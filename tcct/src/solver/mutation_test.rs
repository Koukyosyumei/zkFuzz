use std::collections::HashSet;
use std::io;
use std::io::Write;

use colored::Colorize;
use log::info;
use num_bigint_dig::BigInt;
use num_traits::Zero;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_state::{SymbolicConstraints, SymbolicTrace};
use crate::executor::symbolic_value::{SymbolicName, SymbolicValue};

use crate::solver::mutation_config::MutationConfig;
use crate::solver::utils::{
    extract_variables, gather_runtime_mutable_inputs, get_dependencies, get_slice,
    BaseVerificationConfig, CounterExample, Direction,
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
        &BaseVerificationConfig,
        &MutationConfig,
        &mut StdRng,
    ),
    TraceFitnessFn: Fn(
        &mut SymbolicExecutor,
        &BaseVerificationConfig,
        &SymbolicTrace,
        &SymbolicConstraints,
        &FxHashMap<usize, Direction>,
        &Gene,
        &Vec<FxHashMap<SymbolicName, BigInt>>,
    ) -> (usize, BigInt, Option<CounterExample>, usize),
    TraceEvolutionFn: Fn(
        &[usize],
        &[Gene],
        &[BigInt],
        &BaseVerificationConfig,
        &MutationConfig,
        &mut StdRng,
        &TraceMutationFn,
        &TraceCrossoverFn,
        &TraceSelectionFn,
    ) -> Vec<Gene>,
    TraceMutationFn: Fn(&[usize], &mut Gene, &BaseVerificationConfig, &MutationConfig, &mut StdRng),
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
            SymbolicValue::Assign(_, _, false) | SymbolicValue::AssignCall(_, _, true) => {
                assign_pos.push(i);
            }
            _ => {}
        }
    }

    // Gather input variables
    let mut variables = extract_variables(symbolic_trace);
    variables.append(&mut extract_variables(side_constraints));
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();
    let mut input_variables = Vec::new();
    let mut output_variables = Vec::new();
    for v in variables_set.iter() {
        if v.owner.len() == 1
            && sexe.symbolic_library.template_library
                [&sexe.symbolic_library.name2id[&base_config.target_template_name]]
                .input_ids
                .contains(&v.id)
        {
            input_variables.push(v.clone());
        }
        if v.owner.len() == 1
            && sexe.symbolic_library.template_library
                [&sexe.symbolic_library.name2id[&base_config.target_template_name]]
                .output_ids
                .contains(&v.id)
        {
            output_variables.push(v.clone());
        }
    }

    let dummy_runtime_mutable_positions = FxHashMap::default();
    let runtime_mutable_positions = gather_runtime_mutable_inputs(
        symbolic_trace,
        sexe.symbolic_library,
        &input_variables.iter().cloned().collect(),
    );

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

    println!(
        "{} {}",
        "🎲 Random Seed:",
        seed.to_string().bold().bright_yellow(),
    );

    let mut targets = Vec::new();
    for slice_target in output_variables {
        let mut slice_target_dependencies = FxHashSet::default();
        get_dependencies(
            &symbolic_trace,
            slice_target,
            &mut slice_target_dependencies,
        );

        let mut sliced_assign_pos = Vec::new();
        let sliced_symbolic_trace = get_slice(
            &symbolic_trace,
            &slice_target_dependencies,
            &mut sliced_assign_pos,
        );
        let mut dummy_assign_pos = Vec::default();
        let sliced_side_constraint = get_slice(
            &side_constraints,
            &slice_target_dependencies,
            &mut dummy_assign_pos,
        );
        targets.push((
            sliced_symbolic_trace.len(),
            sliced_assign_pos,
            sliced_symbolic_trace,
            sliced_side_constraint,
        ));
    }
    targets.sort_by(|i, j| i.0.cmp(&j.0));
    if targets[0].0 == symbolic_trace.len() {
        targets = vec![targets[0].clone()];
    }

    for (_, target_assign_pos, target_symbolic_trace, target_side_constraints) in targets {
        // Initial Pupulation of Mutated Inputs
        let mut trace_population = trace_initialization_fn(
            &target_assign_pos,
            mutation_config.program_population_size,
            &target_symbolic_trace,
            base_config,
            &mutation_config,
            &mut rng,
        );
        let mut fitness_scores =
            vec![-base_config.prime.clone(); mutation_config.input_population_size];
        let mut input_population = Vec::new();
        let mut fitness_score_log = if mutation_config.save_fitness_scores {
            Vec::with_capacity(mutation_config.max_generations)
        } else {
            Vec::new()
        };

        let mut binary_input_mode = false;

        for generation in 0..mutation_config.max_generations {
            // Generate input population for this generation
            if generation % mutation_config.input_update_interval == 0 {
                update_input_fn(
                    sexe,
                    &input_variables,
                    &mut input_population,
                    &base_config,
                    &mutation_config,
                    &mut rng,
                );
            }

            // Evolve the trace population
            if !trace_population.is_empty() {
                trace_population = trace_evolution_fn(
                    &target_assign_pos,
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

            // Evaluate the trace population
            let mut evaluations = Vec::new();
            let mut is_extincted_due_to_illegal_subscript = true;
            for individual in &trace_population {
                let fitness = trace_fitness_fn(
                    sexe,
                    &base_config,
                    &target_symbolic_trace,
                    &target_side_constraints,
                    if rng.gen::<f64>() < mutation_config.runtime_mutation_rate {
                        &dummy_runtime_mutable_positions
                    } else {
                        &runtime_mutable_positions
                    },
                    individual,
                    &input_population,
                );
                if fitness.1.is_zero() {
                    evaluations.push(fitness);
                    break;
                }
                is_extincted_due_to_illegal_subscript =
                    is_extincted_due_to_illegal_subscript && fitness.3 == input_population.len();
                evaluations.push(fitness);
            }

            if !binary_input_mode && is_extincted_due_to_illegal_subscript {
                binary_input_mode = true;
                mutation_config.random_value_ranges = vec![(BigInt::from(0), BigInt::from(2))];
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
                &target_assign_pos,
                mutation_config.num_eliminated_individuals,
                &target_symbolic_trace,
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
    }

    MutationTestResult {
        random_seed: seed,
        mutation_config: mutation_config.clone(),
        counter_example: None,
        generation: mutation_config.max_generations,
        fitness_score_log: Vec::new(),
    }
}
