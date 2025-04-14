use std::iter::zip;

use crate::environment::GlobalInformation;
use crate::expression_static::{resolve_inline_array_static, resolve_uniform_array_static};
use crate::namer::{name_opaque_struct, name_template_struct};
use crate::scope_information::ScopeInformation;
use crate::type_check::{unwrap_used_type, wrap_type2used};
use inkwell::types::{ArrayType, BasicType, BasicTypeEnum};
use program_structure::ast::{Access, Expression, Statement, VariableType};

pub fn infer_arg_ty_from_expr<'ctx>(
    env: &GlobalInformation<'ctx>,
    scope_info: &mut ScopeInformation<'ctx>,
    expr: &Expression,
    given_ty: BasicTypeEnum<'ctx>,
) {
    match expr {
        Expression::Call { meta: _, id, args } => {
            // See long_gt as the motivating example.
            if id == scope_info.get_name() {
                // Ignore self-called function.
                return;
            }
            let target_scope_info = env.get_scope_info(id);
            let target_arg_tys = target_scope_info.get_arg_tys();
            assert!(args.len() == target_arg_tys.len());
            for (arg, ty) in zip(args, target_arg_tys) {
                let mut arg_ty = ty.clone();
                arg_ty = unwrap_used_type(&arg_ty);
                infer_arg_ty_from_expr(env, scope_info, arg, arg_ty);
            }
        }
        Expression::InfixOp { lhe, rhe, .. } => {
            infer_arg_ty_from_expr(env, scope_info, &lhe, env.val_ty.as_basic_type_enum());
            infer_arg_ty_from_expr(env, scope_info, &rhe, env.val_ty.as_basic_type_enum());
        }
        Expression::InlineSwitchOp {
            meta: _,
            cond,
            if_true,
            if_false,
        } => {
            infer_arg_ty_from_expr(env, scope_info, &cond, env.val_ty.as_basic_type_enum());
            infer_arg_ty_from_expr(env, scope_info, &if_true, env.val_ty.as_basic_type_enum());
            infer_arg_ty_from_expr(env, scope_info, &if_false, env.val_ty.as_basic_type_enum());
        }
        Expression::PrefixOp { rhe, .. } => {
            infer_arg_ty_from_expr(env, scope_info, &rhe, env.val_ty.as_basic_type_enum());
        }
        Expression::Variable {
            meta: _,
            name,
            access,
        } => {
            if scope_info.is_arg(name) {
                let mut current_ty = given_ty;
                for a in access.iter().rev() {
                    match a {
                        Access::ArrayAccess(_) => {
                            current_ty =
                                construct_array_ty(env, current_ty, 1).as_basic_type_enum();
                        }
                        Access::ComponentAccess(_) => unreachable!(),
                    }
                }
                scope_info.set_var_ty(name, current_ty);
            }
        }
        _ => (),
    }
}

pub fn infer_ty_from_stmt<'ctx>(
    env: &GlobalInformation<'ctx>,
    scope_info: &mut ScopeInformation<'ctx>,
    stmt: &Statement,
) {
    match stmt {
        Statement::Declaration {
            meta: _,
            xtype,
            name,
            dimensions,
            is_constant: _,
        } => {
            let mut val_ty: BasicTypeEnum;
            match xtype {
                VariableType::AnonymousComponent => {
                    println!("Error: AnonymousComponent isn't supported now.");
                    unreachable!()
                }
                VariableType::Var => {
                    val_ty = env.val_ty.as_basic_type_enum();
                }
                VariableType::Component => {
                    let templ_name = scope_info.get_component_name(name);
                    if templ_name == scope_info.get_name() {
                        let strt_name = name_template_struct(templ_name);
                        let strt_ty = env
                            .context
                            .opaque_struct_type(&name_opaque_struct(&strt_name));
                        val_ty = wrap_type2used(&strt_ty.as_basic_type_enum());
                    } else {
                        val_ty = env.get_scope_ret_ty(templ_name);
                    }
                }
                VariableType::Signal(..) => {
                    val_ty = env.val_ty.as_basic_type_enum();
                }
            }
            let dims = dimensions.len();
            if dims == 0 {
                scope_info.set_var_ty(name, val_ty);
            } else {
                val_ty = construct_array_ty(env, val_ty, dims).as_basic_type_enum();
                scope_info.set_var_ty(name, val_ty);
            }
        }
        Statement::Substitution {
            meta,
            var,
            access,
            op: _,
            rhe,
        } => {
            let expr = Expression::Variable {
                meta: meta.clone(),
                name: var.clone(),
                access: access.clone(),
            };
            let ty = get_type_of_expr(env, scope_info, &expr).unwrap();
            infer_arg_ty_from_expr(env, scope_info, &rhe, ty);
        }
        Statement::IfThenElse { cond, .. } => {
            infer_arg_ty_from_expr(env, scope_info, &cond, env.val_ty.as_basic_type_enum());
        }
        Statement::Return { meta: _, value } => {
            infer_arg_ty_from_expr(env, scope_info, value, env.val_ty.as_basic_type_enum());
        }
        Statement::While { cond, .. } => {
            infer_arg_ty_from_expr(env, scope_info, &cond, env.val_ty.as_basic_type_enum());
        }
        _ => (),
    }
}

