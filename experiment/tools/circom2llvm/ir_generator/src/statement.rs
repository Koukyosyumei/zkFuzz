use std::collections::HashSet;

use crate::codegen::CodeGen;
use crate::environment::GlobalInformation;
use crate::expression_codegen::{resolve_dimension_as_record, resolve_expr};
use crate::expression_static::{
    resolve_expr_static, ArgTable, ArgValues, ConcreteValue, SymbolValueManager,
};
use crate::namer::{name_if_block, name_loop_block};
use crate::scope::Scope;
use crate::scope_information::ScopeInformation;
use crate::utils::is_terminated_basicblock;
use inkwell::values::BasicValue;
use inkwell::IntPredicate;
use num_bigint_dig::BigInt;
use num_traits::FromPrimitive;
use program_structure::ast::{Access, AssignOp, Expression, Statement};

pub fn resolve_stmt<'ctx>(
    env: &GlobalInformation<'ctx>,
    codegen: &CodeGen<'ctx>,
    scope: &mut Scope<'ctx>,
    stmt: &Statement,
) {
    match stmt {
        Statement::Assert { meta: _, arg } => {
            let val = resolve_expr(env, codegen, scope, arg);
            if val.get_type().into_int_type() == codegen.context.bool_type() {
                codegen.build_assert(val.into_int_value());
            }
        }
        Statement::Block { meta: _, stmts } => {
            for stmt in stmts {
                resolve_stmt(env, codegen, scope, stmt);
            }
        }
        Statement::ConstraintEquality { meta: _, lhe, rhe } => {
            let lval = resolve_expr(env, codegen, scope, lhe);
            let rval = resolve_expr(env, codegen, scope, rhe);
            codegen.build_constraint(lval, rval);
        }
        Statement::Declaration { .. } => (),
        Statement::IfThenElse {
            meta: _,
            cond,
            if_case,
            else_case,
        } => {
            let current_fnc = scope.get_main_fn();
            let if_bb = codegen
                .context
                .append_basic_block(current_fnc, &name_if_block(true, false));
            let else_bb = codegen
                .context
                .append_basic_block(current_fnc, &name_if_block(false, false));

            // current -> if.true
            let mut cond = resolve_expr(env, codegen, scope, cond).into_int_value();
            if cond.get_type() != codegen.context.bool_type() {
                cond = codegen
                    .builder
                    .build_int_compare(IntPredicate::EQ, cond, env.const_zero, "eq")
                    .expect("REASON");
            }
            codegen
                .builder
                .build_conditional_branch(cond, if_bb, else_bb);

            // if.true
            scope.set_current_exit_block(codegen, if_bb);

            resolve_stmt(env, codegen, scope, &if_case.as_ref());
            let if_end_bb = scope.get_current_exit_block();

            // if.false
            scope.set_current_exit_block(codegen, else_bb);
            match else_case {
                Some(else_stmt) => {
                    resolve_stmt(env, codegen, scope, &else_stmt.as_ref());
                }
                _ => (),
            }
            let else_end_bb = scope.get_current_exit_block();

            if is_terminated_basicblock(&if_end_bb) && is_terminated_basicblock(&else_end_bb) {
                return;
            };

            let exit_bb = codegen
                .context
                .append_basic_block(current_fnc, &name_if_block(false, true));
            // if.true -> if.exit
            codegen.build_block_transferring(if_end_bb, exit_bb);
            // if.false -> if.exit
            codegen.build_block_transferring(else_end_bb, exit_bb);

            scope.set_current_exit_block(codegen, exit_bb);
        }
        Statement::InitializationBlock {
            meta: _,
            xtype: _,
            initializations,
        } => {
            for init in initializations {
                match init {
                    Statement::Substitution {
                        meta: _,
                        var,
                        access,
                        op,
                        rhe,
                    } => match op {
                        AssignOp::AssignVar => {
                            let rval = resolve_expr(env, codegen, scope, rhe);
                            scope.set_var(env, codegen, var, access, rval.as_basic_value_enum());
                        }
                        _ => unreachable!(),
                    },
                    Statement::Declaration {
                        name, dimensions, ..
                    } => {
                        let dims = dimensions
                            .iter()
                            .map(|d| resolve_dimension_as_record(env, codegen, scope, d))
                            .collect();
                        scope.set_var_dims(name, dims);
                    }
                    _ => unreachable!(),
                }
            }
        }
        Statement::LogCall { .. } => (),
        Statement::MultSubstitution { .. } => {
            println!("Error: MultSubstitution isn't supported now.");
            unreachable!();
        }
        Statement::Return { meta: _, value } => {
            let rval = resolve_expr(env, codegen, scope, value).as_basic_value_enum();
            codegen.builder.build_return(Some(&rval));
        }
        Statement::Substitution {
            meta: _,
            var,
            access,
            op,
            rhe,
        } => {
            let rval = resolve_expr(env, codegen, scope, rhe);
            match op {
                AssignOp::AssignConstraintSignal => {
                    scope.set_var(env, codegen, var, access, rval);
                    let lval = scope.get_var(env, codegen, var, access);
                    codegen.build_constraint(lval, rval);
                }
                AssignOp::AssignSignal => {
                    scope.set_var(env, codegen, var, access, rval);
                }
                AssignOp::AssignVar => {
                    scope.set_var(env, codegen, var, access, rval);
                }
                _ => (),
            };
        }
        Statement::While {
            meta: _,
            cond,
            stmt,
        } => {
            let current_func = scope.get_main_fn();

            // Get the body of while and the latch step of while.
            let stmts = match stmt.as_ref() {
                Statement::Block { meta: _, stmts } => stmts,
                _ => unreachable!(),
            };

            let current_bb = scope.get_current_exit_block();

            // current -> loop.header
            let header_bb_name = name_loop_block(true, false, false);
            let header_bb = codegen
                .context
                .append_basic_block(current_func, &header_bb_name);
            codegen.build_block_transferring(current_bb, header_bb);
            scope.set_current_exit_block(codegen, header_bb);
            let cond_var = resolve_expr(env, codegen, scope, cond).into_int_value();

            // loop.header -> loop.body
            let body_bb_name = name_loop_block(false, true, false);
            let body_bb = codegen
                .context
                .append_basic_block(current_func, &body_bb_name);
            scope.set_current_exit_block(codegen, body_bb);
            for stmt in stmts {
                resolve_stmt(env, codegen, scope, stmt);
            }
            let current_bb = scope.get_current_exit_block();
            codegen.build_block_transferring(current_bb, header_bb);

            // loop.header -> loop.exit
            let exit_bb_name = name_loop_block(false, false, true);
            let exit_bb = codegen
                .context
                .append_basic_block(current_func, &exit_bb_name);

            codegen.builder.position_at_end(header_bb);
            codegen
                .builder
                .build_conditional_branch(cond_var, body_bb, exit_bb);

            // loop.exit
            scope.set_current_exit_block(codegen, exit_bb);
        }
    }
}

