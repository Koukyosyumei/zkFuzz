mod utils;

use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;
use num_traits::identities::Zero;
use num_traits::One;

use program_structure::ast::{Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode};

use tcct::executor::debug_ast::{
    DebuggableExpressionInfixOpcode, DebuggableExpressionPrefixOpcode,
};
use tcct::executor::symbolic_execution::SymbolicExecutor;
use tcct::executor::symbolic_setting::{
    get_default_setting_for_concrete_execution, get_default_setting_for_symbolic_execution,
};
use tcct::executor::symbolic_value::{OwnerName, SymbolicAccess, SymbolicName, SymbolicValue};
use tcct::solver::unused_outputs::check_unused_outputs;
use tcct::solver::utils::BaseVerificationConfig;

use tcct::solver::mutation_config::load_config_from_json;
use tcct::solver::mutation_test_crossover_fn::random_crossover;
use tcct::solver::mutation_test_evolution_fn::simple_evolution;
use tcct::solver::mutation_test_trace_fitness_fn::evaluate_trace_fitness_by_error;
use tcct::solver::mutation_test_trace_initialization_fn::initialize_population_with_random_constant_replacement;
use tcct::solver::mutation_test_trace_mutation_fn::mutate_trace_with_random_constant_replacement;
use tcct::solver::mutation_test_trace_selection_fn::roulette_selection;
use tcct::solver::mutation_test_update_input_fn::update_input_population_with_random_sampling;
use tcct::solver::{brute_force::brute_force_search, mutation_test::mutation_test_search};

use crate::utils::{execute, prepare_symbolic_library};

#[test]
fn test_iszero_vuln() {
    let path = "./tests/sample/test_iszero_vuln.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime.clone(), false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let mut main_template_name = "";
    let mut template_param_names = Vec::new();
    let mut template_param_values = Vec::new();
    match &program_archive.initial_template_call {
        Expression::Call { id, args, .. } => {
            main_template_name = id;
            let template = program_archive.templates[id].clone();
            template_param_names = template.get_name_of_params().clone();
            template_param_values = args.clone();
        }
        _ => unimplemented!(),
    }

    let verification_base_config = BaseVerificationConfig {
        target_template_name: main_template_name.to_string(),
        prime: prime.clone(),
        range: prime.clone(),
        quick_mode: false,
        heuristics_mode: false,
        progress_interval: 10000,
        template_param_names: template_param_names,
        template_param_values: template_param_values,
    };

    let subse_base_config = get_default_setting_for_concrete_execution(prime, false);
    let mut conc_executor = SymbolicExecutor::new(&mut sexe.symbolic_library, &subse_base_config);
    conc_executor.feed_arguments(
        &verification_base_config.template_param_names,
        &verification_base_config.template_param_values,
    );

    let mutation_config = load_config_from_json("./tests/parameters/test.json").unwrap();

    let result = mutation_test_search(
        &mut conc_executor,
        &sexe.cur_state.symbolic_trace.clone(),
        &sexe.cur_state.side_constraints.clone(),
        &verification_base_config,
        &mutation_config,
        initialize_population_with_random_constant_replacement,
        update_input_population_with_random_sampling,
        evaluate_trace_fitness_by_error,
        simple_evolution,
        mutate_trace_with_random_constant_replacement,
        random_crossover,
        roulette_selection,
    );

    assert!(result.counter_example.is_some());
}
