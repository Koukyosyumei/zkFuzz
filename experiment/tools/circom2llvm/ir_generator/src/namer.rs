pub fn name_entry_block() -> String {
    return "entry".to_string();
}

pub fn name_exit_block() -> String {
    return "exit".to_string();
}

pub fn name_body_block() -> String {
    return "body".to_string();
}

pub fn name_if_block(true_or_false: bool, is_exit: bool) -> String {
    if is_exit {
        return "if.exit".to_string();
    }
    if true_or_false {
        return "if.true".to_string();
    } else {
        return "if.false".to_string();
    }
}

pub fn name_loop_block(is_header: bool, is_body: bool, is_exit: bool) -> String {
    if is_header {
        return "loop.header".to_string();
    }
    if is_body {
        return "loop.body".to_string();
    }
    if is_exit {
        return "loop.exit".to_string();
    }
    unreachable!();
}

pub fn name_constraint() -> String {
    return "constraint".to_string();
}

pub fn name_intrinsinc_fn(fn_name: &str) -> String {
    return format!("fn_intrinsic_{}", fn_name);
}

pub fn name_template_fn(fn_name: &str, templ_signature: &str) -> String {
    return format!("fn_template_{}_{}", fn_name, templ_signature);
}

pub fn name_template_struct(templ_name: &str) -> String {
    return format!("struct_template_{}", templ_name);
}

pub fn name_opaque_struct(struct_name: &str) -> String {
    return format!("{}.opaque", struct_name);
}
#[derive(PartialEq)]
pub enum ValueTypeEnum {
    InputSignal,
    IntermediateSignal,
    OutputSignal,
    ComponentInput,
    ComponentOutput,

    Argument,
    Constant,
    Variable,
    Component,
}

pub fn print_variable_type(var_ty: &ValueTypeEnum) -> &'static str {
    match var_ty {
        ValueTypeEnum::InputSignal => "input",
        ValueTypeEnum::IntermediateSignal => "inter",
        ValueTypeEnum::OutputSignal => "output",
        ValueTypeEnum::ComponentInput => "compinput",
        ValueTypeEnum::ComponentOutput => "compoutput",

        ValueTypeEnum::Argument => "arg",
        ValueTypeEnum::Constant => "const",
        ValueTypeEnum::Variable => "var",
        ValueTypeEnum::Component => "comp",
    }
}

pub fn name_initial_var(var_name: &str, var_ty: ValueTypeEnum) -> String {
    let operator = "initial";
    let var_ty_abbr = print_variable_type(&var_ty);
    let name = format!("{}.{}.{}", operator, var_name, var_ty_abbr);
    return name;
}

pub fn name_signal(
    templ_name: &str,
    signal_name: &str,
    var_ty: ValueTypeEnum,
) -> String {
    let operator = "gep";
    let name = format!("{}|{}", templ_name, signal_name);
    let var_ty_abbr = print_variable_type(&var_ty);
    let name = format!("{}.{}.{}", operator, name, var_ty_abbr);
    return name;
}

pub fn name_readwrite_var(
    is_read: bool,
    var_name: &str,
    var_ty: ValueTypeEnum,
) -> String {
    let operator = if is_read { "read" } else { "write" };
    let var_ty_abbr = print_variable_type(&var_ty);
    let name = format!("{}.{}.{}", operator, var_name, var_ty_abbr);
    return name;
}

pub fn name_inline_array(scope_name: &str) -> String {
    return format!("{}inlinearray", scope_name);
}

pub fn name_getter(ty: &str) -> String {
    return format!("{}_getter", ty);
}

pub fn name_main_comp(main_signature: &String) -> String {
    return format!("main_comp={}", main_signature);
}
