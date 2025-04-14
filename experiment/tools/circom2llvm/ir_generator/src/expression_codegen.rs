use crate::codegen::CodeGen;
use crate::environment::GlobalInformation;
use crate::expression_static::{
    resolve_expr_static, resolve_inline_array_static, resolve_number_static,
    resolve_uniform_array_static, ConcreteValue,
};
use crate::namer::{name_inline_array, name_template_fn};
use crate::scope::Scope;
use crate::type_check::check_used_value;
use inkwell::builder::BuilderError;
use inkwell::types::BasicType;
use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, IntValue, PointerValue};
use inkwell::IntPredicate;
use program_structure::ast::{
    Access, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, Statement,
};

const FIELD_MODULUS: u64 = 9938766679346745377; // 279926033603397055746914466956409631037;

pub fn resolve_expr<'ctx>(
    env: &GlobalInformation<'ctx>,
    codegen: &CodeGen<'ctx>,
    scope: &Scope<'ctx>,
    expr: &Expression,
) -> BasicValueEnum<'ctx> {
    use Expression::*;
    let res = match expr {
        AnonymousComp { .. } => {
            println!("Error: AnonymousComp isn't supported now.");
            unreachable!()
        }
        ArrayInLine { .. } => {
            let ptr = resolve_inline_array(env, codegen, scope, expr);
            ptr.as_basic_value_enum()
        }
        Call { meta: _, id, args } => {
            let fn_name: String;
            if scope.info.is_component(id) {
                fn_name = name_template_fn("build", id);
            } else {
                fn_name = id.to_string();
            }
            let called_fn = codegen.module.get_function(&fn_name);
            match called_fn {
                Some(called_fn) => {
                    // println!("Func:");
                    // called_fn.print_to_stderr();
                    let arg_vals: Vec<BasicMetadataValueEnum> = args
                        .iter()
                        .map(|a| {
                            let basic_val = resolve_expr(env, codegen, scope, a);
                            // println!("Value: {}", basic_val.print_to_string());
                            // println!("Type: {}", basic_val.get_type().print_to_string());
                            BasicMetadataValueEnum::from(basic_val)
                        })
                        .collect();
                    codegen
                        .builder
                        .build_call(called_fn, &arg_vals, "call")
                        .expect("REASON")
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                }
                None => env.get_scope_ret_ty(id).const_zero(),
            }
        }
        InfixOp {
            meta: _,
            lhe,
            infix_op,
            rhe,
        } => {
            let lval = resolve_expr(env, codegen, scope, lhe.as_ref()).into_int_value();
            let rval = resolve_expr(env, codegen, scope, rhe.as_ref()).into_int_value();

            let res = resolve_infix_op(env, codegen, infix_op, lval, rval);
            res.as_basic_value_enum()
        }
        InlineSwitchOp {
            meta: _,
            cond,
            if_true,
            if_false,
        } => {
            let cond = resolve_expr(env, codegen, scope, cond.as_ref()).into_int_value();
            let lval = resolve_expr(env, codegen, scope, if_true.as_ref()).into_int_value();
            let rval = resolve_expr(env, codegen, scope, if_false.as_ref()).into_int_value();
            codegen.build_switch(cond, lval, rval).as_basic_value_enum()
        }
        Number(..) => {
            let number = resolve_number_static(expr);
            env.val_ty
                .const_int(number as u64, true)
                .as_basic_value_enum()
        }
        ParallelOp { .. } => {
            println!("Error: ParallelOp isn't supported now.");
            unreachable!()
        }
        PrefixOp {
            meta: _,
            prefix_op,
            rhe,
        } => {
            let rval = resolve_expr(env, codegen, scope, rhe.as_ref()).into_int_value();
            let res = resolve_prefix_op(env, codegen, prefix_op, rval);
            res.as_basic_value_enum()
        }
        Tuple { .. } => {
            println!("Error: Tuple isn't supported now.");
            unreachable!()
        }
        UniformArray { .. } => {
            let val = resolve_uniform_array(env, codegen, scope, expr);
            val.as_basic_value_enum()
        }
        Variable {
            meta: _,
            name,
            access,
        } => {
            let val = scope.get_var(env, codegen, name, access);
            val
        }
    };
    check_used_value(&res);
    return res;
}

fn resolve_inline_array<'ctx>(
    env: &GlobalInformation<'ctx>,
    codegen: &CodeGen<'ctx>,
    scope: &Scope<'ctx>,
    expr: &Expression,
) -> PointerValue<'ctx> {
    let (arr_ty, dim_record) = resolve_inline_array_static(env, expr);
    let arr_name = name_inline_array(&scope.get_name());
    let ptr = codegen.build_alloca(arr_ty.as_basic_type_enum(), &arr_name);
    resolve_inline_array_internal(env, codegen, scope, expr, ptr, vec![0]);
    let dims = dim_record
        .iter()
        .map(|d| env.val_ty.const_int(*d as u64, true))
        .collect();
    if scope.info.is_template {
        codegen.build_arraydim(&ptr, &dims);
    }
    ptr
}

