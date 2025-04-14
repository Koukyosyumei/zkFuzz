use std::collections::HashSet;
use std::io;
use std::io::Write;

use colored::Colorize;
use log::info;
use num_bigint_dig::BigInt;
use num_traits::Zero;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_state::{SymbolicConstraints, SymbolicTrace};
use crate::executor::symbolic_value::{
    extract_variables, QuadraticPoly, SymbolicName, SymbolicValue,
};

use crate::executor::utils::solve_quadratic_modulus_equation;
use crate::mutator::mutation_config::MutationConfig;
use crate::mutator::utils::{
    evaluate_symbolic_value, gather_potential_zero_division, gather_runtime_mutable_inputs,
    is_containing_binary_check, BaseVerificationConfig, CounterExample, Direction,
};

pub struct MutationTestResult {
    pub random_seed: u64,
    pub mutation_config: MutationConfig,
    pub counter_example: Option<CounterExample>,
    pub generation: usize,
    pub fitness_score_log: Vec<BigInt>,
}

pub type Gene = FxHashMap<usize, SymbolicValue>;

/// Conducts a mutation-based search to find counterexamples for symbolic trace verification.
///
/// This function applies a genetic algorithm-like approach to search for counterexamples that
/// violate the provided symbolic constraints. It initializes a population of symbolic traces,
/// evolves them through mutation and crossover, evaluates their fitness, and selects the best
/// candidates iteratively until a counterexample is found or a maximum number of generations is reached.
///
/// # Parameters
/// - `sexe`: A mutable reference to the symbolic executor that executes symbolic traces.
/// - `symbolic_trace`: The symbolic trace to be verified.
/// - `side_constraints`: Additional symbolic constraints that must be satisfied.
/// - `base_config`: The base configuration containing general verification settings.
/// - `mutation_config`: The mutation-specific configuration, including parameters such as
///   population size, mutation rate, and maximum number of generations.
/// - `trace_initialization_fn`: A function that initializes the population of symbolic traces.
/// - `update_input_fn`: A function that updates the input population at regular intervals.
/// - `trace_fitness_fn`: A function that evaluates the fitness of a given trace and determines if it violates constraints.
/// - `trace_evolution_fn`: A function that handles the evolution of the trace population by applying
///   mutation, crossover, and selection.
/// - `trace_mutation_fn`: A function that applies mutation to a trace.
/// - `trace_crossover_fn`: A function that combines two parent traces to produce an offspring trace.
/// - `trace_selection_fn`: A function that selects traces from the population based on their fitness scores.
///
/// # Returns
/// A `MutationTestResult` containing:
/// - `random_seed`: The seed used for the random number generator.
/// - `mutation_config`: A copy of the mutation configuration.
/// - `counter_example`: An optional counterexample found during the search.
/// - `generation`: The generation in which the counterexample was found, or the maximum number of generations if no solution was found.
/// - `fitness_score_log`: A log of the best fitness scores across generations.
///
/// # Type Parameters
/// - `TraceInitializationFn`: A closure or function that initializes the population of traces.
/// - `UpdateInputFn`: A closure or function that updates the input population.
/// - `TraceFitnessFn`: A closure or function that evaluates the fitness of a symbolic trace.
/// - `TraceEvolutionFn`: A closure or function that handles trace population evolution.
/// - `TraceMutationFn`: A closure or function that mutates a trace.
/// - `TraceCrossoverFn`: A closure or function that performs crossover between two traces.
/// - `TraceSelectionFn`: A closure or function that selects traces from the population.
///
/// # Algorithm
/// 1. **Initialization**:
///    - Set the random seed.
///    - Identify mutable locations in the symbolic trace.
///    - Extract input variables and constraints.
///    - Initialize the population of symbolic traces.
///
/// 2. **Iterative Search**:
///    - Update the input population at regular intervals.
///    - Evolve the trace population using mutation, crossover, and selection.
///    - Evaluate the fitness of the population.
///    - If a counterexample is found, return it immediately.
///
/// 3. **Termination**:
///    - Stop after reaching the maximum number of generations.
///    - If no solution is found, return a result indicating failure.
///
/// # Notes
/// - This function assumes that all closures and functions provided as parameters are consistent with the structure of the symbolic execution process.
/// - The fitness function must be designed such that a fitness score of zero indicates a counterexample.
pub fn mutation_test_search<
    TraceInitializationFn,
    UpdateInputFn,
    TraceFitnessFn,
    TraceEvolutionFn,
    TraceMutationFn,
    TraceCrossoverFn,
    TraceSelectionFn,
