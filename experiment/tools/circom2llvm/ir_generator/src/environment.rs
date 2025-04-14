use std::collections::HashMap;

use inkwell::{
    context::Context,
    types::{BasicTypeEnum, IntType},
    values::IntValue,
};

use crate::{
    expression_static::{Instantiation, ArgTable}, scope_information::ScopeInformation,
    template::TemplateInformation,
};

pub struct InstantiationManager {
    templ_name2arg2val: HashMap<String, Vec<ArgTable>>,
    instantiations: HashMap<String, Vec<Instantiation>>,
}

impl InstantiationManager {

    pub fn has_arg2val(&self, templ_name: &String) -> bool {
        self.templ_name2arg2val.contains_key(templ_name)
    }
    
    pub fn get_arg2val(&self, templ_name: &String) -> &Vec<ArgTable> {
        &self.templ_name2arg2val[templ_name]
    }

    pub fn set_arg2val(&mut self, templ_name: &String, v: ArgTable) {
        if self.templ_name2arg2val.contains_key(templ_name) {
            let c = self.templ_name2arg2val.get_mut(templ_name).unwrap();
            c.push(v);
        } else {
            self.templ_name2arg2val.insert(templ_name.clone(), vec![v]);
        }
    }

    pub fn get_instantiations(&self, templ_name: &String) -> &Vec<Instantiation> {
        &self.instantiations[templ_name]
    }

    pub fn set_instantiations(&mut self, templ_name: &String, v: Instantiation) {
        if self.instantiations.contains_key(templ_name) {
            let c = self.instantiations.get_mut(templ_name).unwrap();
            c.push(v);
        } else {
            self.instantiations.insert(templ_name.clone(), vec![v]);
        }
    }
}

pub fn init_instantiation_manager() -> InstantiationManager {
    InstantiationManager {
        templ_name2arg2val: HashMap::new(),
        instantiations: HashMap::new(),
    }
}

pub struct GlobalInformation<'ctx> {
    pub arraysize: u32,
    pub is_instantiation: bool,

    pub p: u64,
    pub val_ty: IntType<'ctx>,
    pub const_p: IntValue<'ctx>,
    pub const_zero: IntValue<'ctx>,
    pub context: &'ctx Context,

    // Global template information
    name2template_infos: HashMap<String, TemplateInformation>,

    // Global scope information
    name2scope_infos: HashMap<String, ScopeInformation<'ctx>>,

    fn_scope_names: Vec<String>,
    templ_scope_names: Vec<String>,
}

impl<'ctx> GlobalInformation<'ctx> {
    pub fn get_template_info(&self, templ_name: &String) -> &TemplateInformation {
        return self.name2template_infos.get(templ_name).unwrap();
    }

    pub fn set_template_info(&mut self, templ_name: &String, v: TemplateInformation) {
        self.name2template_infos.insert(templ_name.clone(), v);
    }

    pub fn get_scope_info(&self, scope_name: &String) -> &ScopeInformation<'ctx> {
        return self.name2scope_infos.get(scope_name).unwrap();
    }

    pub fn set_scope_info(&mut self, v: ScopeInformation<'ctx>) {
        let scope_name = v.get_name();
        if v.is_template {
            self.templ_scope_names.push(scope_name.clone());
        } else {
            self.fn_scope_names.push(scope_name.clone());
        }
        self.name2scope_infos.insert(scope_name.clone(), v);
    }

    pub fn get_fn_scope_infos(&self) -> Vec<&ScopeInformation<'ctx>> {
        let mut res = Vec::new();
        for s in &self.fn_scope_names {
            res.push(self.get_scope_info(s))
        }
        res
    }

    pub fn get_templ_scope_infos(&self) -> Vec<(&ScopeInformation<'ctx>, &TemplateInformation)> {
        let mut res = Vec::new();
        for s in &self.templ_scope_names {
            res.push((self.get_scope_info(s), self.get_template_info(s)))
        }
        res
    }

    pub fn get_scope_ret_ty(&self, scope_info_name: &String) -> BasicTypeEnum<'ctx> {
        let scope_ret_ty = self.get_scope_info(scope_info_name).get_ret_ty();
        scope_ret_ty
    }
}

pub fn init_env<'ctx>(
    context: &'ctx Context,
    val_ty: IntType<'ctx>,
    arraysize: u32,
    is_instantiation: bool,
) -> GlobalInformation<'ctx> {
    let p: u64 = 18446744073709551557;
    let const_p = val_ty.const_int(p, false);
    let const_zero = val_ty.const_zero();
    return GlobalInformation {
        arraysize,
        is_instantiation,
        p,
        val_ty,
        const_p,
        const_zero,

        context,
        name2template_infos: HashMap::new(),
        name2scope_infos: HashMap::new(),
        fn_scope_names: Vec::new(),
        templ_scope_names: Vec::new(),
    };
}
