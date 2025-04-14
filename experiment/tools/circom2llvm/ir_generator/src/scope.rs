use crate::codegen::CodeGen;
use crate::environment::GlobalInformation;
use crate::expression_codegen::resolve_expr;
use crate::expression_static::ArgTable;
use crate::namer::{
    name_exit_block, name_getter, name_inline_array, name_readwrite_var, name_signal,
    name_template_fn, print_variable_type, ValueTypeEnum,
};
use crate::scope_information::ScopeInformation;
use crate::type_check::{check_used_type, check_used_value, unwrap_used_type, wrap_type2used};
use inkwell::basic_block::BasicBlock;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue};
use program_structure::ast::Access;
use std::collections::HashMap;

pub struct Scope<'ctx> {
    pub info: ScopeInformation<'ctx>,
    pub fn_val: Option<FunctionValue<'ctx>>,
    pub instantiation: ArgTable,

    signature: String,

    current_exit_block: Option<BasicBlock<'ctx>>,

    arg2val: HashMap<String, BasicValueEnum<'ctx>>,

    var2ptr: HashMap<String, PointerValue<'ctx>>,
    var2dims: HashMap<String, Vec<IntValue<'ctx>>>,
}

impl<'ctx> Scope<'ctx> {
    pub fn get_name(&self) -> &String {
        return self.info.get_name();
    }

    pub fn get_signature(&self) -> &String {
        &self.signature
    }

    pub fn canonicalize_fn_name(&self, fn_name: &String) -> String {
        let res = if self.info.is_component(fn_name) {
            name_template_fn(fn_name, "build")
        } else {
            fn_name.clone()
        };
        res
    }

