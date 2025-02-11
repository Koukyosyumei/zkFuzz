use std::cmp::max;

use rand::rngs::StdRng;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::Rng;

use program_structure::ast::ExpressionInfixOpcode;

use crate::executor::debug_ast::DebuggableExpressionInfixOpcode;
use crate::executor::symbolic_state::SymbolicTrace;
use crate::executor::symbolic_value::SymbolicValue;

use crate::solver::mutation_config::MutationConfig;
use crate::solver::mutation_test::Gene;
use crate::solver::mutation_utils::draw_bigint_with_probabilities;
use crate::solver::utils::BaseVerificationConfig;

/// Initializes a population of `Gene` instances by replacing all symbolic trace positions
/// with random constant values.
///
/// This function generates a population for a given program, where each `Gene` is
/// a mapping from trace positions to symbolic values, all of which are initialized to
/// random constants based on the provided base configuration.
///
/// # Parameters
/// - `pos`: A slice of indices representing positions in the symbolic trace to be initialized.
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
/// - The size of the generated population is determined by `mutation_config.program_population_size`.
pub fn initialize_population_with_random_constant_replacement(
    pos: &[usize],
    program_population_size: usize,
    _symbolic_trace: &SymbolicTrace,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> Vec<Gene> {
    (0..program_population_size)
        .map(|_| {
            let num_mutations =
                rng.gen_range(1, max(pos.len(), mutation_config.max_num_mutation_points));
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

lazy_static::lazy_static! {
    static ref OPERATOR_MUTATION_CANDIDATES: Vec<(ExpressionInfixOpcode,Vec<ExpressionInfixOpcode>)> = {
        vec![
            (ExpressionInfixOpcode::Add, vec![ExpressionInfixOpcode::Sub, ExpressionInfixOpcode::Mul]),
            (ExpressionInfixOpcode::Sub, vec![ExpressionInfixOpcode::Add, ExpressionInfixOpcode::Mul]),
            (ExpressionInfixOpcode::Mul, vec![ExpressionInfixOpcode::Add, ExpressionInfixOpcode::Sub, ExpressionInfixOpcode::Pow]),
            (ExpressionInfixOpcode::Pow, vec![ExpressionInfixOpcode::Mul]),
            (ExpressionInfixOpcode::Div, vec![ExpressionInfixOpcode::IntDiv, ExpressionInfixOpcode::Mul]),
            (ExpressionInfixOpcode::IntDiv, vec![ExpressionInfixOpcode::Div, ExpressionInfixOpcode::Mul]),
            (ExpressionInfixOpcode::Mod, vec![ExpressionInfixOpcode::Div, ExpressionInfixOpcode::IntDiv]),
            (ExpressionInfixOpcode::BitOr, vec![ExpressionInfixOpcode::BitAnd, ExpressionInfixOpcode::BitXor]),
            (ExpressionInfixOpcode::BitAnd, vec![ExpressionInfixOpcode::BitOr, ExpressionInfixOpcode::BitXor]),
            (ExpressionInfixOpcode::BitXor, vec![ExpressionInfixOpcode::BitOr, ExpressionInfixOpcode::BitAnd]),
            (ExpressionInfixOpcode::ShiftL, vec![ExpressionInfixOpcode::ShiftR]),
            (ExpressionInfixOpcode::ShiftR, vec![ExpressionInfixOpcode::ShiftL]),
            (ExpressionInfixOpcode::Lesser, vec![ExpressionInfixOpcode::Greater, ExpressionInfixOpcode::LesserEq]),
            (ExpressionInfixOpcode::Greater, vec![ExpressionInfixOpcode::Lesser, ExpressionInfixOpcode::GreaterEq]),
            (ExpressionInfixOpcode::LesserEq, vec![ExpressionInfixOpcode::GreaterEq, ExpressionInfixOpcode::Lesser]),
            (ExpressionInfixOpcode::GreaterEq, vec![ExpressionInfixOpcode::LesserEq, ExpressionInfixOpcode::Greater]),
            (ExpressionInfixOpcode::Eq, vec![ExpressionInfixOpcode::NotEq]),
            (ExpressionInfixOpcode::NotEq, vec![ExpressionInfixOpcode::Eq]),
        ]
    };
}

/// Initializes a population of `Gene` instances by applying random constant replacement
/// and optional operator mutation to symbolic traces.
///
/// This function generates a population of symbolic mutations for a given program,
/// following the configuration provided in `mutation_config`. Each `Gene` is a mapping
/// from trace positions to their symbolic values.
///
/// # Parameters
/// - `pos`: A slice of indices representing positions in the symbolic trace to be mutated.
/// - `symbolic_trace`: A reference to the symbolic trace (`SymbolicTrace`) containing
///   symbolic expressions associated with the original program under mutation.
/// - `_base_config`: Configuration object providing base parameters for generating random constants.
///   This parameter is currently unused but reserved for potential future enhancements.
/// - `mutation_config`: Configuration object defining mutation parameters, such as population
///   size and operator mutation rate.
/// - `rng`: A mutable reference to a random number generator for consistent randomization.
///
/// # Returns
/// A vector of `Gene` instances, where each `Gene` maps positions to mutated symbolic values.
///
/// # Details
/// - For each position in the trace, this function attempts to mutate the symbolic value:
///   - If the value is a binary operation (`BinaryOp`), it may be mutated to use a related operator
///     (defined in `OPERATOR_MUTATION_CANDIDATES`) with a probability specified by
///     `mutation_config.operator_mutation_rate`.
///   - Otherwise, or if no mutation is applied, a random constant is generated and assigned.
/// - The operator mutation relies on a static mapping (`OPERATOR_MUTATION_CANDIDATES`)
///   that defines groups of related operators.
/// - The generated population size is controlled by `mutation_config.program_population_size`.
///
/// # Panics
/// - Panics if an unsupported opcode is encountered during operator mutation.
/// - Panics if the related operator group for a given opcode is empty.
pub fn initialize_population_with_operator_mutation_and_random_constant_replacement(
    pos: &[usize],
    program_population_size: usize,
    symbolic_trace: &SymbolicTrace,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> Vec<Gene> {
    (0..program_population_size)
        .map(|_| {
            let num_mutations =
                rng.gen_range(1, max(pos.len(), mutation_config.max_num_mutation_points));
            let selected_pos: Vec<_> = pos.choose_multiple(rng, num_mutations).cloned().collect();
            selected_pos
                .iter()
                .map(|p| match &*symbolic_trace[*p] {
                    SymbolicValue::BinaryOp(left, op, right) => {
                        if rng.gen::<f64>() < mutation_config.operator_mutation_rate {
                            let mutated_op = if let Some(related_ops) = OPERATOR_MUTATION_CANDIDATES
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

                            (
                                p.clone(),
                                SymbolicValue::BinaryOp(
                                    left.clone(),
                                    DebuggableExpressionInfixOpcode(mutated_op),
                                    right.clone(),
                                ),
                            )
                        } else {
                            (
                                p.clone(),
                                SymbolicValue::ConstantInt(
                                    draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
                                ),
                            )
                        }
                    }
                    _ => (
                        p.clone(),
                        SymbolicValue::ConstantInt(
                            draw_bigint_with_probabilities(&mutation_config, rng).unwrap(),
                        ),
                    ),
                })
                .collect()
        })
        .collect()
}