>(
    sexe: &mut SymbolicExecutor,
    symbolic_trace: &SymbolicTrace,
    side_constraints: &SymbolicConstraints,
    base_config: &BaseVerificationConfig,
    base_mutation_config: &MutationConfig,
    trace_initialization_fn: TraceInitializationFn,
    update_input_fn: UpdateInputFn,
    trace_fitness_fn: TraceFitnessFn,
    trace_evolution_fn: TraceEvolutionFn,
    trace_mutation_fn: TraceMutationFn,
    trace_crossover_fn: TraceCrossoverFn,
    trace_selection_fn: TraceSelectionFn,
) -> MutationTestResult
where
    TraceInitializationFn: Fn(
        &[usize],
        usize,
        &SymbolicTrace,
        &BaseVerificationConfig,
        &MutationConfig,
        &mut StdRng,
    ) -> Vec<Gene>,
    UpdateInputFn: Fn(
        &mut SymbolicExecutor,
        &[SymbolicName],
        &mut Vec<FxHashMap<SymbolicName, BigInt>>,
        &Vec<BigInt>,
        &BaseVerificationConfig,
        &MutationConfig,
        &mut StdRng,
    ),
    TraceFitnessFn: Fn(
        &mut SymbolicExecutor,
        &BaseVerificationConfig,
        &MutationConfig,
        &SymbolicTrace,
        &SymbolicConstraints,
        &FxHashMap<usize, Direction>,
        &Gene,
        &Vec<FxHashMap<SymbolicName, BigInt>>,
        &mut Vec<BigInt>,
    ) -> (usize, BigInt, Option<CounterExample>, usize),
    TraceEvolutionFn: Fn(
        &[usize],
        &SymbolicTrace,
        &[Gene],
        &[BigInt],
        &BaseVerificationConfig,
        &MutationConfig,
        &mut StdRng,
        &TraceMutationFn,
        &TraceCrossoverFn,
        &TraceSelectionFn,
    ) -> Vec<Gene>,
    TraceMutationFn: Fn(
        &[usize],
        &SymbolicTrace,
        &mut Gene,
        &BaseVerificationConfig,
        &MutationConfig,
        &mut StdRng,
    ),
    TraceCrossoverFn: Fn(&Gene, &Gene, &mut StdRng) -> Gene,
    TraceSelectionFn: for<'a> Fn(&'a [Gene], &[BigInt], &mut StdRng) -> &'a Gene,
{
    let mut mutation_config = base_mutation_config.clone();

    // Set random seed
    let seed = if mutation_config.seed.is_zero() {
        let mut seed_rng = rand::thread_rng();
        seed_rng.gen()
    } else {
        mutation_config.seed
    };
    let mut rng = StdRng::seed_from_u64(seed);

    // Gather mutable locations
    let mut assign_pos = Vec::new();
    for (i, sv) in symbolic_trace.iter().enumerate() {
        match *sv.as_ref() {
            SymbolicValue::Assign(_, _, false, _) | SymbolicValue::AssignCall(_, _, true) => {
                assign_pos.push(i);
            }
            SymbolicValue::Assign(_, _, _, _) | SymbolicValue::AssignCall(_, _, _) => {
                if mutation_config.trace_mutation_method == "naive" {
                    assign_pos.push(i);
                }
            }
            _ => {}
        }
    }

    // Gather input variables
    let mut variables = extract_variables(symbolic_trace);
    variables.append(&mut extract_variables(side_constraints));
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();
    let mut unique_variables: Vec<SymbolicName> = variables_set.iter().cloned().collect();
    unique_variables.sort();
    let mut input_variables = Vec::new();
    for v in unique_variables.iter() {
        if v.owner.len() == 1
            && sexe.symbolic_library.template_library
                [&sexe.symbolic_library.name2id[&base_config.target_template_name]]
                .input_ids
                .contains(&v.id)
        {
            input_variables.push(v.clone());
        }
    }

    let dummy_runtime_mutable_positions = FxHashMap::default();
    let runtime_mutable_positions = if mutation_config.dissable_runtime_mutation_for_hash_check {
        FxHashMap::default()
    } else {
        gather_runtime_mutable_inputs(
            symbolic_trace,
            sexe.symbolic_library,
            &input_variables.iter().cloned().collect(),
        )
    };

    info!(
        "\n⚖️ Constraints Summary:
    ├─ #Trace Constraints : {}
    ├─ #Side Constraints  : {}
    ├─ #Input Variables   : {}
    └─ #Mutation Candidate: {}",
        symbolic_trace.len().to_string().bright_yellow(),
        side_constraints.len().to_string().bright_yellow(),
        input_variables.len().to_string().bright_yellow(),
        assign_pos.len().to_string().bright_yellow()
    );

    // Initial Pupulation of Mutated Inputs
    let mut trace_population = trace_initialization_fn(
        &assign_pos,
        mutation_config.program_population_size,
        &symbolic_trace,
        base_config,
        &mutation_config,
        &mut rng,
    );
    let mut fitness_scores =
        vec![-base_config.prime.clone(); mutation_config.program_population_size + 1];
    let mut fitness_scores_inputs =
        vec![-base_config.prime.clone(); mutation_config.input_population_size];
    let mut input_population = Vec::new();
    let mut fitness_score_log = if mutation_config.save_fitness_scores {
        Vec::with_capacity(mutation_config.max_generations)
    } else {
        Vec::new()
    };

    println!(
        "{} {}",
        "🎲 Random Seed:",
        seed.to_string().bold().bright_yellow(),
    );

    let mut binary_input_mode = false;
    let mut partial_binary_mode = false;
    let original_binary_mode_prob = mutation_config.binary_mode_prob;

    if is_containing_binary_check(&symbolic_trace, mutation_config.binary_mode_search_level) {
        info!("⚡ Binary check detected!");
        partial_binary_mode = true;
    }

    let potential_zero_div_positions = gather_potential_zero_division(symbolic_trace);
    let mut zero_div_cache = FxHashMap::default();

    for generation in 0..mutation_config.max_generations {
        if partial_binary_mode
            && 1 < generation
            && generation
                < (mutation_config.max_generations as f64
                    * mutation_config.binary_mode_warmup_round) as usize
        {
            mutation_config.binary_mode_prob = 1.0;
        } else {
            mutation_config.binary_mode_prob = original_binary_mode_prob;
        }

        // Generate input population for this generation
        if generation % mutation_config.input_update_interval == 0 {
            update_input_fn(
                sexe,
                &input_variables,
                &mut input_population,
                &fitness_scores_inputs,
                &base_config,
                &mutation_config,
                &mut rng,
            );
        }

        // Evolve the trace population
        if !trace_population.is_empty() {
            trace_population = trace_evolution_fn(
                &assign_pos,
                &symbolic_trace,
                &trace_population,
                &fitness_scores,
                base_config,
                &mutation_config,
                &mut rng,
                &trace_mutation_fn,
                &trace_crossover_fn,
                &trace_selection_fn,
            );
        }
        trace_population.push(FxHashMap::default());

        // zero-division-pattern
        if !potential_zero_div_positions.is_empty() {
            for inp in input_population.iter_mut() {
                if rng.gen::<f64>() < mutation_config.zero_div_attempt_prob {
                    zero_div_attempt(
                        inp,
                        sexe,
                        &mut zero_div_cache,
                        base_config,
                        &potential_zero_div_positions,
                        &input_variables.iter().cloned().collect(),
                        &mut rng,
                    );
                }
            }
        }

        // Evaluate the trace population
        let mut evaluations = Vec::new();
        let mut is_extincted_due_to_illegal_subscript = true;
        for individual in &trace_population {
            let fitness = trace_fitness_fn(
                sexe,
                &base_config,
                &mutation_config,
                symbolic_trace,
                side_constraints,
                if rng.gen::<f64>() < mutation_config.runtime_mutation_rate {
                    &dummy_runtime_mutable_positions
                } else {
                    &runtime_mutable_positions
                },
                individual,
                &input_population,
                &mut fitness_scores_inputs,
            );
            if fitness.1.is_zero() {
                evaluations.push(fitness);
                break;
            }
            is_extincted_due_to_illegal_subscript =
                is_extincted_due_to_illegal_subscript && fitness.3 == input_population.len();
            evaluations.push(fitness);
        }

        if !binary_input_mode
            && is_extincted_due_to_illegal_subscript
            && (!mutation_config.dissable_heuristic_for_invalid_array_subscript)
        {
            binary_input_mode = true;
            let mindim = if sexe.mindim >= std::usize::MAX {
                1
            } else {
                sexe.mindim
            };
            mutation_config.random_value_ranges = vec![(BigInt::from(0), BigInt::from(mindim))];
            mutation_config.random_value_probs = vec![1.0];
        }

        let mut evaluation_indices: Vec<usize> = (0..evaluations.len()).collect();
        evaluation_indices.sort_by(|&i, &j| evaluations[i].1.cmp(&evaluations[j].1));

        // Pick the best one
        let best_idx = evaluation_indices.last().unwrap();

        if evaluations[*best_idx].1.is_zero() {
            print!(
                "\r\x1b[2K🧬 Generation: {}/{} ({:.3})",
                generation, mutation_config.max_generations, 0
            );
            println!("\n    └─ Solution found in generation {}", generation);

            return MutationTestResult {
                random_seed: seed,
                mutation_config: mutation_config.clone(),
                counter_example: evaluations[*best_idx].2.clone(),
                generation: generation,
                fitness_score_log: fitness_score_log,
            };
        }

        // Extract the fitness scores
        if mutation_config.fitness_function != "const" {
            fitness_scores = evaluations.iter().map(|v| v.1.clone()).collect();
        }

        print!(
            "\r\x1b[2K🧬 Generation: {}/{} ({:.3})",
            generation, mutation_config.max_generations, fitness_scores[*best_idx]
        );
        io::stdout().flush().unwrap();

        if mutation_config.save_fitness_scores {
            fitness_score_log.push(fitness_scores[*best_idx].clone());
        }

        // Reset individuals with poor fitness score
        let new_trace_population = trace_initialization_fn(
            &assign_pos,
            mutation_config.num_eliminated_individuals,
            &symbolic_trace,
            base_config,
            &mutation_config,
            &mut rng,
        );
        for (i, j) in evaluation_indices
            .into_iter()
            .take(mutation_config.num_eliminated_individuals)
            .enumerate()
        {
            trace_population[j] = new_trace_population[i].clone();
        }
    }

    println!(
        "\n └─ No solution found after {} generations",
        mutation_config.max_generations
    );

    MutationTestResult {
        random_seed: seed,
        mutation_config: mutation_config.clone(),
        counter_example: None,
        generation: mutation_config.max_generations,
        fitness_score_log: fitness_score_log,
    }
}

