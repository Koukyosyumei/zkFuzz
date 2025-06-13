mod executor;
mod mutator;
mod stats;

mod input_user;
mod parser_user;
mod type_analysis_user;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::str::FromStr;
use std::time;

use colored::Colorize;
use env_logger;
use input_user::Input;
use log::{debug, info, warn};
use num_bigint_dig::BigInt;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::json;

use program_structure::ast::Expression;
use program_structure::program_archive::ProgramArchive;

use executor::symbolic_execution::SymbolicExecutor;
use executor::symbolic_setting::{
    get_default_setting_for_concrete_execution, get_default_setting_for_symbolic_execution,
};
use executor::symbolic_value::{OwnerName, SymbolicLibrary};

use mutator::mutation_config::load_config_from_json;
use mutator::mutation_test_crossover_fn::random_crossover;
use mutator::mutation_test_evolution_fn::simple_evolution;
use mutator::mutation_test_trace_fitness_fn::evaluate_trace_fitness_by_error;
use mutator::mutation_test_trace_initialization_fn::{
    initialize_population_with_constant_replacement,
    initialize_population_with_operator_or_const_replacement,
    initialize_population_with_operator_or_const_replacement_or_addition,
    initialize_population_with_operator_or_const_replacement_or_deletion,
};
use mutator::mutation_test_trace_mutation_fn::{
    mutate_trace_with_constant_replacement, mutate_trace_with_operator_or_const_replacement,
    mutate_trace_with_operator_or_const_replacement_or_addition,
    mutate_trace_with_operator_or_const_replacement_or_deletion,
};
use mutator::mutation_test_trace_selection_fn::roulette_selection;
use mutator::mutation_test_update_input_fn::{
    update_input_population_with_coverage_maximization, update_input_population_with_fitness_score,
    update_input_population_with_random_sampling,
};
use mutator::{
    brute_force::brute_force_search, mutation_test::mutation_test_search,
    unused_outputs::check_unused_outputs, utils::BaseVerificationConfig,
};

use stats::ast_stats::ASTStats;
use stats::symbolic_stats::{
    print_constraint_summary_statistics_csv, print_constraint_summary_statistics_pretty,
    ConstraintStatistics,
};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const RESET: &str = "\x1b[0m";
const BACK_GRAY_SCRIPT_BLACK: &str = "\x1b[30;100m"; //94

fn display_zkfuzz_header() {
    let logo = r#"
           zkFuzz      
      ZK Circuit Fuzzer
    "#;

    eprintln!("{}", logo.bright_cyan().bold());
    eprintln!("{}", "Welcome to the zkFuzz".green().bold());
    eprintln!("{}", "══════════════════════════════════".green());
}

fn read_file_to_lines(file_path: &str) -> io::Result<Vec<String>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    Ok(lines)
}

