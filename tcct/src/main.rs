//mod execution_user;
mod input_user;
mod parser_user;
mod stats;
mod symbolic_execution;
mod type_analysis_user;

use ansi_term::Colour;
use env_logger;
use input_user::Input;
use log::{info, warn};
use parser_user::ExtendedStatement;
use program_structure::ast::Expression;
use stats::print_constraint_summary_statistics_pretty;
use std::env;
use symbolic_execution::{simplify_statement, ConstraintStatistics, SymbolicExecutor};

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
    //use compilation_user::CompilerConfig;

    let user_input = Input::new()?;
    let mut program_archive = parser_user::parse_project(&user_input)?;
    type_analysis_user::analyse_project(&mut program_archive)?;

    env_logger::init();

    let mut ts = ConstraintStatistics::new();
    let mut ss = ConstraintStatistics::new();
    let mut sexe = SymbolicExecutor::new(&mut ts, &mut ss);

    for (k, v) in program_archive.templates.clone().into_iter() {
        let body = simplify_statement(&v.get_body().clone());
        sexe.register_library(k.clone(), body.clone(), v.get_name_of_params());

        if user_input.flag_printout_ast {
            println!("🌳 AST Tree for {}", k);
            println!("{:?}", ExtendedStatement::DebugStatement(body.clone()));
            println!("========================================")
        }
    }

    match &program_archive.initial_template_call {
        Expression::Call { id, .. } => {
            let v = program_archive.templates[id].clone();
            let body = simplify_statement(&v.get_body().clone());

            sexe.cur_state.set_owner("main".to_string());
            sexe.execute(
                &vec![
                    ExtendedStatement::DebugStatement(body),
                    ExtendedStatement::Ret,
                ],
                0,
            );

            info!("============================================================");
            for s in &sexe.final_states {
                info!("Final State: {:?}", s);
            }

            println!("================ TCCT Report ================");
            println!("📊 Execution Summary:");
            println!("  - Total Paths Explored: {}", sexe.final_states.len());

            if user_input.flag_printout_stats {
                println!(
                    "--------------------------------------------\n🪶 Stats of Trace Constraint"
                );
                print_constraint_summary_statistics_pretty(&sexe.trace_constraint_stats);
                println!(
                    "--------------------------------------------\n⛓️ Stats of Side Constraint*"
                );
                print_constraint_summary_statistics_pretty(&sexe.side_constraint_stats);
            }
            println!("=============================================");
        }
        _ => {
            warn!("Cannot Find Main Call");
        }
    }

    Result::Ok(())
}
