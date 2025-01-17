use rand::rngs::StdRng;
use rand::seq::IteratorRandom;

use crate::executor::symbolic_value::SymbolicValue;
use crate::solver::mutation_config::MutationConfig;
use crate::solver::mutation_test::Gene;
use crate::solver::mutation_utils::{draw_bigint_with_probabilities, draw_random_constant};
use crate::solver::utils::BaseVerificationConfig;

pub fn mutate_trace_with_random_constant_replacement(
    individual: &mut Gene,
    _base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    if !individual.is_empty() {
        let var = individual.keys().choose(rng).unwrap();
        individual.insert(
            var.clone(),
            SymbolicValue::ConstantInt(
                draw_bigint_with_probabilities(
                    &mutation_config.random_value_ranges,
                    &mutation_config.random_value_probs,
                    rng,
                )
                .unwrap(),
            ),
        );
    }
}
