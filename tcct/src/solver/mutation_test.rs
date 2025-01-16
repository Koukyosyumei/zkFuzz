use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::str::FromStr;

use colored::Colorize;
use log::info;
use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::{One, Zero};
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};
use rustc_hash::FxHashMap;

use program_structure::ast::ExpressionInfixOpcode;

use crate::executor::debug_ast::DebuggableExpressionInfixOpcode;
use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_state::{SymbolicConstraints, SymbolicTrace};
use crate::executor::symbolic_value::{OwnerName, SymbolicName, SymbolicValue, SymbolicValueRef};

use crate::solver::mutation_config::MutationConfig;
use crate::solver::utils::{extract_variables, BaseVerificationConfig, CounterExample};

pub struct MutationTestResult {
    pub random_seed: u64,
    pub mutation_config: MutationConfig,
    pub counter_example: Option<CounterExample>,
    pub generation: usize,
    pub fitness_score_log: Vec<BigInt>,
}

pub type Gene = FxHashMap<usize, SymbolicValue>;

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
    mutation_config: &MutationConfig,
    trace_initialization_fn: TraceInitializationFn,
    update_input_fn: UpdateInputFn,
    trace_fitness_fn: TraceFitnessFn,
    trace_evolution_fn: TraceEvolutionFn,
    trace_mutation_fn: TraceMutationFn,
    trace_crossover_fn: TraceCrossoverFn,
    trace_selection_fn: TraceSelectionFn,
) -> MutationTestResult
where
    TraceInitializationFn:
        Fn(&[usize], &BaseVerificationConfig, &MutationConfig, &mut StdRng) -> Vec<Gene>,
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
        &Gene,
        &Vec<FxHashMap<SymbolicName, BigInt>>,
    ) -> (usize, BigInt, Option<CounterExample>),
    TraceEvolutionFn: Fn(
        &[Gene],
        &[BigInt],
        &BaseVerificationConfig,
        &MutationConfig,
        &mut StdRng,
        &TraceMutationFn,
        &TraceCrossoverFn,
        &TraceSelectionFn,
    ) -> Vec<Gene>,
    TraceMutationFn: Fn(&mut Gene, &BaseVerificationConfig, &mut StdRng),
    TraceCrossoverFn: Fn(&Gene, &Gene, &mut StdRng) -> Gene,
    TraceSelectionFn: for<'a> Fn(&'a [Gene], &[BigInt], &mut StdRng) -> &'a Gene,
{
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
    for v in variables_set.iter() {
        if v.owner.len() == 1
            && sexe.symbolic_library.template_library
                [&sexe.symbolic_library.name2id[&base_config.target_template_name]]
                .input_ids
                .contains(&v.id)
        {
            input_variables.push(v.clone());
        }
    }

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
    let mut trace_population =
        trace_initialization_fn(&assign_pos, base_config, mutation_config, &mut rng);
    let mut fitness_scores =
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
        let evaluations: Vec<_> = trace_population
            .iter()
            .map(|a| {
                trace_fitness_fn(
                    sexe,
                    &base_config,
                    symbolic_trace,
                    side_constraints,
                    a,
                    &input_population,
                )
            })
            .collect();

        // Pick the best one
        let best_idx = evaluations
            .iter()
            .enumerate()
            .max_by_key(|&(_, value)| value.1.clone())
            .map(|(index, _)| index)
            .unwrap();

        // Extract the fitness scores
        if mutation_config.fitness_function != "const" {
            fitness_scores = evaluations.iter().map(|v| v.1.clone()).collect();
        }

        if evaluations[best_idx].1.is_zero() {
            print!(
                "\r\x1b[2K🧬 Generation: {}/{} ({:.3})",
                generation, mutation_config.max_generations, 0
            );
            println!("\n    └─ Solution found in generation {}", generation);

            return MutationTestResult {
                random_seed: seed,
                mutation_config: mutation_config.clone(),
                counter_example: evaluations[best_idx].2.clone(),
                generation: generation,
                fitness_score_log: fitness_score_log,
            };
        }

        print!(
            "\r\x1b[2K🧬 Generation: {}/{} ({:.3})",
            generation, mutation_config.max_generations, fitness_scores[best_idx]
        );
        io::stdout().flush().unwrap();

        if mutation_config.save_fitness_scores {
            fitness_score_log.push(fitness_scores[best_idx].clone());
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
