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
