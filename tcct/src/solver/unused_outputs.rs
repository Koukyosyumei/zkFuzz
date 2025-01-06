use std::collections::HashSet;

use num_bigint_dig::BigInt;
use num_traits::Zero;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{register_array_elements, SymbolicName};

use crate::solver::utils::{
    extract_variables, CounterExample, UnderConstrainedType, VerificationResult,
    VerificationSetting,
};

pub fn check_unused_outputs(
    sexe: &mut SymbolicExecutor,
    setting: &VerificationSetting,
) -> Option<CounterExample> {
    let mut variables: Vec<SymbolicName> = Vec::new();
    variables.append(&mut extract_variables(
        &sexe.cur_state.trace_constraints.clone(),
    ));
    variables.append(&mut extract_variables(
        &sexe.cur_state.trace_constraints.clone(),
    ));
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();

    let mut used_outputs: FxHashMap<SymbolicName, Option<bool>> = FxHashMap::default();
    for oup_name in &sexe.symbolic_library.template_library
        [&sexe.symbolic_library.name2id[&setting.id]]
        .output_ids
        .clone()
    {
        let dims = sexe.evaluate_dimension(
            &sexe.symbolic_library.template_library[&sexe.symbolic_library.name2id[&setting.id]]
                .id2dimensions[&oup_name]
                .clone(),
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
            assignment: dummy_assignment,
        })
    } else {
        None
    }
}
