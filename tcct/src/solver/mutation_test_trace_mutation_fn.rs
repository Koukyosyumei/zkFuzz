use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::Rng;

use crate::executor::symbolic_value::SymbolicValue;
use crate::solver::mutation_config::MutationConfig;
use crate::solver::mutation_test::Gene;
use crate::solver::mutation_utils::draw_bigint_with_probabilities;
use crate::solver::utils::BaseVerificationConfig;

/// Mutates a trace by replacing a randomly selected position with a new random constant value.
///
/// This function introduces randomness into an individual's trace by selecting a position and replacing
/// its value with a newly generated constant, based on configurable probabilities and value ranges.
///
/// # Parameters
/// - `individual`: A mutable reference to a `Gene` (a mapping of position in a trace to symbolic values),
///   representing the mutation of the trace.
/// - `_base_config`: A reference to the `BaseVerificationConfig`, which is currently unused in this function
///   but might be relevant in future extensions.
/// - `mutation_config`: A reference to the `MutationConfig`, specifying the value ranges and probabilities
///   used for generating random constants.
/// - `rng`: A mutable reference to a random number generator (`StdRng`) for producing random values.
///
/// # Behavior
/// - If the `individual` is not empty:
///   - A position is selected randomly from its keys.
///   - The selected position's value is replaced with a new `SymbolicValue::ConstantInt`,
///     generated using `mutation_config.random_value_ranges` and `mutation_config.random_value_probs`.
/// - If the `individual` is empty, the function does nothing.
///
/// # Notes
/// - The generated constants are drawn according to the probabilities and ranges defined in the `MutationConfig`.
///
/// # Future Considerations
/// - `_base_config` could be leveraged to introduce additional constraints or behaviors during mutation.
pub fn mutate_trace_with_random_constant_replacement(
    pos: &[usize],
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
        if individual.len() < mutation_config.max_num_mutation_points && rng.gen::<bool>() {
            let var = pos.into_iter().choose(rng).unwrap();
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
        } else if individual.len() > 1 {
            let var = pos.into_iter().choose(rng).unwrap();
            individual.remove(&var);
        }
    }
}
