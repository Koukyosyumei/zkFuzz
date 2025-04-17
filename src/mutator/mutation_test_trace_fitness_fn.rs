use num_bigint_dig::BigInt;
use num_traits::Zero;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{SymbolicName, SymbolicValue, SymbolicValueRef};
use crate::mutator::mutation_config::MutationConfig;
use crate::mutator::mutation_utils::apply_trace_mutation;
use crate::mutator::utils::{
    accumulate_error_of_constraints, count_error_constraints, emulate_symbolic_trace,
    evaluate_constraints, is_equal_mod, max_error_of_constraints, BaseVerificationConfig,
    CounterExample, Direction, UnderConstrainedType, VerificationResult,
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
/// - `mutation_config`: The mutation-specific configuration, including parameters such as
///   population size, mutation rate, and maximum number of generations.
/// - `symbolic_trace`: A vector of references to symbolic values representing the trace to be evaluated.
/// - `side_constraints`: A vector of references to symbolic values representing additional constraints for the evaluation.
/// - `runtime_mutable_positions`: A map of runtime mutable positions.
/// - `trace_mutation`: A mapping of indices to mutated symbolic values applied to the trace.
/// - `inputs_assignment`: A vector of potential input assignments, where each assignment is a mapping of symbolic names to `BigInt` values.
/// - `fitness_scores_inputs`: A vector to store the fitness scores of inputs.
///
/// # Returns
/// A tuple containing:
/// - `usize`: The index of the input assignment with the best fitness score.
/// - `BigInt`: The maximum fitness score achieved.
/// - `Option<CounterExample>`: An optional counterexample, if the trace is found to be under-constrained or over-constrained.
/// - `usize`: The number of invalid input assignment causing out-of-range subscript
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
/// - This function terminates early if a valid counterexample is found.
pub fn evaluate_trace_fitness_by_error(
    sexe: &mut SymbolicExecutor,
    base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    symbolic_trace: &Vec<SymbolicValueRef>,
    side_constraints: &Vec<SymbolicValueRef>,
    runtime_mutable_positions: &FxHashMap<usize, Direction>,
    trace_mutation: &FxHashMap<usize, SymbolicValue>,
    inputs_assignment: &Vec<FxHashMap<SymbolicName, BigInt>>,
    fitness_scores_inputs: &mut Vec<BigInt>,
) -> (usize, BigInt, Option<CounterExample>, usize) {
    // Apply the given mutations to the symbolic trace.
    let mutated_symbolic_trace = apply_trace_mutation(symbolic_trace, trace_mutation);

    let mut max_idx = 0_usize;
    let mut max_score = -base_config.prime.clone();
    let mut counter_example = None;
    let mut num_invalida_assignments = 0; // invalid assignments causing out-of-range subscript

    for (i, inp) in inputs_assignment.iter().enumerate() {
        // Clone the input assignment for evaluation with the original program.
        let mut assignment_for_original = inp.clone();

        // Emulate the original trace to evaluate its behavior on the given input.
        // Even if an assertion fails, the function proceeds, treating it as a modified trace with no assertions.
        let emulation_result = emulate_symbolic_trace(
            &base_config.prime,
            &symbolic_trace,
            runtime_mutable_positions,
            &mut assignment_for_original,
            &mut sexe.symbolic_library,
        );
        if emulation_result.is_none() {
            num_invalida_assignments += 1;
            continue;
        }
        let (is_original_program_success, original_program_failure_pos) = emulation_result.unwrap();
        // Check if the original trace satisfies the side constraints.
        let is_original_satisfy_sc = evaluate_constraints(
            &base_config.prime,
            side_constraints,
            &assignment_for_original,
            &mut sexe.symbolic_library,
        );
        // The original program succeeds, but the side constraints fail.
        if is_original_program_success && !is_original_satisfy_sc {
            counter_example = Some(CounterExample {
                flag: VerificationResult::OverConstrained,
                target_output: None,
                assignment: assignment_for_original.clone(),
            });
            max_idx = i;
            max_score = BigInt::zero();
            break;
        }

        // The original program fails, but the mutated program, where all assertions are removed,
        // satisfies the side constraints.
        if !is_original_program_success && is_original_satisfy_sc {
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

        // Clone the input assignment for evaluating the mutated trace.
        let mut assignment_for_mutation = inp.clone();

        // Emulate the mutated trace and evaluate the error in side constraints.
        let mutated_emulation_result = emulate_symbolic_trace(
            &base_config.prime,
            &mutated_symbolic_trace,
            runtime_mutable_positions,
            &mut assignment_for_mutation,
            &mut sexe.symbolic_library.clone(),
        );
        if mutated_emulation_result.is_none() {
            break;
        }
        let (_is_mutated_program_success, _mutated_program_failure_pos) =
            mutated_emulation_result.unwrap();
        // Calculate the error in side constraints for the mutated trace.

        let error_of_side_constraints_for_mutated_assignment =
            if mutation_config.fitness_function == "count-error" {
                count_error_constraints(
                    &base_config.prime,
                    side_constraints,
                    &assignment_for_mutation,
                    &mut sexe.symbolic_library,
                )
            } else if mutation_config.fitness_function == "max-error" {
                max_error_of_constraints(
                    &base_config.prime,
                    side_constraints,
                    &assignment_for_mutation,
                    &mut sexe.symbolic_library,
                )
            } else {
                accumulate_error_of_constraints(
                    &base_config.prime,
                    side_constraints,
                    &assignment_for_mutation,
                    &mut sexe.symbolic_library,
                )
            };
        let mut score = -error_of_side_constraints_for_mutated_assignment.clone();

        // Check for valid solutions that satisfy all side constraints.
        if error_of_side_constraints_for_mutated_assignment.is_zero() {
            if !is_original_program_success {
                // the original fails but the mutated satisfies constraints.
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
                // Verify consistency of outputs for valid solutions.
                let mut keys: Vec<_> = assignment_for_original.keys().collect();
                keys.sort();
                for k in keys {
                    let v = assignment_for_original.get(k).unwrap();
                    if k.owner.len() == 1
                        && sexe.symbolic_library.template_library
                            [&sexe.symbolic_library.name2id[&base_config.target_template_name]]
                            .output_ids
                            .contains(&k.id)
                    {
                        // If outputs differ, mark as a non-deterministic under-constrained issue.
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
            // Penalize valid solutions by setting their score to the worst possible value.
            score = -base_config.prime.clone();
        }

        if fitness_scores_inputs[i] > score.clone() {
            fitness_scores_inputs[i] = score.clone();
        }

        if score > max_score {
            max_idx = i;
            max_score = score;
        }
    }

    (
        max_idx,
        max_score,
        counter_example,
        num_invalida_assignments,
    )
}
