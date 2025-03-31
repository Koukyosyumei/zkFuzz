use std::fmt;
use std::fs::File;
use std::str::FromStr;

use colored::Colorize;
use log::info;
use num_bigint_dig::BigInt;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MutationConfig {
    pub seed: u64,
    pub program_population_size: usize,
    pub input_population_size: usize,
    pub max_generations: usize,
    pub input_initialization_method: String,
    pub trace_mutation_method: String,
    pub fitness_function: String,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub operator_mutation_rate: f64,
    pub runtime_mutation_rate: f64,
    pub num_eliminated_individuals: usize,
    pub max_num_mutation_points: usize,
    pub input_update_interval: usize,
    pub input_generation_max_iteration: usize,
    pub input_generation_crossover_rate: f64,
    pub input_generation_mutation_rate: f64,
    pub input_generation_singlepoint_mutation_rate: f64,
    #[serde_as(as = "Vec<(DisplayFromStr, DisplayFromStr)>")]
    pub random_value_ranges: Vec<(BigInt, BigInt)>,
    pub random_value_probs: Vec<f64>,
    pub binary_mode_prob: f64,
    pub binary_mode_search_level: usize,
    pub binary_mode_warmup_round: f64,
    pub zero_div_attempt_prob: f64,
    pub statement_deletion_prob: f64,
    pub add_random_const_prob: f64,
    pub dissable_runtime_mutation_for_hash_check: bool,
    pub dissable_heuristic_for_invalid_array_subscript: bool,
    pub save_fitness_scores: bool,
}

impl Default for MutationConfig {
    fn default() -> Self {
        MutationConfig {
            seed: 0,
            program_population_size: 30,
            input_population_size: 30,
            max_generations: 500,
            input_initialization_method: "random".to_string(),
            trace_mutation_method: "constant_operator".to_string(),
            fitness_function: "error".to_string(),
            mutation_rate: 0.3,
            crossover_rate: 0.5,
            operator_mutation_rate:0.1,
            runtime_mutation_rate:0.3,
            num_eliminated_individuals:5,
            max_num_mutation_points:10,
            input_update_interval: 1,
            input_generation_max_iteration: 30,
            input_generation_crossover_rate: 0.66,
            input_generation_mutation_rate: 0.5,
            input_generation_singlepoint_mutation_rate: 0.5,
            random_value_ranges: vec![
                (BigInt::from(0), BigInt::from(2)),
                (BigInt::from(2), BigInt::from(11)),
                (BigInt::from_str("11").unwrap(),
                 BigInt::from_str("21888242871839275222246405745257275088548364400416034343698204186575808495517").unwrap()),
                (BigInt::from_str("21888242871839275222246405745257275088548364400416034343698204186575808495517").unwrap(),
                 BigInt::from_str("21888242871839275222246405745257275088548364400416034343698204186575808495617").unwrap()),
            ],
            random_value_probs: vec![0.15, 0.34, 0.01, 0.5],
            binary_mode_prob: 0.0,
            binary_mode_search_level: 1,
            binary_mode_warmup_round: 0.0,
            zero_div_attempt_prob:0.2,
            statement_deletion_prob: 0.2,
            add_random_const_prob: 0.2,
            dissable_runtime_mutation_for_hash_check:false,
            dissable_heuristic_for_invalid_array_subscript:false,
            save_fitness_scores: false,
        }
    }
}

impl fmt::Display for MutationConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "🧬 Mutation Settings:
    ├─ Program Population Size                    : {}
    ├─ Input Population Size                      : {}
    ├─ Maximum Number of Generations              : {}
    ├─ Input Initialization Method                : {} 
    ├─ Fitness Function                           : {} 
    ├─ Trace Mutation Rate                        : {}
    ├─ Trace Crossover Rate                       : {}
    ├─ Operator Mutation Rate                     : {}
    ├─ Runimte Mutation Rate                      : {}
    ├─ Maximum Number of Mutated Points           : {}
    ├─ Input Generation Interval                  : {} 
    ├─ Input Generation Maximum Iteration         : {} 
    ├─ Input Generation Crossover Rate            : {}
    ├─ Input Generation Mutation Rate             : {}
    └─ Input Generation Singlepoint Mutation Rate : {}",
            self.program_population_size.to_string().bright_yellow(),
            self.input_population_size.to_string().bright_yellow(),
            self.max_generations.to_string().bright_yellow(),
            self.input_initialization_method.bright_yellow(),
            self.fitness_function.bright_yellow(),
            self.mutation_rate.to_string().bright_yellow(),
            self.crossover_rate.to_string().bright_yellow(),
            self.operator_mutation_rate.to_string().bright_yellow(),
            self.runtime_mutation_rate.to_string().bright_yellow(),
            self.max_num_mutation_points.to_string().bright_yellow(),
            self.input_update_interval.to_string().bright_yellow(),
            self.input_generation_max_iteration
                .to_string()
                .bright_yellow(),
            self.input_generation_crossover_rate
                .to_string()
                .bright_yellow(),
            self.input_generation_mutation_rate
                .to_string()
                .bright_yellow(),
            self.input_generation_singlepoint_mutation_rate
                .to_string()
                .bright_yellow()
        )
    }
}

pub fn load_config_from_json(file_path: &str) -> Result<MutationConfig, serde_json::Error> {
    let file = File::open(file_path);
    if file.is_ok() {
        let settings: MutationConfig = serde_json::from_reader(file.unwrap())?;
        Ok(settings)
    } else {
        info!("Use the default setting for mutation testing");
        Ok(MutationConfig::default())
    }
}