fn zero_div_attempt(
    inp: &mut FxHashMap<SymbolicName, BigInt>,
    sexe: &mut SymbolicExecutor,
    cache: &mut FxHashMap<[BigInt; 3], BigInt>,
    base_config: &BaseVerificationConfig,
    potential_zero_div_positions: &Vec<(usize, (Vec<QuadraticPoly>, Vec<QuadraticPoly>))>,
    input_variables: &FxHashSet<SymbolicName>,
    rng: &mut StdRng,
) {
    let zero_div_info = potential_zero_div_positions.choose(rng);
    let mut dummy_inp = inp.clone();

    if let Some((_, (numerator_polys, denominator_polys))) = zero_div_info {
        if !numerator_polys.is_empty() {
            let numerator = numerator_polys.choose(rng);
            if let Some((numerator_var_name, numerator_coefs)) = numerator {
                if input_variables.contains(numerator_var_name) {
                    let tmp_val = dummy_inp.remove(numerator_var_name);
                    let numerator_coefficients: Option<Vec<_>> = numerator_coefs
                        .iter()
                        .map(|expr| {
                            evaluate_symbolic_value(
                                &base_config.prime,
                                expr,
                                &dummy_inp,
                                sexe.symbolic_library,
                            )
                            .and_then(|val| match val {
                                SymbolicValue::ConstantInt(c) => Some(c),
                                _ => None,
                            })
                        })
                        .collect();
                    if let Some(coefs) = numerator_coefficients {
                        let coefs_slice = [coefs[0].clone(), coefs[1].clone(), coefs[2].clone()];
                        if let Some(ans_val) = cache.get(&coefs_slice) {
                            inp.insert(numerator_var_name.clone(), ans_val.clone());
                        } else if let Some(ans_val) =
                            solve_quadratic_modulus_equation(&coefs_slice, &base_config.prime)
                        {
                            inp.insert(numerator_var_name.clone(), ans_val.clone());
                            cache.insert(coefs_slice, ans_val);
                        }
                    }
                    if let Some(tv) = tmp_val {
                        dummy_inp.insert(numerator_var_name.clone(), tv);
                    }
                }
            }
        }
        if !denominator_polys.is_empty() {
            let denominator = denominator_polys.choose(rng);
            if let Some((denominator_var_name, denominator_coefs)) = denominator {
                if input_variables.contains(denominator_var_name) {
                    dummy_inp.remove(denominator_var_name);
                    let denominator_coefficients: Option<Vec<_>> = denominator_coefs
                        .iter()
                        .map(|expr| {
                            evaluate_symbolic_value(
                                &base_config.prime,
                                expr,
                                &dummy_inp,
                                sexe.symbolic_library,
                            )
                            .and_then(|val| match val {
                                SymbolicValue::ConstantInt(c) => Some(c),
                                _ => None,
                            })
                        })
                        .collect();
                    if let Some(coefs) = denominator_coefficients {
                        let coefs_slice = [coefs[0].clone(), coefs[1].clone(), coefs[2].clone()];
                        if let Some(ans_val) = cache.get(&coefs_slice) {
                            inp.insert(denominator_var_name.clone(), ans_val.clone());
                        } else if let Some(ans_val) =
                            solve_quadratic_modulus_equation(&coefs_slice, &base_config.prime)
                        {
                            inp.insert(denominator_var_name.clone(), ans_val.clone());
                            cache.insert(coefs_slice, ans_val);
                        }
                    }
                }
            }
        }
    }
}