pub fn flat_statements(stmt: &Statement) -> Vec<&Statement> {
    let mut all_stmts: Vec<&Statement> = vec![stmt];
    match stmt {
        Statement::Block { meta: _, stmts } => {
            for _stmt in stmts {
                all_stmts.append(&mut flat_statements(_stmt));
            }
        }
        Statement::IfThenElse {
            if_case, else_case, ..
        } => {
            all_stmts.append(&mut flat_statements(if_case.as_ref()));
            match else_case.as_ref() {
                Some(_stmt) => {
                    all_stmts.append(&mut flat_statements(_stmt.as_ref()));
                }
                None => (),
            }
        }
        Statement::InitializationBlock {
            initializations, ..
        } => {
            for init in initializations {
                all_stmts.append(&mut flat_statements(init));
            }
        }
        Statement::While { stmt, .. } => {
            all_stmts.append(&mut flat_statements(stmt.as_ref()));
        }
        _ => (),
    }
    return all_stmts;
}

pub fn print_stmt(stmt: &Statement) -> &'static str {
    match stmt {
        Statement::Assert { .. } => "Assert",
        Statement::Block { .. } => "Block",
        Statement::ConstraintEquality { .. } => "ConstraintEquality",
        Statement::Declaration { .. } => "Declaration",
        Statement::IfThenElse { .. } => "IfThenElse",
        Statement::InitializationBlock { .. } => "InitializationBlock",
        Statement::LogCall { .. } => "LogCall",
        Statement::MultSubstitution { .. } => "MultSubstitution",
        Statement::Return { .. } => "Return",
        Statement::Substitution { .. } => "Substitution",
        Statement::While { .. } => "While",
    }
}

