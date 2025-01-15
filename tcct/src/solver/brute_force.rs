use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use num_bigint_dig::BigInt;
use num_traits::{One, Zero};
use rustc_hash::FxHashMap;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{SymbolicName, SymbolicValueRef};

use crate::solver::utils::{
    extract_variables, is_vulnerable, verify_assignment, CounterExample, VerificationResult,
    VerificationSetting,
};

/// Performs a brute-force search over variable assignments to evaluate constraints.
///
/// # Parameters
/// - `sexe`: A mutable reference to the symbolic executor.
/// - `symbolic_trace`: A vector of constraints representing the program trace.
/// - `side_constraints`: A vector of additional constraints for validation.
/// - `setting`: The verification settings.
///
/// # Returns
/// An `Option<CounterExample>` containing a counterexample if constraints are invalid, or `None` otherwise.
pub fn brute_force_search(
    sexe: &mut SymbolicExecutor,
    symbolic_trace: &Vec<SymbolicValueRef>,
    side_constraints: &Vec<SymbolicValueRef>,
    setting: &VerificationSetting,
) -> Option<CounterExample> {
    let mut trace_variables = extract_variables(symbolic_trace);
    let mut side_variables = extract_variables(side_constraints);

    let mut variables = Vec::new();
    variables.append(&mut trace_variables);
    variables.append(&mut side_variables);
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();
    variables = variables_set.into_iter().collect();

    let mut assignment = FxHashMap::default();
    let current_iteration = Arc::new(AtomicUsize::new(0));

    fn search(
        sexe: &mut SymbolicExecutor,
        symbolic_trace: &[SymbolicValueRef],
        side_constraints: &[SymbolicValueRef],
        setting: &VerificationSetting,
        index: usize,
        variables: &[SymbolicName],
        assignment: &mut FxHashMap<SymbolicName, BigInt>,
        current_iteration: &Arc<AtomicUsize>,
    ) -> VerificationResult {
        if index == variables.len() {
            let iter = current_iteration.fetch_add(1, Ordering::SeqCst);
            if iter % setting.progress_interval == 0 {
                print!(
                    "\rProgress: {} / {}^{}",
                    iter,
                    &setting.prime,
                    variables.len()
                );
                io::stdout().flush().unwrap();
            }

            return verify_assignment(sexe, symbolic_trace, side_constraints, assignment, setting);
        }

        let var = &variables[index];
        if setting.quick_mode {
            let candidates = vec![BigInt::zero(), BigInt::one(), -1 * BigInt::one()];
            for c in candidates.into_iter() {
                assignment.insert(var.clone(), c.clone());
                let result = search(
                    sexe,
                    symbolic_trace,
                    side_constraints,
                    setting,
                    index + 1,
                    variables,
                    assignment,
                    current_iteration,
                );
                if is_vulnerable(&result) {
                    return result;
                }
                assignment.remove(var);
            }
        } else if setting.heuristics_mode {
            let mut value = -&setting.range;
            while value <= setting.range {
                assignment.insert(var.clone(), value.clone());

                let result = search(
                    sexe,
                    symbolic_trace,
                    side_constraints,
                    setting,
                    index + 1,
                    variables,
                    assignment,
                    current_iteration,
                );

                if is_vulnerable(&result) {
                    return result;
                }
                assignment.remove(&var);
                value += BigInt::one();
            }
            let mut value = &setting.prime - &setting.range;

            while value < setting.prime {
                assignment.insert(var.clone(), value.clone());

                let result = search(
                    sexe,
                    symbolic_trace,
                    side_constraints,
                    setting,
                    index + 1,
                    variables,
                    assignment,
                    current_iteration,
                );

                if is_vulnerable(&result) {
                    return result;
                }
                assignment.remove(&var);
                value += BigInt::one();
            }
        } else {
            let mut value = BigInt::zero();
            while value < setting.prime {
                assignment.insert(var.clone(), value.clone());
                let result = search(
                    sexe,
                    symbolic_trace,
                    side_constraints,
                    setting,
                    index + 1,
                    variables,
                    assignment,
                    current_iteration,
                );
                if is_vulnerable(&result) {
                    return result;
                }
                assignment.remove(var);
                value += BigInt::one();
            }
        }
        VerificationResult::WellConstrained
    }

    let flag = search(
        sexe,
        &symbolic_trace,
        &side_constraints,
        setting,
        0,
        &variables,
        &mut assignment,
        &current_iteration,
    );

    print!(
        "\rProgress: {} / {}^{}",
        current_iteration.load(Ordering::SeqCst),
        setting.prime,
        variables.len()
    );
    io::stdout().flush().unwrap();

    println!("\n • Search completed");
    println!(
        "     ├─ Total iterations: {}",
        current_iteration.load(Ordering::SeqCst)
    );
    println!("     └─ Verification result: {}", flag);

    if is_vulnerable(&flag) {
        Some(CounterExample {
            flag: flag,
            target_output: None,
            assignment: assignment,
        })
    } else {
        None
    }
}
