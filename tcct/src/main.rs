//mod execution_user;
mod input_user;
mod parser_user;
mod solver;
mod stats;
mod symbolic_execution;
mod type_analysis_user;
mod utils;

use ansi_term::Colour;
use env_logger;
use input_user::Input;
use log::{error, info, warn};
use num_bigint_dig::BigInt;
use stats::print_constraint_summary_statistics_pretty;
use std::env;
use std::str::FromStr;
use std::time;

use parser_user::ExtendedStatement;
use program_structure::ast::Expression;
use solver::brute_force_search;
use symbolic_execution::{simplify_statement, ConstraintStatistics, SymbolicExecutor};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const RESET: &str = "\x1b[0m";
const BACK_GRAY_SCRIPT_BLACK: &str = "\x1b[30;100m"; //94

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
    let mut sexe = SymbolicExecutor::new(
        user_input.flag_propagate_substitution,
        BigInt::from_str(&user_input.debug_prime()).unwrap(),
        &mut ts,
        &mut ss,
    );

    println!("{}", Colour::Green.paint("🧩 Parsing Templates..."));
    for (k, v) in program_archive.templates.clone().into_iter() {
        let body = simplify_statement(&v.get_body().clone());
        sexe.register_library(k.clone(), body.clone(), v.get_name_of_params());

        if user_input.flag_printout_ast {
            println!(
                "{}{} {}{}",
                BACK_GRAY_SCRIPT_BLACK, "🌳 AST Tree for", k, RESET
            );
            println!("{:?}", ExtendedStatement::DebugStatement(body.clone()));
        }
    }

    println!("{}", Colour::Green.paint("⚙️ Parsing Function..."));
    for (k, v) in program_archive.functions.clone().into_iter() {
        let body = simplify_statement(&v.get_body().clone());
        sexe.register_function(k.clone(), body.clone(), v.get_name_of_params());

        if user_input.flag_printout_ast {
            println!(
                "{}{} {}{}",
                BACK_GRAY_SCRIPT_BLACK, "🌴 AST Tree for", k, RESET
            );
            println!("{:?}", ExtendedStatement::DebugStatement(body.clone()));
        }
    }

    match &program_archive.initial_template_call {
        Expression::Call { id, args, .. } => {
            let start_time = time::Instant::now();
            let template = program_archive.templates[id].clone();
            let body = simplify_statement(&template.get_body().clone());

            println!(
                "{}",
                Colour::Green.paint("🛒 Gathering Trace/Side Constraints...")
            );
            sexe.cur_state.set_owner("main".to_string());
            if !user_input.flag_symbolic_template_params {
                sexe.feed_arguments(template.get_name_of_params(), args);
            }
            sexe.execute(
                &vec![
                    ExtendedStatement::DebugStatement(body),
                    ExtendedStatement::Ret,
                ],
                0,
            );

            println!("===========================================================");
            for s in &sexe.final_states {
                info!("Final State: {:?}", s);
            }
            println!("===========================================================");

            println!("{}", Colour::Green.paint("🩺 Scanning TCCT Instances..."));
            let mut sub_ts = ConstraintStatistics::new();
            let mut sub_ss = ConstraintStatistics::new();
            let mut sub_sexe = SymbolicExecutor::new(
                user_input.flag_propagate_substitution,
                BigInt::from_str(&user_input.debug_prime()).unwrap(),
                &mut sub_ts,
                &mut sub_ss,
            );
            for (k, v) in program_archive.templates.clone().into_iter() {
                let body = simplify_statement(&v.get_body().clone());
                sub_sexe.register_library(k.clone(), body.clone(), v.get_name_of_params());
            }
            for (k, v) in program_archive.functions.clone().into_iter() {
                let body = simplify_statement(&v.get_body().clone());
                sub_sexe.register_function(k.clone(), body.clone(), v.get_name_of_params());
            }
            let mut main_template_id = "";
            match &program_archive.initial_template_call {
                Expression::Call { id, args, .. } => {
                    main_template_id = id;
                    let template = program_archive.templates[id].clone();
                    let body = simplify_statement(&template.get_body().clone());
                    sub_sexe.cur_state.set_owner("main".to_string());
                    if !user_input.flag_symbolic_template_params {
                        sub_sexe.feed_arguments(template.get_name_of_params(), args);
                    }
                }
                _ => unimplemented!(),
            }

            let mut is_safe = "🆗 Probably Safe";
            for s in &sexe.final_states {
                let assignment = brute_force_search(
                    BigInt::from_str(&user_input.debug_prime()).unwrap(),
                    main_template_id.to_string(),
                    &mut sub_sexe,
                    &s.trace_constraints.clone(),
                    &s.side_constraints.clone(),
                );
                if assignment.is_some() {
                    error!("Malformed TCCT Instance:");
                    is_safe = "💥 NOT SAFE 💥";
                    for (k, v) in assignment.unwrap().into_iter() {
                        println!("  - {}: {}", k, v);
                    }
                }
            }

            println!(
                "{}",
                Colour::Green.paint("======================= TCCT Report =======================")
            );
            println!("📊 Execution Summary:");
            println!("  - Prime Number        : {}", user_input.debug_prime());
            println!("  - Total Paths Explored: {}", sexe.final_states.len());
            println!(
                "  - Compression Rate    : {:.2}% ({}/{})",
                (sexe.side_constraint_stats.total_constraints as f64
                    / sexe.trace_constraint_stats.total_constraints as f64)
                    * 100 as f64,
                sexe.side_constraint_stats.total_constraints,
                sexe.trace_constraint_stats.total_constraints
            );
            println!("  - Verification        : {}", is_safe);
            println!("  - Execution Time      : {:?}", start_time.elapsed());

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
            println!("===========================================================");
        }
        _ => {
            warn!("Cannot Find Main Call");
        }
    }

    Result::Ok(())
}
