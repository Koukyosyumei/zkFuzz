use std::collections::HashSet;

use num_bigint_dig::BigInt;
use num_traits::Zero;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{extract_variables, register_array_elements, SymbolicName};
use crate::mutator::utils::{
    BaseVerificationConfig, CounterExample, UnderConstrainedType, VerificationResult,
};

/// Checks for unused outputs in the symbolic execution trace and returns a counterexample if any are found.
///
/// # Parameters
/// - `sexe`: A mutable reference to the `SymbolicExecutor`, which holds the current state of symbolic execution,
///   including the symbolic trace and symbolic library.
/// - `base_config`: A reference to the `BaseVerificationConfig`, which contains configuration information
///   such as the target template name to analyze.
///
/// # Returns
/// An `Option<CounterExample>` containing:
/// - `Some(CounterExample)` if unused outputs are detected, providing details about the unused outputs
///   and a dummy assignment.
/// - `None` if all outputs are used, indicating no under-constrained outputs.
///
/// # Behavior
/// 1. Extracts all variables used in the current symbolic execution trace.
/// 2. Collects all outputs defined in the target template specified in `base_config`.
/// 3. Compares the collected outputs against the used variables to identify unused outputs.
/// 4. If unused outputs are found:
///    - Constructs a `CounterExample` with the unused outputs marked as under-constrained.
///    - Assigns dummy values (e.g., zero) to the unused outputs for illustrative purposes.
/// 5. If all outputs are used, returns `None`.
///
/// # Notes
/// - This function assumes that the `SymbolicExecutor` contains a valid symbolic trace and a populated
///   symbolic library.
/// - The returned `CounterExample` highlights unused outputs as a potential issue, classified under
///   `UnderConstrainedType::UnusedOutput`.
///
/// # Performance
/// - The function iterates through the symbolic trace and template outputs, which may incur overhead
///   proportional to the number of variables and outputs. Use cautiously in performance-critical contexts.
pub fn check_unused_outputs(
    sexe: &mut SymbolicExecutor,
    base_config: &BaseVerificationConfig,
) -> Option<CounterExample> {
    let mut variables: Vec<SymbolicName> = Vec::new();
    variables.append(&mut extract_variables(
        &sexe.cur_state.symbolic_trace.clone(),
    ));
    variables.append(&mut extract_variables(
        &sexe.cur_state.symbolic_trace.clone(),
    ));
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();

    let mut used_outputs: FxHashMap<SymbolicName, Option<bool>> = FxHashMap::default();
    for oup_name in &sexe.symbolic_library.template_library
        [&sexe.symbolic_library.name2id[&base_config.target_template_name]]
        .output_ids
        .clone()
    {
        let dims = sexe.evaluate_dimension(
            &sexe.symbolic_library.template_library
                [&sexe.symbolic_library.name2id[&base_config.target_template_name]]
                .id2dimension_expressions[&oup_name]
                .clone(),
            usize::MAX,
        );
        register_array_elements(
            *oup_name,
            &dims,
            Some(sexe.cur_state.owner_name.clone()),
            &mut used_outputs,
        );
    }
    let unused_outputs: Vec<SymbolicName> = used_outputs
        .keys()
        .filter(|key| !variables_set.contains(*key))
        .cloned()
        .collect();
    if !unused_outputs.is_empty() {
        let dummy_assignment: FxHashMap<SymbolicName, BigInt> = unused_outputs
            .iter()
            .map(|uo| (uo.clone(), BigInt::zero()))
            .collect();
        Some(CounterExample {
            flag: VerificationResult::UnderConstrained(UnderConstrainedType::UnusedOutput),
            target_output: None,
            assignment: dummy_assignment,
        })
    } else {
        None
    }
}