fn instant_access<'ctx>(
    env: &GlobalInformation<'ctx>,
    scope_info: &ScopeInformation<'ctx>,
    arg2val: &ArgTable,
    access: &Vec<Access>,
) -> Vec<Access> {
    let new_access = access
        .iter()
        .map(|a| match a {
            Access::ArrayAccess(expr) => {
                let a_expr =
                    instant_expr(env, scope_info, arg2val, &SymbolValueManager::init(), expr);
                if !a_expr.is_number() {
                    println!("Error: Non-static access!");
                    unreachable!();
                }
                Access::ArrayAccess(a_expr)
            }
            Access::ComponentAccess(..) => a.clone(),
        })
        .collect();
    new_access
}

fn instant_expr<'ctx>(
    env: &GlobalInformation<'ctx>,
    scope_info: &ScopeInformation<'ctx>,
    arg2val: &ArgTable,
    var2val: &SymbolValueManager,
    expr: &Expression,
) -> Expression {
    let res = resolve_expr_static(env, scope_info, arg2val, expr);
    if res.is_int() {
        return Expression::Number(
            expr.get_meta().clone(),
            BigInt::from_i128(res.as_int()).unwrap(),
        );
    }
    match expr {
        Expression::Call { meta, id, args } => {
            if scope_info.is_component(id) {
                let instantiation = instant_subcomp(env, scope_info, &arg2val, args);
                let target_scope_info = env.get_scope_info(id);
                let arg2val = target_scope_info.gen_arg2val(&instantiation);
                let target_signature = target_scope_info.gen_signature(&arg2val);
                let new_expr = Expression::Call {
                    meta: meta.clone(),
                    id: target_signature,
                    args: args.clone(),
                };
                new_expr
            } else {
                let args = args
                    .iter()
                    .map(|a| instant_expr(env, scope_info, arg2val, var2val, a))
                    .collect();
                Expression::Call {
                    meta: meta.clone(),
                    id: id.clone(),
                    args,
                }
            }
        }
        Expression::InfixOp {
            meta,
            lhe,
            infix_op,
            rhe,
        } => {
            let lhe = Box::new(instant_expr(
                env,
                scope_info,
                arg2val,
                var2val,
                lhe.as_ref(),
            ));
            let rhe = Box::new(instant_expr(
                env,
                scope_info,
                arg2val,
                var2val,
                rhe.as_ref(),
            ));
            Expression::InfixOp {
                meta: meta.clone(),
                lhe,
                infix_op: infix_op.clone(),
                rhe,
            }
        }
        Expression::InlineSwitchOp {
            meta,
            cond,
            if_true,
            if_false,
        } => {
            let cond = instant_expr(env, scope_info, arg2val, var2val, cond);
            let if_true = instant_expr(env, scope_info, arg2val, var2val, if_true.as_ref());
            let if_false = instant_expr(env, scope_info, arg2val, var2val, if_false.as_ref());
            let res = resolve_expr_static(env, scope_info, &arg2val, &cond);
            match res {
                ConcreteValue::Int(i) => {
                    if i == 1 {
                        if_true
                    } else {
                        if_false
                    }
                }
                ConcreteValue::Array(..) => unreachable!(),
                ConcreteValue::Unknown => Expression::InlineSwitchOp {
                    meta: meta.clone(),
                    cond: Box::new(cond),
                    if_true: Box::new(if_true),
                    if_false: Box::new(if_false),
                },
            }
        }
        Expression::PrefixOp {
            meta,
            prefix_op,
            rhe,
        } => {
            let rhe = Box::new(instant_expr(
                env,
                scope_info,
                arg2val,
                var2val,
                rhe.as_ref(),
            ));
            Expression::PrefixOp {
                meta: meta.clone(),
                prefix_op: prefix_op.clone(),
                rhe,
            }
        }
        Expression::UniformArray {
            meta,
            value,
            dimension,
        } => {
            let value = Box::new(instant_expr(
                env,
                scope_info,
                arg2val,
                var2val,
                value.as_ref(),
            ));
            let dimension = Box::new(instant_expr(
                env,
                scope_info,
                arg2val,
                var2val,
                dimension.as_ref(),
            ));
            Expression::UniformArray {
                meta: meta.clone(),
                value,
                dimension,
            }
        }
        Expression::Variable { meta, name, access } => {
            let access = instant_access(env, scope_info, arg2val, access);
            if scope_info.is_var(env, name) {
                let access_idxes: Vec<usize> = access
                    .iter()
                    .rev()
                    .map(|a| match a {
                        Access::ArrayAccess(a) => {
                            resolve_expr_static(env, scope_info, arg2val, a).as_int() as usize
                        }
                        Access::ComponentAccess(..) => unreachable!(),
                    })
                    .collect();
                let expr = var2val.get_expr(name, &access_idxes);
                match expr {
                    Some(expr) => {
                        return expr.clone();
                    }
                    None => (),
                }
            }
            Expression::Variable {
                meta: meta.clone(),
                name: name.clone(),
                access,
            }
        }
        _ => expr.clone(),
    }
}

