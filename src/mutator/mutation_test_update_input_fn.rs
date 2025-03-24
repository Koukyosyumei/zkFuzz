use num_bigint_dig::BigInt;
use rand::rngs::StdRng;
use rand::Rng;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{OwnerName, SymbolicName};

use crate::mutator::mutation_config::MutationConfig;
use crate::mutator::mutation_test_crossover_fn::random_crossover;
use crate::mutator::mutation_test_trace_selection_fn::roulette_selection;
use crate::mutator::mutation_utils::draw_bigint_with_probabilities;
use crate::mutator::utils::BaseVerificationConfig;

/// Updates the input population with randomly generated samples.
///
/// This function initializes the input population by generating random values for each input
/// variable. The random values are drawn based on the specified value ranges and probabilities
/// in the mutation configuration.
///
/// # Parameters
/// - `_sexe`: A mutable reference to the symbolic executor. Not used in this implementation.
/// - `input_variables`: A slice of symbolic names representing the input variables.
/// - `inputs_population`: A mutable vector of hash maps representing the current input population.
///   This will be cleared and replaced with the new randomly generated population.
/// - `_base_config`: A reference to the base verification configuration. Not used in this implementation.
/// - `mutation_config`: The configuration that defines mutation parameters, including population size
///   and random value ranges.
/// - `rng`: A mutable reference to the random number generator.
///
/// # Behavior
/// The function creates a new population of inputs, with each input consisting of values
/// randomly sampled according to the mutation configuration. The existing input population is replaced
/// with the new one.
pub fn update_input_population_with_random_sampling(
    _sexe: &mut SymbolicExecutor,
    input_variables: &[SymbolicName],
    inputs_population: &mut Vec<FxHashMap<SymbolicName, BigInt>>,
    _inputs_population_score: &Vec<BigInt>,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    let mut new_inputs_population: Vec<_> = (0..mutation_config.input_population_size)
        .map(|_| {
            input_variables
                .iter()
                .map(|var| {
                    (
                        var.clone(),
                        draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
                    )
                })
                .collect::<FxHashMap<SymbolicName, BigInt>>()
        })
        .collect();
    inputs_population.clear();
    inputs_population.append(&mut new_inputs_population);
}

pub fn update_input_population_with_fitness_score(
    sexe: &mut SymbolicExecutor,
    input_variables: &[SymbolicName],
    inputs_population: &mut Vec<FxHashMap<SymbolicName, BigInt>>,
    inputs_population_score: &Vec<BigInt>,
    base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    if inputs_population.is_empty() {
        update_input_population_with_random_sampling(
            sexe,
            input_variables,
            inputs_population,
            inputs_population_score,
            base_config,
            mutation_config,
            rng,
        );
    }
    let mut updated_inputs_population = (0..mutation_config.input_population_size)
        .map(|_| {
            let parent1 = roulette_selection(inputs_population, inputs_population_score, rng);
            let parent2 = roulette_selection(inputs_population, inputs_population_score, rng);
            let mut child = if rng.gen::<f64>() < mutation_config.crossover_rate {
                random_crossover(&parent1, &parent2, rng)
            } else {
                parent1.clone()
            };
            let mut keys: Vec<_> = child.keys().cloned().collect();
            keys.sort();
            for k in keys.iter() {
                //let val = child.get(k).unwrap();
                if rng.gen::<f64>() < mutation_config.mutation_rate {
                    //*val = draw_bigint_with_probabilities(&mutation_config, rng).unwrap();
                    child.insert(
                        k.clone().clone(),
                        draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
                    );
                }
            }
            child
        })
        .collect::<Vec<_>>();
    inputs_population.clear();
    inputs_population.append(&mut updated_inputs_population);
}

/// Evaluates the coverage achieved by a given set of inputs.
///
/// This function runs the symbolic executor with the provided inputs and measures
/// the coverage based on the paths explored during execution.
///
/// # Parameters
/// - `sexe`: A mutable reference to the symbolic executor used for evaluation.
/// - `inputs`: A reference to a hash map representing the input values to evaluate.
/// - `base_config`: A reference to the base verification configuration containing execution parameters.
///
/// # Returns
/// The number of coverage points (e.g., branches or paths) achieved by the inputs.
///
/// # Behavior
/// The symbolic executor is cleared and set up for coverage tracking. The inputs are fed into the executor,
/// and the coverage is recorded. The coverage tracking is then disabled, and the coverage count is returned.
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

/// Updates the input population to maximize coverage.
///
/// This function uses a combination of random sampling, mutation, and crossover techniques
/// to evolve the input population towards achieving higher coverage. It iteratively refines
/// the population by evaluating and retaining inputs that increase the coverage.
///
/// # Parameters
/// - `sexe`: A mutable reference to the symbolic executor used for coverage evaluation.
/// - `input_variables`: A slice of symbolic names representing the input variables.
/// - `inputs_population`: A mutable vector of hash maps representing the current input population.
///   This will be updated to contain inputs that maximize coverage.
/// - `base_config`: A reference to the base verification configuration containing execution parameters.
/// - `mutation_config`: The configuration that defines mutation parameters, including population size,
///   mutation rates, and random value ranges.
/// - `rng`: A mutable reference to the random number generator.
///
/// # Behavior
/// 1. Initializes the population with random inputs.
/// 2. Evaluates each input for coverage and retains those that increase coverage.
/// 3. Iteratively performs mutations and crossovers on the population to explore new inputs,
///    retaining inputs that further increase coverage.
/// 4. The process stops when the population reaches the maximum size or the specified number
///    of iterations is completed.
pub fn update_input_population_with_coverage_maximization(
    sexe: &mut SymbolicExecutor,
    input_variables: &[SymbolicName],
    inputs_population: &mut Vec<FxHashMap<SymbolicName, BigInt>>,
    inputs_population_score: &Vec<BigInt>,
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
        inputs_population_score,
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
                    let mutation = draw_bigint_with_probabilities(&mutation_config, rng).unwrap();
                    new_input.insert(var.clone(), mutation);
                } else {
                    // Mutate each input variable with a small probability
                    for var in input_variables {
                        if rng.gen::<bool>() {
                            let mutation =
                                draw_bigint_with_probabilities(&mutation_config, rng).unwrap();
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
