use std::fmt;
use std::fs::File;

use colored::Colorize;
use log::info;
use serde::{Deserialize, Serialize};

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
    pub coverage_based_input_generation_max_iteration: usize,
    pub coverage_based_input_generation_crossover_rate: f64,
    pub coverage_based_input_generation_mutation_rate: f64,
    pub coverage_based_input_generation_singlepoint_mutation_rate: f64,
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
            coverage_based_input_generation_max_iteration: 30,
            coverage_based_input_generation_crossover_rate: 0.66,
            coverage_based_input_generation_mutation_rate: 0.5,
            coverage_based_input_generation_singlepoint_mutation_rate: 0.5,
            save_fitness_scores: false,
        }
    }
}

impl fmt::Display for MutationConfig {
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
