use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, IntType};
use program_structure::ast::Statement;
use std::{collections::HashMap, iter::zip};

use crate::environment::GlobalInformation;
use crate::namer::ValueTypeEnum;

use crate::{
    expression_codegen::flat_expressions_from_statement,
    expression_static::{ArgTable, ArgValues},
    info_collector::{collect_depended_components, collect_dependences},
    namer::name_template_fn,
    statement::flat_statements,
    type_check::wrap_type2used,
};

#[derive(Clone)]
pub struct ScopeInformation<'ctx> {
    pub name: String,
    pub args: Vec<String>,
    pub is_template: bool,
    pub default_val_ty: IntType<'ctx>,

    arg_tys: Vec<BasicTypeEnum<'ctx>>,
    dependences: Vec<String>,
    ret_ty: Option<BasicTypeEnum<'ctx>>,
    var2ty: HashMap<String, BasicTypeEnum<'ctx>>,
    var2comp: HashMap<String, String>,
}

impl<'ctx> ScopeInformation<'ctx> {
    pub fn get_name(&self) -> &String {
        return &self.name;
    }

    pub fn get_component_name(&self, var_name: &String) -> &String {
        if var_name == &self.name {
            return &self.name;
        }
        match self.var2comp.get(var_name) {
            None => {
                println!("This variable: {} is NOT a component", var_name);
                unreachable!();
            }
            Some(comp_name) => comp_name,
        }
    }

    pub fn set_component_var(&mut self, var_name: &String, comp_name: &String) {
        if !self.var2comp.contains_key(var_name) && comp_name == "" {
            self.var2comp
                .insert(var_name.clone(), "unknown".to_string());
        } else {
            self.var2comp.insert(var_name.clone(), comp_name.clone());
        }
    }

    pub fn is_component_var(&self, var_name: &String) -> bool {
        return self.var2comp.contains_key(var_name);
    }

    pub fn is_component(&self, comp_or_fn_name: &String) -> bool {
        for (_, comp) in &self.var2comp {
            if comp_or_fn_name.starts_with(comp) {
                return true;
            }
        }
        return false;
    }

    pub fn get_dependences(&self) -> &Vec<String> {
        return &self.dependences;
    }

    pub fn set_dependence(&mut self, name: &String) {
        if !self.dependences.contains(name) {
            self.dependences.push(name.clone());
        }
    }

    pub fn get_var_ty(&self, name: &String) -> BasicTypeEnum<'ctx> {
        let res = self.var2ty.get(name);
        match res {
            Some(ty) => *ty,
            None => self.default_val_ty.as_basic_type_enum(),
        }
    }

    pub fn set_var_ty(&mut self, name: &String, ty: BasicTypeEnum<'ctx>) {
        if !self.var2ty.contains_key(name) {
            self.var2ty.insert(name.clone(), ty);
        } else {
            let current_ty = self.var2ty.get(name).unwrap();
            if current_ty != &ty {
                println!("Error: Different type is set to var: {}!", name);
                println!("Current type is: {}", current_ty.print_to_string());
                println!("New type is: {}", ty.print_to_string());
                unreachable!();
            }
        }
    }

    pub fn has_var_ty(&self, name: &String) -> bool {
        return self.var2ty.contains_key(name);
    }

    pub fn get_var_used_ty(&self, name: &String) -> BasicTypeEnum<'ctx> {
        let ty = self.get_var_ty(name);
        wrap_type2used(&ty)
    }

    pub fn gen_arg2val(&self, instantiation: &ArgValues) -> ArgTable {
        let mut arg2val = HashMap::new();
        for (arg_name, arg_val) in zip(&self.args, instantiation) {
            if arg_val.is_unknown() {
                println!("Error: Argument {} is unknown", arg_name);
                unreachable!();
            } else {
                arg2val.insert(arg_name.clone(), arg_val.clone());
            }
        }
        arg2val
    }

    pub fn gen_signature(&self, i: &ArgTable) -> String {
        let mut s = self.name.clone();
        if i.len() > 0 {
            for a in &self.args {
                s += "@";
                s += a;
                s += "=";
                s += &i[a].to_string();
            }
        }
        s
    }

    pub fn is_arg(&self, arg_name: &String) -> bool {
        for a in &self.args {
            if a == arg_name {
                return true;
            }
        }
        return false;
    }

    pub fn canonicalize_fn_name(&self, fn_name: &String) -> String {
        let res = if self.is_component(fn_name) {
            name_template_fn(fn_name, "build")
        } else {
            fn_name.clone()
        };
        res
    }

    pub fn get_ret_ty(&self) -> BasicTypeEnum<'ctx> {
        self.ret_ty.unwrap()
    }

    pub fn set_ret_ty(&mut self, ty: BasicTypeEnum<'ctx>) {
        self.ret_ty = Some(ty);
    }

    pub fn gen_arg_metadata_tys(&self) -> Vec<BasicMetadataTypeEnum<'ctx>> {
        let mut arg_tys: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::new();
        for a in &self.arg_tys {
            arg_tys.push(a.clone().into());
        }
        arg_tys
    }

    pub fn get_var2ty(&self) -> Vec<(String, BasicTypeEnum<'ctx>)> {
        let mut res = Vec::new();
        for (name, ty) in &self.var2ty {
            res.push((name.clone(), ty.clone()))
        }
        res
    }

    pub fn get_arg_tys(&self) -> &Vec<BasicTypeEnum<'ctx>> {
        &self.arg_tys
    }

    pub fn set_arg_tys(&mut self, arg_tys: Vec<BasicTypeEnum<'ctx>>) {
        self.arg_tys = arg_tys;
    }

    pub fn resolve_dependences(&mut self, body: &Statement) {
        let stmts = flat_statements(body);
        let mut exprs = Vec::new();
        for stmt in stmts {
            collect_depended_components(stmt, self);
            exprs.append(&mut flat_expressions_from_statement(stmt));
        }
        for expr in exprs {
            collect_dependences(expr, self);
        }
    }

    pub fn check(&self) {
        assert!(self.args.len() == self.arg_tys.len());
        assert!(self.ret_ty.is_some());
    }

    pub fn get_var_ty_enum(&self, env: &GlobalInformation, var_name: &String) -> ValueTypeEnum {
        if self.is_arg(var_name) {
            return ValueTypeEnum::Argument;
        }
        if self.is_template {
            if self.is_component_var(var_name) {
                return ValueTypeEnum::Component;
            };
            let templ_info = env.get_template_info(self.get_name());
            let res = templ_info.get_signal_info(var_name);
            match res {
                Some((_, val_ty_enum)) => val_ty_enum,
                None => ValueTypeEnum::Variable,
            }
        } else {
            ValueTypeEnum::Variable
        }
    }

    pub fn is_var(&self, env: &GlobalInformation, var_name: &String) -> bool {
        return self.get_var_ty_enum(env, var_name) == ValueTypeEnum::Variable && !self.is_component_var(var_name);
    }
}

pub fn init_scope_info<'ctx>(
    is_template: bool,
    val_ty: IntType<'ctx>,
    name: String,
    args: Vec<String>,
) -> ScopeInformation<'ctx> {
    return ScopeInformation {
        is_template,
        name,
        args,

        arg_tys: Vec::new(),
        default_val_ty: val_ty,
        dependences: Vec::new(),
        ret_ty: None,
        var2ty: HashMap::new(),
        var2comp: HashMap::new(),
    };
}
