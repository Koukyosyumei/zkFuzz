use std::cmp::min;
use std::rc::Rc;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;

use program_structure::ast::ExpressionInfixOpcode;

use crate::executor::debug_ast::DebuggableExpressionInfixOpcode;
use crate::executor::symbolic_state::SymbolicTrace;
use crate::executor::symbolic_value::SymbolicValue;

use crate::mutator::mutation_config::MutationConfig;
use crate::mutator::mutation_test::Gene;
use crate::mutator::mutation_utils::{
    draw_bigint_with_probabilities, draw_operator_mutation_or_random_constant,
};
use crate::mutator::utils::BaseVerificationConfig;

/// Initializes a population of `Gene` instances by replacing all symbolic trace positions
/// with random constant values.
///
/// This function generates a population for a given program, where each `Gene` is
/// a mapping from trace positions to symbolic values, all of which are initialized to
/// random constants based on the provided base configuration.
///
/// # Parameters
/// - `pos`: A slice of indices representing positions in the symbolic trace to be initialized.
/// - `program_population_size`: The size of the generated population
/// - `_symbolic_trace`: A reference to the symbolic trace (`SymbolicTrace`). This parameter
///   is currently unused but reserved for potential future enhancements.
/// - `_base_config`: Configuration object providing base parameters for generating random constants.
///   This parameter is currently unused but reserved for potential future enhancements.
/// - `mutation_config`: Configuration object defining mutation parameters, such as the population size.
/// - `rng`: A mutable reference to a random number generator for consistent randomization.
///
/// # Returns
/// A vector of `Gene` instances, where each `Gene` maps trace positions to randomly generated constant values.
///
/// # Details
/// - For each position in the trace, a random constant value is generated using the
///   `draw_bigint_with_probabilities` function and assigned as the symbolic value.
pub fn initialize_population_with_constant_replacement(
    pos: &[usize],
    program_population_size: usize,
    _symbolic_trace: &SymbolicTrace,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> Vec<Gene> {
    (0..program_population_size)
        .map(|_| {
            let num_mutations = if pos.len() > 1 {
                rng.gen_range(1, min(pos.len(), mutation_config.max_num_mutation_points))
            } else {
                1
            };
            let selected_pos: Vec<_> = pos.choose_multiple(rng, num_mutations).cloned().collect();
            selected_pos
                .iter()
                .map(|p| {
                    (
                        p.clone(),
                        SymbolicValue::ConstantInt(
                            draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
                        ),
                    )
                })
                .collect()
        })
        .collect()
}

pub fn initialize_population_with_operator_or_const_replacement(
    pos: &[usize],
    program_population_size: usize,
    symbolic_trace: &SymbolicTrace,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> Vec<Gene> {
    (0..program_population_size)
        .map(|_| {
            let num_mutations = if pos.len() > 1 {
                rng.gen_range(1, min(pos.len(), mutation_config.max_num_mutation_points))
            } else {
                1
            };
            let selected_pos: Vec<_> = pos.choose_multiple(rng, num_mutations).cloned().collect();
            selected_pos
                .iter()
                .map(|p| {
                    (
                        p.clone(),
                        draw_operator_mutation_or_random_constant(
                            &*symbolic_trace[*p],
                            mutation_config,
                            rng,
                        ),
                    )
                })
                .collect()
        })
        .collect()
}

pub fn initialize_population_with_operator_or_const_replacement_or_addition(
    pos: &[usize],
    program_population_size: usize,
    symbolic_trace: &SymbolicTrace,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> Vec<Gene> {
    (0..program_population_size)
        .map(|_| {
            let num_mutations = if pos.len() > 1 {
                rng.gen_range(1, min(pos.len(), mutation_config.max_num_mutation_points))
            } else {
                1
            };
            let selected_pos: Vec<_> = pos.choose_multiple(rng, num_mutations).cloned().collect();
            selected_pos
                .iter()
                .map(|p| {
                    if rng.gen::<f64>() < mutation_config.add_random_const_prob {
                        (
                            p.clone(),
                            SymbolicValue::BinaryOp(
                                symbolic_trace[*p].clone(),
                                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                                Rc::new(SymbolicValue::ConstantInt(
                                    draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
                                )),
                            ),
                        )
                    } else {
                        (
                            p.clone(),
                            draw_operator_mutation_or_random_constant(
                                &*symbolic_trace[*p],
                                mutation_config,
                                rng,
                            ),
                        )
                    }
                })
                .collect()
        })
        .collect()
}

pub fn initialize_population_with_operator_or_const_replacement_or_deletion(
    pos: &[usize],
    program_population_size: usize,
    symbolic_trace: &SymbolicTrace,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> Vec<Gene> {
    (0..program_population_size)
        .map(|_| {
            let num_mutations = if pos.len() > 1 {
                rng.gen_range(1, min(pos.len(), mutation_config.max_num_mutation_points))
            } else {
                1
            };
            let selected_pos: Vec<_> = pos.choose_multiple(rng, num_mutations).cloned().collect();
            selected_pos
                .iter()
                .map(|p| {
                    if rng.gen::<f64>() < mutation_config.statement_deletion_prob {
                        (p.clone(), SymbolicValue::NOP)
                    } else {
                        (
                            p.clone(),
                            draw_operator_mutation_or_random_constant(
                                &*symbolic_trace[*p],
                                mutation_config,
                                rng,
                            ),
                        )
                    }
                })
                .collect()
        })
        .collect()
}
