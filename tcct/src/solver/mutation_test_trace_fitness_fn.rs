use num_bigint_dig::BigInt;
use num_traits::Zero;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{SymbolicName, SymbolicValue, SymbolicValueRef};

use crate::solver::mutation_utils::apply_trace_mutation;
use crate::solver::utils::{
    accumulate_error_of_constraints, emulate_symbolic_values, is_vulnerable, verify_assignment,
    BaseVerificationConfig, CounterExample, UnderConstrainedType, VerificationResult,
};

pub fn evaluate_trace_fitness_by_error(
    sexe: &mut SymbolicExecutor,
    base_config: &BaseVerificationConfig,
    symbolic_trace: &Vec<SymbolicValueRef>,
    side_constraints: &Vec<SymbolicValueRef>,
    trace_mutation: &FxHashMap<usize, SymbolicValue>,
    inputs_assignment: &Vec<FxHashMap<SymbolicName, BigInt>>,
) -> (usize, BigInt, Option<CounterExample>) {
    let mutated_symbolic_trace = apply_trace_mutation(symbolic_trace, trace_mutation);

    let mut max_idx = 0_usize;
    let mut max_score = -base_config.prime.clone();
    let mut counter_example = None;

    for (i, inp) in inputs_assignment.iter().enumerate() {
        let mut assignment = inp.clone();

        let (is_success, failure_pos) = emulate_symbolic_values(
            &base_config.prime,
            &mutated_symbolic_trace,
            &mut assignment,
            &mut sexe.symbolic_library,
        );
        let error_of_side_constraints = accumulate_error_of_constraints(
            &base_config.prime,
            side_constraints,
            &assignment,
            &mut sexe.symbolic_library,
        );
        let mut score = -error_of_side_constraints.clone();

        if error_of_side_constraints.is_zero() {
            if is_success {
                let flag = verify_assignment(
                    sexe,
                    symbolic_trace,
                    side_constraints,
                    &assignment,
                    base_config,
                );
                if is_vulnerable(&flag) {
                    max_idx = i;
                    max_score = BigInt::zero();
                    counter_example = if let VerificationResult::UnderConstrained(
                        UnderConstrainedType::NonDeterministic(sym_name, _, _),
                    ) = &flag
                    {
                        Some(CounterExample {
                            flag: flag.clone(),
                            target_output: Some(sym_name.clone()),
                            assignment: assignment.clone(),
                        })
                    } else {
                        Some(CounterExample {
                            flag: flag,
                            target_output: None,
                            assignment: assignment.clone(),
                        })
                    };
                    break;
                } else {
                    score = -base_config.prime.clone();
                }
            } else {
                if trace_mutation.is_empty() {
                    max_idx = i;
                    max_score = BigInt::zero();
                    counter_example = Some(CounterExample {
                        flag: VerificationResult::UnderConstrained(
                            UnderConstrainedType::UnexpectedInput(
                                failure_pos,
                                mutated_symbolic_trace[failure_pos]
                                    .lookup_fmt(&sexe.symbolic_library.id2name),
                            ),
                        ),
                        target_output: None,
                        assignment: assignment.clone(),
                    });
                    break;
                }
            }
        } else {
            if trace_mutation.is_empty() && is_success {
                max_idx = i;
                max_score = BigInt::zero();
                counter_example = Some(CounterExample {
                    flag: VerificationResult::OverConstrained,
                    target_output: None,
                    assignment: assignment.clone(),
                });
                break;
            }
        }

        if score > max_score {
            max_idx = i;
            max_score = score;
        }
    }

    (max_idx, max_score, counter_example)
}
