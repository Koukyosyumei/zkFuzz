//mod execution_user;
mod debug_ast;
mod input_user;
mod parser_user;
mod solver;
mod stats;
mod symbolic_execution;
mod symbolic_value;
mod type_analysis_user;
mod utils;

use std::env;
use std::str::FromStr;
use std::time;

use ansi_term::Colour;
use colored::Colorize;
use env_logger;
use input_user::Input;
use log::{info, warn};
use num_bigint_dig::BigInt;
use rustc_hash::FxHashMap;

use debug_ast::simplify_statement;
use program_structure::ast::Expression;
use solver::{brute_force_search, VerificationSetting};
use stats::{print_constraint_summary_statistics_pretty, ConstraintStatistics};
use symbolic_execution::{SymbolicExecutor, SymbolicExecutorSetting};
use symbolic_value::{OwnerName, SymbolicLibrary};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const RESET: &str = "\x1b[0m";
const BACK_GRAY_SCRIPT_BLACK: &str = "\x1b[30;100m"; //94

fn display_tcct_logo() {
    let logo = r#"
  ████████╗ ██████╗ ██████╗████████╗
  ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝
     ██║   ██║     ██║        ██║   
     ██║   ██║     ██║        ██║   
     ██║   ╚██████╗╚██████╗   ██║   
     ╚═╝    ╚═════╝ ╚═════╝   ╚═╝   
 Trace-Constraint Consistency Test
     ZKP Circuit Debugger v0.0
    "#;

    println!("{}", logo.bright_cyan().bold());
    println!("{}", "Welcome to the TCCT Debugging Tool".green().bold());
    println!(
        "{}",
        "════════════════════════════════════════════════════════════════".green()
    );
}

