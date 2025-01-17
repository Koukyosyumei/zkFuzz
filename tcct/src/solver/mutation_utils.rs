use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::One;
use rand::rngs::StdRng;
use rand::Rng;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_value::{SymbolicValue, SymbolicValueRef};

use crate::solver::utils::BaseVerificationConfig;

pub fn draw_random_constant(base_config: &BaseVerificationConfig, rng: &mut StdRng) -> BigInt {
    if rng.gen::<bool>() {
        rng.gen_bigint_range(
            &(BigInt::from_str("10").unwrap() * -BigInt::one()),
            &(BigInt::from_str("10").unwrap()),
        )
    } else {
        rng.gen_bigint_range(
            &(base_config.prime.clone() - BigInt::from_str("100").unwrap()),
            &(base_config.prime),
        )
    }
}

/// Draws a random BigInt from specified ranges based on given probabilities.
///
/// # Arguments
/// - `ranges`: A slice of tuples [(x1, y1), (x2, y2), ...], where each tuple defines a range [x, y).
/// - `probabilities`: A slice of probabilities [p1, p2, ...] corresponding to each range.
///
/// # Returns
/// A random BigInt drawn from one of the specified ranges based on the probabilities,
/// or `None` if the input is invalid (e.g., mismatched lengths of ranges and probabilities).
pub fn draw_bigint_with_probabilities(
    ranges: &[(BigInt, BigInt)],
    probabilities: &[f64],
    rng: &mut StdRng,
) -> Option<BigInt> {
    // Ensure the number of ranges matches the number of probabilities
    if ranges.len() != probabilities.len() {
        return None;
    }

    // Normalize the probabilities to sum to 1
    // let total_prob: f64 = probabilities.iter().sum();
    // let normalized_probabilities: Vec<f64> = probabilities.iter().map(|p| p / total_prob).collect();

    // Roulette selection to choose a range based on probabilities
    let mut cumulative_prob = 0.0;
    let random_value: f64 = rng.gen();
    let mut selected_range = None;

    for (i, range) in ranges.iter().enumerate() {
        cumulative_prob += probabilities[i];
        if random_value <= cumulative_prob {
            selected_range = Some(range);
            break;
        }
    }

    if let Some((start, end)) = selected_range {
        // Generate a random BigInt within the selected range
        Some(rng.gen_bigint_range(&start, &end))
    } else {
        None
    }
}

pub fn apply_trace_mutation(
    symbolic_trace: &Vec<SymbolicValueRef>,
    trace_mutation: &FxHashMap<usize, SymbolicValue>,
) -> Vec<SymbolicValueRef> {
    let mut mutated_constraints = symbolic_trace.clone();
    for (index, value) in trace_mutation {
        if let SymbolicValue::Assign(lv, _, is_safe) = mutated_constraints[*index].as_ref().clone()
        {
            mutated_constraints[*index] = Rc::new(SymbolicValue::Assign(
                lv.clone(),
                Rc::new(value.clone()),
                is_safe,
            ));
        } else if let SymbolicValue::AssignCall(lv, _, is_mutable) =
            mutated_constraints[*index].as_ref().clone()
        {
            mutated_constraints[*index] = Rc::new(SymbolicValue::Assign(
                lv.clone(),
                Rc::new(value.clone()),
                !is_mutable,
            ));
        } else {
            panic!("We can only mutate SymbolicValue::Assign");
        }
    }
    mutated_constraints
}
