use std::rc::Rc;

use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::{One, Signed, Zero};
use rand::rngs::StdRng;
use rand::Rng;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{SymbolicName, SymbolicValue, SymbolicValueRef};

use crate::solver::utils::{
    accumulate_error_of_constraints, emulate_symbolic_values, is_vulnerable, verify_assignment,
    CounterExample, UnderConstrainedType, VerificationResult, VerificationSetting,
};

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

pub fn apply_trace_mutation(
    trace_constraints: &Vec<SymbolicValueRef>,
    trace_mutation: &FxHashMap<usize, SymbolicValue>,
) -> Vec<SymbolicValueRef> {
    let mut mutated_constraints = trace_constraints.clone();
    for (index, value) in trace_mutation {
        if let SymbolicValue::Assign(lv, _, is_safe) = mutated_constraints[*index].as_ref().clone()
        {
            mutated_constraints[*index] = Rc::new(SymbolicValue::Assign(
                lv.clone(),
                Rc::new(value.clone()),
                is_safe,
            ));
        } else if let SymbolicValue::AssignCall(lv, _, is_mutable) =
            mutated_constraints[*index].as_ref().clone()
        {
            mutated_constraints[*index] = Rc::new(SymbolicValue::Assign(
                lv.clone(),
                Rc::new(value.clone()),
                !is_mutable,
            ));
        } else {
            panic!("We can only mutate SymbolicValue::Assign");
        }
    }
    mutated_constraints
}

pub fn evaluate_trace_fitness_by_error(
    sexe: &mut SymbolicExecutor,
    setting: &VerificationSetting,
    trace_constraints: &Vec<SymbolicValueRef>,
    side_constraints: &Vec<SymbolicValueRef>,
    trace_mutation: &FxHashMap<usize, SymbolicValue>,
    inputs_assignment: &Vec<FxHashMap<SymbolicName, BigInt>>,
) -> (usize, BigInt, Option<CounterExample>) {
    let mutated_trace_constraints = apply_trace_mutation(trace_constraints, trace_mutation);

    let mut max_idx = 0_usize;
    let mut max_score = -setting.prime.clone();
    let mut counter_example = None;

    for (i, inp) in inputs_assignment.iter().enumerate() {
        let mut assignment = inp.clone();

        let (is_success, failure_pos) = emulate_symbolic_values(
            &setting.prime,
            &mutated_trace_constraints,
            &mut assignment,
            &mut sexe.symbolic_library,
        );
        let error_of_side_constraints = accumulate_error_of_constraints(
            &setting.prime,
            side_constraints,
            &assignment,
            &mut sexe.symbolic_library,
        );
        let mut score = -error_of_side_constraints.clone();

        if error_of_side_constraints.is_zero() {
            if is_success {
                let flag = verify_assignment(
                    sexe,
                    trace_constraints,
                    side_constraints,
                    &assignment,
                    setting,
                );
                if is_vulnerable(&flag) {
                    max_idx = i;
                    max_score = BigInt::zero();
                    counter_example = Some(CounterExample {
                        flag: flag,
                        assignment: assignment.clone(),
                    });
                    break;
                } else {
                    score = -setting.prime.clone();
                }
            } else {
                if trace_mutation.is_empty() {
                    max_idx = i;
                    max_score = BigInt::zero();
                    counter_example = Some(CounterExample {
                        flag: VerificationResult::UnderConstrained(
                            UnderConstrainedType::UnexpectedTrace(
                                mutated_trace_constraints[failure_pos]
                                    .lookup_fmt(&sexe.symbolic_library.id2name),
                            ),
                        ),
                        assignment: assignment.clone(),
                    });
                    break;
                }
            }
        }

        if score > max_score {
            max_idx = i;
            max_score = score;
        }
    }

    (max_idx, max_score, counter_example)
}
