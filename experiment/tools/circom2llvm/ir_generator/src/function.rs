use crate::codegen::CodeGen;
use crate::environment::GlobalInformation;
use crate::namer::{name_body_block, name_entry_block};
use crate::scope::Scope;
use crate::scope_information::ScopeInformation;
use crate::statement::{flat_statements, resolve_stmt};
use crate::type_infer::{get_type_of_expr, infer_ty_from_stmt};
use inkwell::types::BasicType;
use program_structure::ast::Statement;

pub struct Function<'ctx> {
    pub scope: Scope<'ctx>,
}

pub fn infer_fn<'ctx>(
    env: &GlobalInformation<'ctx>,
    scope_info: &mut ScopeInformation<'ctx>,
    body: &Statement,
) {
    let mut ret_ty = env.val_ty.as_basic_type_enum();
    let stmts = flat_statements(body);
    for stmt in &stmts {
        infer_ty_from_stmt(env, scope_info, stmt);
    }
    for stmt in &stmts {
        match stmt {
            Statement::Return { meta: _, value } => {
                let ty = get_type_of_expr(env, &scope_info, value);
                match ty {
                    Some(ty) => ret_ty = ty,
                    None => (),
                }
            }
            _ => (),
        }
    }
    scope_info.set_ret_ty(ret_ty);
    let mut arg_tys = Vec::new();
    for a in &scope_info.args {
        arg_tys.push(scope_info.get_var_used_ty(a));
    }
    scope_info.set_arg_tys(arg_tys);
    scope_info.check();
}

impl<'ctx> Function<'ctx> {
    pub fn build(
        &mut self,
        env: &GlobalInformation<'ctx>,
        codegen: &CodeGen<'ctx>,
        body: &Statement,
    ) {
        let fn_name = self.scope.get_name().clone();
        let ret_ty = self.scope.info.get_ret_ty();
        let fn_ty = ret_ty.fn_type(&self.scope.info.gen_arg_metadata_tys(), false);
        let fn_val = codegen.module.add_function(&fn_name, fn_ty, None);
        self.scope.set_main_fn(fn_val);

        let entry_bb = codegen
            .context
            .append_basic_block(fn_val, &name_entry_block());
        self.scope.set_current_exit_block(codegen, entry_bb);

        // Bind args
        for (idx, arg) in self.scope.info.args.clone().iter().enumerate() {
            let val = fn_val.get_nth_param(idx as u32).unwrap();
            self.scope.set_arg_val(arg, &val);
        }

        // Initial variable
        let var_table = self.scope.info.get_var2ty();
        for (name, ty) in &var_table {
            if self.scope.info.is_arg(&name) {
                continue;
            }
            let alloca_name = name;
            self.scope.initial_var(codegen, name, alloca_name, ty, true);
        }

        let body_bb = codegen
            .context
            .append_basic_block(fn_val, &name_body_block());
        codegen.build_block_transferring(entry_bb, body_bb);
        self.scope.set_current_exit_block(codegen, body_bb);

        match body {
            Statement::Block { meta: _, stmts } => {
                for stmt in stmts {
                    if stmt.is_return() {
                        self.scope.build_exit(codegen);
                    }
                    resolve_stmt(env, codegen, &mut self.scope, stmt);
                }
            }
            _ => unreachable!(),
        }
    }
}
