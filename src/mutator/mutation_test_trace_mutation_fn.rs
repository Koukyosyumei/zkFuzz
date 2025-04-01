use std::rc::Rc;

use program_structure::ast::ExpressionInfixOpcode;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::Rng;

use crate::executor::debug_ast::DebuggableExpressionInfixOpcode;
use crate::executor::symbolic_state::SymbolicTrace;
use crate::executor::symbolic_value::SymbolicValue;
use crate::mutator::mutation_config::MutationConfig;
use crate::mutator::mutation_test::Gene;
use crate::mutator::mutation_utils::{
    draw_bigint_with_probabilities, draw_operator_mutation_or_random_constant,
};
use crate::mutator::utils::BaseVerificationConfig;

/// Mutates a trace by replacing a randomly selected position with a new random constant value.
///
/// This function introduces randomness into an individual's trace by selecting a position and replacing
/// its value with a newly generated constant, based on configurable probabilities and value ranges.
///
/// # Parameters
/// - `pos`: A slice of indices representing mutable positions in the symbolic trace.
/// - `individual`: A mutable reference to a `Gene` (a mapping of position in a trace to symbolic values),
///   representing the mutation of the trace.
/// - `_base_config`: A reference to the `BaseVerificationConfig`, which is currently unused in this function
///   but might be relevant in future extensions.
/// - `mutation_config`: A reference to the `MutationConfig`, specifying the value ranges and probabilities
///   used for generating random constants.
/// - `rng`: A mutable reference to a random number generator (`StdRng`) for producing random values.
///
/// # Behavior
/// - If the `individual` is not empty:
///   - A position is selected randomly from its keys.
///     - The selected position's value is replaced with a new `SymbolicValue::ConstantInt`,
///       generated using `mutation_config.random_value_ranges` and `mutation_config.random_value_probs`.
///   - If the size of `individual` is smaller than `max_num_mutation_points`:
///     - There is a 50% chance to add one more mutation at a new position.
///   - If the size of `individual` is larger than 1 and no mutation was added in the previous step:
///     - Remove one existing mutation point.
/// - If the `individual` is empty, the function does nothing.
///
/// # Notes
/// - The generated constants are drawn according to the probabilities and ranges defined in the `MutationConfig`.
///
/// # Future Considerations
/// - `_base_config` could be leveraged to introduce additional constraints or behaviors during mutation.
pub fn mutate_trace_with_constant_replacement(
    pos: &[usize],
    _symbolic_trace: &SymbolicTrace,
    individual: &mut Gene,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    if !individual.is_empty() {
        let mut keys: Vec<usize> = individual.keys().copied().collect();
        keys.sort();
        let var = keys.iter().choose(rng).unwrap();
        individual.insert(
            var.clone(),
            SymbolicValue::ConstantInt(
                draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
            ),
        );
        if individual.len() < mutation_config.max_num_mutation_points && rng.gen::<bool>() {
            let var = pos.into_iter().choose(rng).unwrap();
            individual.insert(
                var.clone(),
                SymbolicValue::ConstantInt(
                    draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
                ),
            );
        } else if individual.len() > 1 && rng.gen::<bool>() {
            let mut keys: Vec<usize> = individual.keys().copied().collect();
            keys.sort();
            let var = keys.iter().choose(rng).unwrap();
            individual.remove(&var);
        }
    }
}

pub fn mutate_trace_with_operator_or_const_replacement(
    pos: &[usize],
    symbolic_trace: &SymbolicTrace,
    individual: &mut Gene,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    if !individual.is_empty() {
        let mut keys: Vec<usize> = individual.keys().copied().collect();
        keys.sort();
        let var = keys.iter().choose(rng).unwrap();
        individual.insert(
            var.clone(),
            draw_operator_mutation_or_random_constant(&*symbolic_trace[*var], mutation_config, rng),
        );
        if individual.len() < mutation_config.max_num_mutation_points && rng.gen::<bool>() {
            let var = pos.into_iter().choose(rng).unwrap();
            individual.insert(
                var.clone(),
                draw_operator_mutation_or_random_constant(
                    &*symbolic_trace[*var],
                    mutation_config,
                    rng,
                ),
            );
        } else if individual.len() > 1 && rng.gen::<bool>() {
            let mut keys: Vec<usize> = individual.keys().copied().collect();
            keys.sort();
            let var = keys.iter().choose(rng).unwrap();
            individual.remove(&var);
        }
    }
}

pub fn mutate_trace_with_operator_or_const_replacement_or_addition(
    pos: &[usize],
    symbolic_trace: &SymbolicTrace,
    individual: &mut Gene,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    if !individual.is_empty() {
        let mut keys: Vec<usize> = individual.keys().copied().collect();
        keys.sort();
        let var = keys.iter().choose(rng).unwrap();
        individual.insert(
            var.clone(),
            if rng.gen::<f64>() < mutation_config.add_random_const_prob {
                SymbolicValue::BinaryOp(
                    symbolic_trace[*var].clone(),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    Rc::new(SymbolicValue::ConstantInt(
                        draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
                    )),
                )
            } else {
                draw_operator_mutation_or_random_constant(
                    &*symbolic_trace[*var],
                    mutation_config,
                    rng,
                )
            },
        );
        if individual.len() < mutation_config.max_num_mutation_points && rng.gen::<bool>() {
            let var = pos.into_iter().choose(rng).unwrap();
            individual.insert(
                var.clone(),
                if rng.gen::<f64>() < mutation_config.add_random_const_prob {
                    SymbolicValue::BinaryOp(
                        symbolic_trace[*var].clone(),
                        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                        Rc::new(SymbolicValue::ConstantInt(
                            draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
                        )),
                    )
                } else {
                    draw_operator_mutation_or_random_constant(
                        &*symbolic_trace[*var],
                        mutation_config,
                        rng,
                    )
                },
            );
        } else if individual.len() > 1 && rng.gen::<bool>() {
            let mut keys: Vec<usize> = individual.keys().copied().collect();
            keys.sort();
            let var = keys.iter().choose(rng).unwrap();
            individual.remove(&var);
        }
    }
}

pub fn mutate_trace_with_operator_or_const_replacement_or_deletion(
    pos: &[usize],
    symbolic_trace: &SymbolicTrace,
    individual: &mut Gene,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    if !individual.is_empty() {
        let mut keys: Vec<usize> = individual.keys().copied().collect();
        keys.sort();
        let var = keys.iter().choose(rng).unwrap();
        individual.insert(
            var.clone(),
            if rng.gen::<f64>() < mutation_config.statement_deletion_prob {
                SymbolicValue::NOP
            } else {
                draw_operator_mutation_or_random_constant(
                    &*symbolic_trace[*var],
                    mutation_config,
                    rng,
                )
            },
        );
        if individual.len() < mutation_config.max_num_mutation_points && rng.gen::<bool>() {
            let var = pos.into_iter().choose(rng).unwrap();
            individual.insert(
                var.clone(),
                if rng.gen::<f64>() < mutation_config.statement_deletion_prob {
                    SymbolicValue::NOP
                } else {
                    draw_operator_mutation_or_random_constant(
                        &*symbolic_trace[*var],
                        mutation_config,
                        rng,
                    )
                },
            );
        } else if individual.len() > 1 && rng.gen::<bool>() {
            let mut keys: Vec<usize> = individual.keys().copied().collect();
            keys.sort();
            let var = keys.iter().choose(rng).unwrap();
            individual.remove(&var);
        }
    }
}
