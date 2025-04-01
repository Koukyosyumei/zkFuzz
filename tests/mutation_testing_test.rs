mod utils;

use std::str::FromStr;

use num_bigint_dig::BigInt;

use program_structure::ast::Expression;

use zkfuzz::executor::symbolic_execution::SymbolicExecutor;
use zkfuzz::executor::symbolic_setting::{
    get_default_setting_for_concrete_execution, get_default_setting_for_symbolic_execution,
};
use zkfuzz::mutator::utils::{
    BaseVerificationConfig, CounterExample, UnderConstrainedType, VerificationResult,
};

use zkfuzz::mutator::mutation_config::load_config_from_json;
use zkfuzz::mutator::mutation_test::{mutation_test_search, MutationTestResult};
use zkfuzz::mutator::mutation_test_crossover_fn::random_crossover;
use zkfuzz::mutator::mutation_test_evolution_fn::simple_evolution;
use zkfuzz::mutator::mutation_test_trace_fitness_fn::evaluate_trace_fitness_by_error;
use zkfuzz::mutator::mutation_test_trace_initialization_fn::initialize_population_with_operator_or_const_replacement;
use zkfuzz::mutator::mutation_test_trace_mutation_fn::mutate_trace_with_operator_or_const_replacement;
use zkfuzz::mutator::mutation_test_trace_selection_fn::roulette_selection;
use zkfuzz::mutator::mutation_test_update_input_fn::{
    update_input_population_with_fitness_score, update_input_population_with_random_sampling,
};

use crate::utils::{execute, prepare_symbolic_library};

fn conduct_mutation_testing(path: String, update_input_method: String) -> MutationTestResult {
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime.clone(), false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

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

    let update_func = if update_input_method == "fitness" {
        update_input_population_with_fitness_score
    } else {
        update_input_population_with_random_sampling
    };

    mutation_test_search(
        &mut conc_executor,
        &sexe.cur_state.symbolic_trace.clone(),
        &sexe.cur_state.side_constraints.clone(),
        &verification_base_config,
        &mutation_config,
        initialize_population_with_operator_or_const_replacement,
        update_func,
        evaluate_trace_fitness_by_error,
        simple_evolution,
        mutate_trace_with_operator_or_const_replacement,
        random_crossover,
        roulette_selection,
    )
}

#[test]
fn test_vuln_iszero() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_vuln_iszero.circom".to_string(),
        "random".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic(..)),
            ..
        })
    ));
}

#[test]
fn test_vuln_iszero_fitness() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_vuln_iszero.circom".to_string(),
        "fitness".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic(..)),
            ..
        })
    ));
}

#[test]
fn test_vuln_average() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_vuln_average.circom".to_string(),
        "random".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic(..)),
            ..
        })
    ));
}

#[test]
fn test_vuln_scholarshipcheck() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_vuln_scholarshipcheck.circom".to_string(),
        "random".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic(..)),
            ..
        })
    ));
}

#[test]
fn test_vuln_rshift1() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_vuln_rshift1.circom".to_string(),
        "random".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic(..)),
            ..
        })
    ));
}

#[test]
fn test_lessthan() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_lessthan.circom".to_string(),
        "random".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::UnexpectedInput(..)),
            ..
        })
    ));
}

#[test]
fn test_input_subscript_1d() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_input_subscript_1d.circom".to_string(),
        "random".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic(..)),
            ..
        })
    ));
}

#[test]
fn test_input_subscript_2d() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_input_subscript_2d.circom".to_string(),
        "random".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic(..)),
            ..
        })
    ));
}

#[test]
fn test_montgomerydouble() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_montgomerydouble.circom".to_string(),
        "random".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic(..)),
            ..
        })
    ));
}

#[test]
fn test_decreasing_for_loop() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_decreasing_for_loop.circom".to_string(),
        "random".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic(..)),
            ..
        })
    ));
}

#[test]
fn test_array_template_parameter() {
    let result = conduct_mutation_testing(
        "./tests/sample/test_array_template_parameter.circom".to_string(),
        "random".to_string(),
    );

    assert!(matches!(
        result.counter_example,
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic(..)),
            ..
        })
    ));
}
