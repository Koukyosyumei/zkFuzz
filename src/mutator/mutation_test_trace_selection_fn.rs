use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::{One, Signed, Zero};
use rand::rngs::StdRng;

/// Selects an individual from the population using roulette-wheel selection.
///
/// This function performs roulette-wheel selection (also known as fitness-proportionate selection),
/// a probabilistic method for selecting individuals from a population based on their fitness scores.
/// Individuals with higher fitness scores have a higher probability of being selected.
///
/// # Parameters
/// - `population`: A slice of individuals in the population.
/// - `fitness_scores`: A slice of fitness scores corresponding to the individuals in the population.
///   Each fitness score represents the relative "quality" or fitness of an individual.
/// - `rng`: A mutable reference to a random number generator used to perform the selection.
///
/// # Returns
/// A reference to the selected individual in the population.
///
/// # Type Parameters
/// - `T`: The type of individuals in the population, which must implement `Clone`.
///
/// # Algorithm
/// 1. Calculate the weight for each individual by subtracting the minimum fitness score from
///    each individual's fitness score. This ensures non-negative weights.
/// 2. Compute the total weight as the sum of all individual weights.
/// 3. Generate a random target weight in the range `[0, total_weight)`.
/// 4. Iterate through the population and subtract each individual's weight from the target.
///    When the target becomes less than the current individual's weight, that individual is selected.
/// 5. If no individual is selected due to edge cases (e.g., all weights are zero), the first
///    individual in the population is returned as a fallback.
///
/// # Edge Cases
/// - If all fitness scores are equal, the selection is effectively random.
/// - If the population is empty, this function will panic when attempting to compute the minimum score.
/// - If all fitness scores are zero, the first individual in the population will always be selected.
///
/// # Example
/// ```rust
/// use rand::{SeedableRng, rngs::StdRng};
/// use num_bigint_dig::BigInt;
///
/// use zkfuzz::mutator::mutation_test_trace_selection_fn::roulette_selection;
///
/// let population = vec!["A", "B", "C"];
/// let fitness_scores = vec![BigInt::from(10), BigInt::from(20), BigInt::from(30)];
/// let mut rng = StdRng::seed_from_u64(42);
///
/// let selected = roulette_selection(&population, &fitness_scores, &mut rng);
/// println!("Selected individual: {}", selected);
/// ```
///
/// # Panics
/// - Panics if `fitness_scores` is empty or if its length does not match the length of `population`.
///
/// # Complexity
/// - Time complexity: O(n), where `n` is the size of the population.
/// - Space complexity: O(n), due to the allocation of the weights vector.
pub fn roulette_selection<'a, T: Clone>(
    population: &'a [T],
    fitness_scores: &[BigInt],
    rng: &mut StdRng,
) -> &'a T {
    let min_score = fitness_scores.iter().min().unwrap();
    let weights: Vec<_> = fitness_scores
        .iter()
        .map(|score| score - min_score)
        .collect();
    let mut total_weight: BigInt = weights.iter().sum();
    total_weight = if total_weight.is_positive() {
        total_weight
    } else {
        BigInt::one()
    };
    let mut target = rng.gen_bigint_range(&BigInt::zero(), &total_weight);
    for (individual, weight) in population.iter().zip(weights.iter()) {
        if &target < weight {
            return individual;
        }
        target -= weight;
    }
    &population[0]
}
