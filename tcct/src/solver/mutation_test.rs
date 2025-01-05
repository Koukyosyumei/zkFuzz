use std::collections::HashSet;
use std::io;
use std::io::Write;

use colored::Colorize;
use log::info;
use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::{One, Signed, Zero};
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;
use rustc_hash::FxHashMap;
use std::str::FromStr;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{SymbolicName, SymbolicValue, SymbolicValueRef};

use crate::solver::eval::evaluate_trace_fitness;
use crate::solver::utils::{extract_variables, CounterExample, VerificationSetting};

pub fn mutation_test_search(
    sexe: &mut SymbolicExecutor,
    trace_constraints: &Vec<SymbolicValueRef>,
    side_constraints: &Vec<SymbolicValueRef>,
    setting: &VerificationSetting,
) -> Option<CounterExample> {
    // Parameters
    let program_population_size = 30;
    let input_population_size = 30;
    let max_generations = 300;
    let mutation_rate = 0.3;
    let crossover_rate = 0.5;
    let mut rng = rand::thread_rng();

    // Initial Population of Mutated Programs
    let mut assign_pos = Vec::new();
    for (i, sv) in trace_constraints.iter().enumerate() {
        match *sv.clone() {
            SymbolicValue::Assign(_, _, false) | SymbolicValue::AssignCall(_, _, true) => {
                assign_pos.push(i);
            }
            _ => {}
        }
    }

    // Initial Pupulation of Mutated Inputs
    let mut variables = extract_variables(trace_constraints);
    variables.append(&mut extract_variables(side_constraints));
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();
    let mut input_variables = Vec::new();
    for v in variables_set.iter() {
        if v.owner.len() == 1
            && sexe.symbolic_library.template_library[&sexe.symbolic_library.name2id[&setting.id]]
                .inputs
                .contains(&v.name)
        {
            input_variables.push(v.clone());
        }
    }

    info!(
        "{}: {}",
        "#Trace Constraints ".cyan(),
        trace_constraints.len()
    );
    info!(
        "{}: {}",
        "#Side Constraints  ".cyan(),
        side_constraints.len()
    );
    info!(
        "{}: {}",
        "#Input Variables   ".cyan(),
        input_variables.len()
    );
    info!("{}: {}", "#Mutation Candidate".cyan(), assign_pos.len());

    let mut trace_population =
        initialize_trace_mutation(&assign_pos, program_population_size, setting, &mut rng);
    let mut fitness_scores = vec![-setting.prime.clone(); input_population_size];

    for generation in 0..max_generations {
        // Generate input population for this generation
        let input_population = initialize_input_population(
            &input_variables,
            input_population_size,
            &setting,
            &mut rng,
        );

        // Evolve the trace population
        trace_population = if !trace_population.is_empty() {
            evolve_population(
                &trace_population,
                &fitness_scores,
                program_population_size,
                mutation_rate,
                crossover_rate,
                setting,
                &mut rng,
                |individual, setting, rng| trace_mutate(individual, setting, rng),
                |parent1, parent2, rng| trace_crossover(parent1, parent2, rng),
            )
        } else {
            vec![FxHashMap::default()]
        };

        let evaluations: Vec<_> = trace_population
            .iter()
            .map(|a| {
                evaluate_trace_fitness(
                    sexe,
                    &setting,
                    trace_constraints,
                    side_constraints,
                    a,
                    &input_population,
                )
            })
            .collect();
        let best_idx = evaluations
            .iter()
            .enumerate()
            .max_by_key(|&(_, value)| value.1.clone())
            .map(|(index, _)| index)
            .unwrap();
        fitness_scores = evaluations.iter().map(|v| v.1.clone()).collect();

        if evaluations[best_idx].1.is_zero() {
            print!(
                "\r\x1b[2KGeneration: {}/{} ({:.3})",
                generation, max_generations, 0
            );
            println!("\n └─ Solution found in generation {}", generation);
            return evaluations[best_idx].2.clone();
        }

        print!(
            "\r\x1b[2KGeneration: {}/{} ({:.3})",
            generation, max_generations, evaluations[best_idx].1
        );
        io::stdout().flush().unwrap();
    }

    println!(
        "\n └─ No solution found after {} generations",
        max_generations
    );
    None
}

