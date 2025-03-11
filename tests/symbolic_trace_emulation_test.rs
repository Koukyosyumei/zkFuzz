mod utils;

use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;
use num_traits::identities::Zero;
use num_traits::One;

use rustc_hash::{FxHashMap, FxHashSet};
use zkfuzz::executor::symbolic_execution::SymbolicExecutor;
use zkfuzz::executor::symbolic_setting::get_default_setting_for_symbolic_execution;
use zkfuzz::executor::symbolic_value::{OwnerName, SymbolicAccess, SymbolicName, SymbolicValue};
use zkfuzz::mutator::utils::{emulate_symbolic_trace, gather_runtime_mutable_inputs, Direction};

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

    let runtime_mutable_positions = FxHashMap::default();
    let mut assignment = FxHashMap::from_iter([(main_in.clone(), BigInt::zero())]);
    let _ = emulate_symbolic_trace(
        &prime,
        &sexe.cur_state.symbolic_trace,
        &runtime_mutable_positions,
        &mut assignment,
        &mut sexe.symbolic_library,
    );
    assert_eq!(assignment[&main_out], BigInt::one());

    assignment.insert(main_in, BigInt::one());
    let _ = emulate_symbolic_trace(
        &prime,
        &sexe.cur_state.symbolic_trace,
        &runtime_mutable_positions,
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

    let runtime_mutable_positions = FxHashMap::default();
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
        &runtime_mutable_positions,
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

#[test]
fn test_call_const_template() {
    let path = "./tests/sample/test_call_const_template.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime.clone(), false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let main_a = SymbolicName::new(
        sexe.symbolic_library.name2id["a"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    );
    let main_b = SymbolicName::new(
        sexe.symbolic_library.name2id["b"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    );
    let main_c = SymbolicName::new(
        sexe.symbolic_library.name2id["c"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    );

    let runtime_mutable_positions = FxHashMap::default();
    let mut assignment = FxHashMap::from_iter([
        (main_a.clone(), BigInt::from(3)),
        (main_b.clone(), BigInt::from(4)),
    ]);
    let _ = emulate_symbolic_trace(
        &prime,
        &sexe.cur_state.symbolic_trace,
        &runtime_mutable_positions,
        &mut assignment,
        &mut sexe.symbolic_library,
    );
    assert_eq!(assignment[&main_c], BigInt::from(8));
}

#[test]
fn test_hash_break() {
    let path = "./tests/sample/test_hash_break.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime.clone(), false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let main_x = SymbolicName::new(
        sexe.symbolic_library.name2id["x"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    );
    let main_y = SymbolicName::new(
        sexe.symbolic_library.name2id["y"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    );
    let main_z = SymbolicName::new(
        sexe.symbolic_library.name2id["z"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    );

    let input_variables = FxHashSet::from_iter([main_x, main_y, main_z]);

    let runtime_mutable_positions = gather_runtime_mutable_inputs(
        &sexe.cur_state.symbolic_trace,
        sexe.symbolic_library,
        &input_variables,
    );
    assert_eq!(runtime_mutable_positions.len(), 1);
    assert_eq!(*runtime_mutable_positions.get(&2).unwrap(), Direction::Left);
}
