use crate::namer::{
    name_constraint, name_entry_block, name_if_block, name_inline_array, name_intrinsinc_fn,
    name_main_comp,
};
use crate::utils::is_terminated_basicblock;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::intrinsics::Intrinsic;
use inkwell::module::{Linkage, Module};
use inkwell::types::{BasicType, BasicTypeEnum, IntType};
use inkwell::values::{
    AnyValue, ArrayValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue,
    IntValue, PointerValue,
};
use inkwell::{AddressSpace, IntPredicate};

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    // Internal utils
    _utils_constraint_fn_val: FunctionValue<'ctx>,
    _utils_constraint_array_fn_val: FunctionValue<'ctx>,
    _utils_switch_fn_val: FunctionValue<'ctx>,
    _utils_powi_fn_val: FunctionValue<'ctx>,
    _utils_init_fn_val: FunctionValue<'ctx>,
    _utils_assert_fn_val: FunctionValue<'ctx>,
    _utils_arraydim_fn_val: FunctionValue<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn build_alloca(
        &self,
        val_ty: BasicTypeEnum<'ctx>,
        alloca_name: &String,
    ) -> PointerValue<'ctx> {
        let current_block = self.builder.get_insert_block().unwrap();
        let entry_block = current_block
            .get_parent()
            .unwrap()
            .get_first_basic_block()
            .unwrap();
        if current_block == entry_block {
            self.builder
                .build_alloca(val_ty, alloca_name)
                .expect("REASON")
        } else {
            // Last instruction is branch.
            self.builder
                .position_at(entry_block, &entry_block.get_last_instruction().unwrap());
            let res = self.builder.build_alloca(val_ty, alloca_name);
            self.builder.position_at_end(current_block);
            res.expect("REASON")
        }
    }

    pub fn build_constraint(&self, lval: BasicValueEnum<'ctx>, rval: BasicValueEnum<'ctx>) {
        let gv = self
            .module
            .add_global(self.context.bool_type(), None, &name_constraint());
        if lval.get_type() != rval.get_type() {
            println!(
                "Error: Left value and right value should be the same type in the constraint."
            );
            println!("Left value is: {}", lval.print_to_string());
            println!("Right value is: {}", rval.print_to_string());
            unreachable!();
        }
        if lval.is_int_value() {
            self.builder.build_call(
                self._utils_constraint_fn_val,
                &[
                    lval.into_int_value().into(),
                    rval.into_int_value().into(),
                    gv.as_basic_value_enum().into(),
                ],
                &name_constraint(),
            );
        }
        if lval.is_array_value() {
            self.builder.build_call(
                self._utils_constraint_array_fn_val,
                &[
                    lval.into_pointer_value().into(),
                    rval.into_pointer_value().into(),
                    gv.as_basic_value_enum().into(),
                ],
                &name_constraint(),
            );
        }
    }

    pub fn build_switch(
        &self,
        cond: IntValue<'ctx>,
        lval: IntValue<'ctx>,
        rval: IntValue<'ctx>,
    ) -> IntValue<'ctx> {
        let inst_name = "utils_switch";
        let res = self.builder.build_call(
            self._utils_switch_fn_val,
            &[cond.into(), lval.into(), rval.into()],
            inst_name,
        );
        return res
            .expect("REASON")
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
    }

    // Prevent constant folding
    pub fn build_initial_var(&self, inst_name: &String) -> BasicValueEnum<'ctx> {
        let res = self
            .builder
            .build_call(self._utils_init_fn_val, &[], inst_name);
        return res.expect("REASON").try_as_basic_value().left().unwrap();
    }

    pub fn build_result_modulo(&self, value: IntValue<'ctx>) -> IntValue<'ctx> {
        // if unsafe { APPLY_MOD } {
        //     let name = &format!("{}.mod", value.get_name().to_str().unwrap())[0..];
        //     return self
        //         .builder
        //         .build_int_signed_rem(value, self._global_p, name);
        // } else {
        //     return value;
        // }
        return value;
    }

    pub fn build_pow(&self, args: &[BasicMetadataValueEnum<'ctx>], name: &str) -> IntValue<'ctx> {
        return self
            .builder
            .build_call(self._utils_powi_fn_val, args, name)
            .expect("REASON")
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();
    }

    pub fn build_block_transferring(
        &self,
        source_bb: BasicBlock<'ctx>,
        destination_bb: BasicBlock<'ctx>,
    ) {
        if !is_terminated_basicblock(&source_bb) {
            self.builder.position_at_end(source_bb);
            self.builder.build_unconditional_branch(destination_bb);
        }
        self.builder.position_at_end(destination_bb);
    }

    pub fn build_assert(&self, val: IntValue<'ctx>) {
        self.builder
            .build_call(self._utils_assert_fn_val, &[val.into()], "assert");
    }

    pub fn build_arraydim(&self, ptr: &PointerValue<'ctx>, dims: &Vec<IntValue<'ctx>>) {
        let default_ptr_ty = self.context.i128_type().ptr_type(AddressSpace::default());
        let _ptr = self
            .builder
            .build_pointer_cast(ptr.clone(), default_ptr_ty, "ptr_cast");
        let _ptr = _ptr.unwrap(); // Or use `?` to propagate the error
        let mut vals = vec![_ptr.into()];
        // let mut vals = vec![_ptr.into()];
        for d in dims {
            if !d.is_const() {
                println!(
                    "Error: Dimension is not a constant, current: {}",
                    d.print_to_string()
                );
                unreachable!();
            }
            vals.push(d.clone().into());
        }
        self.builder
            .build_call(self._utils_arraydim_fn_val, &vals, "arraydim");
    }

    pub fn build_instantiation_flag(&self, main_signature: &String) {
        self.module
            .add_global(self.context.bool_type(), None, "is_instantiation");
        self.module.add_global(
            self.context.bool_type(),
            None,
            &name_main_comp(&main_signature),
        );
    }

    pub fn build_direct_array_store(
        &self,
        arr_val: ArrayValue<'ctx>,
        scope_name: &String,
    ) -> PointerValue<'ctx> {
        let arr_name = &name_inline_array(scope_name);
        let ptr = self.build_alloca(arr_val.get_type().as_basic_type_enum(), arr_name);
        // Example name: tempinlinearray
        self.builder.build_store(ptr, arr_val);
        ptr
    }
}

pub fn init_codegen<'ctx>(
    context: &'ctx Context,
    module: Module<'ctx>,
    val_ty: IntType<'ctx>,
    arraysize: u32,
) -> CodeGen<'ctx> {
    let builder = context.create_builder();
    let bool_ty = context.bool_type();
    let const_zero = val_ty.const_zero();

    // Add constraint function
    let utils_constraint_gv_ptr_ty = bool_ty.ptr_type(AddressSpace::default());
    let utils_constraint_fn_args_ty = [
        val_ty.into(),
        val_ty.into(),
        utils_constraint_gv_ptr_ty.into(),
    ];
    let utils_constraint_fn_ret_ty = context.void_type();
    let utils_constraint_fn_ty =
        utils_constraint_fn_ret_ty.fn_type(&utils_constraint_fn_args_ty, false);
    let utils_constraint_fn_name = name_intrinsinc_fn("utils_constraint");
    let utils_constraint_fn_val =
        module.add_function(&utils_constraint_fn_name, utils_constraint_fn_ty, None);
    let utils_constraint_fn_entry_bb =
        context.append_basic_block(utils_constraint_fn_val, &name_entry_block());
    builder.position_at_end(utils_constraint_fn_entry_bb);

    let inst_name = name_constraint();
    let eq_val = builder.build_int_compare(
        IntPredicate::EQ,
        utils_constraint_fn_val
            .get_first_param()
            .unwrap()
            .into_int_value(),
        utils_constraint_fn_val
            .get_nth_param(1)
            .unwrap()
            .into_int_value(),
        &inst_name,
    );
    let gv_val = utils_constraint_fn_val
        .get_last_param()
        .unwrap()
        .into_pointer_value();
    builder.build_store(gv_val, eq_val.unwrap());
    builder.build_return(None);

    // Add constraint array function
    let utils_constraint_array_fn_args_ty = [
        val_ty
            .array_type(arraysize as u32)
            .ptr_type(AddressSpace::default())
            .into(),
        val_ty
            .array_type(arraysize as u32)
            .ptr_type(AddressSpace::default())
            .into(),
        utils_constraint_gv_ptr_ty.into(),
    ];
    let utils_constraint_array_fn_ret_ty = context.void_type();
    let utils_constraint_array_fn_ty =
        utils_constraint_array_fn_ret_ty.fn_type(&utils_constraint_array_fn_args_ty, false);
    let utils_constraint_array_fn_name = name_intrinsinc_fn("utils_constraint_array");
    let utils_constraint_array_fn_val = module.add_function(
        &utils_constraint_array_fn_name,
        utils_constraint_array_fn_ty,
        None,
    );
    let utils_constraint_array_fn_entry_bb =
        context.append_basic_block(utils_constraint_array_fn_val, &name_entry_block());
    builder.position_at_end(utils_constraint_array_fn_entry_bb);
    builder.build_return(None);

    // Add inline switch function
    let utils_switch_fn_args_ty = [bool_ty.into(), val_ty.into(), val_ty.into()];
    let utils_switch_fn_ret_ty = val_ty;
    let utils_switch_fn_ty = utils_switch_fn_ret_ty.fn_type(&utils_switch_fn_args_ty, false);
    let utils_switch_fn_name = name_intrinsinc_fn("utils_switch");
    let utils_switch_fn_val = module.add_function(&utils_switch_fn_name, utils_switch_fn_ty, None);
    let utils_switch_fn_entry_bb =
        context.append_basic_block(utils_switch_fn_val, &name_entry_block());
    let utils_switch_fn_t_bb =
        context.append_basic_block(utils_switch_fn_val, &name_if_block(true, false));
    let utils_switch_fn_f_bb =
        context.append_basic_block(utils_switch_fn_val, &name_if_block(false, false));
    builder.position_at_end(utils_switch_fn_entry_bb);

    builder.build_conditional_branch(
        utils_switch_fn_val
            .get_first_param()
            .unwrap()
            .into_int_value(),
        utils_switch_fn_t_bb,
        utils_switch_fn_f_bb,
    );
    builder.position_at_end(utils_switch_fn_t_bb);
    builder.build_return(Some(&utils_switch_fn_val.get_nth_param(1).unwrap()));
    builder.position_at_end(utils_switch_fn_f_bb);
    builder.build_return(Some(&utils_switch_fn_val.get_nth_param(2).unwrap()));

    // Add powi function
    let powi_base_ty = context.f128_type();
    let powi_power_ty = context.i32_type();
    let powi_intrinsic = Intrinsic::find("llvm.powi").unwrap();
    let powi_fn_val = powi_intrinsic
        .get_declaration(&module, &[powi_base_ty.into(), powi_power_ty.into()])
        .unwrap();

    let utils_powi_fn_args_ty = [val_ty.into(), val_ty.into()];
    let utils_powi_fn_ret_ty = val_ty;
    let utils_powi_fn_ty = utils_powi_fn_ret_ty.fn_type(&utils_powi_fn_args_ty, false);
    let utils_powi_fn_name = name_intrinsinc_fn("utils_powi");
    let utils_powi_fn_val = module.add_function(&utils_powi_fn_name, utils_powi_fn_ty, None);
    let utils_powi_fn_entry_bb = context.append_basic_block(utils_powi_fn_val, &name_entry_block());
    builder.position_at_end(utils_powi_fn_entry_bb);

    let origin_base = utils_powi_fn_val
        .get_first_param()
        .unwrap()
        .into_int_value();
    let origin_power = utils_powi_fn_val.get_last_param().unwrap().into_int_value();
    let base = builder.build_unsigned_int_to_float(origin_base, powi_base_ty, "utils_powi.base");
    let power = builder.build_int_cast(origin_power, powi_power_ty, "utils_powi.power");
    let powi = builder
        .build_call(
            powi_fn_val,
            &[base.unwrap().into(), power.unwrap().into()],
            "utils_powi.cal",
        )
        .expect("REASON")
        .try_as_basic_value()
        .unwrap_left();
    let ret =
        builder.build_float_to_unsigned_int(powi.into_float_value(), val_ty, "utils_powi.ret");
    builder.build_return(Some(&ret.unwrap()));

    let utils_init_fn_ret_ty = val_ty;
    let utils_init_fn_ty = utils_init_fn_ret_ty.fn_type(&[], false);
    let utils_init_fn_name = name_intrinsinc_fn("utils_init");
    let utils_init_fn_val = module.add_function(&utils_init_fn_name, utils_init_fn_ty, None);
    let utils_init_fn_entry_bb = context.append_basic_block(utils_init_fn_val, &name_entry_block());
    builder.position_at_end(utils_init_fn_entry_bb);
    builder.build_return(Some(&const_zero));

    let utils_assert_fn_args_ty = [context.bool_type().into()];
    let utils_assert_fn_ret_ty = context.void_type();
    let utils_assert_fn_ty = utils_assert_fn_ret_ty.fn_type(&utils_assert_fn_args_ty, false);
    let utils_assert_fn_name = name_intrinsinc_fn("utils_assert");
    let utils_assert_fn_val = module.add_function(&utils_assert_fn_name, utils_assert_fn_ty, None);
    let utils_assert_fn_entry_bb =
        context.append_basic_block(utils_assert_fn_val, &name_entry_block());
    builder.position_at_end(utils_assert_fn_entry_bb);
    builder.build_return(None);

    let default_ptr_ty = val_ty.ptr_type(AddressSpace::default());
    let utils_arraydim_fn_args_ty = [default_ptr_ty.into()];
    let utils_arraydim_fn_ret_ty = context.void_type();
    let utils_arraydim_fn_ty = utils_arraydim_fn_ret_ty.fn_type(&utils_arraydim_fn_args_ty, true);
    let utils_arraydim_fn_name = name_intrinsinc_fn("utils_arraydim");
    let utils_arraydim_fn_val =
        module.add_function(&utils_arraydim_fn_name, utils_arraydim_fn_ty, None);
    let utils_arraydim_fn_entry_bb =
        context.append_basic_block(utils_arraydim_fn_val, &name_entry_block());
    builder.position_at_end(utils_arraydim_fn_entry_bb);
    builder.build_return(None);

    let i128_type = context.i128_type();
    let mod_operation_fn_ty = i128_type.fn_type(
        &[i128_type.into(), i128_type.into(), i128_type.into()],
        false,
    );
    module.add_function("mod_add", mod_operation_fn_ty, Some(Linkage::External));
    module.add_function("mod_sub", mod_operation_fn_ty, Some(Linkage::External));
    module.add_function("mod_mul", mod_operation_fn_ty, Some(Linkage::External));
    module.add_function("mod_div", mod_operation_fn_ty, Some(Linkage::External));

    let codegen = CodeGen {
        context,
        module,
        builder,

        _utils_constraint_fn_val: utils_constraint_fn_val,
        _utils_constraint_array_fn_val: utils_constraint_array_fn_val,
        _utils_switch_fn_val: utils_switch_fn_val,
        _utils_powi_fn_val: utils_powi_fn_val,
        _utils_init_fn_val: utils_init_fn_val,
        _utils_assert_fn_val: utils_assert_fn_val,
        _utils_arraydim_fn_val: utils_arraydim_fn_val,
    };
    return codegen;
}