    pub fn initial_var(
        &mut self,
        codegen: &CodeGen<'ctx>,
        name: &String,
        alloca_name: &String,
        ty: &BasicTypeEnum<'ctx>,
        alloca: bool,
    ) -> PointerValue<'ctx> {
        let used_ty = wrap_type2used(ty);
        check_used_type(&used_ty);
        // Example name: initial.a.output
        let ptr = codegen.build_alloca(used_ty, alloca_name);
        if ty.is_array_type() {
            let arr_name = &name_inline_array(&self.get_name());
            let basic_ty = unwrap_used_type(ty);
            let arr_ptr = if alloca {
                codegen.build_alloca(basic_ty, arr_name)
            } else {
                codegen.builder.build_malloc(basic_ty, arr_name).unwrap()
            };
            codegen.builder.build_store(ptr, arr_ptr);
        }
        self.var2ptr.insert(name.clone(), ptr);
        return ptr;
    }

    pub fn get_var(
        &self,
        env: &GlobalInformation<'ctx>,
        codegen: &CodeGen<'ctx>,
        name: &String,
        access: &Vec<Access>,
    ) -> BasicValueEnum<'ctx> {
        let stored_val;
        let val_ty_enum = self.info.get_var_ty_enum(env, name);
        let final_inst_name = name_readwrite_var(true, name, val_ty_enum);
        if self.arg2val.contains_key(name) {
            stored_val = self.arg2val[name];
        } else {
            let ptr = self.var2ptr[name];
            if access.len() == 0 {
                stored_val = codegen
                    .builder
                    .build_load(ptr, &final_inst_name)
                    .expect("REASON");
            } else {
                let inst_name = name_getter("var");
                stored_val = codegen.builder.build_load(ptr, &inst_name).expect("REASON");
            }
        };
        let mut val;
        if access.len() == 0 {
            val = stored_val;
        } else {
            let mut comp_access_idx: Option<usize> = None;
            let mut templ_name = "".to_string();
            let mut signal_name = "".to_string();
            for (idx, a) in access.iter().enumerate() {
                match a {
                    Access::ArrayAccess(..) => (),
                    Access::ComponentAccess(signal) => {
                        comp_access_idx = Some(idx);
                        templ_name = self.info.get_component_name(name).clone();
                        signal_name = signal.clone();
                    }
                }
            }
            match comp_access_idx {
                Some(idx) => {
                    let inst_name0 = name_getter("array");
                    let arr_ptr0 = stored_val;
                    let strt_ptr = self
                        .get_from_array(env, codegen, &access[0..idx], arr_ptr0, &inst_name0)
                        .into_pointer_value();
                    let gep = self.build_struct_gep(
                        env,
                        codegen,
                        &templ_name,
                        &signal_name,
                        strt_ptr,
                        false,
                    );
                    let inst_name1 = if idx == access.len() - 1 {
                        final_inst_name.clone()
                    } else {
                        name_getter("struct")
                    };
                    let arr_ptr1 = self.get_from_struct(codegen, gep, &inst_name1);
                    val = self.get_from_array(
                        env,
                        codegen,
                        &access[idx + 1..],
                        arr_ptr1.as_basic_value_enum(),
                        &final_inst_name,
                    );
                }
                None => {
                    val = self.get_from_array(env, codegen, access, stored_val, &final_inst_name);
                }
            }
            if val.is_array_value() {
                val = codegen
                    .build_direct_array_store(val.into_array_value(), &self.get_name())
                    .as_basic_value_enum()
            }
        }
        check_used_value(&val);
        return val;
    }

    pub fn set_var(
        &mut self,
        env: &GlobalInformation<'ctx>,
        codegen: &CodeGen<'ctx>,
        name: &String,
        access: &Vec<Access>,
        value: BasicValueEnum<'ctx>,
    ) {
        check_used_value(&value);
        let val_ty_enum = self.info.get_var_ty_enum(env, name);
        let mut stored_val = value;
        if stored_val.is_pointer_value() {
            let strt_ptr = stored_val.into_pointer_value();
            if strt_ptr.get_type().get_element_type().is_struct_type() {
                stored_val = self
                    .build_struct_ptr_cast(env, codegen, self.get_name(), strt_ptr)
                    .as_basic_value_enum();
            }
        }
        let mut ptr = self.var2ptr[name];
        let final_inst_name = name_readwrite_var(false, name, val_ty_enum);
        if access.len() == 0 {
            // Example name: initial.a.output
            codegen.builder.build_store(ptr, stored_val);
        } else {
            let inst_name = name_getter("var");
            ptr = codegen
                .builder
                .build_load(ptr, &inst_name)
                .expect("REASON")
                .into_pointer_value();
            let mut comp_access_idx: Option<usize> = None;
            let mut templ_name = "".to_string();
            let mut signal_name = "".to_string();
            for (idx, a) in access.iter().enumerate() {
                match a {
                    Access::ArrayAccess(..) => (),
                    Access::ComponentAccess(signal) => {
                        comp_access_idx = Some(idx);
                        templ_name = self.info.get_component_name(name).clone();
                        signal_name = signal.clone();
                    }
                }
            }
            match comp_access_idx {
                Some(idx) => {
                    if idx == 0 {
                        let gep = self.build_struct_gep(
                            env,
                            codegen,
                            &templ_name,
                            &signal_name,
                            ptr,
                            false,
                        );
                        if access.len() == 1 {
                            self.set_to_struct(codegen, gep, stored_val);
                        } else {
                            let inst_name = name_getter("struct");
                            let arr_ptr = self
                                .get_from_struct(codegen, gep, &inst_name)
                                .into_pointer_value();
                            self.set_to_array(
                                env,
                                codegen,
                                &access[1..],
                                arr_ptr,
                                &final_inst_name,
                                stored_val,
                            );
                        }
                    } else {
                        let struct_ptr = self
                            .get_from_array(
                                env,
                                codegen,
                                &access[0..idx],
                                ptr.as_basic_value_enum(),
                                name,
                            )
                            .into_pointer_value();
                        let gep = self.build_struct_gep(
                            env,
                            codegen,
                            &templ_name,
                            &signal_name,
                            struct_ptr,
                            false,
                        );
                        if idx == access.len() - 1 {
                            self.set_to_struct(codegen, gep, stored_val);
                        } else {
                            let inst_name = name_getter("struct");
                            let arr_ptr = self
                                .get_from_struct(codegen, gep, &inst_name)
                                .into_pointer_value();
                            self.set_to_array(
                                env,
                                codegen,
                                &access[idx + 1..],
                                arr_ptr,
                                &final_inst_name,
                                stored_val,
                            );
                        }
                    }
                }
                None => self.set_to_array(env, codegen, access, ptr, &final_inst_name, stored_val),
            }
        }
    }

    fn get_from_array(
        &self,
        env: &GlobalInformation<'ctx>,
        codegen: &CodeGen<'ctx>,
        access: &[Access],
        arr_ptr: BasicValueEnum<'ctx>,
        inst_name: &str,
    ) -> BasicValueEnum<'ctx> {
        if access.len() == 0 {
            return arr_ptr;
        }
        let indexes = self.access_to_indexes(env, codegen, access);
        let gep = self.build_array_gep(
            codegen,
            &indexes,
            arr_ptr.into_pointer_value(),
            &name_getter("array"),
        );
        let res = codegen.builder.build_load(gep, inst_name);
        res.expect("REASON")
    }

    fn set_to_array(
        &self,
        env: &GlobalInformation<'ctx>,
        codegen: &CodeGen<'ctx>,
        access: &[Access],
        arr_ptr: PointerValue<'ctx>,
        inst_name: &str,
        val: BasicValueEnum<'ctx>,
    ) {
        if access.len() == 0 {
            return;
        }
        let indexes = self.access_to_indexes(env, codegen, access);
        let gep = self.build_array_gep(codegen, &indexes, arr_ptr, inst_name);
        if gep.get_type().get_element_type().is_array_type() {
            let size = env.val_ty.const_int(env.arraysize as u64, false);
            _ = codegen
                .builder
                .build_memcpy(gep, 8, val.into_pointer_value(), 8, size);
        } else {
            codegen.builder.build_store(gep, val);
        }
    }

    fn access_to_indexes(
        &self,
        env: &GlobalInformation<'ctx>,
        codegen: &CodeGen<'ctx>,
        access: &[Access],
    ) -> Vec<IntValue<'ctx>> {
        let mut indexes: Vec<IntValue<'ctx>> = access
            .iter()
            .map(|s| match s {
                Access::ComponentAccess(_) => unreachable!(),
                Access::ArrayAccess(expr) => {
                    resolve_expr(env, codegen, self, &expr).into_int_value()
                }
            })
            .collect();
        indexes.insert(0, env.const_zero);
        indexes
    }

    pub fn build_array_gep(
        &self,
        codegen: &CodeGen<'ctx>,
        indexes: &Vec<IntValue<'ctx>>,
        ptr: PointerValue<'ctx>,
        inst_name: &str,
    ) -> PointerValue<'ctx> {
        unsafe {
            codegen
                .builder
                .build_in_bounds_gep(ptr, indexes, inst_name)
                .expect("REASON")
        }
    }

    pub fn get_from_struct(
        &self,
        codegen: &CodeGen<'ctx>,
        gep: PointerValue<'ctx>,
        inst_name: &String,
    ) -> BasicValueEnum<'ctx> {
        codegen.builder.build_load(gep, inst_name).expect("REASON")
    }

    pub fn set_to_struct(
        &self,
        codegen: &CodeGen<'ctx>,
        gep: PointerValue<'ctx>,
        val: BasicValueEnum<'ctx>,
    ) {
        codegen.builder.build_store(gep, val);
    }

    pub fn build_struct_gep(
        &self,
        env: &GlobalInformation<'ctx>,
        codegen: &CodeGen<'ctx>,
        templ_name: &String,
        signal_name: &String,
        ptr: PointerValue<'ctx>,
        is_inner: bool,
    ) -> PointerValue<'ctx> {
        let template_info = env.get_template_info(templ_name);
        let (index, mut var_ty) = template_info.get_signal_info(signal_name).unwrap();
        if !is_inner {
            if var_ty == ValueTypeEnum::InputSignal {
                var_ty = ValueTypeEnum::ComponentInput
            } else if var_ty == ValueTypeEnum::OutputSignal {
                var_ty = ValueTypeEnum::ComponentOutput
            } else {
                println!(
                    "Error: Unexpected value type: {}",
                    print_variable_type(&var_ty)
                );
            }
        }
        let inst_name = name_signal(templ_name, signal_name, var_ty);
        let real_struct_ptr = self.build_struct_ptr_cast(env, codegen, templ_name, ptr);
        let res = codegen
            .builder
            .build_struct_gep(real_struct_ptr, index, &inst_name)
            .unwrap();
        res
    }

    pub fn build_struct_ptr_cast(
        &self,
        env: &GlobalInformation<'ctx>,
        codegen: &CodeGen<'ctx>,
        templ_name: &String,
        struct_ptr: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let strt_ty = struct_ptr.get_type().get_element_type().into_struct_type();
        let real_struct_ptr;
        if strt_ty.count_fields() == 0 {
            let real_strt_ty = env.get_scope_ret_ty(templ_name).into_pointer_type();
            real_struct_ptr =
                codegen
                    .builder
                    .build_pointer_cast(struct_ptr, real_strt_ty, "ptr_cast");
        } else {
            real_struct_ptr = Ok(struct_ptr);
        }
        return real_struct_ptr.expect("REASON");
    }

    pub fn set_arg_val(&mut self, arg_name: &String, value: &BasicValueEnum<'ctx>) {
        check_used_value(value);
        self.arg2val.insert(arg_name.clone(), value.clone());
    }

    pub fn get_var_dims(&self, name: &String) -> Option<&Vec<IntValue<'ctx>>> {
        return self.var2dims.get(name);
    }

    pub fn set_var_dims(&mut self, name: &String, dims: Vec<IntValue<'ctx>>) {
        if dims.len() == 0 {
            return;
        }
        self.var2dims.insert(name.clone(), dims);
    }

    pub fn build_exit(&self, codegen: &CodeGen<'ctx>) {
        let current_bb = codegen.builder.get_insert_block().unwrap();
        let fn_val = self.get_main_fn();

        let exit_bb = codegen
            .context
            .append_basic_block(fn_val, &name_exit_block());
        codegen.build_block_transferring(current_bb, exit_bb);

        if self.info.is_template {
            for (name, ptr) in &self.var2ptr {
                let dims_op = self.get_var_dims(name);
                match dims_op {
                    Some(dims) => {
                        codegen.build_arraydim(ptr, &dims);
                    }
                    None => (),
                }
            }
        }
    }

    pub fn get_main_fn(&self) -> FunctionValue<'ctx> {
        return self.fn_val.unwrap();
    }

    pub fn set_main_fn(&mut self, fn_val: FunctionValue<'ctx>) {
        self.fn_val = Some(fn_val);
    }

    pub fn get_current_exit_block(&self) -> BasicBlock<'ctx> {
        return self.current_exit_block.unwrap();
    }

    pub fn set_current_exit_block(&mut self, codegen: &CodeGen<'ctx>, bb: BasicBlock<'ctx>) {
        self.current_exit_block = Some(bb);
        codegen.builder.position_at_end(bb);
    }
}

pub fn init_scope<'ctx>(info: ScopeInformation<'ctx>, instantiation: ArgTable) -> Scope<'ctx> {
    let signature = info.gen_signature(&instantiation);
    return Scope {
        info,
        instantiation,
        signature,

        fn_val: None,
        current_exit_block: None,

        arg2val: HashMap::new(),
        var2ptr: HashMap::new(),
        var2dims: HashMap::new(),
    };
}
