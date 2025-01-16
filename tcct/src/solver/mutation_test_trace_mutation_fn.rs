use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::str::FromStr;

use colored::Colorize;
use log::info;
use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::{One, Zero};
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};
use rustc_hash::FxHashMap;

use program_structure::ast::ExpressionInfixOpcode;

use crate::executor::debug_ast::DebuggableExpressionInfixOpcode;
use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_state::{SymbolicConstraints, SymbolicTrace};
use crate::executor::symbolic_value::{OwnerName, SymbolicName, SymbolicValue, SymbolicValueRef};

use crate::solver::mutation_config::MutationConfig;
use crate::solver::mutation_test::Gene;
use crate::solver::mutation_utils::draw_random_constant;
use crate::solver::utils::{extract_variables, BaseVerificationConfig, CounterExample};

pub fn trace_mutate(individual: &mut Gene, base_config: &BaseVerificationConfig, rng: &mut StdRng) {
    if !individual.is_empty() {
        let var = individual.keys().choose(rng).unwrap();
        individual.insert(
            var.clone(),
            SymbolicValue::ConstantInt(draw_random_constant(base_config, rng)),
        );
    }
}
