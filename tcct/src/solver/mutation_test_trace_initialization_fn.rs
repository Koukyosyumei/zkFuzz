use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::Rng;

use program_structure::ast::ExpressionInfixOpcode;

use crate::executor::debug_ast::DebuggableExpressionInfixOpcode;
use crate::executor::symbolic_value::{SymbolicValue, SymbolicValueRef};

use crate::solver::mutation_config::MutationConfig;
use crate::solver::mutation_test::Gene;
use crate::solver::mutation_utils::draw_random_constant;
use crate::solver::utils::BaseVerificationConfig;

pub fn initialize_population_with_random_constant_replacement(
    pos: &[usize],
    base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> Vec<Gene> {
    (0..mutation_config.program_population_size)
        .map(|_| {
            pos.iter()
                .map(|p| {
                    (
                        p.clone(),
                        SymbolicValue::ConstantInt(draw_random_constant(base_config, rng)),
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

fn initialize_population_with_operator_mutation_and_random_constant_replacement(
    pos: &[usize],
    symbolic_trace: &[SymbolicValueRef],
    operator_mutation_rate: f64,
    base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> Vec<Gene> {
    (0..mutation_config.program_population_size)
        .map(|_| {
            pos.iter()
                .map(|p| match &*symbolic_trace[*p] {
                    SymbolicValue::BinaryOp(left, op, right) => {
                        if rng.gen::<f64>() < operator_mutation_rate {
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
                                SymbolicValue::ConstantInt(draw_random_constant(base_config, rng)),
                            )
                        }
                    }
                    _ => (
                        p.clone(),
                        SymbolicValue::ConstantInt(draw_random_constant(base_config, rng)),
                    ),
                })
                .collect()
        })
        .collect()
}
