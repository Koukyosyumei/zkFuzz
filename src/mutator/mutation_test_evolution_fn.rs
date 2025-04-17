use num_bigint_dig::BigInt;
use rand::rngs::StdRng;
use rand::Rng;

use crate::executor::symbolic_state::SymbolicTrace;
use crate::mutator::mutation_config::MutationConfig;
use crate::mutator::utils::BaseVerificationConfig;

/// Performs a basic evolutionary step to generate the next population of individuals.
///
/// This function implements a straightforward genetic algorithm to evolve a population
/// using selection, crossover, and mutation. It is intended as a foundational operation
/// for population evolution processes.
///
/// # Parameters
/// - `assign_pos`: A slice of indices representing mutable positions in the symbolic trace.
/// - `symbolic_trace`: A symbolic trace of the target program.
/// - `prev_population`: A slice of the current population of individuals.
/// - `prev_evaluations`: A slice of evaluation scores corresponding to the individuals
///   in the population. Higher scores typically indicate better fitness.
/// - `base_base_config`: Configuration parameters used for verification or mutation.
/// - `mutation_config`: Configuration parameters controlling mutation and crossover rates,
///   as well as the size of the new population.
/// - `rng`: A mutable reference to a random number generator used for probabilistic operations
///   like crossover and mutation.
/// - `trace_mutation_fn`: A function that applies mutation to an individual. It takes
///   a mutable reference to an individual, the base verification configuration, and
///   a random number generator.
/// - `trace_crossover_fn`: A function that performs crossover between two parent individuals
///   to produce a child individual. It takes two parent references and a random number generator.
/// - `trace_selection_fn`: A function that selects a parent individual from the population based
///   on their evaluation scores. It takes a slice of individuals, their evaluations, and a random
///   number generator.
///
/// # Returns
/// A `Vec<T>` representing the next generation of individuals after applying selection, crossover,
/// and mutation.
///
/// # Type Parameters
/// - `T`: The type representing an individual in the population, which must implement `Clone`.
/// - `MutationFn`: A callable function type for mutating an individual.
/// - `CrossoverFn`: A callable function type for performing crossover between two individuals.
/// - `SelectionFn`: A callable function type for selecting individuals based on fitness.
///
/// # Algorithm
/// 1. For each new individual in the population:
///     - Select two parent individuals using `trace_selection_fn`.
///     - With a probability defined in `mutation_config.crossover_rate`, create a child
///       by applying `trace_crossover_fn` to the parents. Otherwise, clone one parent.
///     - With a probability defined in `mutation_config.mutation_rate`, apply `trace_mutation_fn`
///       to the child.
/// 2. Collect all generated individuals into a new population.
pub fn simple_evolution<T: Clone, MutationFn, CrossoverFn, SelectionFn>(
    assign_pos: &[usize],
    symbolic_trace: &SymbolicTrace,
    prev_population: &[T],
    prev_evaluations: &[BigInt],
    base_base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
    mutation_fn: &MutationFn,
    crossover_fn: &CrossoverFn,
    selection_fn: &SelectionFn,
) -> Vec<T>
where
    MutationFn:
        Fn(&[usize], &SymbolicTrace, &mut T, &BaseVerificationConfig, &MutationConfig, &mut StdRng),
    CrossoverFn: Fn(&T, &T, &mut StdRng) -> T,
    SelectionFn: for<'a> Fn(&'a [T], &[BigInt], &mut StdRng) -> &'a T,
{
    (0..mutation_config.program_population_size)
        .map(|_| {
            let parent1 = selection_fn(prev_population, prev_evaluations, rng);
            let parent2 = selection_fn(prev_population, prev_evaluations, rng);
            let mut child = if rng.gen::<f64>() < mutation_config.crossover_rate {
                crossover_fn(&parent1, &parent2, rng)
            } else {
                parent1.clone()
            };
            if rng.gen::<f64>() < mutation_config.mutation_rate {
                mutation_fn(
                    assign_pos,
                    symbolic_trace,
                    &mut child,
                    base_base_config,
                    mutation_config,
                    rng,
                );
            }
            child
        })
        .collect()
}
