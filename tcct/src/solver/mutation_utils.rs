use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::One;
use rand::rngs::StdRng;
use rand::Rng;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_value::{SymbolicValue, SymbolicValueRef};

use crate::solver::utils::BaseVerificationConfig;

pub fn draw_random_constant(base_config: &BaseVerificationConfig, rng: &mut StdRng) -> BigInt {
    if rng.gen::<bool>() {
        rng.gen_bigint_range(
            &(BigInt::from_str("10").unwrap() * -BigInt::one()),
            &(BigInt::from_str("10").unwrap()),
        )
    } else {
        rng.gen_bigint_range(
            &(base_config.prime.clone() - BigInt::from_str("100").unwrap()),
            &(base_config.prime),
        )
    }
}

pub fn apply_trace_mutation(
    symbolic_trace: &Vec<SymbolicValueRef>,
    trace_mutation: &FxHashMap<usize, SymbolicValue>,
) -> Vec<SymbolicValueRef> {
    let mut mutated_constraints = symbolic_trace.clone();
    for (index, value) in trace_mutation {
        if let SymbolicValue::Assign(lv, _, is_safe) = mutated_constraints[*index].as_ref().clone()
        {
            mutated_constraints[*index] = Rc::new(SymbolicValue::Assign(
                lv.clone(),
                Rc::new(value.clone()),
                is_safe,
            ));
        } else if let SymbolicValue::AssignCall(lv, _, is_mutable) =
            mutated_constraints[*index].as_ref().clone()
        {
            mutated_constraints[*index] = Rc::new(SymbolicValue::Assign(
                lv.clone(),
                Rc::new(value.clone()),
                !is_mutable,
            ));
        } else {
            panic!("We can only mutate SymbolicValue::Assign");
        }
    }
    mutated_constraints
}
