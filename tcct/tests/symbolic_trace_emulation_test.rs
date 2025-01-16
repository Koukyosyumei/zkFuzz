mod utils;

use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;
use num_traits::identities::Zero;
use num_traits::One;

use program_structure::ast::{Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode};

use tcct::executor::debug_ast::{
    DebuggableExpressionInfixOpcode, DebuggableExpressionPrefixOpcode,
};
use tcct::executor::symbolic_execution::SymbolicExecutor;
use tcct::executor::symbolic_setting::get_default_setting_for_symbolic_execution;
use tcct::executor::symbolic_value::{OwnerName, SymbolicAccess, SymbolicName, SymbolicValue};
use tcct::solver::unused_outputs::check_unused_outputs;
use tcct::solver::utils::BaseVerificationConfig;

use crate::utils::{execute, prepare_symbolic_library};

fn test_emulate_if_else() {
    let path = "./tests/sample/test_if_else.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);
}
