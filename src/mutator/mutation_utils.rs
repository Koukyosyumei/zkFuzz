use std::rc::Rc;

use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::Zero;
use program_structure::ast::ExpressionInfixOpcode;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::Rng;
use rustc_hash::FxHashMap;

use crate::executor::debug_ast::DebuggableExpressionInfixOpcode;
use crate::executor::symbolic_state::SymbolicTrace;
use crate::executor::symbolic_value::SymbolicValue;
use crate::mutator::mutation_config::MutationConfig;

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
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> Option<BigInt> {
    if rng.gen::<f64>() < mutation_config.binary_mode_prob {
        Some(rng.gen_bigint_range(&BigInt::zero(), &BigInt::from(2)))
    } else {
        // Ensure the number of ranges matches the number of probabilities
        if mutation_config.random_value_ranges.len() != mutation_config.random_value_probs.len() {
            return None;
        }

        // Roulette selection to choose a range based on probabilities
        let mut cumulative_prob = 0.0;
        let random_value: f64 = rng.gen();
        let mut selected_range = None;

        for (i, range) in mutation_config.random_value_ranges.iter().enumerate() {
            cumulative_prob += mutation_config.random_value_probs[i];
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
    let mut keys: Vec<_> = trace_mutation.keys().collect();
    keys.sort();

    for index in keys {
        let value = trace_mutation.get(index).unwrap();
        if let SymbolicValue::Assign(lv, _, is_safe, _) =
            mutated_constraints[*index].as_ref().clone()
        {
            mutated_constraints[*index] = Rc::new(SymbolicValue::Assign(
                lv.clone(),
                Rc::new(value.clone()),
                is_safe,
                None,
            ));
        } else if let SymbolicValue::AssignCall(lv, _, is_mutable) =
            mutated_constraints[*index].as_ref().clone()
        {
            mutated_constraints[*index] = Rc::new(SymbolicValue::Assign(
                lv.clone(),
                Rc::new(value.clone()),
                !is_mutable,
                None,
            ));
        } else {
            panic!("We can only mutate SymbolicValue::Assign");
        }
    }
    mutated_constraints
}

lazy_static::lazy_static! {
    static ref OPERATOR_MUTATION_CANDIDATES_STRICT: Vec<(ExpressionInfixOpcode,Vec<ExpressionInfixOpcode>)> = {
        vec![
            (ExpressionInfixOpcode::Add, vec![ExpressionInfixOpcode::Sub, ExpressionInfixOpcode::Mul, ExpressionInfixOpcode::Pow, ExpressionInfixOpcode::Div, ExpressionInfixOpcode::IntDiv, ExpressionInfixOpcode::Mod]),
            (ExpressionInfixOpcode::Sub, vec![ExpressionInfixOpcode::Add, ExpressionInfixOpcode::Mul, ExpressionInfixOpcode::Pow, ExpressionInfixOpcode::Div, ExpressionInfixOpcode::IntDiv, ExpressionInfixOpcode::Mod]),
            (ExpressionInfixOpcode::Mul, vec![ExpressionInfixOpcode::Add, ExpressionInfixOpcode::Sub, ExpressionInfixOpcode::Pow, ExpressionInfixOpcode::Div, ExpressionInfixOpcode::IntDiv, ExpressionInfixOpcode::Mod]),
            (ExpressionInfixOpcode::Pow, vec![ExpressionInfixOpcode::Add, ExpressionInfixOpcode::Sub, ExpressionInfixOpcode::Mul, ExpressionInfixOpcode::Div, ExpressionInfixOpcode::IntDiv, ExpressionInfixOpcode::Mod]),
            (ExpressionInfixOpcode::Div, vec![ExpressionInfixOpcode::Add, ExpressionInfixOpcode::Sub, ExpressionInfixOpcode::Mul, ExpressionInfixOpcode::Pow, ExpressionInfixOpcode::IntDiv, ExpressionInfixOpcode::Mod]),
            (ExpressionInfixOpcode::IntDiv, vec![ExpressionInfixOpcode::Add, ExpressionInfixOpcode::Sub, ExpressionInfixOpcode::Mul, ExpressionInfixOpcode::Pow, ExpressionInfixOpcode::Div, ExpressionInfixOpcode::Mod]),
            (ExpressionInfixOpcode::Mod, vec![ExpressionInfixOpcode::Add, ExpressionInfixOpcode::Sub, ExpressionInfixOpcode::Mul, ExpressionInfixOpcode::Pow, ExpressionInfixOpcode::Div, ExpressionInfixOpcode::IntDiv]),
            (ExpressionInfixOpcode::BitOr, vec![ExpressionInfixOpcode::BitAnd, ExpressionInfixOpcode::BitXor]),
            (ExpressionInfixOpcode::BitAnd, vec![ExpressionInfixOpcode::BitOr, ExpressionInfixOpcode::BitXor]),
            (ExpressionInfixOpcode::BitXor, vec![ExpressionInfixOpcode::BitOr, ExpressionInfixOpcode::BitAnd]),
            (ExpressionInfixOpcode::ShiftL, vec![ExpressionInfixOpcode::ShiftR]),
            (ExpressionInfixOpcode::ShiftR, vec![ExpressionInfixOpcode::ShiftL]),
            (ExpressionInfixOpcode::Lesser, vec![ExpressionInfixOpcode::NotEq, ExpressionInfixOpcode::Eq, ExpressionInfixOpcode::Greater, ExpressionInfixOpcode::GreaterEq, ExpressionInfixOpcode::LesserEq]),
            (ExpressionInfixOpcode::Greater, vec![ExpressionInfixOpcode::NotEq, ExpressionInfixOpcode::Eq, ExpressionInfixOpcode::Lesser, ExpressionInfixOpcode::GreaterEq, ExpressionInfixOpcode::LesserEq]),
            (ExpressionInfixOpcode::LesserEq, vec![ExpressionInfixOpcode::NotEq, ExpressionInfixOpcode::Eq, ExpressionInfixOpcode::Lesser, ExpressionInfixOpcode::Greater, ExpressionInfixOpcode::GreaterEq]),
            (ExpressionInfixOpcode::GreaterEq, vec![ExpressionInfixOpcode::NotEq, ExpressionInfixOpcode::Eq, ExpressionInfixOpcode::Lesser, ExpressionInfixOpcode::Greater, ExpressionInfixOpcode::LesserEq]),
            (ExpressionInfixOpcode::Eq, vec![ExpressionInfixOpcode::NotEq, ExpressionInfixOpcode::Lesser, ExpressionInfixOpcode::Greater, ExpressionInfixOpcode::GreaterEq, ExpressionInfixOpcode::LesserEq]),
            (ExpressionInfixOpcode::NotEq, vec![ExpressionInfixOpcode::Eq, ExpressionInfixOpcode::Lesser, ExpressionInfixOpcode::Greater, ExpressionInfixOpcode::GreaterEq, ExpressionInfixOpcode::LesserEq]),
        ]
    };
}

pub fn draw_operator_mutation_or_random_constant(
    target: &SymbolicValue,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> SymbolicValue {
    match target {
        SymbolicValue::BinaryOp(left, op, right) => {
            if rng.gen::<f64>() < mutation_config.operator_mutation_rate {
                let mutated_op = if let Some(related_ops) = OPERATOR_MUTATION_CANDIDATES_STRICT
                    .iter()
                    .find(|&&(key, _)| key == op.0)
                    .map(|&(_, ref ops)| ops)
                {
                    *related_ops
                        .iter()
                        .choose(rng)
                        .expect("Related operator group cannot be empty")
                } else {
                    panic!("No group defined for the given opcode: {:?}", op);
                };

                SymbolicValue::BinaryOp(
                    left.clone(),
                    DebuggableExpressionInfixOpcode(mutated_op),
                    right.clone(),
                )
            } else {
                SymbolicValue::ConstantInt(
                    draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
                )
            }
        }
        _ => SymbolicValue::ConstantInt(
            draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
        ),
    }
}
