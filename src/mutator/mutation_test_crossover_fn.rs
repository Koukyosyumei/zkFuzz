use rand::rngs::StdRng;
use rand::Rng;
use rustc_hash::FxHashMap;

/// Generates a new `FxHashMap` by performing a random crossover between two parent maps.
///
/// # Parameters
/// - `parent1`: A reference to the first parent map containing keys and values.
/// - `parent2`: A reference to the second parent map containing keys and values.
/// - `rng`: A mutable reference to a random number generator implementing `StdRng`.
///
/// # Returns
/// A new `FxHashMap` where each key-value pair is selected either from `parent1` or `parent2`:
/// - For each key in `parent1`, the corresponding value is randomly chosen from `parent1` or `parent2` using the provided RNG.
/// - If a key exists in `parent1` but not in `parent2`, the value is taken from `parent1`.
///
/// # Type Parameters
/// - `K`: The key type, which must implement `Clone`, `Hash`, and `Eq`.
/// - `V`: The value type, which must implement `Clone`.
///
/// # Examples
/// ```
/// use rustc_hash::FxHashMap;
/// use rand::{SeedableRng, rngs::StdRng};
/// use zkfuzz::mutator::mutation_test_crossover_fn::random_crossover;
///
/// let mut rng = StdRng::seed_from_u64(42);
///
/// let mut parent1: FxHashMap<String, i32> = FxHashMap::default();
/// parent1.insert("a".to_string(), 1);
/// parent1.insert("b".to_string(), 2);
///
/// let mut parent2: FxHashMap<String, i32> = FxHashMap::default();
/// parent2.insert("b".to_string(), 3);
/// parent2.insert("c".to_string(), 4);
///
/// let child = random_crossover(&parent1, &parent2, &mut rng);
///
/// // `child` will contain a random combination of values from `parent1` and `parent2`.
/// ```
pub fn random_crossover<K, V>(
    parent1: &FxHashMap<K, V>,
    parent2: &FxHashMap<K, V>,
    rng: &mut StdRng,
) -> FxHashMap<K, V>
where
    K: Clone + std::hash::Hash + std::cmp::Eq + std::cmp::Ord,
    V: Clone,
{
    let mut keys: Vec<&K> = parent1.keys().collect();
    keys.sort();

    keys.into_iter()
        .map(|var| {
            let val = parent1.get(var).unwrap();
            if rng.gen::<bool>() {
                (var.clone(), val.clone())
            } else {
                if parent2.contains_key(var) {
                    (var.clone(), parent2[var].clone())
                } else {
                    (var.clone(), val.clone())
                }
            }
        })
        .collect()
}
