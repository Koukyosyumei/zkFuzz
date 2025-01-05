//mod execution_user;
mod executor;
mod input_user;
mod parser_user;
mod solver;
mod stats;
mod type_analysis_user;

use std::env;
use std::str::FromStr;
use std::time;

use colored::Colorize;
use env_logger;
use input_user::Input;
use log::{info, warn};
use num_bigint_dig::BigInt;
use rustc_hash::FxHashMap;

use program_structure::ast::Expression;
use program_structure::program_archive::ProgramArchive;

use executor::symbolic_execution::{SymbolicExecutor, SymbolicExecutorSetting};
use executor::symbolic_value::{OwnerName, SymbolicLibrary};
use solver::{
    brute_force::brute_force_search, mutation_test::mutation_test_search,
    unused_outputs::check_unused_outputs, utils::VerificationSetting,
};
use stats::ast_stats::ASTStats;
use stats::symbolic_stats::{print_constraint_summary_statistics_pretty, ConstraintStatistics};

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

    eprintln!("{}", logo.bright_cyan().bold());
    eprintln!("{}", "Welcome to the TCCT Debugging Tool".green().bold());
    eprintln!(
        "{}",
        "════════════════════════════════════════════════════════════════".green()
    );
}

fn main() {
    display_tcct_logo();

    let result = start();
    if result.is_err() {
        eprintln!("{}", "previous errors were found".red());
        std::process::exit(1);
    } else {
        eprintln!("{}", "Everything went okay".green());
        //std::process::exit(0);
    }
}

fn show_stats(program_archive: &ProgramArchive) {
    println!("template_name,num_statements,num_variables,num_if_then_else,num_while,num_constraint_equality,num_assign_var,num_assign_constraint_signal,num_assign_signal,avg_loc_constraint_equality,avg_loc_assign_constraint_signal,avg_loc_assign_signal");
    for (k, v) in program_archive.templates.clone().into_iter() {
        let mut ass = ASTStats::default();
        ass.collect_stats(v.get_body());
        println!("{},{}", k, ass.get_csv());
    }
}

