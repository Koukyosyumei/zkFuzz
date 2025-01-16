use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Write;
use std::str::FromStr;

use colored::Colorize;
use log::info;
use num_bigint_dig::BigInt;
use num_bigint_dig::RandBigInt;
use num_traits::{One, Zero};
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use program_structure::ast::ExpressionInfixOpcode;

use crate::executor::debug_ast::DebuggableExpressionInfixOpcode;
use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_value::{OwnerName, SymbolicName, SymbolicValue, SymbolicValueRef};

use crate::solver::mutation_utils::{
    evaluate_trace_fitness_by_error, random_crossover, roulette_selection,
};
use crate::solver::utils::{extract_variables, CounterExample, VerificationSetting};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MutationSettings {
    seed: u64,
    program_population_size: usize,
    input_population_size: usize,
    max_generations: usize,
    input_initialization_method: String,
    fitness_function: String,
    mutation_rate: f64,
    crossover_rate: f64,
    coverage_based_input_generation_max_iteration: usize,
    coverage_based_input_generation_crossover_rate: f64,
    coverage_based_input_generation_mutation_rate: f64,
    coverage_based_input_generation_singlepoint_mutation_rate: f64,
    save_fitness_scores: bool,
}

impl Default for MutationSettings {
    fn default() -> Self {
        MutationSettings {
            seed: 0,
            program_population_size: 30,
            input_population_size: 30,
            max_generations: 300,
            input_initialization_method: "random".to_string(),
            fitness_function: "error".to_string(),
            mutation_rate: 0.3,
            crossover_rate: 0.5,
            coverage_based_input_generation_max_iteration: 30,
            coverage_based_input_generation_crossover_rate: 0.66,
            coverage_based_input_generation_mutation_rate: 0.5,
            coverage_based_input_generation_singlepoint_mutation_rate: 0.5,
            save_fitness_scores: false,
        }
    }
}

impl fmt::Display for MutationSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "🧬 Mutation Settings:
    ├─ Program Population Size    : {}
    ├─ Input Population Size      : {}
    ├─ Max Generations            : {}
    ├─ Input Initialization Method: {} 
    ├─ Fitness Function           : {} 
    ├─ Trace Mutation Rate        : {}
    └─ Trace Crossover Rate       : {}",
            self.program_population_size.to_string().bright_yellow(),
            self.input_population_size.to_string().bright_yellow(),
            self.max_generations.to_string().bright_yellow(),
            self.input_initialization_method.bright_yellow(),
            self.fitness_function.bright_yellow(),
            self.mutation_rate.to_string().bright_yellow(),
            self.crossover_rate.to_string().bright_yellow()
        )
    }
}

pub struct MutationTestResult {
    pub random_seed: u64,
    pub mutation_setting: MutationSettings,
    pub counter_example: Option<CounterExample>,
    pub generation: usize,
    pub fitness_score_log: Vec<BigInt>,
}

fn load_settings_from_json(file_path: &str) -> Result<MutationSettings, serde_json::Error> {
    let file = File::open(file_path);
    if file.is_ok() {
        let settings: MutationSettings = serde_json::from_reader(file.unwrap())?;
        Ok(settings)
    } else {
        info!("Use the default setting for mutation testing");
        Ok(MutationSettings::default())
    }
}

