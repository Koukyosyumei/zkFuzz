use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::Signed;
use num_traits::One;
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;
use rustc_hash::FxHashMap;
use std::str::FromStr;


use crate::symbolic_execution::SymbolicExecutor;
use crate::symbolic_value::SymbolicName;
use crate::symbolic_value::SymbolicValue;

use crate::solver::utils::{
    count_satisfied_constraints, extract_variables, is_vulnerable,
    verify_assignment, CounterExample, VerificationSetting,
};

pub fn genetic_algorithm_search(
    sexe: &mut SymbolicExecutor,
    trace_constraints: &Vec<Rc<SymbolicValue>>,
    side_constraints: &Vec<Rc<SymbolicValue>>,
    setting: &VerificationSetting,
) -> Option<CounterExample> {
    let mut variables = extract_variables(trace_constraints);
    variables.append(&mut extract_variables(side_constraints));
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();
    variables = variables_set.into_iter().collect();

    let population_size = 100;
    let max_generations = 1000;
    let mutation_rate = 0.3;
    let crossover_rate = 0.5;

    let mut rng = rand::thread_rng();
    let mut population = initialize_population(&variables, &setting.prime, population_size);
    let current_iteration = Arc::new(AtomicUsize::new(0));

    for generation in 0..max_generations {
        let mut new_population = Vec::new();

        for _ in 0..population_size {
            let parent1 = selection(&population, &mut rng);
            let parent2 = selection(&population, &mut rng);

            let mut child = if rng.gen::<f64>() < crossover_rate {
                crossover(parent1, parent2, &mut rng)
            } else {
                parent1.clone()
            };

            if rng.gen::<f64>() < mutation_rate {
                mutate(&mut child, &setting.prime, &mut rng);
            }

            new_population.push(child);
        }

        population = new_population;

        // In your genetic algorithm function
        let best_individual = population
            .iter()
            .max_by(|a, b| {
                let fitness_a = fitness(&setting.prime, trace_constraints, side_constraints, a);
                let fitness_b = fitness(&setting.prime, trace_constraints, side_constraints, b);
                fitness_a
                    .partial_cmp(&fitness_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        let best_score = fitness(
            &setting.prime,
            trace_constraints,
            side_constraints,
            best_individual,
        );

        if best_score == 1.0 {
            let flag = verify_assignment(
                sexe,
                trace_constraints,
                side_constraints,
                best_individual,
                setting,
            );
            if is_vulnerable(&flag) {
                print!(
                    "\rGeneration: {}/{} ({:.3})",
                    generation, max_generations, best_score
                );
                println!("\n └─ Solution found in generation {}", generation);
                return Some(CounterExample {
                    flag: flag,
                    assignment: best_individual.clone(),
                });
            }
        }

        if generation % 10 == 0 {
            print!(
                "\rGeneration: {}/{} ({:.3})",
                generation, max_generations, best_score
            );
            io::stdout().flush().unwrap();
        }
    }

    println!(
        "\n └─ No solution found after {} generations",
        max_generations
    );
    None
}

fn initialize_population(
    variables: &[SymbolicName],
    prime: &BigInt,
    size: usize,
) -> Vec<FxHashMap<SymbolicName, BigInt>> {
    let mut rng = rand::thread_rng();
    (0..size)
        .map(|_| {
            variables
                .iter()
                .map(|var| {
                    (
                        var.clone(),
                        rng.gen_bigint_range(
                            &(BigInt::from_str("2").unwrap() * -BigInt::one()),
                            &(BigInt::from_str("2").unwrap() * BigInt::one()),
                        ),
                    )
                })
                .collect()
        })
        .collect()
}

fn selection<'a>(
    population: &'a [FxHashMap<SymbolicName, BigInt>],
    rng: &mut ThreadRng,
) -> &'a FxHashMap<SymbolicName, BigInt> {
    population.choose(rng).unwrap()
}

fn crossover(
    parent1: &FxHashMap<SymbolicName, BigInt>,
    parent2: &FxHashMap<SymbolicName, BigInt>,
    rng: &mut ThreadRng,
) -> FxHashMap<SymbolicName, BigInt> {
    parent1
        .iter()
        .map(|(var, val)| {
            if rng.gen::<bool>() {
                (var.clone(), val.clone())
            } else {
                (var.clone(), parent2[var].clone())
            }
        })
        .collect()
}

fn mutate(individual: &mut FxHashMap<SymbolicName, BigInt>, prime: &BigInt, rng: &mut ThreadRng) {
    let var = individual.keys().choose(rng).unwrap();
    individual.insert(
        var.clone(),
        rng.gen_bigint_range(
            &(BigInt::from_str("2").unwrap() * -BigInt::one()),
            &(BigInt::from_str("2").unwrap() * BigInt::one()),
        ),
    );
}

fn fitness(
    prime: &BigInt,
    trace_constraints: &[Rc<SymbolicValue>],
    side_constraints: &[Rc<SymbolicValue>],
    assignment: &FxHashMap<SymbolicName, BigInt>,
) -> f64 {
    let total_constraints = trace_constraints.len() + side_constraints.len();
    let satisfied_trace = count_satisfied_constraints(prime, trace_constraints, assignment);
    let satisfied_side = count_satisfied_constraints(prime, side_constraints, assignment);

    let trace_ratio = satisfied_trace as f64 / trace_constraints.len() as f64;
    let side_ratio = satisfied_side as f64 / side_constraints.len() as f64;

    if (trace_ratio == 1.0 && side_ratio < 1.0) || (trace_ratio < 1.0 && side_ratio == 1.0) {
        1.0
    } else if trace_ratio == 1.0 && side_ratio == 1.0 {
        0.5
    } else {
        let distance_to_desired = (1.0 - trace_ratio).abs() + (1.0 - side_ratio).abs();
        1.0 / (1.0 + distance_to_desired)
    }
}