pub fn get_type_of_expr<'ctx>(
    env: &GlobalInformation<'ctx>,
    scope_info: &ScopeInformation<'ctx>,
    expr: &Expression,
) -> Option<BasicTypeEnum<'ctx>> {
    let res = match expr {
        Expression::AnonymousComp { .. } => {
            println!("Error: AnonymousComp isn't supported now.");
            unreachable!();
        }
        Expression::ArrayInLine { .. } => {
            let (arr_ty, _) = resolve_inline_array_static(env, expr);
            Some(arr_ty.as_basic_type_enum())
        }
        Expression::Call { id, .. } => Some(env.get_scope_ret_ty(id).clone()),
        Expression::InfixOp { lhe, rhe, .. } => {
            let lty = get_type_of_expr(env, scope_info, lhe);
            let rty = get_type_of_expr(env, scope_info, rhe);
            merge_type(lty, rty)
        }
        Expression::InlineSwitchOp {
            if_true, if_false, ..
        } => {
            let lty = get_type_of_expr(env, scope_info, if_true.as_ref());
            let rty = get_type_of_expr(env, scope_info, if_false.as_ref());
            merge_type(lty, rty)
        }
        Expression::Number(..) => Some(env.val_ty.as_basic_type_enum()),
        Expression::ParallelOp { rhe, .. } => get_type_of_expr(env, scope_info, rhe.as_ref()),
        Expression::PrefixOp { rhe, .. } => get_type_of_expr(env, scope_info, rhe.as_ref()),
        Expression::Tuple { .. } => {
            println!("Error: Tuple isn't supported now.");
            unreachable!();
        }
        Expression::UniformArray { .. } => {
            let arr = resolve_uniform_array_static(env, scope_info, expr);
            let arr_ty = arr.get_type();
            Some(arr_ty.as_basic_type_enum())
        }
        Expression::Variable { name, access, .. } => {
            let mut var_ty = scope_info.get_var_ty(name);
            for a in access {
                match a {
                    Access::ArrayAccess(..) => {
                        var_ty = var_ty.into_array_type().get_element_type();
                    }
                    Access::ComponentAccess(signal) => {
                        let templ_name = scope_info.get_component_name(name);
                        let target_scope_info = if templ_name == scope_info.get_name() {
                            scope_info
                        } else {
                            env.get_scope_info(templ_name)
                        };
                        var_ty = target_scope_info.get_var_ty(signal)
                    }
                }
            }
            Some(var_ty)
        }
    };
    match res {
        Some(r) => Some(wrap_type2used(&r)),
        None => None,
    }
}

pub fn construct_array_ty<'ctx>(
    env: &GlobalInformation<'ctx>,
    val_ty: BasicTypeEnum<'ctx>,
    dims: usize,
) -> ArrayType<'ctx> {
    assert!(dims > 0);
    let mut arr_ty = val_ty.array_type(env.arraysize);
    for _ in 1..dims {
        arr_ty = arr_ty.array_type(env.arraysize);
    }
    arr_ty
}

fn merge_type<'ctx>(
    a: Option<BasicTypeEnum<'ctx>>,
    b: Option<BasicTypeEnum<'ctx>>,
) -> Option<BasicTypeEnum<'ctx>> {
    match a {
        Some(..) => {
            return a;
        }
        None => match b {
            Some(..) => {
                return b;
            }
            None => {
                return None;
            }
        },
    }
}
