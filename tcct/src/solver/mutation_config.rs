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
    pub fitness_function: String,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub operator_mutation_rate: f64,
    pub input_update_interval: usize,
    pub input_generation_max_iteration: usize,
    pub input_generation_crossover_rate: f64,
    pub input_generation_mutation_rate: f64,
    pub input_generation_singlepoint_mutation_rate: f64,
    #[serde_as(as = "Vec<(DisplayFromStr, DisplayFromStr)>")]
    pub random_value_ranges: Vec<(BigInt, BigInt)>,
    pub random_value_probs: Vec<f64>,
    pub save_fitness_scores: bool,
}

impl Default for MutationConfig {
    fn default() -> Self {
        MutationConfig {
            seed: 0,
            program_population_size: 30,
            input_population_size: 30,
            max_generations: 300,
            input_initialization_method: "random".to_string(),
            fitness_function: "error".to_string(),
            mutation_rate: 0.3,
            crossover_rate: 0.5,
            operator_mutation_rate:0.2,
            input_update_interval: 1,
            input_generation_max_iteration: 30,
            input_generation_crossover_rate: 0.66,
            input_generation_mutation_rate: 0.5,
            input_generation_singlepoint_mutation_rate: 0.5,
            random_value_ranges: vec![
                (BigInt::from(-10), BigInt::from(10)),
                (BigInt::from_str("21888242871839275222246405745257275088548364400416034343698204186575808495517").unwrap(),
                 BigInt::from_str("21888242871839275222246405745257275088548364400416034343698204186575808495617").unwrap()),
            ],
            random_value_probs: vec![0.5, 0.5],
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
    ├─ Max Generations                            : {}
    ├─ Input Initialization Method                : {} 
    ├─ Fitness Function                           : {} 
    ├─ Trace Mutation Rate                        : {}
    ├─ Trace Crossover Rate                       : {}
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
