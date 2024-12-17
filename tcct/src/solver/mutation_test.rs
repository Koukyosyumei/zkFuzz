use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::rc::Rc;

use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::One;
use num_traits::Zero;
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;
use rustc_hash::FxHashMap;
use std::str::FromStr;

use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{
    register_array_elements, OwnerName, SymbolicName, SymbolicValue, SymbolicValueRef,
};

use crate::solver::utils::{
    count_satisfied_constraints, emulate_symbolic_values, evaluate_constraints, extract_variables,
    is_vulnerable, verify_assignment, CounterExample, UnderConstrainedType, VerificationResult,
    VerificationSetting,
};

pub fn mutation_test_search(
    sexe: &mut SymbolicExecutor,
    trace_constraints: &Vec<SymbolicValueRef>,
    side_constraints: &Vec<SymbolicValueRef>,
    setting: &VerificationSetting,
) -> Option<CounterExample> {
    // Parameters
    let program_population_size = 100;
    let input_population_size = 30;
    let max_generations = 100;
    let mutation_rate = 0.3;
    let crossover_rate = 0.5;
    let mut rng = rand::thread_rng();

    // Initial Population of Mutated Programs
    let mut assign_pos = Vec::new();
    for (i, sv) in trace_constraints.iter().enumerate() {
        match *sv.clone() {
            SymbolicValue::Assign(_, _) => {
                assign_pos.push(i);
            }
            _ => {}
        }
    }

    // Initial Pupulation of Mutated Inputs
    let mut variables = extract_variables(trace_constraints);
    variables.append(&mut extract_variables(side_constraints));
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();
    //variables = variables_set.into_iter().collect();
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

    if assign_pos.is_empty() {
        return None;
    }

    let mut trace_population =
        initialize_trace_mutation(&assign_pos, program_population_size, setting, &mut rng);

    for generation in 0..max_generations {
        let input_population = initialize_input_population(
            &input_variables,
            input_population_size,
            &setting,
            &mut rng,
        );

        let mut new_trace_population = vec![FxHashMap::default()];
        for _ in 0..program_population_size {
            let parent1 = trace_selection(&trace_population, &mut rng);
            let parent2 = trace_selection(&trace_population, &mut rng);

            let mut child = if rng.gen::<f64>() < crossover_rate {
                trace_crossover(parent1, parent2, &mut rng)
            } else {
                parent1.clone()
            };

            if rng.gen::<f64>() < mutation_rate {
                trace_mutate(&mut child, setting, &mut rng);
            }

            new_trace_population.push(child);
        }
        trace_population = new_trace_population;

        let best_mutated_trace = trace_population
            .iter()
            .max_by(|a, b| {
                let fitness_a = trace_fitness(
                    sexe,
                    &setting,
                    trace_constraints,
                    side_constraints,
                    a,
                    &input_population,
                );
                let fitness_b = trace_fitness(
                    sexe,
                    &setting,
                    trace_constraints,
                    side_constraints,
                    b,
                    &input_population,
                );
                fitness_a
                    .1
                    .partial_cmp(&fitness_b.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        let best_score = trace_fitness(
            sexe,
            &setting,
            trace_constraints,
            side_constraints,
            best_mutated_trace,
            &input_population,
        );

        if best_score.1 >= 1.0 {
            let mut mutated_trace_constraints = trace_constraints.clone();
            for (k, v) in best_mutated_trace {
                if let SymbolicValue::Assign(lv, rv) =
                    mutated_trace_constraints[*k].as_ref().clone()
                {
                    mutated_trace_constraints[*k] =
                        Rc::new(SymbolicValue::Assign(lv.clone(), Rc::new(v.clone())));
                } else {
                    panic!("We can only mutate SymbolicValue::Assign");
                }
            }

            let mut assignment = input_population[best_score.0].clone();
            if emulate_symbolic_values(
                &setting.prime,
                &mutated_trace_constraints,
                &mut assignment,
                &mut sexe.symbolic_library,
            ) {
                let flag = verify_assignment(
                    sexe,
                    trace_constraints,
                    side_constraints,
                    &assignment,
                    setting,
                );
                if is_vulnerable(&flag) {
                    print!(
                        "\rGeneration: {}/{} ({:.3})",
                        generation, max_generations, best_score.1
                    );
                    println!("\n └─ Solution found in generation {}", generation);
                    return Some(CounterExample {
                        flag: flag,
                        assignment: assignment.clone(),
                    });
                }
            } else {
                if evaluate_constraints(
                    &setting.prime,
                    side_constraints,
                    &assignment,
                    &mut sexe.symbolic_library,
                ) {
                    return Some(CounterExample {
                        flag: VerificationResult::UnderConstrained(
                            UnderConstrainedType::Deterministic,
                        ),
                        assignment: assignment.clone(),
                    });
                }
            }
        }

        if generation % 10 == 0 {
            print!(
                "\rGeneration: {}/{} ({:.3})",
                generation, max_generations, best_score.1
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

fn draw_random_constant(setting: &VerificationSetting, rng: &mut ThreadRng) -> BigInt {
    if rng.gen::<bool>() {
        rng.gen_bigint_range(
            &(BigInt::from_str("2").unwrap() * -BigInt::one()),
            &(BigInt::from_str("2").unwrap()),
        )
    } else {
        rng.gen_bigint_range(
            &(setting.prime.clone() - BigInt::from_str("2").unwrap()),
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

fn trace_selection<'a>(
    population: &'a [FxHashMap<usize, SymbolicValue>],
    rng: &mut ThreadRng,
) -> &'a FxHashMap<usize, SymbolicValue> {
    population.choose(rng).unwrap()
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
        individual.insert(
            var.clone(),
            SymbolicValue::ConstantInt(draw_random_constant(setting, rng)),
        );
    }
}

fn trace_fitness(
    sexe: &mut SymbolicExecutor,
    setting: &VerificationSetting,
    trace_constraints: &Vec<SymbolicValueRef>,
    side_constraints: &Vec<SymbolicValueRef>,
    trace_mutation: &FxHashMap<usize, SymbolicValue>,
    inputs: &Vec<FxHashMap<SymbolicName, BigInt>>,
) -> (usize, f64) {
    let mut mutated_trace_constraints = trace_constraints.clone();
    for (k, v) in trace_mutation {
        if let SymbolicValue::Assign(lv, rv) = mutated_trace_constraints[*k].as_ref().clone() {
            mutated_trace_constraints[*k] =
                Rc::new(SymbolicValue::Assign(lv.clone(), Rc::new(v.clone())));
        } else {
            panic!("We can only mutate SymbolicValue::Assign");
        }
    }

    let mut max_idx = 0_usize;
    let mut max_score = 0 as f64;
    for (i, inp) in inputs.iter().enumerate() {
        let mut assignment = inp.clone();
        let is_success = emulate_symbolic_values(
            &setting.prime,
            &mutated_trace_constraints,
            &mut assignment,
            &mut sexe.symbolic_library,
        );
        {
            let satisfied_side = count_satisfied_constraints(
                &setting.prime,
                side_constraints,
                &assignment,
                &mut sexe.symbolic_library,
            );
            let mut side_ratio = if side_constraints.is_empty() {
                1.0 as f64
            } else {
                satisfied_side as f64 / side_constraints.len() as f64
            };

            if side_ratio == 1.0 as f64 {
                if is_success {
                    let flag = verify_assignment(
                        sexe,
                        trace_constraints,
                        side_constraints,
                        &assignment,
                        setting,
                    );
                    if !is_vulnerable(&flag) {
                        side_ratio = 0.9;
                    } else {
                        side_ratio = 1.1;
                    }
                }
            }

            if side_ratio > max_score {
                max_idx = i;
                max_score = side_ratio;
            }
        }
    }

    (max_idx, max_score)
}