fn resolve_inline_array_internal<'ctx>(
    env: &GlobalInformation<'ctx>,
    codegen: &CodeGen<'ctx>,
    scope: &Scope<'ctx>,
    expr: &Expression,
    arr_ptr: PointerValue<'ctx>,
    indexes: Vec<u32>,
) {
    use Expression::*;
    match expr {
        ArrayInLine { meta: _, values } => {
            for (idx, v) in values.iter().enumerate() {
                let mut new_indexes = indexes.clone();
                new_indexes.push(idx as u32);
                resolve_inline_array_internal(env, codegen, scope, v, arr_ptr, new_indexes);
            }
        }
        _ => {
            let val = resolve_expr(env, codegen, scope, expr);
            let indexes: Vec<IntValue> = indexes
                .iter()
                .map(|i| env.val_ty.const_int(*i as u64, true))
                .collect();
            let gep = scope.build_array_gep(codegen, &indexes, arr_ptr, "init_inlinearray");
            codegen.builder.build_store(gep, val);
        }
    }
}

fn resolve_uniform_array_dims<'ctx>(
    env: &GlobalInformation<'ctx>,
    codegen: &CodeGen<'ctx>,
    scope: &Scope<'ctx>,
    expr: &Expression,
) -> Vec<IntValue<'ctx>> {
    let mut dims_record = Vec::new();
    let mut end = false;
    let mut current_expr = expr;
    while !end {
        match current_expr {
            Expression::UniformArray {
                meta: _,
                value,
                dimension,
            } => {
                dims_record.push(resolve_dimension_as_record(env, codegen, scope, dimension));
                current_expr = value.as_ref();
            }
            _ => {
                end = true;
            }
        }
    }
    dims_record
}

fn resolve_uniform_array<'ctx>(
    env: &GlobalInformation<'ctx>,
    codegen: &CodeGen<'ctx>,
    scope: &Scope<'ctx>,
    expr: &Expression,
) -> PointerValue<'ctx> {
    let dims = resolve_uniform_array_dims(env, codegen, scope, expr);
    let arr_val = resolve_uniform_array_static(env, &scope.info, expr);
    let ptr = codegen.build_direct_array_store(arr_val, &scope.get_name());
    if scope.info.is_template {
        codegen.build_arraydim(&ptr, &dims);
    }
    return ptr;
}

fn resolve_prefix_op<'ctx>(
    env: &GlobalInformation<'ctx>,
    codegen: &CodeGen<'ctx>,
    prefix_op: &ExpressionPrefixOpcode,
    rval: IntValue<'ctx>,
) -> IntValue<'ctx> {
    use ExpressionPrefixOpcode::*;
    let temp = match prefix_op {
        //Sub => codegen.builder.build_int_sub(env.const_zero, rval, "neg"),
        Sub => {
            let mod_sub_fn = codegen
                .module
                .get_function("mod_sub")
                .expect("mod_sub function not found");
            let call_site_value = codegen
                .builder
                .build_call(
                    mod_sub_fn,
                    &[
                        inkwell::values::BasicMetadataValueEnum::IntValue(env.const_zero),
                        inkwell::values::BasicMetadataValueEnum::IntValue(rval),
                        inkwell::values::BasicMetadataValueEnum::IntValue(
                            codegen.context.i128_type().const_int(FIELD_MODULUS, false),
                        ),
                    ],
                    "mod_sub",
                )
                .expect("Failed to build mod_sub call");
            // Convert CallSiteValue to IntValue
            match call_site_value.try_as_basic_value().left() {
                Some(basic_value) => Ok(basic_value.into_int_value()),
                None => panic!("mod_sub call returned void"),
            }
        }
        BoolNot => codegen.builder.build_not(rval, "not"),
        Complement => {
            println!("Error: Complement isn't supported now.");
            unreachable!();
        }
    };
    codegen.build_result_modulo(temp.expect("REASON"))
    //.expect("REASON")
}

