use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::rc::Rc;

use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::One;
use num_traits::Zero;
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;
use rustc_hash::FxHashMap;
use std::str::FromStr;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{
    register_array_elements, OwnerName, SymbolicName, SymbolicValue, SymbolicValueRef,
};

use crate::solver::utils::{
    count_satisfied_constraints, emulate_symbolic_values, evaluate_constraints, extract_variables,
    is_vulnerable, verify_assignment, CounterExample, UnderConstrainedType, VerificationResult,
    VerificationSetting,
};

pub fn check_unused_outputs(
    sexe: &mut SymbolicExecutor,
    setting: &VerificationSetting,
) -> Option<CounterExample> {
    let mut variables: Vec<SymbolicName> = Vec::new();
    for s in &sexe.symbolic_store.final_states {
        variables.append(&mut extract_variables(&s.trace_constraints.clone()));
        variables.append(&mut extract_variables(&s.trace_constraints.clone()));
    }
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();

    let mut used_outputs: FxHashMap<SymbolicName, Option<bool>> = FxHashMap::default();
    for oup_name in &sexe.symbolic_library.template_library
        [&sexe.symbolic_library.name2id[&setting.id]]
        .outputs
        .clone()
    {
        let dims = sexe.evaluate_dimension(
            &sexe.symbolic_library.template_library[&sexe.symbolic_library.name2id[&setting.id]]
                .output_dimensions[&oup_name]
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
