use crate::{
    function::Function,
    namer::name_template_fn,
    template::Template,
};
use inkwell::types::{
    AnyTypeEnum, ArrayType, BasicTypeEnum, IntType, PointerType, StructType, VoidType,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct SummaryGen {
    version: String,
    compiler: String,
    framework: String,
    meta: Meta,
    components: Vec<ComponentSummary>,
    functions: Vec<FunctionSummary>,
}

#[derive(Serialize, Deserialize)]
struct Meta {
    is_ir_ssa: bool,
}

#[derive(Serialize, Deserialize)]
struct TypeDescriber {
    type_name: String,
    size: u32,
    fields: Vec<TypeDescriber>,
    element_type: Option<Box<TypeDescriber>>,
}

#[derive(Serialize, Deserialize)]
struct SignalDescriber {
    name: String,
    signal_type: TypeDescriber,
    visibility: String,
    io: String,
}

#[derive(Serialize, Deserialize)]
struct ComponentSummary {
    name: String,
    main: bool,
    storage: TypeDescriber,
    params: Vec<TypeDescriber>,
    signals: Vec<SignalDescriber>,
    logic_fn_name: String,
}

#[derive(Serialize, Deserialize)]
struct FunctionSummary {
    name: String,
    params: Vec<TypeDescriber>,
    logic_fn_name: String,
    return_type: TypeDescriber,
    returns: bool,
}

trait LLVMType2TypeDescriber {
    fn gen(&self) -> TypeDescriber;
}

impl LLVMType2TypeDescriber for BasicTypeEnum<'_> {
    fn gen(&self) -> TypeDescriber {
        return match self {
            BasicTypeEnum::ArrayType(array_ty) => array_ty.gen(),
            BasicTypeEnum::IntType(int_ty) => int_ty.gen(),
            BasicTypeEnum::PointerType(pointer_ty) => pointer_ty.gen(),
            BasicTypeEnum::StructType(struct_ty) => struct_ty.gen(),
            _ => {
                unreachable!();
            }
        };
    }
}

impl LLVMType2TypeDescriber for AnyTypeEnum<'_> {
    fn gen(&self) -> TypeDescriber {
        return match self {
            AnyTypeEnum::ArrayType(array_ty) => array_ty.gen(),
            AnyTypeEnum::IntType(int_ty) => int_ty.gen(),
            AnyTypeEnum::PointerType(pointer_ty) => pointer_ty.gen(),
            AnyTypeEnum::StructType(struct_ty) => struct_ty.gen(),
            AnyTypeEnum::VoidType(void_ty) => void_ty.gen(),
            _ => {
                unreachable!();
            }
        };
    }
}

impl LLVMType2TypeDescriber for ArrayType<'_> {
    fn gen(&self) -> TypeDescriber {
        let ele_ty = Box::new(self.get_element_type().gen());
        return TypeDescriber {
            type_name: "array".to_string(),
            size: self.len(),
            fields: Vec::new(),
            element_type: Some(ele_ty),
        };
    }
}

impl LLVMType2TypeDescriber for IntType<'_> {
    fn gen(&self) -> TypeDescriber {
        let type_name = format!("i{}", self.get_bit_width());
        return TypeDescriber {
            type_name,
            size: 0,
            fields: Vec::new(),
            element_type: None,
        };
    }
}

impl LLVMType2TypeDescriber for PointerType<'_> {
    fn gen(&self) -> TypeDescriber {
        let ele_ty = Box::new(self.get_element_type().gen());
        return TypeDescriber {
            type_name: "pointer".to_string(),
            size: 0,
            fields: Vec::new(),
            element_type: Some(ele_ty),
        };
    }
}

impl LLVMType2TypeDescriber for StructType<'_> {
    fn gen(&self) -> TypeDescriber {
        let fields = self.get_field_types().iter().map(|f| f.gen()).collect();
        return TypeDescriber {
            type_name: "struct".to_string(),
            size: 0,
            fields,
            element_type: None,
        };
    }
}

impl LLVMType2TypeDescriber for VoidType<'_> {
    fn gen(&self) -> TypeDescriber {
        return TypeDescriber {
            type_name: "void".to_string(),
            size: 0,
            fields: Vec::new(),
            element_type: None,
        };
    }
}

impl SummaryGen {
    pub fn add_component(&mut self, template: &Template) {
        let templ_name = template.scope.get_signature().clone();
        let func_val = template.scope.fn_val.unwrap();
        let struct_ty = func_val.get_first_param().unwrap().get_type();
        let mut signals = Vec::new();
        for input in &template.templ_info.inputs {
            let signal_type = template.scope.info.get_var_used_ty(input);
            let signal = SignalDescriber {
                name: input.clone(),
                signal_type: signal_type.gen(),
                visibility: "public".to_string(),
                io: "input".to_string(),
            };
            signals.push(signal);
        }
        for output in &template.templ_info.outputs {
            let signal_type = template.scope.info.get_var_used_ty(output);
            let signal = SignalDescriber {
                name: output.clone(),
                signal_type: signal_type.gen(),
                visibility: "public".to_string(),
                io: "output".to_string(),
            };
            signals.push(signal);
        }
        for inter in &template.templ_info.inters {
            let signal_type = template.scope.info.get_var_used_ty(inter);
            let signal = SignalDescriber {
                name: inter.clone(),
                signal_type: signal_type.gen(),
                visibility: "public".to_string(),
                io: "inter".to_string(),
            };
            signals.push(signal);
        }
        let component = ComponentSummary {
            name: templ_name.clone(),
            main: template.scope.get_name() == "main",
            storage: struct_ty.gen(),
            params: template.scope.info.get_arg_tys().iter().map(|a| a.gen()).collect(),
            signals,
            logic_fn_name: name_template_fn("init", &templ_name),
        };
        self.components.push(component);
    }

    pub fn add_function(&mut self, function: &Function) {
        let func_name = function.scope.get_name().clone();
        let func_val = function.scope.fn_val.unwrap();
        let return_type = func_val.get_type().get_return_type().unwrap().gen();
        let returns = return_type.type_name == "void";
        let func = FunctionSummary {
            name: func_name.clone(),
            params: function.scope.info.get_arg_tys().iter().map(|a| a.gen()).collect(),
            logic_fn_name: func_name.clone(),
            return_type,
            returns,
        };
        self.functions.push(func);
    }

    pub fn print_to_file(&self, path: &PathBuf) -> Result<(), serde_json::Error> {
        let writer = File::create(path).unwrap();
        return serde_json::to_writer(writer, &self);
    }
}

pub fn init_summarygen() -> SummaryGen {
    let meta = Meta {
        is_ir_ssa: false,
    };
    let summarygen = SummaryGen {
        compiler: "circom".to_string(),
        framework: "".to_string(),
        version: "2.0.0".to_string(),
        meta,
        components: Vec::new(),
        functions: Vec::new(),
    };
    return summarygen;
}
