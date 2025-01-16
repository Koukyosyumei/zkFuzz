use std::collections::HashSet;
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

use program_structure::ast::ExpressionInfixOpcode;

use crate::executor::debug_ast::DebuggableExpressionInfixOpcode;
use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_state::{SymbolicConstraints, SymbolicTrace};
use crate::executor::symbolic_value::{OwnerName, SymbolicName, SymbolicValue, SymbolicValueRef};

use crate::solver::mutation_config::MutationConfig;
use crate::solver::mutation_utils::{random_crossover, roulette_selection};
use crate::solver::utils::{extract_variables, BaseVerificationConfig, CounterExample};

pub struct MutationTestResult {
    pub random_seed: u64,
    pub mutation_config: MutationConfig,
    pub counter_example: Option<CounterExample>,
    pub generation: usize,
    pub fitness_score_log: Vec<BigInt>,
}

type Gene = FxHashMap<usize, SymbolicValue>;

pub fn mutation_test_search<
    InitializeTraceFn,
    UpdateInputFn,
    FitnessFn,
    EvolveFn,
    MutateFn,
    CrossoverFn,
    SelectionFn,
>(
    sexe: &mut SymbolicExecutor,
    symbolic_trace: &SymbolicTrace,
    side_constraints: &SymbolicConstraints,
    base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    trace_initialization_fn: InitializeTraceFn,
    input_update_fn: UpdateInputFn,
    fitness_fn: FitnessFn,
    evolve_fn: EvolveFn,
    mutate_fn: MutateFn,
    crossover_fn: CrossoverFn,
    selection_fn: SelectionFn,
) -> MutationTestResult
where
    InitializeTraceFn:
        Fn(&[usize], &BaseVerificationConfig, &MutationConfig, &mut StdRng) -> Vec<Gene>,
    UpdateInputFn: Fn(
        &mut SymbolicExecutor,
        &[SymbolicName],
        &mut Vec<FxHashMap<SymbolicName, BigInt>>,
        &BaseVerificationConfig,
        &MutationConfig,
        &mut StdRng,
    ),
    FitnessFn: Fn(
        &mut SymbolicExecutor,
        &BaseVerificationConfig,
        &SymbolicTrace,
        &SymbolicConstraints,
        &Gene,
        &Vec<FxHashMap<SymbolicName, BigInt>>,
    ) -> (usize, BigInt, Option<CounterExample>),
    EvolveFn: Fn(
        &[Gene],
        &[BigInt],
        &BaseVerificationConfig,
        &MutationConfig,
        &mut StdRng,
        &MutateFn,
        &CrossoverFn,
        &SelectionFn,
    ) -> Vec<Gene>,
    MutateFn: Fn(&mut Gene, &BaseVerificationConfig, &mut StdRng),
    CrossoverFn: Fn(&Gene, &Gene, &mut StdRng) -> Gene,
    SelectionFn: for<'a> Fn(&'a [Gene], &[BigInt], &mut StdRng) -> &'a Gene,
{
    // Set random seed
    let seed = if mutation_config.seed.is_zero() {
        let mut seed_rng = rand::thread_rng();
        seed_rng.gen()
    } else {
        mutation_config.seed
    };
    let mut rng = StdRng::seed_from_u64(seed);

    // Gather mutable locations
    let mut assign_pos = Vec::new();
    for (i, sv) in symbolic_trace.iter().enumerate() {
        match *sv.as_ref() {
            SymbolicValue::Assign(_, _, false) | SymbolicValue::AssignCall(_, _, true) => {
                assign_pos.push(i);
            }
            _ => {}
        }
    }

    // Gather input variables
    let mut variables = extract_variables(symbolic_trace);
    variables.append(&mut extract_variables(side_constraints));
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();
    let mut input_variables = Vec::new();
    for v in variables_set.iter() {
        if v.owner.len() == 1
            && sexe.symbolic_library.template_library
                [&sexe.symbolic_library.name2id[&base_config.target_template_name]]
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

    // Initial Pupulation of Mutated Inputs
    let mut trace_population =
        trace_initialization_fn(&assign_pos, base_config, mutation_config, &mut rng);
    let mut fitness_scores =
        vec![-base_config.prime.clone(); mutation_config.input_population_size];
    let mut input_population = Vec::new();
    let mut fitness_score_log = if mutation_config.save_fitness_scores {
        Vec::with_capacity(mutation_config.max_generations)
    } else {
        Vec::new()
    };

    println!(
        "{} {}",
        "🎲 Random Seed:",
        seed.to_string().bold().bright_yellow(),
    );

    for generation in 0..mutation_config.max_generations {
        // Generate input population for this generation
        if generation % mutation_config.input_update_interval == 0 {
            input_update_fn(
                sexe,
                &input_variables,
                &mut input_population,
                &base_config,
                &mutation_config,
                &mut rng,
            );
        }

        // Evolve the trace population
        if !trace_population.is_empty() {
            trace_population = evolve_fn(
                &trace_population,
                &fitness_scores,
                base_config,
                &mutation_config,
                &mut rng,
                &mutate_fn,
                &crossover_fn,
                &selection_fn,
            );
        }
        trace_population.push(FxHashMap::default());

        // Evaluate the trace population
        let evaluations: Vec<_> = trace_population
            .iter()
            .map(|a| {
                fitness_fn(
                    sexe,
                    &base_config,
                    symbolic_trace,
                    side_constraints,
                    a,
                    &input_population,
                )
            })
            .collect();

        // Pick the best one
        let best_idx = evaluations
            .iter()
            .enumerate()
            .max_by_key(|&(_, value)| value.1.clone())
            .map(|(index, _)| index)
            .unwrap();

        // Extract the fitness scores
        if mutation_config.fitness_function != "const" {
            fitness_scores = evaluations.iter().map(|v| v.1.clone()).collect();
        }

        if evaluations[best_idx].1.is_zero() {
            print!(
                "\r\x1b[2K🧬 Generation: {}/{} ({:.3})",
                generation, mutation_config.max_generations, 0
            );
            println!("\n    └─ Solution found in generation {}", generation);

            return MutationTestResult {
                random_seed: seed,
                mutation_config: mutation_config.clone(),
                counter_example: evaluations[best_idx].2.clone(),
                generation: generation,
                fitness_score_log: fitness_score_log,
            };
        }

        print!(
            "\r\x1b[2K🧬 Generation: {}/{} ({:.3})",
            generation, mutation_config.max_generations, fitness_scores[best_idx]
        );
        io::stdout().flush().unwrap();

        if mutation_config.save_fitness_scores {
            fitness_score_log.push(fitness_scores[best_idx].clone());
        }
    }

    println!(
        "\n └─ No solution found after {} generations",
        mutation_config.max_generations
    );

    MutationTestResult {
        random_seed: seed,
        mutation_config: mutation_config.clone(),
        counter_example: None,
        generation: mutation_config.max_generations,
        fitness_score_log: fitness_score_log,
    }
}

fn draw_random_constant(base_config: &BaseVerificationConfig, rng: &mut StdRng) -> BigInt {
    if rng.gen::<bool>() {
        rng.gen_bigint_range(
            &(BigInt::from_str("10").unwrap() * -BigInt::one()),
            &(BigInt::from_str("10").unwrap()),
        )
    } else {
        rng.gen_bigint_range(
            &(base_config.prime.clone() - BigInt::from_str("100").unwrap()),
            &(base_config.prime),
        )
    }
}

pub fn update_input_population_with_random_sampling(
    _sexe: &mut SymbolicExecutor,
    input_variables: &[SymbolicName],
    inputs_population: &mut Vec<FxHashMap<SymbolicName, BigInt>>,
    base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    let mut new_inputs_population: Vec<_> = (0..mutation_config.input_population_size)
        .map(|_| {
            input_variables
                .iter()
                .map(|var| (var.clone(), draw_random_constant(base_config, rng)))
                .collect::<FxHashMap<SymbolicName, BigInt>>()
        })
        .collect();
    inputs_population.clear();
    inputs_population.append(&mut new_inputs_population);
}

pub fn evaluate_coverage(
    sexe: &mut SymbolicExecutor,
    inputs: &FxHashMap<SymbolicName, BigInt>,
    base_config: &BaseVerificationConfig,
) -> usize {
    sexe.clear();
    sexe.turn_on_coverage_tracking();
    sexe.cur_state.add_owner(&OwnerName {
        id: sexe.symbolic_library.name2id["main"],
        counter: 0,
        access: None,
    });
    sexe.feed_arguments(
        &base_config.template_param_names,
        &base_config.template_param_values,
    );
    sexe.concrete_execute(&base_config.target_template_name, inputs);
    sexe.record_path();
    sexe.turn_off_coverage_tracking();
    sexe.coverage_count()
}

pub fn update_input_population_with_coverage_maximization(
    sexe: &mut SymbolicExecutor,
    input_variables: &[SymbolicName],
    inputs_population: &mut Vec<FxHashMap<SymbolicName, BigInt>>,
    base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) {
    sexe.clear_coverage_tracker();
    let mut total_coverage = 0_usize;
    inputs_population.clear();

    let mut initial_input_population = Vec::new();
    update_input_population_with_random_sampling(
        sexe,
        input_variables,
        &mut initial_input_population,
        &base_config,
        &mutation_config,
        rng,
    );

    for input in &initial_input_population {
        let new_coverage = evaluate_coverage(sexe, &input, base_config);
        if new_coverage > total_coverage {
            inputs_population.push(input.clone());
            total_coverage = new_coverage;
        }
    }

    for _ in 0..mutation_config.input_generation_max_iteration {
        let mut new_inputs_population = Vec::new();

        // Iterate through the population and attempt mutations
        for input in inputs_population.iter() {
            let mut new_input = input.clone();

            if rng.gen::<f64>() < mutation_config.input_generation_crossover_rate {
                // Crossover
                let other = inputs_population[rng.gen_range(0, inputs_population.len())].clone();
                new_input = random_crossover(input, &other, rng);
            }
            if rng.gen::<f64>() < mutation_config.input_generation_mutation_rate {
                if rng.gen::<f64>() < mutation_config.input_generation_singlepoint_mutation_rate {
                    // Mutate only one input variable
                    let var = &input_variables[rng.gen_range(0, input_variables.len())];
                    let mutation = draw_random_constant(base_config, rng);
                    new_input.insert(var.clone(), mutation);
                } else {
                    // Mutate each input variable with a small probability
                    for var in input_variables {
                        // rng.gen_bool(0.2)
                        if rng.gen::<bool>() {
                            let mutation = draw_random_constant(base_config, rng);
                            new_input.insert(var.clone(), mutation);
                        }
                    }
                }
            }

            // Evaluate the new input
            let new_coverage = evaluate_coverage(sexe, &new_input, base_config);
            if new_coverage > total_coverage {
                new_inputs_population.push(new_input);
                total_coverage = new_coverage;
            }
        }
        inputs_population.append(&mut new_inputs_population);

        if inputs_population.len() > mutation_config.input_population_size {
            break;
        }
    }
}

pub fn initialize_trace_mutation_only_constant(
    pos: &[usize],
    base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
) -> Vec<Gene> {
    (0..mutation_config.program_population_size)
        .map(|_| {
            pos.iter()
                .map(|p| {
                    (
                        p.clone(),
                        SymbolicValue::ConstantInt(draw_random_constant(base_config, rng)),
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
    base_config: &BaseVerificationConfig,
    rng: &mut StdRng,
) -> Vec<Gene> {
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
                                SymbolicValue::ConstantInt(draw_random_constant(base_config, rng)),
                            )
                        }
                    }
                    _ => (
                        p.clone(),
                        SymbolicValue::ConstantInt(draw_random_constant(base_config, rng)),
                    ),
                })
                .collect()
        })
        .collect()
}