fn resolve_infix_op<'ctx>(
    env: &GlobalInformation<'ctx>,
    codegen: &CodeGen<'ctx>,
    infix_op: &ExpressionInfixOpcode,
    lval: IntValue<'ctx>,
    rval: IntValue<'ctx>,
) -> IntValue<'ctx> {
    use ExpressionInfixOpcode::*;
    let mut lval = lval;
    let mut rval = rval;
    if lval.get_type() != rval.get_type() {
        if rval.get_type() == env.val_ty {
            lval = codegen
                .builder
                .build_int_cast(lval, env.val_ty, "intcast")
                .expect("REASON");
        } else {
            rval = codegen
                .builder
                .build_int_cast(rval, env.val_ty, "intcast")
                .expect("REASON");
        }
    }
    let temp = match infix_op {
        // Add => codegen.builder.build_int_add(lval, rval, "add"),
        Add => {
            let mod_add_fn = codegen
                .module
                .get_function("mod_add")
                .expect("mod_add function not found");
            let call_site_value = codegen
                .builder
                .build_call(
                    mod_add_fn,
                    &[
                        inkwell::values::BasicMetadataValueEnum::IntValue(lval),
                        inkwell::values::BasicMetadataValueEnum::IntValue(rval),
                        inkwell::values::BasicMetadataValueEnum::IntValue(
                            codegen.context.i128_type().const_int(FIELD_MODULUS, false),
                        ),
                    ],
                    "mod_add",
                )
                .expect("Failed to build mod_add call");
            // Convert CallSiteValue to IntValue
            match call_site_value.try_as_basic_value().left() {
                Some(basic_value) => Ok(basic_value.into_int_value()),
                None => panic!("mod_add call returned void"),
            }
        }
        BitAnd => codegen.builder.build_and(lval, rval, "and"),
        BitOr => codegen.builder.build_or(lval, rval, "or"),
        BitXor => codegen.builder.build_xor(lval, rval, "xor"),
        BoolAnd => codegen.builder.build_and(lval, rval, "and"),
        BoolOr => codegen.builder.build_and(lval, rval, "or"),
        // Div => codegen.builder.build_int_signed_div(lval, rval, "sdiv"),
        Div => {
            let mod_div_fn = codegen
                .module
                .get_function("mod_div")
                .expect("mod_div function not found");
            let call_site_value = codegen
                .builder
                .build_call(
                    mod_div_fn,
                    &[
                        inkwell::values::BasicMetadataValueEnum::IntValue(lval),
                        inkwell::values::BasicMetadataValueEnum::IntValue(rval),
                        inkwell::values::BasicMetadataValueEnum::IntValue(
                            codegen.context.i128_type().const_int(FIELD_MODULUS, false),
                        ),
                    ],
                    "mod_div",
                )
                .expect("Failed to build mod_div call");
            // Convert CallSiteValue to IntValue
            match call_site_value.try_as_basic_value().left() {
                Some(basic_value) => Ok(basic_value.into_int_value()),
                None => panic!("mod_div call returned void"),
            }
        }
        IntDiv => codegen.builder.build_int_signed_div(lval, rval, "sdiv"),
        Mod => codegen.builder.build_int_signed_rem(lval, rval, "mod"),
        //Mul => codegen.builder.build_int_mul(lval, rval, "mul"),
        Mul => {
            let mod_mul_fn = codegen
                .module
                .get_function("mod_mul")
                .expect("mod_mul function not found");
            let call_site_value = codegen
                .builder
                .build_call(
                    mod_mul_fn,
                    &[
                        inkwell::values::BasicMetadataValueEnum::IntValue(lval),
                        inkwell::values::BasicMetadataValueEnum::IntValue(rval),
                        inkwell::values::BasicMetadataValueEnum::IntValue(
                            codegen.context.i128_type().const_int(FIELD_MODULUS, false),
                        ),
                    ],
                    "mod_mul",
                )
                .expect("Failed to build mod_mul call");
            // Convert CallSiteValue to IntValue
            match call_site_value.try_as_basic_value().left() {
                Some(basic_value) => Ok(basic_value.into_int_value()),
                None => panic!("mod_mul call returned void"),
            }
        }
        Pow => Ok(codegen.build_pow(&[lval.into(), rval.into()], "pow")),
        ShiftL => codegen.builder.build_left_shift(lval, rval, "lshift"),
        ShiftR => codegen
            .builder
            .build_right_shift(lval, rval, true, "rshift"),
        // Sub => codegen.builder.build_int_sub(lval, rval, "sub"),
        Sub => {
            let mod_sub_fn = codegen
                .module
                .get_function("mod_sub")
                .expect("mod_sub function not found");
            let call_site_value = codegen
                .builder
                .build_call(
                    mod_sub_fn,
                    &[
                        inkwell::values::BasicMetadataValueEnum::IntValue(lval),
                        inkwell::values::BasicMetadataValueEnum::IntValue(rval),
                        inkwell::values::BasicMetadataValueEnum::IntValue(
                            codegen.context.i128_type().const_int(FIELD_MODULUS, false),
                        ),
                    ],
                    "mod_sub",
                )
                .expect("Failed to build mod_sub call");
            // Convert CallSiteValue to IntValue
            match call_site_value.try_as_basic_value().left() {
                Some(basic_value) => Ok(basic_value.into_int_value()),
                None => panic!("mod_sub call returned void"),
            }
        }

        // Comparison
        Eq => codegen
            .builder
            .build_int_compare(IntPredicate::EQ, lval, rval, "eq"),
        Greater => codegen
            .builder
            .build_int_compare(IntPredicate::SGT, lval, rval, "sgt"),
        GreaterEq => codegen
            .builder
            .build_int_compare(IntPredicate::SGE, lval, rval, "sge"),
        NotEq => codegen
            .builder
            .build_int_compare(IntPredicate::NE, lval, rval, "ne"),
        Lesser => codegen
            .builder
            .build_int_compare(IntPredicate::SLT, lval, rval, "slt"),
        LesserEq => codegen
            .builder
            .build_int_compare(IntPredicate::SLE, lval, rval, "sle"),
    };
    codegen.build_result_modulo(temp.expect("REASON"))
}

