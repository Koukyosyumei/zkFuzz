
use num_bigint_dig::BigInt;
use rand::rngs::StdRng;
use rand::Rng;



use crate::solver::mutation_config::MutationConfig;
use crate::solver::utils::BaseVerificationConfig;

pub fn evolve_population<T: Clone, TraceMutationFn, TraceCrossoverFn, TraceSelectionFn>(
    prev_population: &[T],
    prev_evaluations: &[BigInt],
    base_base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
    trace_mutation_fn: &TraceMutationFn,
    trace_crossover_fn: &TraceCrossoverFn,
    trace_selection_fn: &TraceSelectionFn,
) -> Vec<T>
where
    TraceMutationFn: Fn(&mut T, &BaseVerificationConfig, &mut StdRng),
    TraceCrossoverFn: Fn(&T, &T, &mut StdRng) -> T,
    TraceSelectionFn: for<'a> Fn(&'a [T], &[BigInt], &mut StdRng) -> &'a T,
{
    (0..mutation_config.program_population_size)
        .map(|_| {
            let parent1 = trace_selection_fn(prev_population, prev_evaluations, rng);
            let parent2 = trace_selection_fn(prev_population, prev_evaluations, rng);
            let mut child = if rng.gen::<f64>() < mutation_config.crossover_rate {
                trace_crossover_fn(&parent1, &parent2, rng)
            } else {
                parent1.clone()
            };
            if rng.gen::<f64>() < mutation_config.mutation_rate {
                trace_mutation_fn(&mut child, base_base_config, rng);
            }
            child
        })
        .collect()
}