pub fn mutation_test_search(
    sexe: &mut SymbolicExecutor,
    symbolic_trace: &Vec<SymbolicValueRef>,
    side_constraints: &Vec<SymbolicValueRef>,
    setting: &VerificationSetting,
    path_to_mutation_setting: &String,
) -> MutationTestResult {
    let mutation_setting = load_settings_from_json(path_to_mutation_setting).unwrap();
    info!("\n{}", mutation_setting);

    let seed = if mutation_setting.seed.is_zero() {
        let mut seed_rng = rand::thread_rng();
        seed_rng.gen()
    } else {
        mutation_setting.seed
    };
    let mut rng = StdRng::seed_from_u64(seed);

    // Initial Population of Mutated Programs
    let mut assign_pos = Vec::new();
    for (i, sv) in symbolic_trace.iter().enumerate() {
        match *sv.as_ref() {
            SymbolicValue::Assign(_, _, false) | SymbolicValue::AssignCall(_, _, true) => {
                assign_pos.push(i);
            }
            _ => {}
        }
    }

    // Initial Pupulation of Mutated Inputs
    let mut variables = extract_variables(symbolic_trace);
    variables.append(&mut extract_variables(side_constraints));
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();
    let mut input_variables = Vec::new();
    for v in variables_set.iter() {
        if v.owner.len() == 1
            && sexe.symbolic_library.template_library
                [&sexe.symbolic_library.name2id[&setting.target_template_name]]
                .input_ids
                .contains(&v.id)
        {
            input_variables.push(v.clone());
        }
    }

    info!(
        "\n⚖️ Constraints Summary:
    ├─ #Trace Constraints : {}
    ├─ #Side Constraints  : {}
    ├─ #Input Variables   : {}
    └─ #Mutation Candidate: {}",
        symbolic_trace.len().to_string().bright_yellow(),
        side_constraints.len().to_string().bright_yellow(),
        input_variables.len().to_string().bright_yellow(),
        assign_pos.len().to_string().bright_yellow()
    );

    let mut trace_population = initialize_trace_mutation_only_constant(
        &assign_pos,
        mutation_setting.program_population_size,
        setting,
        &mut rng,
    );
    let mut fitness_scores = vec![-setting.prime.clone(); mutation_setting.input_population_size];
    let mut input_population = Vec::new();
    let mut fitness_score_log = if mutation_setting.save_fitness_scores {
        Vec::with_capacity(mutation_setting.max_generations)
    } else {
        Vec::new()
    };

    println!(
        "{} {}",
        "🎲 Random Seed:",
        seed.to_string().bold().bright_yellow(),
    );

    for generation in 0..mutation_setting.max_generations {
        // Generate input population for this generation
        if mutation_setting.input_initialization_method == "coverage" {
            if generation % 4 == 3 {
                sexe.clear_coverage_tracker();
                mutate_input_population_with_coverage_maximization(
                    sexe,
                    &input_variables,
                    &mut input_population,
                    mutation_setting.input_population_size / 2 as usize,
                    mutation_setting.input_population_size,
                    mutation_setting.max_generations,
                    mutation_setting.coverage_based_input_generation_crossover_rate,
                    mutation_setting.coverage_based_input_generation_mutation_rate,
                    mutation_setting.coverage_based_input_generation_singlepoint_mutation_rate,
                    &setting,
                    &mut rng,
                );
            }
        } else if mutation_setting.input_initialization_method == "random" {
            input_population = initialize_input_population(
                &input_variables,
                mutation_setting.input_population_size,
                &setting,
                &mut rng,
            );
        } else {
            panic!("mutation_setting.input_initialization_method should be one of [`coverage`, `random`]");
        }

        // Evolve the trace population
        if !trace_population.is_empty() {
            trace_population = evolve_population(
                &trace_population,
                &fitness_scores,
                mutation_setting.program_population_size,
                mutation_setting.mutation_rate,
                mutation_setting.crossover_rate,
                setting,
                &mut rng,
                |individual, setting, rng| trace_mutate(individual, setting, rng),
                |parent1, parent2, rng| random_crossover(parent1, parent2, rng),
            );
        }
        trace_population.push(FxHashMap::default());

        let evaluations: Vec<_> = trace_population
            .iter()
            .map(|a| {
                evaluate_trace_fitness_by_error(
                    sexe,
                    &setting,
                    symbolic_trace,
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
        if mutation_setting.fitness_function == "error" {
            fitness_scores = evaluations.iter().map(|v| v.1.clone()).collect();
        } else if mutation_setting.fitness_function == "constant" {
        } else {
            panic!("mutation_setting.fitness_function should be one of [`error`, `constant`]");
        }

        if evaluations[best_idx].1.is_zero() {
            print!(
                "\r\x1b[2K🧬 Generation: {}/{} ({:.3})",
                generation, mutation_setting.max_generations, 0
            );
            println!("\n    └─ Solution found in generation {}", generation);

            return MutationTestResult {
                random_seed: seed,
                mutation_setting: mutation_setting,
                counter_example: evaluations[best_idx].2.clone(),
                generation: generation,
                fitness_score_log: fitness_score_log,
            };
        }

        print!(
            "\r\x1b[2K🧬 Generation: {}/{} ({:.3})",
            generation, mutation_setting.max_generations, fitness_scores[best_idx]
        );
        io::stdout().flush().unwrap();

        if mutation_setting.save_fitness_scores {
            fitness_score_log.push(fitness_scores[best_idx].clone());
        }
    }

    println!(
        "\n └─ No solution found after {} generations",
        mutation_setting.max_generations
    );

    MutationTestResult {
        random_seed: seed,
        mutation_setting: mutation_setting.clone(),
        counter_example: None,
        generation: mutation_setting.max_generations,
        fitness_score_log: fitness_score_log,
    }
}

fn draw_random_constant(setting: &VerificationSetting, rng: &mut StdRng) -> BigInt {
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
    rng: &mut StdRng,
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

fn evaluate_coverage(
    sexe: &mut SymbolicExecutor,
    inputs: &FxHashMap<SymbolicName, BigInt>,
    setting: &VerificationSetting,
) -> usize {
    sexe.clear();
    sexe.turn_on_coverage_tracking();
    sexe.cur_state.add_owner(&OwnerName {
        id: sexe.symbolic_library.name2id["main"],
        counter: 0,
        access: None,
    });
    sexe.feed_arguments(
        &setting.template_param_names,
        &setting.template_param_values,
    );
    sexe.concrete_execute(&setting.target_template_name, inputs);
    sexe.record_path();
    sexe.turn_off_coverage_tracking();
    sexe.coverage_count()
}

fn mutate_input_population_with_coverage_maximization(
    sexe: &mut SymbolicExecutor,
    input_variables: &[SymbolicName],
    inputs_population: &mut Vec<FxHashMap<SymbolicName, BigInt>>,
    input_population_size: usize,
    maximum_size: usize,
    max_iteration: usize,
    cross_over_rate: f64,
    mutation_rate: f64,
    singlepoint_mutation_rate: f64,
    setting: &VerificationSetting,
    rng: &mut StdRng,
) {
    let mut total_coverage = 0_usize;
    inputs_population.clear();

    let initial_input_population =
        initialize_input_population(input_variables, input_population_size, &setting, rng);

    for input in &initial_input_population {
        let new_coverage = evaluate_coverage(sexe, &input, setting);
        if new_coverage > total_coverage {
            inputs_population.push(input.clone());
            total_coverage = new_coverage;
        }
    }

    for _ in 0..max_iteration {
        let mut new_inputs_population = Vec::new();

        // Iterate through the population and attempt mutations
        for input in inputs_population.iter() {
            let mut new_input = input.clone();

            if rng.gen::<f64>() < cross_over_rate {
                // Crossover
                let other = inputs_population[rng.gen_range(0, inputs_population.len())].clone();
                new_input = random_crossover(input, &other, rng);
            }
            if rng.gen::<f64>() < mutation_rate {
                if rng.gen::<f64>() < singlepoint_mutation_rate {
                    // Mutate only one input variable
                    let var = &input_variables[rng.gen_range(0, input_variables.len())];
                    let mutation = draw_random_constant(setting, rng);
                    new_input.insert(var.clone(), mutation);
                } else {
                    // Mutate each input variable with a small probability
                    for var in input_variables {
                        // rng.gen_bool(0.2)
                        if rng.gen::<bool>() {
                            let mutation = draw_random_constant(setting, rng);
                            new_input.insert(var.clone(), mutation);
                        }
                    }
                }
            }

            // Evaluate the new input
            let new_coverage = evaluate_coverage(sexe, &new_input, setting);
            if new_coverage > total_coverage {
                new_inputs_population.push(new_input);
                total_coverage = new_coverage;
            }
        }
        inputs_population.append(&mut new_inputs_population);

        if inputs_population.len() > maximum_size {
            break;
        }
    }
}

fn initialize_trace_mutation_only_constant(
    pos: &[usize],
    size: usize,
    setting: &VerificationSetting,
    rng: &mut StdRng,
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

lazy_static::lazy_static! {
    static ref OPERATOR_MUTATION_CANDIDATES: Vec<(ExpressionInfixOpcode,Vec<ExpressionInfixOpcode>)> = {
        vec![
            (ExpressionInfixOpcode::Add, vec![ExpressionInfixOpcode::Sub, ExpressionInfixOpcode::Mul]),
            (ExpressionInfixOpcode::Sub, vec![ExpressionInfixOpcode::Add, ExpressionInfixOpcode::Mul]),
            (ExpressionInfixOpcode::Mul, vec![ExpressionInfixOpcode::Add, ExpressionInfixOpcode::Sub, ExpressionInfixOpcode::Pow]),
            (ExpressionInfixOpcode::Pow, vec![ExpressionInfixOpcode::Mul]),
            (ExpressionInfixOpcode::Div, vec![ExpressionInfixOpcode::IntDiv, ExpressionInfixOpcode::Mul]),
            (ExpressionInfixOpcode::IntDiv, vec![ExpressionInfixOpcode::Div, ExpressionInfixOpcode::Mul]),
            (ExpressionInfixOpcode::Mod, vec![ExpressionInfixOpcode::Div, ExpressionInfixOpcode::IntDiv]),
            (ExpressionInfixOpcode::BitOr, vec![ExpressionInfixOpcode::BitAnd, ExpressionInfixOpcode::BitXor]),
            (ExpressionInfixOpcode::BitAnd, vec![ExpressionInfixOpcode::BitOr, ExpressionInfixOpcode::BitXor]),
            (ExpressionInfixOpcode::BitXor, vec![ExpressionInfixOpcode::BitOr, ExpressionInfixOpcode::BitAnd]),
            (ExpressionInfixOpcode::ShiftL, vec![ExpressionInfixOpcode::ShiftR]),
            (ExpressionInfixOpcode::ShiftR, vec![ExpressionInfixOpcode::ShiftL]),
            (ExpressionInfixOpcode::Lesser, vec![ExpressionInfixOpcode::Greater, ExpressionInfixOpcode::LesserEq]),
            (ExpressionInfixOpcode::Greater, vec![ExpressionInfixOpcode::Lesser, ExpressionInfixOpcode::GreaterEq]),
            (ExpressionInfixOpcode::LesserEq, vec![ExpressionInfixOpcode::GreaterEq, ExpressionInfixOpcode::Lesser]),
            (ExpressionInfixOpcode::GreaterEq, vec![ExpressionInfixOpcode::LesserEq, ExpressionInfixOpcode::Greater]),
            (ExpressionInfixOpcode::Eq, vec![ExpressionInfixOpcode::NotEq]),
            (ExpressionInfixOpcode::NotEq, vec![ExpressionInfixOpcode::Eq]),
        ]
    };
}

fn initialize_trace_mutation_operator_mutation_and_constant(
    pos: &[usize],
    size: usize,
    symbolic_trace: &[SymbolicValueRef],
    operator_mutation_rate: f64,
    setting: &VerificationSetting,
    rng: &mut StdRng,
) -> Vec<FxHashMap<usize, SymbolicValue>> {
    (0..size)
        .map(|_| {
            pos.iter()
                .map(|p| match &*symbolic_trace[*p] {
                    SymbolicValue::BinaryOp(left, op, right) => {
                        if rng.gen::<f64>() < operator_mutation_rate {
                            let mutated_op = if let Some(related_ops) = OPERATOR_MUTATION_CANDIDATES
                                .iter()
                                .find(|&&(key, _)| key == op.0)
                                .map(|&(_, ref ops)| ops)
                            {
                                *related_ops
                                    .iter()
                                    .choose(rng)
                                    .expect("Related operator group cannot be empty")
                            } else {
                                panic!("No group defined for the given opcode: {:?}", op);
                            };

                            (
                                p.clone(),
                                SymbolicValue::BinaryOp(
                                    left.clone(),
                                    DebuggableExpressionInfixOpcode(mutated_op),
                                    right.clone(),
                                ),
                            )
                        } else {
                            (
                                p.clone(),
                                SymbolicValue::ConstantInt(draw_random_constant(setting, rng)),
                            )
                        }
                    }
                    _ => (
                        p.clone(),
                        SymbolicValue::ConstantInt(draw_random_constant(setting, rng)),
                    ),
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
    rng: &mut StdRng,
    mutate_fn: impl Fn(&mut T, &VerificationSetting, &mut StdRng),
    crossover_fn: impl Fn(&T, &T, &mut StdRng) -> T,
) -> Vec<T> {
    (0..population_size)
        .map(|_| {
            let parent1 = roulette_selection(current_population, evaluations, rng);
            let parent2 = roulette_selection(current_population, evaluations, rng);
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

fn trace_mutate(
    individual: &mut FxHashMap<usize, SymbolicValue>,
    setting: &VerificationSetting,
    rng: &mut StdRng,
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
