use std::rc::Rc;

use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use rand::rngs::StdRng;
use rand::Rng;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_state::SymbolicTrace;
use crate::executor::symbolic_value::SymbolicValue;

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

/// Applies trace mutations to a symbolic trace by replacing specific symbolic values.
///
/// # Parameters
/// - `symbolic_trace`: A reference to SymbolicTrace, a vector of `SymbolicValueRef` representing the current symbolic execution trace.
/// - `trace_mutation`: A reference to a hash map where the key is the index in the trace to mutate,
///   and the value is the new `SymbolicValue` to apply.
///
/// # Returns
/// A new `SymbolicTrace` representing the mutated symbolic trace.
///
/// # Behavior
/// 1. Clones the provided `symbolic_trace` to prepare for mutations.
/// 2. Iterates over the `trace_mutation` map and applies updates:
///    - If the value at the specified index is a `SymbolicValue::Assign`, it is replaced with a new assignment
///      using the provided value while preserving the left-hand side and safety flag.
///    - If the value is a `SymbolicValue::AssignCall`, it is replaced with a new assignment,
///      flipping the mutability flag while preserving the left-hand side.
/// 3. Panics if an entry at the specified index is neither `SymbolicValue::Assign` nor `SymbolicValue::AssignCall`,
///    as these are the only supported mutation targets.
///
/// # Panics
/// - The function panics if a mutation is attempted on a value that is not of type `SymbolicValue::Assign`
///   or `SymbolicValue::AssignCall`.
///
/// # Notes
/// - The original `symbolic_trace` is not modified; all changes are applied to a cloned version.
/// - This function assumes that indices in `trace_mutation` are valid within the range of the `symbolic_trace`.
pub fn apply_trace_mutation(
    symbolic_trace: &SymbolicTrace,
    trace_mutation: &FxHashMap<usize, SymbolicValue>,
) -> SymbolicTrace {
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
