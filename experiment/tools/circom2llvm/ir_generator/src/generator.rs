use crate::after_process::remove_opaque_struct_name;
use crate::codegen::init_codegen;
use crate::environment::{init_env, init_instantiation_manager};
use crate::expression_static::{resolve_expr_static, Instantiation, SymbolValueManager};
use crate::function::{infer_fn, Function};
use crate::scope::init_scope;
use crate::scope_information::{init_scope_info, ScopeInformation};
use crate::statement::instant_stmt;
use crate::summarygen::init_summarygen;
use crate::template::{infer_templ, Template};
use inkwell::context::Context;
use program_structure::ast::{Definition, Expression, Statement};
use std::collections::{HashMap, HashSet};
use std::iter::zip;
use std::path::PathBuf;

pub fn resolve_dependence(dependence_graph: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut all = dependence_graph.len().clone();
    let mut output: Vec<String> = Vec::new();
    while all > 0 {
        let last_all = all.clone();
        for (k, deps) in dependence_graph {
            if output.contains(&k) {
                continue;
            }
            let mut satisfied = true;
            for dep in deps {
                if !output.contains(&dep) && dep != k {
                    satisfied = false;
                }
            }
            if satisfied {
                output.push(k.clone());
                all -= 1;
            }
        }
        if last_all == all {
            println!("Error: Cannot resolve dependences! Perhaps some includes are missing.");
            for (k, deps) in dependence_graph {
                if output.contains(&k) {
                    continue;
                } else {
                    let dep_strs: Vec<&str> = deps
                        .iter()
                        .filter(|s| !output.contains(s))
                        .map(|s| s.as_str())
                        .collect();
                    println!(
                        "Unresolved: {}, Unresolved dependences: {}.",
                        k,
                        dep_strs.join("  ")
                    );
                }
            }
            unreachable!();
        }
    }
    return output;
}

pub fn resolve_main_dependence(
    dependence_graph: &HashMap<String, Vec<String>>,
    main_comp: &String,
) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
    let mut queue = vec![main_comp];
    while queue.len() > 0 {
        let cur = queue.pop().unwrap();
        for d in &dependence_graph[cur] {
            if d != cur {
                queue.insert(0, d)
            }
        }
        output.insert(0, cur.clone())
    }
    output
}

