mod utils;

use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;
use num_traits::identities::Zero;
use num_traits::One;
use rustc_hash::FxHashMap;

use program_structure::ast::Expression;
use program_structure::program_archive::ProgramArchive;

use zkfuzz::executor::symbolic_execution::SymbolicExecutor;
use zkfuzz::executor::symbolic_setting::get_default_setting_for_concrete_execution;
use zkfuzz::executor::symbolic_value::{OwnerName, SymbolicAccess, SymbolicName, SymbolicValue};

use crate::utils::prepare_symbolic_library;

fn get_inputs(cexe: &SymbolicExecutor, inputs: &[BigInt]) -> FxHashMap<SymbolicName, BigInt> {
    let mut map = FxHashMap::default();
    map.insert(
        SymbolicName::new(
            cexe.symbolic_library.name2id["in"],
            Rc::new(vec![OwnerName {
                id: cexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            Some(vec![SymbolicAccess::ArrayAccess(
                SymbolicValue::ConstantInt(BigInt::zero()),
            )]),
        ),
        inputs[0].clone(),
    );
    map.insert(
        SymbolicName::new(
            cexe.symbolic_library.name2id["in"],
            Rc::new(vec![OwnerName {
                id: cexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            Some(vec![SymbolicAccess::ArrayAccess(
                SymbolicValue::ConstantInt(BigInt::one()),
            )]),
        ),
        inputs[1].clone(),
    );
    map
}

pub fn concrete_execute(
    cexe: &mut SymbolicExecutor,
    program_archive: &ProgramArchive,
    inputs: &[BigInt],
) {
    match &program_archive.initial_template_call {
        Expression::Call { id, args, .. } => {
            let template = program_archive.templates[id].clone();

            cexe.symbolic_library
                .name2id
                .insert("main".to_string(), cexe.symbolic_library.name2id.len());
            cexe.symbolic_library
                .id2name
                .insert(cexe.symbolic_library.name2id["main"], "main".to_string());
            cexe.cur_state
                .set_template_id(cexe.symbolic_library.name2id[id]);

            cexe.cur_state.add_owner(&OwnerName {
                id: cexe.symbolic_library.name2id["main"],
                counter: 0,
                access: None,
            });
            cexe.cur_state
                .set_template_id(cexe.symbolic_library.name2id[id]);

            cexe.feed_arguments(template.get_name_of_params(), args);

            let assignment = get_inputs(&cexe, inputs);
            cexe.concrete_execute(&"Main".to_string(), &assignment);
        }
        _ => {
            panic!("Cannot Find Main Call");
        }
    }
}

#[test]
fn test_coverage_test() {
    let path = "./tests/sample/test_coverage.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_concrete_execution(prime, false);

    let mut cexe = SymbolicExecutor::new(&mut symbolic_library, &setting);

    cexe.turn_on_coverage_tracking();
    concrete_execute(
        &mut cexe,
        &program_archive,
        &[BigInt::zero(), BigInt::zero()],
    );
    assert_eq!(0, cexe.coverage_count());
    cexe.record_path();
    assert_eq!(1, cexe.coverage_count());
    cexe.record_path();
    assert_eq!(1, cexe.coverage_count());

    cexe.clear();
    concrete_execute(
        &mut cexe,
        &program_archive,
        &[BigInt::zero(), BigInt::one()],
    );
    assert_eq!(1, cexe.coverage_count());
    cexe.record_path();
    assert_eq!(2, cexe.coverage_count());

    cexe.clear();
    concrete_execute(
        &mut cexe,
        &program_archive,
        &[BigInt::zero(), BigInt::zero()],
    );
    assert_eq!(2, cexe.coverage_count());
    cexe.record_path();
    assert_eq!(2, cexe.coverage_count());

    cexe.clear();
    concrete_execute(
        &mut cexe,
        &program_archive,
        &[BigInt::one(), BigInt::zero()],
    );
    assert_eq!(2, cexe.coverage_count());
    cexe.record_path();
    assert_eq!(3, cexe.coverage_count());

    cexe.clear();
    concrete_execute(
        &mut cexe,
        &program_archive,
        &[BigInt::from_str("3").unwrap(), BigInt::zero()],
    );
    assert_eq!(3, cexe.coverage_count());
    cexe.record_path();
    assert_eq!(3, cexe.coverage_count());

    cexe.clear();
    concrete_execute(&mut cexe, &program_archive, &[BigInt::one(), BigInt::one()]);
    assert_eq!(3, cexe.coverage_count());
    cexe.record_path();
    assert_eq!(4, cexe.coverage_count());
}