pub fn instant_subcomp<'ctx>(
    env: &GlobalInformation<'ctx>,
    scope_info: &ScopeInformation<'ctx>,
    arg2val: &ArgTable,
    args: &Vec<Expression>,
) -> ArgValues {
    let instantiation: ArgValues = args
        .iter()
        .map(|a| {
            let v = resolve_expr_static(env, scope_info, &arg2val, a);
            v
        })
        .collect();
    instantiation
}

pub fn instant_stmt<'ctx>(
    env: &GlobalInformation<'ctx>,
    scope_info: &ScopeInformation<'ctx>,
    arg2val: &mut ArgTable,
    var2val: &mut SymbolValueManager,
    n: &mut HashSet<(String, Vec<ConcreteValue>)>,
    stmt: &Statement,
) -> Statement {
    use Statement::*;
    match stmt {
        Block { meta, stmts } => {
            let stmts = stmts
                .iter()
                .map(|s| instant_stmt(env, scope_info, arg2val, var2val, n, s))
                .collect();
            Block {
                meta: meta.clone(),
                stmts,
            }
        }
        IfThenElse {
            meta,
            cond,
            if_case,
            else_case,
        } => {
            let res = resolve_expr_static(env, scope_info, &arg2val, cond);
            match res {
                ConcreteValue::Int(i) => {
                    if i == 1 {
                        instant_stmt(env, scope_info, arg2val, var2val, n, if_case.as_ref())
                    } else {
                        match else_case {
                            Some(else_case) => instant_stmt(
                                env,
                                scope_info,
                                arg2val,
                                var2val,
                                n,
                                else_case.as_ref(),
                            ),
                            // Empty Statement
                            None => Block {
                                meta: meta.clone(),
                                stmts: Vec::new(),
                            },
                        }
                    }
                }
                ConcreteValue::Array(..) => unreachable!(),
                ConcreteValue::Unknown => stmt.clone(),
            }
        }
        InitializationBlock {
            meta,
            xtype,
            initializations,
        } => {
            let initializations = initializations
                .iter()
                .map(|i| instant_stmt(env, scope_info, arg2val, var2val, n, i))
                .collect();
            InitializationBlock {
                meta: meta.clone(),
                xtype: xtype.clone(),
                initializations,
            }
        }
        Substitution {
            meta,
            var,
            access,
            op,
            rhe,
        } => {
            match op {
                AssignOp::AssignVar => match rhe {
                    Expression::Call { meta: _, id, args } => {
                        if scope_info.is_component(id) {
                            let instantiation = instant_subcomp(env, scope_info, &arg2val, args);
                            let p = (id.clone(), instantiation);
                            n.insert(p);
                        }
                    }
                    _ => (),
                },
                AssignOp::AssignSignal => (),
                AssignOp::AssignConstraintSignal => (),
            }
            let new_access = instant_access(env, scope_info, arg2val, access);
            let new_rhe = instant_expr(env, scope_info, arg2val, var2val, rhe);
            let res = resolve_expr_static(env, scope_info, arg2val, rhe);
            if access.len() == 0 {
                if !res.is_unknown() {
                    arg2val.insert(var.clone(), res);
                } else {
                    arg2val.remove(var);
                }
            }
            if scope_info.is_var(env, var) {
                let access_idxes: Vec<usize> = access
                    .iter()
                    .rev()
                    .map(|a| match a {
                        Access::ArrayAccess(a) => {
                            resolve_expr_static(env, scope_info, arg2val, a).as_int() as usize
                        }
                        Access::ComponentAccess(..) => unreachable!(),
                    })
                    .collect();
                var2val.set_expr(var, &access_idxes, &new_rhe);
            }
            Substitution {
                meta: meta.clone(),
                var: var.clone(),
                access: new_access,
                op: op.clone(),
                rhe: new_rhe,
            }
        }
        While {
            meta,
            cond,
            stmt: i_stmt,
        } => {
            let res = resolve_expr_static(env, scope_info, arg2val, cond);
            match res {
                ConcreteValue::Int(i) => {
                    let mut initial_cond = i;
                    let mut stmts = vec![];
                    while initial_cond != 0 {
                        stmts.push(instant_stmt(env, scope_info, arg2val, var2val, n, i_stmt));
                        initial_cond = resolve_expr_static(env, scope_info, arg2val, cond).as_int();
                    }
                    Block {
                        meta: meta.clone(),
                        stmts,
                    }
                }
                ConcreteValue::Array(..) => unreachable!(),
                ConcreteValue::Unknown => stmt.clone(),
            }
        }
        ConstraintEquality { meta, lhe, rhe } => {
            let lhe = instant_expr(env, scope_info, arg2val, var2val, lhe);
            let rhe = instant_expr(env, scope_info, arg2val, var2val, rhe);
            ConstraintEquality {
                meta: meta.clone(),
                lhe,
                rhe,
            }
        }
        Return { meta, value } => {
            let value = instant_expr(env, scope_info, arg2val, var2val, value);
            Return {
                meta: meta.clone(),
                value,
            }
        }
        Declaration {
            meta,
            xtype,
            name,
            dimensions,
            is_constant,
        } => {
            let dimensions: Vec<Expression> = dimensions
                .iter()
                .map(|d| instant_expr(env, scope_info, arg2val, var2val, d))
                .collect();
            if scope_info.is_var(env, name) {
                let dimensions: Vec<usize> = dimensions
                    .iter()
                    .rev()
                    .map(|d| resolve_expr_static(env, scope_info, arg2val, d).as_int() as usize)
                    .collect();
                var2val.add_dims(name, dimensions);
            }
            Declaration {
                meta: meta.clone(),
                xtype: xtype.clone(),
                name: name.clone(),
                dimensions,
                is_constant: is_constant.clone(),
            }
        }
        _ => stmt.clone(),
    }
}