fn main() {
    display_zkfuzz_header();

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
    let start_time = time::Instant::now();
    //use compilation_user::CompilerConfig;

    let user_input = Input::new()?;
    let mut program_archive = parser_user::parse_project(&user_input)?;
    type_analysis_user::analyse_project(&mut program_archive)?;

    if user_input.show_stats_of_ast {
        show_stats(&program_archive);
        return Result::Ok(());
    }

    env_logger::init();

    eprintln!("{}", "🧾 Loading Whitelists...".green());
    let whitelist = if user_input.path_to_whitelist() == "none" {
        FxHashSet::from_iter(["IsZero".to_string(), "Num2Bits".to_string()])
    } else {
        FxHashSet::from_iter(
            read_file_to_lines(&&&user_input.path_to_whitelist())
                .unwrap()
                .into_iter(),
        )
    };

    let mut symbolic_library = SymbolicLibrary {
        template_library: FxHashMap::default(),
        name2id: FxHashMap::default(),
        id2name: FxHashMap::default(),
        function_library: FxHashMap::default(),
        function_counter: FxHashMap::default(),
    };

    eprintln!("{}", "🧩 Parsing Templates...".green());
    let mut templates_names = program_archive
        .templates
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    templates_names.sort();
    for k in templates_names {
        let v = program_archive.templates.get(&k).unwrap();
        let body = v.get_body().clone();
        symbolic_library.register_template(
            k.clone(),
            &body.clone(),
            v.get_name_of_params(),
            &whitelist,
            user_input.lessthan_dissabled_flag,
        );

        if user_input.flag_printout_ast {
            eprintln!(
                "{}{} {}{}",
                BACK_GRAY_SCRIPT_BLACK, "🌳 AST Tree for", k, RESET
            );
            eprintln!(
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

    eprintln!("{}", "⚙️ Parsing Function...".green());
    let mut function_names = program_archive
        .functions
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    function_names.sort();
    for k in function_names {
        let v = program_archive.functions.get(&k).unwrap();
        let body = v.get_body().clone();
        symbolic_library.register_function(k.clone(), body.clone(), v.get_name_of_params());

        if user_input.flag_printout_ast {
            eprintln!(
                "{}{} {}{}",
                BACK_GRAY_SCRIPT_BLACK, "🌴 AST Tree for", k, RESET
            );
            eprintln!(
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

    let base_config = get_default_setting_for_symbolic_execution(
        BigInt::from_str(&user_input.debug_prime()).unwrap(),
        user_input.constraint_assert_dissabled_flag(),
    );
    let mut sym_executor = SymbolicExecutor::new(&mut symbolic_library, &base_config);

    match &program_archive.initial_template_call {
        Expression::Call { id, args, .. } => {
            let template = program_archive.templates[id].clone();

            eprintln!("{}", "🛒 Gathering Trace/Side Constraints...".green());

            sym_executor.symbolic_library.name2id.insert(
                "main".to_string(),
                sym_executor.symbolic_library.name2id.len(),
            );
            sym_executor.symbolic_library.id2name.insert(
                sym_executor.symbolic_library.name2id["main"],
                "main".to_string(),
            );

            sym_executor.cur_state.add_owner(&OwnerName {
                id: sym_executor.symbolic_library.name2id["main"],
                counter: 0,
                access: None,
            });
            sym_executor
                .cur_state
                .set_template_id(sym_executor.symbolic_library.name2id[id]);

            if !user_input.flag_symbolic_template_params {
                sym_executor.feed_arguments(template.get_name_of_params(), args);
            }

            let body = sym_executor.symbolic_library.template_library
                [&sym_executor.symbolic_library.name2id[id]]
                .body
                .clone();
            sym_executor.execute(&body, 0);

            eprintln!("{}", "══════════════════════════════════".green());
            let mut ts = ConstraintStatistics::new();
            let mut ss = ConstraintStatistics::new();
            for c in &sym_executor.cur_state.symbolic_trace {
                ts.update(c);
            }
            for c in &sym_executor.cur_state.side_constraints {
                ss.update(c);
            }
            debug!(
                "Final State: {}",
                sym_executor
                    .cur_state
                    .lookup_fmt(&sym_executor.symbolic_library.id2name)
            );

            let mut is_safe = true;
            if user_input.search_mode != "off" {
                eprintln!("{}", "══════════════════════════════════".green());
                eprintln!("{}", "🩺 Scanning TCCT Instances...".green());

                let (main_template_name, template_param_names, template_param_values) =
                    match &program_archive.initial_template_call {
                        Expression::Call { id, args, .. } => {
                            let template = &program_archive.templates[id];
                            (id, template.get_name_of_params().clone(), args.clone())
                        }
                        _ => unimplemented!(),
                    };

                let verification_base_config = BaseVerificationConfig {
                    target_template_name: main_template_name.to_string(),
                    prime: BigInt::from_str(&user_input.debug_prime()).unwrap(),
                    range: BigInt::from_str(&user_input.heuristics_range()).unwrap(),
                    quick_mode: &*user_input.search_mode == "quick",
                    heuristics_mode: &*user_input.search_mode == "heuristics",
                    progress_interval: 10000,
                    template_param_names: template_param_names,
                    template_param_values: template_param_values,
                };

                let mut new_base_config = base_config.clone();
                new_base_config.off_trace = true;
                sym_executor.setting = &new_base_config;

                let mut counter_example =
                    check_unused_outputs(&mut sym_executor, &verification_base_config);
                let mut auxiliary_result = json!({});
                if let Some(_) = &counter_example {
                    is_safe = false;
                } else {
                    let subse_base_config = get_default_setting_for_concrete_execution(
                        BigInt::from_str(&user_input.debug_prime()).unwrap(),
                        user_input.constraint_assert_dissabled_flag(),
                    );
                    let mut conc_executor = SymbolicExecutor::new(
                        &mut sym_executor.symbolic_library,
                        &subse_base_config,
                    );
                    conc_executor.feed_arguments(
                        &verification_base_config.template_param_names,
                        &verification_base_config.template_param_values,
                    );

                    counter_example = match &*user_input.search_mode() {
                        "quick" => brute_force_search(
                            &mut conc_executor,
                            &sym_executor.cur_state.symbolic_trace.clone(),
                            &sym_executor.cur_state.side_constraints.clone(),
                            &verification_base_config,
                        ),
                        "full" => brute_force_search(
                            &mut conc_executor,
                            &sym_executor.cur_state.symbolic_trace.clone(),
                            &sym_executor.cur_state.side_constraints.clone(),
                            &verification_base_config,
                        ),
                        "heuristics" => brute_force_search(
                            &mut conc_executor,
                            &sym_executor.cur_state.symbolic_trace.clone(),
                            &sym_executor.cur_state.side_constraints.clone(),
                            &verification_base_config,
                        ),
                        "ga" => {
                            let mutation_config =
                                load_config_from_json(&&user_input.path_to_mutation_setting())
                                    .unwrap();
                            info!("\n{}", mutation_config);

                            let trace_initialization_fn = match mutation_config.trace_mutation_method.as_str() {
                                "naive" => initialize_population_with_constant_replacement,
                                "constant" => initialize_population_with_constant_replacement,
                                "constant_operator" => initialize_population_with_operator_or_const_replacement,
                                "constant_operator_add" => initialize_population_with_operator_or_const_replacement_or_addition,
                                "constant_operator_delete" => initialize_population_with_operator_or_const_replacement_or_deletion,
                                _ => panic!("`trace_mutation_method` should be one of [`constant`, `constant_operator`, `constant_operator_add`, `constant_operator_delete`]")
                            };

                            let trace_mutation_fn = match mutation_config.trace_mutation_method.as_str() {
                                "naive" => mutate_trace_with_constant_replacement,
                                "constant" => mutate_trace_with_constant_replacement,
                                "constant_operator" => mutate_trace_with_operator_or_const_replacement,
                                "constant_operator_add" => mutate_trace_with_operator_or_const_replacement_or_addition,
                                "constant_operator_delete" => mutate_trace_with_operator_or_const_replacement_or_deletion,
                                _ => panic!("`trace_mutation_method` should be one of [`constant`, `constant_operator`, `constant_operator_add`, `constant_operator_delete`]")
                            };

                            let update_input_fn = match mutation_config
                                .input_initialization_method
                                .as_str()
                            {
                                "random" => update_input_population_with_random_sampling,
                                "fitness" => update_input_population_with_fitness_score,
                                "coverage" => update_input_population_with_coverage_maximization,
                                _ => panic!("`input_initialization_method` should be one of [`random`, `fitness`, `coverage`]")
                            };

                            let result = mutation_test_search(
                                &mut conc_executor,
                                &sym_executor.cur_state.symbolic_trace.clone(),
                                &sym_executor.cur_state.side_constraints.clone(),
                                &verification_base_config,
                                &mutation_config,
                                trace_initialization_fn,
                                update_input_fn,
                                evaluate_trace_fitness_by_error,
                                simple_evolution,
                                trace_mutation_fn,
                                random_crossover,
                                roulette_selection,
                            );
                            auxiliary_result["mutation_test_config"] =
                                serde_json::to_value(result.mutation_config)
                                    .expect("Failed to serialize to JSON");
                            auxiliary_result["mutation_test_log"] = json!({"random_seed":result.random_seed,"generation":result.generation, "fitness_score_log":result.fitness_score_log});
                            result.counter_example
                        }
                        _ => panic!(
                            "search_mode={} is not supported",
                            user_input.search_mode.to_string()
                        ),
                    };
                }
                if let Some(ce) = &counter_example {
                    is_safe = false;
                    if user_input.flag_save_output {
                        // Save the output as JSON
                        let ce_meta = FxHashMap::from_iter([
                            (
                                "0_target_path".to_string(),
                                user_input.input_file().to_string(),
                            ),
                            ("1_main_template".to_string(), id.to_string()),
                            ("2_search_mode".to_string(), user_input.search_mode()),
                            (
                                "3_execution_time".to_string(),
                                format!("{:?}", start_time.elapsed()),
                            ),
                            (
                                "4_git_hash_of_zkfuzz".to_string(),
                                format!("{}", option_env!("GIT_HASH").unwrap_or("unknown")),
                            ),
                        ]);

                        let mut json_output =
                            ce.to_json_with_meta(&sym_executor.symbolic_library.id2name, &ce_meta);
                        json_output["8_auxiliary_result"] = auxiliary_result;

                        let mut file_path = user_input.input_file().to_string();
                        file_path.push('_');
                        let random_string: String = thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(10)
                            .map(char::from)
                            .collect();
                        file_path.push_str(&random_string);
                        file_path.push_str("_counterexample.json");
                        eprintln!("{} {}", "💾 Saving the output to:", file_path.cyan(),);

                        let mut file = File::create(file_path).expect("Unable to create file");
                        let json_string = serde_json::to_string_pretty(&json_output).unwrap();
                        file.write_all(json_string.as_bytes())
                            .expect("Unable to write data");
                    } else {
                        eprintln!("{}", ce.lookup_fmt(&sym_executor.symbolic_library.id2name));
                    }
                }
            }

            eprintln!(
                "{}",
                "╔═══════════════════════════════════════════════════════════════╗".green()
            );
            eprintln!(
                "{}",
                "║                        zkFuzz Report                        ║".green()
            );
            eprintln!(
                "{}",
                "╚═══════════════════════════════════════════════════════════════╝".green()
            );
            eprintln!("{}", "📊 Execution Summary:".cyan().bold());
            eprintln!(" ├─ Prime Number      : {}", user_input.debug_prime());
            eprintln!(
                " ├─ Compression Rate  : {:.2}% ({}/{})",
                (ss.total_constraints as f64 / ts.total_constraints as f64) * 100 as f64,
                ss.total_constraints,
                ts.total_constraints
            );
            eprintln!(
                " ├─ Verification      : {}",
                if is_safe {
                    "🆗 No Counter Example Found".green().bold()
                } else {
                    "💥 NOT SAFE 💥".red().bold()
                }
            );
            eprintln!(" └─ Execution Time    : {:?}", start_time.elapsed());

            if user_input.flag_printout_stats {
                println!(
                    "\n{}",
                    "🪶 Stats of Symbolic Trace  ══════════════════════"
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
            } else if user_input.flag_printout_stats_csv {
                let headers = vec![
                    "Total_Constraints",
                    "Constant_Counts",
                    "Conditional_Counts",
                    "Array_Counts",
                    "Avg_Depth",
                    "Max_Depth",
                    "Count_Assign",
                    "Count_AssignEq",
                    "Count_AssignCall",
                    "Count_QuadZeroDiv",
                    "Count_Mul",
                    "Count_Div",
                    "Count_Add",
                    "Count_Sub",
                    "Count_Pow",
                    "Count_IntDiv",
                    "Count_Mod",
                    "Count_ShiftL",
                    "Count_ShiftR",
                    "Count_LesserEq",
                    "Count_GreaterEq",
                    "Count_Lesser",
                    "Count_Greater",
                    "Count_Eq",
                    "Count_NotEq",
                    "Count_BoolOr",
                    "Count_BoolAnd",
                    "Count_BitOr",
                    "Count_BitAnd",
                    "Count_BitXor",
                    "Number_of_Variable",
                    "Variable_Avg_Count",
                    "Variable_Max_Count",
                    "Function_Avg_Count",
                    "Function_Max_Count",
                ];
                println!("{}", headers.join(","));
                print_constraint_summary_statistics_csv(&ts);
                print_constraint_summary_statistics_csv(&ss);
            }
            eprintln!(
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
