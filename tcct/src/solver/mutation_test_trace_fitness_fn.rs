use num_bigint_dig::BigInt;
use num_traits::Zero;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{SymbolicName, SymbolicValue, SymbolicValueRef};

use crate::solver::mutation_utils::apply_trace_mutation;
use crate::solver::utils::{
    accumulate_error_of_constraints, emulate_symbolic_trace, evaluate_constraints, is_equal_mod,
    BaseVerificationConfig, CounterExample, UnderConstrainedType, VerificationResult,
};

/// Evaluates the fitness of a mutated symbolic execution trace by calculating the error score.
///
/// This function applies a mutation to a symbolic trace and evaluates the fitness of the trace
/// based on its ability to satisfy both the trace's symbolic constraints and the given side constraints.
/// If the trace produces a counterexample, such as an under-constrained or over-constrained assignment,
/// it is returned along with the fitness score.
///
/// # Parameters
/// - `sexe`: A mutable reference to a `SymbolicExecutor` instance responsible for symbolic execution.
/// - `base_config`: The base verification configuration, containing the prime modulus and other verification parameters.
/// - `symbolic_trace`: A vector of references to symbolic values representing the trace to be evaluated.
/// - `side_constraints`: A vector of references to symbolic values representing additional constraints for the evaluation.
/// - `trace_mutation`: A mapping of indices to mutated symbolic values applied to the trace.
/// - `inputs_assignment`: A vector of potential input assignments, where each assignment is a mapping of symbolic names to `BigInt` values.
///
/// # Returns
/// A tuple containing:
/// - `usize`: The index of the input assignment with the best fitness score.
/// - `BigInt`: The maximum fitness score achieved.
/// - `Option<CounterExample>`: An optional counterexample, if the trace is found to be under-constrained or over-constrained.
///
/// # Behavior
/// 1. Applies the provided mutation to the symbolic trace.
/// 2. For each input assignment:
///    - Simulates the trace using the assignment and evaluates errors in the side constraints.
///    - Checks if the trace successfully satisfies the constraints and whether it results in a counterexample.
/// 3. Tracks the highest fitness score and the associated input assignment.
/// 4. If a counterexample is found, the evaluation halts early and returns the result.
///
/// # Fitness Scoring
/// - Fitness scores are calculated based on the negated error of the side constraints.
/// - A score of zero indicates either an over-constrained or under-constrained trace with a corresponding counterexample.
///
/// # Notes
/// - If the provided `trace_mutation` is empty, the function evaluates the original trace directly.
/// - This function terminates early if a valid counterexample is found.
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
        // Run the original program on `inp`
        let mut assignment_for_original = inp.clone();

        // Note: Even if the assert condition is violated, `emulate_symbolic_trace` continues the execution,
        // and we can view it as a mutated program where all asserts are removed.
        let (is_original_program_success, original_program_failure_pos) = emulate_symbolic_trace(
            &base_config.prime,
            &symbolic_trace,
            &mut assignment_for_original,
            &mut sexe.symbolic_library,
        );
        let is_original_satisfy_sc = evaluate_constraints(
            &base_config.prime,
            side_constraints,
            &assignment_for_original,
            &mut sexe.symbolic_library,
        );

        if is_original_program_success && !is_original_satisfy_sc {
            // The original program does not fail on this input, while the side-constraints
            // does not accept its witness.
            counter_example = Some(CounterExample {
                flag: VerificationResult::OverConstrained,
                target_output: None,
                assignment: assignment_for_original.clone(),
            });
            max_idx = i;
            max_score = BigInt::zero();
            break;
        }
        if !is_original_program_success && is_original_satisfy_sc {
            // The original program crashes on this input, while the witness from the
            // mutated program, where all asserts are removed, satisfies the side-constraints.
            counter_example = Some(CounterExample {
                flag: VerificationResult::UnderConstrained(UnderConstrainedType::UnexpectedInput(
                    original_program_failure_pos,
                    symbolic_trace[original_program_failure_pos]
                        .lookup_fmt(&sexe.symbolic_library.id2name),
                )),
                target_output: None,
                assignment: assignment_for_original.clone(),
            });
            max_idx = i;
            max_score = BigInt::zero();
            break;
        }

        let mut assignment_for_mutation = inp.clone();

        // We can view that asserts are removed from the mutated program.
        let (_is_mutated_program_success, _mutated_program_failure_pos) = emulate_symbolic_trace(
            &base_config.prime,
            &mutated_symbolic_trace,
            &mut assignment_for_mutation,
            &mut sexe.symbolic_library,
        );
        let error_of_side_constraints_for_mutated_assignment = accumulate_error_of_constraints(
            &base_config.prime,
            side_constraints,
            &assignment_for_mutation,
            &mut sexe.symbolic_library,
        );
        let mut score = -error_of_side_constraints_for_mutated_assignment.clone();

        if error_of_side_constraints_for_mutated_assignment.is_zero() {
            // the witness from the mutated program satisfies all the side-constraints
            if !is_original_program_success {
                // the original program is supposed to crash on this input, while there
                // exists an assignment for this input that satisfies the side-constraints
                counter_example = Some(CounterExample {
                    flag: VerificationResult::UnderConstrained(
                        UnderConstrainedType::UnexpectedInput(
                            original_program_failure_pos,
                            symbolic_trace[original_program_failure_pos]
                                .lookup_fmt(&sexe.symbolic_library.id2name),
                        ),
                    ),
                    target_output: None,
                    assignment: assignment_for_mutation.clone(),
                });
                max_idx = i;
                max_score = BigInt::zero();
                break;
            } else {
                // check the consistency of the outputs
                for (k, v) in assignment_for_original {
                    if k.owner.len() == 1
                        && sexe.symbolic_library.template_library
                            [&sexe.symbolic_library.name2id[&base_config.target_template_name]]
                            .output_ids
                            .contains(&k.id)
                    {
                        if !is_equal_mod(&v, &assignment_for_mutation[&k], &base_config.prime) {
                            counter_example = Some(CounterExample {
                                flag: VerificationResult::UnderConstrained(
                                    UnderConstrainedType::NonDeterministic(
                                        k.clone(),
                                        k.lookup_fmt(&sexe.symbolic_library.id2name),
                                        v.clone(),
                                    ),
                                ),
                                target_output: Some(k.clone()),
                                assignment: assignment_for_mutation,
                            });
                            break;
                        }
                    }
                }
                if counter_example.is_some() {
                    max_idx = i;
                    max_score = BigInt::zero();
                    break;
                }
            }
            score = -base_config.prime.clone();
        }

        if score > max_score {
            max_idx = i;
            max_score = score;
        }
    }

    (max_idx, max_score, counter_example)
}