fn main() {
    display_tcct_logo();

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

    let mut symbolic_library = SymbolicLibrary {
        template_library: FxHashMap::default(),
        name2id: FxHashMap::default(),
        id2name: FxHashMap::default(),
        function_library: FxHashMap::default(),
        function_counter: FxHashMap::default(),
    };

    println!("{}", Colour::Green.paint("🧩 Parsing Templates..."));
    for (k, v) in program_archive.templates.clone().into_iter() {
        let body = simplify_statement(&v.get_body().clone());
        symbolic_library.register_library(k.clone(), &body.clone(), v.get_name_of_params());

        if user_input.flag_printout_ast {
            println!(
                "{}{} {}{}",
                BACK_GRAY_SCRIPT_BLACK, "🌳 AST Tree for", k, RESET
            );
            println!(
                "{:?}",
                symbolic_library.template_library[&symbolic_library.name2id[&k]].body
            );
        }
    }

    println!("{}", Colour::Green.paint("⚙️ Parsing Function..."));
    for (k, v) in program_archive.functions.clone().into_iter() {
        let body = simplify_statement(&v.get_body().clone());
        symbolic_library.register_function(k.clone(), body.clone(), v.get_name_of_params());

        if user_input.flag_printout_ast {
            println!(
                "{}{} {}{}",
                BACK_GRAY_SCRIPT_BLACK, "🌴 AST Tree for", k, RESET
            );
            println!(
                "{:?}",
                symbolic_library.function_library[&symbolic_library.name2id[&k]].body
            );
        }
    }

    let setting = SymbolicExecutorSetting {
        prime: BigInt::from_str(&user_input.debug_prime()).unwrap(),
        propagate_substitution: user_input.flag_propagate_substitution,
        skip_initialization_blocks: false,
        off_trace: false,
        keep_track_constraints: true,
        keep_track_unrolled_offset: true,
    };
    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);

    match &program_archive.initial_template_call {
        Expression::Call { id, args, .. } => {
            let start_time = time::Instant::now();
            let template = program_archive.templates[id].clone();

            println!(
                "{}",
                Colour::Green.paint("🛒 Gathering Trace/Side Constraints...")
            );

            sexe.symbolic_library
                .name2id
                .insert("main".to_string(), sexe.symbolic_library.name2id.len());
            sexe.symbolic_library
                .id2name
                .insert(sexe.symbolic_library.name2id["main"], "main".to_string());

            sexe.cur_state.add_owner(&OwnerName {
                name: sexe.symbolic_library.name2id["main"],
                counter: 0,
            });
            sexe.cur_state
                .set_template_id(sexe.symbolic_library.name2id[id]);

            if !user_input.flag_symbolic_template_params {
                sexe.feed_arguments(template.get_name_of_params(), args);
            }

            let body = sexe.symbolic_library.template_library[&sexe.symbolic_library.name2id[id]]
                .body
                .clone();
            sexe.execute(&body, 0);

            println!(
                "{}",
                Colour::Green
                    .paint("════════════════════════════════════════════════════════════════")
            );
            let mut ts = ConstraintStatistics::new();
            let mut ss = ConstraintStatistics::new();
            for s in &sexe.symbolic_store.final_states {
                for c in &s.trace_constraints {
                    ts.update(c);
                }
                for c in &s.side_constraints {
                    ss.update(c);
                }
                info!(
                    "Final State: {}",
                    s.lookup_fmt(&sexe.symbolic_library.id2name)
                );
            }

            let mut is_safe = true;
            if user_input.search_mode != "none" {
                println!(
                    "{}",
                    Colour::Green
                        .paint("════════════════════════════════════════════════════════════════")
                );
                println!("{}", Colour::Green.paint("🩺 Scanning TCCT Instances..."));

                let sub_setting = SymbolicExecutorSetting {
                    prime: BigInt::from_str(&user_input.debug_prime()).unwrap(),
                    propagate_substitution: user_input.flag_propagate_substitution,
                    skip_initialization_blocks: true,
                    off_trace: true,
                    keep_track_constraints: false,
                    keep_track_unrolled_offset: false,
                };
                let mut sub_sexe = SymbolicExecutor::new(&mut sexe.symbolic_library, &sub_setting);

                let mut main_template_id = "";
                let mut template_param_names = Vec::new();
                let mut template_param_values = Vec::new();
                match &program_archive.initial_template_call {
                    Expression::Call { id, args, .. } => {
                        main_template_id = id;
                        let template = program_archive.templates[id].clone();
                        if !user_input.flag_symbolic_template_params {
                            template_param_names = template.get_name_of_params().clone();
                            template_param_values = args.clone();
                            sub_sexe.feed_arguments(template.get_name_of_params(), args);
                        }
                    }
                    _ => unimplemented!(),
                }

                let verification_setting = VerificationSetting {
                    id: main_template_id.to_string(),
                    prime: BigInt::from_str(&user_input.debug_prime()).unwrap(),
                    quick_mode: &*user_input.search_mode == "quick",
                    progress_interval: 10000,
                    template_param_names: template_param_names,
                    template_param_values: template_param_values,
                };

                for s in &sexe.symbolic_store.final_states {
                    let counterexample = match &*user_input.search_mode {
                        "quick" => brute_force_search(
                            &mut sub_sexe,
                            &s.trace_constraints.clone(),
                            &s.side_constraints.clone(),
                            &verification_setting,
                        ),
                        "full" => brute_force_search(
                            &mut sub_sexe,
                            &s.trace_constraints.clone(),
                            &s.side_constraints.clone(),
                            &verification_setting,
                        ),
                        _ => panic!(
                            "search_mode={} is not supported",
                            user_input.search_mode.to_string()
                        ),
                    };
                    if counterexample.is_some() {
                        is_safe = false;
                        println!(
                            "{}",
                            counterexample
                                .unwrap()
                                .lookup_fmt(&sub_sexe.symbolic_library.id2name)
                        );
                        break;
                    }
                }
            }

            println!(
                "{}",
                Colour::Green
                    .paint("╔═══════════════════════════════════════════════════════════════╗")
            );
            println!(
                "{}",
                Colour::Green
                    .paint("║                        TCCT Report                            ║")
            );
            println!(
                "{}",
                Colour::Green
                    .paint("╚═══════════════════════════════════════════════════════════════╝")
            );
            println!("{}", Colour::Cyan.bold().paint("📊 Execution Summary:"));
            println!(" ├─ Prime Number      : {}", user_input.debug_prime());
            println!(
                " ├─ Total Paths       : {}",
                sexe.symbolic_store.final_states.len()
            );
            println!(
                " ├─ Compression Rate  : {:.2}% ({}/{})",
                (ss.total_constraints as f64 / ts.total_constraints as f64) * 100 as f64,
                ss.total_constraints,
                ts.total_constraints
            );
            println!(
                " ├─ Verification      : {}",
                if is_safe {
                    Colour::Green.bold().paint("🆗 No Counter Example Found")
                } else {
                    Colour::Red.bold().paint("💥 NOT SAFE 💥")
                }
            );
            println!(" └─ Execution Time    : {:?}", start_time.elapsed());

            if user_input.flag_printout_stats {
                println!(
                    "\n{}",
                    Colour::Yellow
                        .bold()
                        .paint("🪶 Stats of Trace Constraint ══════════════════════")
                );
                print_constraint_summary_statistics_pretty(&ts);
                println!(
                    "\n{}",
                    Colour::Yellow
                        .bold()
                        .paint("⛓️ Stats of Side Constraint ══════════════════════")
                );
                print_constraint_summary_statistics_pretty(&ss);
            }
            println!(
                "{}",
                Colour::Green
                    .paint("════════════════════════════════════════════════════════════════")
            );
        }
        _ => {
            warn!("Cannot Find Main Call");
        }
    }

    Result::Ok(())
}