fn draw_random_constant(setting: &VerificationSetting, rng: &mut ThreadRng) -> BigInt {
    if rng.gen::<bool>() {
        rng.gen_bigint_range(
            &(BigInt::from_str("10").unwrap() * -BigInt::one()),
            &(BigInt::from_str("10").unwrap()),
        )
    } else {
        rng.gen_bigint_range(
            &(setting.prime.clone() - BigInt::from_str("100").unwrap()),
            &(setting.prime),
        )
    }
}

fn initialize_input_population(
    variables: &[SymbolicName],
    size: usize,
    setting: &VerificationSetting,
    rng: &mut ThreadRng,
) -> Vec<FxHashMap<SymbolicName, BigInt>> {
    (0..size)
        .map(|_| {
            variables
                .iter()
                .map(|var| (var.clone(), draw_random_constant(setting, rng)))
                .collect()
        })
        .collect()
}

fn initialize_trace_mutation(
    pos: &[usize],
    size: usize,
    setting: &VerificationSetting,
    rng: &mut ThreadRng,
) -> Vec<FxHashMap<usize, SymbolicValue>> {
    (0..size)
        .map(|_| {
            pos.iter()
                .map(|p| {
                    (
                        p.clone(),
                        SymbolicValue::ConstantInt(draw_random_constant(setting, rng)),
                    )
                })
                .collect()
        })
        .collect()
}

fn evolve_population<T: Clone>(
    current_population: &[T],
    evaluations: &[BigInt],
    population_size: usize,
    mutation_rate: f64,
    crossover_rate: f64,
    setting: &VerificationSetting,
    rng: &mut ThreadRng,
    mutate_fn: impl Fn(&mut T, &VerificationSetting, &mut ThreadRng),
    crossover_fn: impl Fn(&T, &T, &mut ThreadRng) -> T,
) -> Vec<T> {
    (0..population_size)
        .map(|_| {
            let parent1 = selection(current_population, evaluations, rng);
            let parent2 = selection(current_population, evaluations, rng);
            let mut child = if rng.gen::<f64>() < crossover_rate {
                crossover_fn(&parent1, &parent2, rng)
            } else {
                parent1.clone()
            };
            if rng.gen::<f64>() < mutation_rate {
                mutate_fn(&mut child, setting, rng);
            }
            child
        })
        .collect()
}

fn selection<'a, T: Clone>(
    population: &'a [T],
    fitness_scores: &[BigInt],
    rng: &mut ThreadRng,
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

fn trace_crossover(
    parent1: &FxHashMap<usize, SymbolicValue>,
    parent2: &FxHashMap<usize, SymbolicValue>,
    rng: &mut ThreadRng,
) -> FxHashMap<usize, SymbolicValue> {
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

fn trace_mutate(
    individual: &mut FxHashMap<usize, SymbolicValue>,
    setting: &VerificationSetting,
    rng: &mut ThreadRng,
) {
    if !individual.is_empty() {
        let var = individual.keys().choose(rng).unwrap();
        /*
        if let SymbolicValue::ConstantInt(val) = &individual[var] {
            individual.insert(
                var.clone(),
                SymbolicValue::ConstantInt(
                    val + rng.gen_bigint_range(
                        &(BigInt::from_str("2").unwrap() * -BigInt::one()),
                        &(BigInt::from_str("2").unwrap()),
                    ),
                ),
            );
        }*/

        individual.insert(
            var.clone(),
            SymbolicValue::ConstantInt(draw_random_constant(setting, rng)),
        );
    }
}
