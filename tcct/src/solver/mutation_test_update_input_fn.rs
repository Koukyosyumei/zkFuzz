
use colored::Colorize;
use num_bigint_dig::BigInt;
use rand::rngs::StdRng;
use rand::Rng;
use rustc_hash::FxHashMap;


use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{OwnerName, SymbolicName};

use crate::solver::mutation_config::MutationConfig;
use crate::solver::mutation_test_crossover_fn::random_crossover;
use crate::solver::mutation_utils::draw_random_constant;
use crate::solver::utils::BaseVerificationConfig;

pub fn update_input_population_with_random_sampling(
    _sexe: &mut SymbolicExecutor,
    input_variables: &[SymbolicName],
    inputs_population: &mut Vec<FxHashMap<SymbolicName, BigInt>>,
    base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    let mut new_inputs_population: Vec<_> = (0..mutation_config.input_population_size)
        .map(|_| {
            input_variables
                .iter()
                .map(|var| (var.clone(), draw_random_constant(base_config, rng)))
                .collect::<FxHashMap<SymbolicName, BigInt>>()
        })
        .collect();
    inputs_population.clear();
    inputs_population.append(&mut new_inputs_population);
}

pub fn evaluate_coverage(
    sexe: &mut SymbolicExecutor,
    inputs: &FxHashMap<SymbolicName, BigInt>,
    base_config: &BaseVerificationConfig,
) -> usize {
    sexe.clear();
    sexe.turn_on_coverage_tracking();
    sexe.cur_state.add_owner(&OwnerName {
        id: sexe.symbolic_library.name2id["main"],
        counter: 0,
        access: None,
    });
    sexe.feed_arguments(
        &base_config.template_param_names,
        &base_config.template_param_values,
    );
    sexe.concrete_execute(&base_config.target_template_name, inputs);
    sexe.record_path();
    sexe.turn_off_coverage_tracking();
    sexe.coverage_count()
}

pub fn update_input_population_with_coverage_maximization(
    sexe: &mut SymbolicExecutor,
    input_variables: &[SymbolicName],
    inputs_population: &mut Vec<FxHashMap<SymbolicName, BigInt>>,
    base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    sexe.clear_coverage_tracker();
    let mut total_coverage = 0_usize;
    inputs_population.clear();

    let mut initial_input_population = Vec::new();
    update_input_population_with_random_sampling(
        sexe,
        input_variables,
        &mut initial_input_population,
        &base_config,
        &mutation_config,
        rng,
    );

    for input in &initial_input_population {
        let new_coverage = evaluate_coverage(sexe, &input, base_config);
        if new_coverage > total_coverage {
            inputs_population.push(input.clone());
            total_coverage = new_coverage;
        }
    }

    for _ in 0..mutation_config.input_generation_max_iteration {
        let mut new_inputs_population = Vec::new();

        // Iterate through the population and attempt mutations
        for input in inputs_population.iter() {
            let mut new_input = input.clone();

            if rng.gen::<f64>() < mutation_config.input_generation_crossover_rate {
                // Crossover
                let other = inputs_population[rng.gen_range(0, inputs_population.len())].clone();
                new_input = random_crossover(input, &other, rng);
            }
            if rng.gen::<f64>() < mutation_config.input_generation_mutation_rate {
                if rng.gen::<f64>() < mutation_config.input_generation_singlepoint_mutation_rate {
                    // Mutate only one input variable
                    let var = &input_variables[rng.gen_range(0, input_variables.len())];
                    let mutation = draw_random_constant(base_config, rng);
                    new_input.insert(var.clone(), mutation);
                } else {
                    // Mutate each input variable with a small probability
                    for var in input_variables {
                        // rng.gen_bool(0.2)
                        if rng.gen::<bool>() {
                            let mutation = draw_random_constant(base_config, rng);
                            new_input.insert(var.clone(), mutation);
                        }
                    }
                }
            }

            // Evaluate the new input
            let new_coverage = evaluate_coverage(sexe, &new_input, base_config);
            if new_coverage > total_coverage {
                new_inputs_population.push(new_input);
                total_coverage = new_coverage;
            }
        }
        inputs_population.append(&mut new_inputs_population);

        if inputs_population.len() > mutation_config.input_population_size {
            break;
        }
    }
}