fn start() -> Result<(), ()> {
    //use compilation_user::CompilerConfig;

    let user_input = Input::new()?;
    let mut program_archive = parser_user::parse_project(&user_input)?;
    type_analysis_user::analyse_project(&mut program_archive)?;

    if user_input.show_stats_of_ast {
        show_stats(&program_archive);
        return Result::Ok(());
    }

    env_logger::init();

    let mut symbolic_library = SymbolicLibrary {
        template_library: FxHashMap::default(),
        name2id: FxHashMap::default(),
        id2name: FxHashMap::default(),
        function_library: FxHashMap::default(),
        function_counter: FxHashMap::default(),
    };

    println!("{}", "🧩 Parsing Templates...".green());
    for (k, v) in program_archive.templates.clone().into_iter() {
        let body = v.get_body().clone();
        symbolic_library.register_template(k.clone(), &body.clone(), v.get_name_of_params());

        if user_input.flag_printout_ast {
            println!(
                "{}{} {}{}",
                BACK_GRAY_SCRIPT_BLACK, "🌳 AST Tree for", k, RESET
            );
            println!(
                "{}",
                symbolic_library.template_library[&symbolic_library.name2id[&k]]
                    .body
                    .iter()
                    .map(|b| b.lookup_fmt(&symbolic_library.id2name, 0))
                    .collect::<Vec<_>>()
                    .join("")
            );
        }
    }

    println!("{}", "⚙️ Parsing Function...".green());
    for (k, v) in program_archive.functions.clone().into_iter() {
        let body = v.get_body().clone();
        symbolic_library.register_function(k.clone(), body.clone(), v.get_name_of_params());

        if user_input.flag_printout_ast {
            println!(
                "{}{} {}{}",
                BACK_GRAY_SCRIPT_BLACK, "🌴 AST Tree for", k, RESET
            );
            println!(
                "{}",
                symbolic_library.function_library[&symbolic_library.name2id[&k]]
                    .body
                    .iter()
                    .map(|b| b.lookup_fmt(&symbolic_library.id2name, 0))
                    .collect::<Vec<_>>()
                    .join("")
            );
        }
    }

    let setting = SymbolicExecutorSetting {
        prime: BigInt::from_str(&user_input.debug_prime()).unwrap(),
        skip_initialization_blocks: false,
        only_initialization_blocks: false,
        off_trace: false,
        keep_track_constraints: true,
        substitute_output: false,
        propagate_assignments: false,
    };
    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);

    match &program_archive.initial_template_call {
        Expression::Call { id, args, .. } => {
            let start_time = time::Instant::now();
            let template = program_archive.templates[id].clone();

            println!("{}", "🛒 Gathering Trace/Side Constraints...".green());

            sexe.symbolic_library
                .name2id
                .insert("main".to_string(), sexe.symbolic_library.name2id.len());
            sexe.symbolic_library
                .id2name
                .insert(sexe.symbolic_library.name2id["main"], "main".to_string());

            sexe.cur_state.add_owner(&OwnerName {
                name: sexe.symbolic_library.name2id["main"],
                counter: 0,
                access: None,
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
                "════════════════════════════════════════════════════════════════".green()
            );
            let mut ts = ConstraintStatistics::new();
            let mut ss = ConstraintStatistics::new();
            for c in &sexe.cur_state.trace_constraints {
                ts.update(c);
            }
            for c in &sexe.cur_state.side_constraints {
                ss.update(c);
            }
            info!(
                "Final State: {}",
                sexe.cur_state.lookup_fmt(&sexe.symbolic_library.id2name)
            );

            let mut is_safe = true;
            if user_input.search_mode != "none" {
                println!(
                    "{}",
                    "════════════════════════════════════════════════════════════════".green()
                );
                println!("{}", "🩺 Scanning TCCT Instances...".green());

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

                if let Some(counter_example_for_unused_outputs) =
                    check_unused_outputs(&mut sexe, &verification_setting)
                {
                    is_safe = false;
                    println!(
                        "{}",
                        counter_example_for_unused_outputs
                            .lookup_fmt(&sexe.symbolic_library.id2name)
                    );
                } else {
                    let sub_setting = SymbolicExecutorSetting {
                        prime: BigInt::from_str(&user_input.debug_prime()).unwrap(),
                        skip_initialization_blocks: true,
                        only_initialization_blocks: false,
                        off_trace: true,
                        keep_track_constraints: false,
                        substitute_output: true,
                        propagate_assignments: true,
                    };
                    let mut sub_sexe =
                        SymbolicExecutor::new(&mut sexe.symbolic_library, &sub_setting);
                    sub_sexe.feed_arguments(
                        &verification_setting.template_param_names,
                        &verification_setting.template_param_values,
                    );

                    let counterexample = match &*user_input.search_mode {
                        "quick" => brute_force_search(
                            &mut sub_sexe,
                            &sexe.cur_state.trace_constraints.clone(),
                            &sexe.cur_state.side_constraints.clone(),
                            &verification_setting,
                        ),
                        "full" => brute_force_search(
                            &mut sub_sexe,
                            &sexe.cur_state.trace_constraints.clone(),
                            &sexe.cur_state.side_constraints.clone(),
                            &verification_setting,
                        ),
                        "ga" => mutation_test_search(
                            &mut sub_sexe,
                            &sexe.cur_state.trace_constraints.clone(),
                            &sexe.cur_state.side_constraints.clone(),
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
                    }
                }
            }

            println!(
                "{}",
                "╔═══════════════════════════════════════════════════════════════╗".green()
            );
            println!(
                "{}",
                "║                        TCCT Report                            ║".green()
            );
            println!(
                "{}",
                "╚═══════════════════════════════════════════════════════════════╝".green()
            );
            println!("{}", "📊 Execution Summary:".cyan().bold());
            println!(" ├─ Prime Number      : {}", user_input.debug_prime());
            println!(
                " ├─ Compression Rate  : {:.2}% ({}/{})",
                (ss.total_constraints as f64 / ts.total_constraints as f64) * 100 as f64,
                ss.total_constraints,
                ts.total_constraints
            );
            println!(
                " ├─ Verification      : {}",
                if is_safe {
                    "🆗 No Counter Example Found".green().bold()
                } else {
                    "💥 NOT SAFE 💥".red().bold()
                }
            );
            println!(" └─ Execution Time    : {:?}", start_time.elapsed());

            if user_input.flag_printout_stats {
                println!(
                    "\n{}",
                    "🪶 Stats of Trace Constraint ══════════════════════"
                        .yellow()
                        .bold()
                );
                print_constraint_summary_statistics_pretty(&ts);
                println!(
                    "\n{}",
                    "⛓️ Stats of Side Constraint ══════════════════════"
                        .yellow()
                        .bold()
                );
                print_constraint_summary_statistics_pretty(&ss);
            }
            println!(
                "{}",
                "════════════════════════════════════════════════════════════════".green()
            );
        }
        _ => {
            warn!("Cannot Find Main Call");
        }
    }

    Result::Ok(())
}