pub fn generate(
    is_instantiation: bool,
    arraysize: u32,
    main_expr: Option<Expression>,
    definitions: Vec<&Definition>,
    input_path: &PathBuf,
    output_path: &PathBuf,
    output_summary_path: &PathBuf,
    generate_summary: bool,
) {
    let context = Context::create();
    let file_path = input_path.as_os_str().to_str().unwrap();
    let file_name = input_path.file_name().unwrap().to_str().unwrap();
    let module = context.create_module(file_name);
    module.set_source_file_name(file_path);
    let val_ty = context.i128_type();
    let codegen = init_codegen(&context, module, val_ty, arraysize);
    let mut env = init_env(&context, val_ty, arraysize, is_instantiation);
    let mut i_manager = init_instantiation_manager();
    let mut summarygen = init_summarygen();
    let mut scope_info_stmt_pairs: Vec<(ScopeInformation, &Statement)> = Vec::new();
    let val_ty = env.val_ty.clone();
    for defin in definitions {
        let (mut scope_info, body) = match defin {
            Definition::Template {
                meta: _,
                name,
                args,
                arg_location: _,
                body,
                parallel: _,
                is_custom_gate: _,
            } => (
                init_scope_info(true, val_ty, name.clone(), args.clone()),
                body,
            ),
            Definition::Function {
                meta: _,
                name,
                args,
                arg_location: _,
                body,
            } => (
                init_scope_info(false, val_ty, name.clone(), args.clone()),
                body,
            ),
        };
        scope_info.resolve_dependences(body);
        scope_info_stmt_pairs.push((scope_info, body));
    }
    let mut dependence_graph: HashMap<String, Vec<String>> = HashMap::new();
    for (scope_info, _) in &scope_info_stmt_pairs {
        let owned_deps = scope_info
            .get_dependences()
            .iter()
            .map(|s| s.clone())
            .collect();
        dependence_graph.insert(scope_info.get_name().clone(), owned_deps);
    }

    let compile_order = if is_instantiation {
        match &main_expr {
            Some(expr) => match expr {
                Expression::Call {
                    meta: _,
                    id,
                    args: _,
                } => resolve_main_dependence(&dependence_graph, id),
                _ => unreachable!(),
            },
            None => {
                println!("Error: No main component is provided under instantiation compilation.");
                unreachable!();
            }
        }
    } else {
        resolve_dependence(&dependence_graph)
    };

    let mut unique_compile_order: Vec<String> = Vec::new();
    for c in compile_order {
        if !unique_compile_order.contains(&c) {
            unique_compile_order.push(c);
        }
    }

    let get_index = |s: &(ScopeInformation, &Statement)| -> Option<usize> {
        unique_compile_order
            .iter()
            .position(|r| r == s.0.get_name())
    };

    scope_info_stmt_pairs = scope_info_stmt_pairs
        .into_iter()
        .filter(|s| get_index(s).is_some())
        .collect();

    scope_info_stmt_pairs.sort_by(|a, b| get_index(a).cmp(&get_index(b)));

    let mut templ_pairs: Vec<(ScopeInformation, &Statement)> = Vec::new();
    let mut fn_pairs: Vec<(ScopeInformation, &Statement)> = Vec::new();
    let mut templ_name_pairs: Vec<(String, &Statement)> = Vec::new();

    for (scope_info, body) in scope_info_stmt_pairs.into_iter() {
        if scope_info.is_template {
            templ_name_pairs.push((scope_info.get_name().clone(), body));
            templ_pairs.push((scope_info, body));
        } else {
            fn_pairs.push((scope_info, body));
        }
    }

    for (mut scope_info, body) in fn_pairs {
        infer_fn(&env, &mut scope_info, body);
        env.set_scope_info(scope_info.clone());
        let i = HashMap::new();
        let scope = init_scope(scope_info.clone(), i);
        let mut f = Function { scope };
        f.build(&env, &codegen, body);
        summarygen.add_function(&f);
    }

    let mut templates: Vec<(Template, Statement)> = Vec::new();

    for (mut scope_info, body) in templ_pairs.into_iter() {
        let scope_name = scope_info.get_name().clone();
        let templ_info = infer_templ(&context, &mut env, &mut scope_info, body);
        env.set_scope_info(scope_info);
        env.set_template_info(&scope_name, templ_info.clone());
    }

    if is_instantiation {
        let main_expr = &main_expr.unwrap();
        match main_expr {
            Expression::Call { meta: _, id, args } => {
                // empty scope info
                let empty_scope_info =
                    init_scope_info(true, val_ty, "main".to_string(), Vec::new());
                let target_scope_info = env.get_scope_info(id);
                let mut arg2val = HashMap::new();
                let arg_names = &target_scope_info.args;
                for (a, expr) in zip(arg_names, args) {
                    let v = resolve_expr_static(&env, &empty_scope_info, &HashMap::new(), &expr);
                    arg2val.insert(a.clone(), v);
                }
                let signature = target_scope_info.gen_signature(&arg2val);
                codegen.build_instantiation_flag(&signature);
                i_manager.set_arg2val(&id, arg2val);
            }
            _ => unreachable!(),
        }

        // Collect instantiations from the main component to other components, so we use .rev().
        for (scope_name, body) in templ_name_pairs.iter().rev() {
            if !i_manager.has_arg2val(scope_name) {
                // This sub-component is deleted during the rewriting.
                continue;
            }
            let scope_info = env.get_scope_info(scope_name);
            let mut arg2vals = i_manager.get_arg2val(scope_name).clone();
            let mut instantiations: Vec<Instantiation> = Vec::new();
            let mut sub_templ_arg_vals = HashSet::new();
            let mut consumed_templ_signatures = HashSet::new();

            while arg2vals.len() > 0 {
                let origin_arg2val = arg2vals.pop().unwrap();
                let signature = scope_info.gen_signature(&origin_arg2val);
                if consumed_templ_signatures.contains(&signature) {
                    continue;
                }
                consumed_templ_signatures.insert(scope_info.gen_signature(&origin_arg2val));
                let mut arg2val = origin_arg2val.clone();
                // Rewrite the body by possible argument->concrete_value mappings.
                let new_body = instant_stmt(
                    &env,
                    scope_info,
                    &mut arg2val,
                    &mut SymbolValueManager::init(),
                    &mut sub_templ_arg_vals,
                    &body,
                );
                instantiations.push((origin_arg2val, new_body));
                // Add all argument->concrete_value mappings of sub-components which are collected during the rewriting.
                for (sub_templ_name, arg_vals) in &sub_templ_arg_vals {
                    let target_scope_info = env.get_scope_info(&sub_templ_name);
                    let sub_templ_arg2val = target_scope_info.gen_arg2val(&arg_vals);
                    if sub_templ_name != scope_name {
                        i_manager.set_arg2val(sub_templ_name, sub_templ_arg2val);
                    } else {
                        arg2vals.push(sub_templ_arg2val);
                    }
                }
            }

            // Build all possible circuits of the current template.
            for (arg2val, body) in instantiations.into_iter() {
                let scope_info = env.get_scope_info(scope_name).clone();
                let templ_info = env.get_template_info(scope_name).clone();
                let scope = init_scope(scope_info, arg2val);
                let new_t = Template { scope, templ_info };
                let mut has = false;
                for (t, _) in &templates {
                    if t.scope.get_signature() == new_t.scope.get_signature() {
                        has = true;
                        break;
                    }
                }
                if !has {
                    templates.insert(0, (new_t, body));
                }
            }
        }
    } else {
        for (scope_name, body) in templ_name_pairs {
            let scope_info = env.get_scope_info(&scope_name).clone();
            let templ_info = env.get_template_info(&scope_name).clone();
            let arg2val = HashMap::new();
            let scope = init_scope(scope_info, arg2val);
            let t = Template { scope, templ_info };
            templates.push((t, body.clone()));
        }
    }

    for (t, body) in &mut templates {
        t.build_function(&env, &codegen, body);
    }

    for (t, body) in &mut templates {
        t.build_instrustions(&env, &codegen, body);
        summarygen.add_component(t);
    }
    if generate_summary {
        let json_result = summarygen.print_to_file(output_summary_path);
        match json_result {
            Ok(..) => (),
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }

    let result = codegen.module.print_to_file(&output_path);
    match result {
        Ok(_) => {
            remove_opaque_struct_name(&output_path);
        }
        Err(err) => {
            println!("Error: {}", err.to_string());
        }
    }
}
