//mod execution_user;
mod input_user;
mod parser_user;
//mod solver;
//mod stats;
mod symbolic_execution;
mod type_analysis_user;
mod utils;

use ansi_term::Colour;
use env_logger;
use input_user::Input;
use log::{error, info, warn};
use num_bigint_dig::BigInt;
use rustc_hash::FxHashMap;
use std::env;
use std::str::FromStr;
use std::time;

use parser_user::DebugStatement;
use program_structure::ast::Expression;
//use solver::brute_force_search;
//use stats::{print_constraint_summary_statistics_pretty, ConstraintStatistics};
use symbolic_execution::{register_library, simplify_statement, SymbolicExecutor};

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

    let mut template_library = FxHashMap::default();
    let mut name2id = FxHashMap::default();
    let mut id2name = FxHashMap::default();

    println!("{}", Colour::Green.paint("🧩 Parsing Templates..."));
    for (k, v) in program_archive.templates.clone().into_iter() {
        let body = simplify_statement(&v.get_body().clone());
        register_library(
            &mut template_library,
            &mut name2id,
            &mut id2name,
            k.clone(),
            &body.clone(),
            v.get_name_of_params(),
        );

        if user_input.flag_printout_ast {
            println!(
                "{}{} {}{}",
                BACK_GRAY_SCRIPT_BLACK, "🌳 AST Tree for", k, RESET
            );
            println!("{:?}", template_library[&name2id[&k]].body);
        }
    }

    let mut sexe = SymbolicExecutor::new(
        Box::new(template_library.clone()),
        &mut name2id,
        &mut id2name,
        user_input.flag_propagate_substitution,
        BigInt::from_str(&user_input.debug_prime()).unwrap(),
    );

    println!("{}", Colour::Green.paint("⚙️ Parsing Function..."));
    for (k, v) in program_archive.functions.clone().into_iter() {
        let body = simplify_statement(&v.get_body().clone());
        sexe.register_function(k.clone(), body.clone(), v.get_name_of_params());

        if user_input.flag_printout_ast {
            println!(
                "{}{} {}{}",
                BACK_GRAY_SCRIPT_BLACK, "🌴 AST Tree for", k, RESET
            );
            println!("{:?}", sexe.function_library[&sexe.name2id[&k]].body);
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
            sexe.name2id.insert("main".to_string(), sexe.name2id.len());
            sexe.id2name
                .insert(sexe.name2id["main"], "main".to_string());
            sexe.cur_state.add_owner(sexe.name2id.len() - 1, 0);
            sexe.cur_state.set_template_id(sexe.name2id[id]);
            if !user_input.flag_symbolic_template_params {
                sexe.feed_arguments(template.get_name_of_params(), args);
            }
            let body = sexe.template_library[&sexe.name2id[id]].body.clone();
            sexe.execute(&body, 0);

            println!("===========================================================");
            //let mut ts = ConstraintStatistics::new();
            //let mut ss = ConstraintStatistics::new();
            for s in &sexe.final_states {
                /*
                for c in &s.trace_constraints {
                    ts.update(c);
                }
                for c in &s.side_constraints {
                    ss.update(c);
                }*/
                info!("Final State: {}", s.lookup_fmt(&sexe.id2name));
            }
            println!("===========================================================");

            /*
            let mut is_safe = true;
            if user_input.search_mode != "none" {
                println!("{}", Colour::Green.paint("🩺 Scanning TCCT Instances..."));
                let mut sub_sexe = sexe.clone();
                sub_sexe.clear();

                let mut main_template_id = "";
                let mut template_param_names = Vec::new();
                let mut template_param_values = Vec::new();
                match &program_archive.initial_template_call {
                    Expression::Call { id, args, .. } => {
                        main_template_id = id;
                        let template = program_archive.templates[id].clone();
                        sub_sexe.cur_state.set_owner("main".to_string());
                        sub_sexe
                            .cur_state
                            .set_template_id(main_template_id.to_string());
                        if !user_input.flag_symbolic_template_params {
                            template_param_names = template.get_name_of_params().clone();
                            template_param_values = args.clone();
                            sub_sexe.feed_arguments(template.get_name_of_params(), args);
                        }
                    }
                    _ => unimplemented!(),
                }
                for s in &sexe.final_states {
                    let counterexample = match &*user_input.search_mode {
                        "quick" => brute_force_search(
                            BigInt::from_str(&user_input.debug_prime()).unwrap(),
                            main_template_id.to_string(),
                            &mut sub_sexe,
                            &s.trace_constraints.clone(),
                            &s.side_constraints.clone(),
                            true,
                            &template_param_names,
                            &template_param_values,
                        ),
                        "full" => brute_force_search(
                            BigInt::from_str(&user_input.debug_prime()).unwrap(),
                            main_template_id.to_string(),
                            &mut sub_sexe,
                            &s.trace_constraints.clone(),
                            &s.side_constraints.clone(),
                            false,
                            &template_param_names,
                            &template_param_values,
                        ),
                        _ => panic!(
                            "search_mode={} is not supported",
                            user_input.search_mode.to_string()
                        ),
                    };
                    if counterexample.is_some() {
                        is_safe = false;
                        println!("{:?}", counterexample.unwrap());
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
                (ss.total_constraints as f64 / ts.total_constraints as f64) * 100 as f64,
                ss.total_constraints,
                ts.total_constraints
            );
            println!(
                "  - Verification        : {}",
                if is_safe {
                    Colour::Green.paint("🆗 No Counter Example Found")
                } else {
                    Colour::Red.paint("💥 NOT SAFE 💥")
                }
            );
            println!("  - Execution Time      : {:?}", start_time.elapsed());

            if user_input.flag_printout_stats {
                println!(
                    "--------------------------------------------\n🪶 Stats of Trace Constraint"
                );
                print_constraint_summary_statistics_pretty(&ts);
                println!(
                    "--------------------------------------------\n⛓️ Stats of Side Constraint"
                );
                print_constraint_summary_statistics_pretty(&ss);
            }
            println!("===========================================================");
            */
        }
        _ => {
            warn!("Cannot Find Main Call");
        }
    }

    Result::Ok(())
}
