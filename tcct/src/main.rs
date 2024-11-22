//mod execution_user;
mod input_user;
mod parser_user;
mod symbolic_execution;
mod type_analysis_user;

use ansi_term::Colour;
use env_logger;
use input_user::Input;
use log::{debug, info, warn};
use parser_user::ExtendedStatement;
use program_structure::ast::Expression;
use std::env;
use symbolic_execution::{
    print_constraint_summary_statistics, simplify_statement, SymbolicExecutor,
};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let result = start();
    if result.is_err() {
        eprintln!("{}", Colour::Red.paint("previous errors were found"));
        std::process::exit(1);
    } else {
        println!("{}", Colour::Green.paint("Everything went okay"));
        //std::process::exit(0);
    }
}

fn start() -> Result<(), ()> {
    use crate::parser_user::DebugExpression;
    //use compilation_user::CompilerConfig;

    let user_input = Input::new()?;
    let mut program_archive = parser_user::parse_project(&user_input)?;
    type_analysis_user::analyse_project(&mut program_archive)?;

    env_logger::init();
    let mut sexe = SymbolicExecutor::new();

    for (k, v) in program_archive.templates.clone().into_iter() {
        let body = simplify_statement(&v.get_body().clone());
        sexe.register_library(k, body);
    }

    match &program_archive.initial_template_call {
        Expression::Call { id, args, .. } => {
            let v = program_archive.templates["LessThan"].clone();
            let body = simplify_statement(&v.get_body().clone());
            debug!(
                "body:\n{:?}",
                ExtendedStatement::DebugStatement(body.clone())
            );
            sexe.execute(
                &vec![
                    ExtendedStatement::DebugStatement(body),
                    ExtendedStatement::Ret,
                ],
                0,
            );

            for s in &sexe.final_states {
                info!("final_state: {:?}", s);
            }
            println!("template_name,num_of_params,max_depth");
            //println!("{},{},{}", k, v.get_num_of_params(), sexe.max_depth);
            print_constraint_summary_statistics(&sexe.trace_constraint_stats);
            print_constraint_summary_statistics(&sexe.side_constraint_stats);
        }
        _ => {
            warn!("Cannot Find Main Call");
        }
    }

    /*
    for (k, v) in program_archive.templates.clone().into_iter() {
        let mut sexe = SymbolicExecutor::new();
        let body = simplify_statement(&v.get_body().clone());
        debug!(
            "body:\n{:?}",
            ExtendedStatement::DebugStatement(body.clone())
        );
        sexe.execute(
            &vec![
                ExtendedStatement::DebugStatement(body),
                ExtendedStatement::Ret,
            ],
            0,
        );

        for s in &sexe.final_states {
            info!("final_state: {:?}", s);
        }
        println!("template_name,num_of_params,max_depth");
        println!("{},{},{}", k, v.get_num_of_params(), sexe.max_depth);
        print_constraint_summary_statistics(&sexe.trace_constraint_stats);
        print_constraint_summary_statistics(&sexe.side_constraint_stats);
    }*/

    /*
    let config = ExecutionConfig {
        no_rounds: user_input.no_rounds(),
        flag_p: user_input.parallel_simplification_flag(),
        flag_s: user_input.reduced_simplification_flag(),
        flag_f: user_input.unsimplified_flag(),
        flag_old_heuristics: user_input.flag_old_heuristics(),
        flag_verbose: user_input.flag_verbose(),
        inspect_constraints_flag: user_input.inspect_constraints_flag(),
        r1cs_flag: user_input.r1cs_flag(),
        json_constraint_flag: user_input.json_constraints_flag(),
        json_substitution_flag: user_input.json_substitutions_flag(),
        sym_flag: user_input.sym_flag(),
        sym: user_input.sym_file().to_string(),
        r1cs: user_input.r1cs_file().to_string(),
        json_constraints: user_input.json_constraints_file().to_string(),
        json_substitutions: user_input.json_substitutions_file().to_string(),
        prime: user_input.prime(),
    };
    let circuit = execution_user::execute_project(program_archive, config)?;
    */

    /*
    let compilation_config = CompilerConfig {
        vcp: circuit,
        debug_output: user_input.print_ir_flag(),
        c_flag: user_input.c_flag(),
        wasm_flag: user_input.wasm_flag(),
        wat_flag: user_input.wat_flag(),
        js_folder: user_input.js_folder().to_string(),
        wasm_name: user_input.wasm_name().to_string(),
        c_folder: user_input.c_folder().to_string(),
        c_run_name: user_input.c_run_name().to_string(),
        c_file: user_input.c_file().to_string(),
        dat_file: user_input.dat_file().to_string(),
        wat_file: user_input.wat_file().to_string(),
        wasm_file: user_input.wasm_file().to_string(),
        produce_input_log: user_input.main_inputs_flag(),
        constraint_assert_dissabled_flag: user_input.constraint_assert_dissabled_flag(),
    };
    compilation_user::compile(compilation_config)?;
    */
    Result::Ok(())
}