pub fn evolve_population<T: Clone, MutateFn, CrossoverFn, SelectionFn>(
    prev_population: &[T],
    prev_evaluations: &[BigInt],
    base_base_config: &BaseVerificationConfig,
    mutation_config: &MutationConfig,
    rng: &mut StdRng,
    mutate_fn: &MutateFn,
    crossover_fn: &CrossoverFn,
    selection_fn: &SelectionFn,
) -> Vec<T>
where
    MutateFn: Fn(&mut T, &BaseVerificationConfig, &mut StdRng),
    CrossoverFn: Fn(&T, &T, &mut StdRng) -> T,
    SelectionFn: for<'a> Fn(&'a [T], &[BigInt], &mut StdRng) -> &'a T,
{
    (0..mutation_config.program_population_size)
        .map(|_| {
            let parent1 = selection_fn(prev_population, prev_evaluations, rng);
            let parent2 = selection_fn(prev_population, prev_evaluations, rng);
            let mut child = if rng.gen::<f64>() < mutation_config.crossover_rate {
                crossover_fn(&parent1, &parent2, rng)
            } else {
                parent1.clone()
            };
            if rng.gen::<f64>() < mutation_config.mutation_rate {
                mutate_fn(&mut child, base_base_config, rng);
            }
            child
        })
        .collect()
}

pub fn trace_mutate(individual: &mut Gene, base_config: &BaseVerificationConfig, rng: &mut StdRng) {
    if !individual.is_empty() {
        let var = individual.keys().choose(rng).unwrap();
        individual.insert(
            var.clone(),
            SymbolicValue::ConstantInt(draw_random_constant(base_config, rng)),
        );
    }
}
