use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::str::FromStr;

use colored::Colorize;
use log::info;
use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::{One, Signed, Zero};
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

pub fn roulette_selection<'a, T: Clone>(
    population: &'a [T],
    fitness_scores: &[BigInt],
    rng: &mut StdRng,
) -> &'a T {
    let min_score = fitness_scores.iter().min().unwrap();
    let weights: Vec<_> = fitness_scores
        .iter()
        .map(|score| score - min_score)
        .collect();
    let mut total_weight: BigInt = weights.iter().sum();
    total_weight = if total_weight.is_positive() {
        total_weight
    } else {
        BigInt::one()
    };
    let mut target = rng.gen_bigint_range(&BigInt::zero(), &total_weight);
    for (individual, weight) in population.iter().zip(weights.iter()) {
        if &target < weight {
            return individual;
        }
        target -= weight;
    }
    &population[0]
}
