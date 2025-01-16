use rand::rngs::StdRng;
use rand::seq::IteratorRandom;

use crate::executor::symbolic_value::SymbolicValue;

use crate::solver::mutation_test::Gene;
use crate::solver::mutation_utils::draw_random_constant;
use crate::solver::utils::BaseVerificationConfig;

pub fn trace_mutate(individual: &mut Gene, base_config: &BaseVerificationConfig, rng: &mut StdRng) {
    if !individual.is_empty() {
        let var = individual.keys().choose(rng).unwrap();
        individual.insert(
            var.clone(),
            SymbolicValue::ConstantInt(draw_random_constant(base_config, rng)),
        );
    }
}
