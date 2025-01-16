mod utils;

use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;
use num_traits::identities::Zero;
use num_traits::One;

use rustc_hash::FxHashMap;
use tcct::executor::symbolic_execution::SymbolicExecutor;
use tcct::executor::symbolic_setting::get_default_setting_for_symbolic_execution;
use tcct::executor::symbolic_value::{OwnerName, SymbolicAccess, SymbolicName, SymbolicValue};
use tcct::solver::utils::emulate_symbolic_trace;

use crate::utils::{execute, prepare_symbolic_library};

#[test]
fn test_emulate_if_else() {
    let path = "./tests/sample/test_if_else.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime.clone(), false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let main_in = SymbolicName::new(
        sexe.symbolic_library.name2id["in"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    );
    let main_out = SymbolicName::new(
        sexe.symbolic_library.name2id["out"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    );

    let mut assignment = FxHashMap::from_iter([(main_in.clone(), BigInt::zero())]);
    let _ = emulate_symbolic_trace(
        &prime,
        &sexe.cur_state.symbolic_trace,
        &mut assignment,
        &mut sexe.symbolic_library,
    );
    assert_eq!(assignment[&main_out], BigInt::one());

    assignment.insert(main_in, BigInt::one());
    let _ = emulate_symbolic_trace(
        &prime,
        &sexe.cur_state.symbolic_trace,
        &mut assignment,
        &mut sexe.symbolic_library,
    );
    assert_eq!(assignment[&main_out], BigInt::zero());
}

#[test]
fn test_recursive_call() {
    let path = "./tests/sample/test_recursive_call.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime.clone(), false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let mut assignment = FxHashMap::from_iter((0..8).into_iter().map(|i| {
        (
            SymbolicName::new(
                sexe.symbolic_library.name2id["inputs"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::from(i)),
                )]),
            ),
            BigInt::from(i),
        )
    }));

    let _ = emulate_symbolic_trace(
        &prime,
        &sexe.cur_state.symbolic_trace,
        &mut assignment,
        &mut sexe.symbolic_library,
    );

    let main_out = SymbolicName::new(
        sexe.symbolic_library.name2id["out"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    );

    assert_eq!(assignment[&main_out], BigInt::from(19));
}