pub fn flat_expressions<'ctx>(expr: &Expression) -> Vec<&Expression> {
    use Expression::*;
    let mut all_exprs: Vec<&Expression> = vec![expr];
    match expr {
        AnonymousComp {
            params, signals, ..
        } => {
            for p in params {
                all_exprs.append(&mut flat_expressions(p));
            }
            for s in signals {
                all_exprs.append(&mut flat_expressions(s));
            }
        }
        ArrayInLine { values, .. } => {
            for v in values {
                all_exprs.append(&mut flat_expressions(v));
            }
        }
        Call { args, .. } => {
            for a in args {
                all_exprs.append(&mut flat_expressions(a));
            }
        }
        InfixOp { lhe, rhe, .. } => {
            all_exprs.append(&mut flat_expressions(lhe.as_ref()));
            all_exprs.append(&mut flat_expressions(rhe.as_ref()));
        }
        InlineSwitchOp {
            meta: _,
            cond,
            if_true,
            if_false,
        } => {
            all_exprs.append(&mut flat_expressions(cond.as_ref()));
            all_exprs.append(&mut flat_expressions(if_true.as_ref()));
            all_exprs.append(&mut flat_expressions(if_false.as_ref()));
        }
        Number(..) => (),
        ParallelOp { meta: _, rhe } => {
            all_exprs.append(&mut flat_expressions(rhe.as_ref()));
        }
        PrefixOp { rhe, .. } => {
            all_exprs.append(&mut flat_expressions(rhe.as_ref()));
        }
        Tuple { meta: _, values } => {
            for v in values {
                all_exprs.append(&mut flat_expressions(v));
            }
        }
        UniformArray {
            meta: _,
            value,
            dimension,
        } => {
            all_exprs.append(&mut flat_expressions(value.as_ref()));
            all_exprs.append(&mut flat_expressions(dimension.as_ref()));
        }
        Variable { .. } => (),
    }
    return all_exprs;
}

pub fn flat_expressions_from_statement<'ctx>(stmt: &Statement) -> Vec<&Expression> {
    let mut all_exprs: Vec<&Expression> = vec![];
    use Statement::*;
    match stmt {
        Assert { meta: _, arg } => {
            all_exprs.append(&mut flat_expressions(arg));
        }
        Block { .. } => (),
        ConstraintEquality { meta: _, lhe, rhe } => {
            all_exprs.append(&mut flat_expressions(lhe));
            all_exprs.append(&mut flat_expressions(rhe));
        }
        Declaration { dimensions, .. } => {
            for d in dimensions {
                all_exprs.append(&mut flat_expressions(d));
            }
        }
        IfThenElse { cond, .. } => {
            all_exprs.append(&mut flat_expressions(cond));
        }
        InitializationBlock { .. } => (),
        LogCall { .. } => (),
        MultSubstitution { lhe, rhe, .. } => {
            all_exprs.append(&mut flat_expressions(lhe));
            all_exprs.append(&mut flat_expressions(rhe));
        }
        Return { meta: _, value } => {
            all_exprs.append(&mut flat_expressions(value));
        }
        Substitution { access, rhe, .. } => {
            for a in access {
                match a {
                    Access::ArrayAccess(expr) => {
                        all_exprs.append(&mut flat_expressions(expr));
                    }
                    Access::ComponentAccess(..) => (),
                }
            }
            all_exprs.append(&mut flat_expressions(rhe));
        }
        While { cond, .. } => {
            all_exprs.append(&mut flat_expressions(cond));
        }
    }
    return all_exprs;
}

pub fn resolve_dimension_as_record<'ctx>(
    env: &GlobalInformation<'ctx>,
    codegen: &CodeGen<'ctx>,
    scope: &Scope<'ctx>,
    expr: &Expression,
) -> IntValue<'ctx> {
    let res = resolve_expr_static(env, &scope.info, &scope.instantiation, expr);
    match res {
        ConcreteValue::Int(i) => env.val_ty.const_int(i as u64, true),
        _ => resolve_expr(env, codegen, scope, expr).into_int_value(),
    }
}
