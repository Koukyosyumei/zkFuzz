
use rand::rngs::StdRng;
use rand::Rng;
use rustc_hash::FxHashMap;




pub fn random_crossover<K, V>(
    parent1: &FxHashMap<K, V>,
    parent2: &FxHashMap<K, V>,
    rng: &mut StdRng,
) -> FxHashMap<K, V>
where
    K: Clone + std::hash::Hash + std::cmp::Eq,
    V: Clone,
{
    parent1
        .iter()
        .map(|(var, val)| {
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
