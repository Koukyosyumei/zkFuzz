# zkFuzz

> [!IMPORTANT]
> This tool is under active development. Usage patterns and features may change over time.

<p align="center">
  <a href="https://arxiv.org/abs/2504.11961" target="_blank">
      <img src="./img/zkfuzz-logo.png" alt="zkFuzz Logo" height="126">
  </a>
</p>

![example workflow](https://github.com/Koukyosyumei/zkFuzz/actions/workflows/test.yaml/badge.svg)
![GPL-3.0](https://img.shields.io/github/license/Koukyosyumei/zkFuzz?color=blue)
![Fuzzing Powered](https://img.shields.io/badge/Fuzzing-powered--by--program--mutation-orange)

**zkFuzz** is a ZK circuit fuzzer designed to help you identify vulnerabilities in [zero-knowledge proof](https://en.wikipedia.org/wiki/Non-interactive_zero-knowledge_proof) circuits. It leverages fuzzing with program mutation to uncover counterexamples that reveal under-constrained or over-constrained behavior in your circuits. zkFuzz currently supports [Circom](https://docs.circom.io/), with support for additional languages coming soon.

## 🚀 Install

**Option 1: Install via Cargo**

```bash
cargo install --git https://github.com/Koukyosyumei/zkFuzz zkfuzz
```

**Option 2: Build from Source**

```bash
git clone https://github.com/Koukyosyumei/zkFuzz.git
cd zkFuzz
cargo build --release
```

## 🧰 Basic Usage

zkFuzz’s CLI provides numerous options to tailor your fuzzing session. Below is a summary of the available commands and flags:

```
ZK Circuit Fuzzer

USAGE:
    zkfuzz [FLAGS] [OPTIONS] [--] [input]

FLAGS:
        --constraint_assert_dissabled    Does not add asserts in the generated code for === constraint equalities
        --lessthan_dissabled             (zkFuzz) Does not detect overflow erros due to LessThan template
        --print_ast                      (zkFuzz) Prints AST
        --show_stats_of_ast              (zkFuzz) Prints the basic stats of AST
        --print_stats                    (zkFuzz) Prints the stats of constraints
        --print_stats_csv                (zkFuzz) Prints the stats of constraints in CSV format
        --symbolic_template_params       (zkFuzz) Treats the template parameters of the main template as symbolic values
        --save_output                    (zkFuzz) Save the output when the counterexample is found
    -h, --help                           Prints help information
    -V, --version                        Prints version information

OPTIONS:
    -l <link_libraries>                  Adds directory to library search path
    -p, --prime <prime>
            To choose the prime number to use to generate the circuit. Receives the name of the curve (bn128, bls12381,
            goldilocks, grumpkin, pallas, vesta, secq256r1) [default: bn128]
        --debug_prime <debug_prime>
            (zkFuzz) Prime number for zkFuzz [default:
            21888242871839275222246405745257275088548364400416034343698204186575808495617]
        --search_mode <search_mode>
            (zkFuzz) Search mode to find the counter example that shows the given circuit is not well-constrained [default: ga]
        --heuristics_range <heuristics_range>
            (zkFuzz) Heuristics range for zkFuzz [default: 100]
        --path_to_mutation_setting <path_to_mutation_setting>
            (zkFuzz) Path to the setting file for Mutation Testing [default: none]
        --path_to_whitelist <path_to_whitelist>                  
            (zkFuzz) Path to the white-lists file [default: none]

ARGS:
    <input>    Path to a circuit with a main component [default: ./circuit.circom]
```

**Example Command:**

Run zkFuzz using your circuit file written in Circom:

```bash
# Using the debug build:
./target/debug/zkfuzz ./tests/sample/iszero_vuln.circom

# Using the release build:
./target/release/zkfuzz ./tests/sample/iszero_vuln.circom
```

**Example Output:**

<img src="img/main_result.png" alt="Result" width=700>

## 🔬 Fuzzing with Program Mutation

Fuzzing with program mutation mode (`ga` mode) suppots a detailed configuration through the `path_to_mutation_setting` option. The configuration is specified as a JSON file.

### Schema Overview

Here is an example of the JSON configuration schema:

```json
{
    "seed": 0,
    "program_population_size": 30,
    "input_population_size": 30,
    "max_generations": 300,
    "input_initialization_method": "random",
    "trace_mutation_method": "constant",
    "fitness_function": "error",
    "mutation_rate": 0.3,
    "crossover_rate": 0.5,
    "operator_mutation_rate": 0.2,
    "num_eliminated_individuals": 5,
    "max_num_mutation_points": 10,
    "input_update_interval": 1,
    "input_generation_max_iteration": 30,
    "input_generation_crossover_rate": 0.66,
    "input_generation_mutation_rate": 0.5,
    "input_generation_singlepoint_mutation_rate": 0.5,
    "random_value_ranges": [
        [   "-10", 
            "10"
        ],
        [
            "21888242871839275222246405745257275088548364400416034343698204186575808495517", 
            "21888242871839275222246405745257275088548364400416034343698204186575808495617"
        ]
    ],
    "random_value_probs": [
        0.5,
        0.5
    ],
    "save_fitness_scores": false
}
```

If the configuration JSON file omits some keys, the default values are used for those omitted keys.

<details>
<summary><strong>Field Descriptions – Click to view all configuration options</strong></summary>

```yaml
- seed (u64)
  - Purpose: Seed for random number generation to ensure reproducibility. If set to 0, a new seed is internally generated using the thread-local random number generator.
  - Default: 0

- program_population_size (usize)
  - Purpose: Size of the program population in the genetic algorithm.
  - Default: 30

- input_population_size (usize)
  - Purpose: Size of the input population in the genetic algorithm.
  - Default: 30

- max_generations (usize)
  - Purpose: Maximum number of generations for the evolutionary process.
  - Default: 500

- input_initialization_method (String)
  - Purpose: Method used to initialize inputs ("random", "fitness", "coverage").
  - Default: "random"

- trace_mutation_method (String)
  - Purpose: Method used for trace mutation ("naive", "constant", "constant_operator", "constant_operator_add", "constant_operator_delete").
  - Default: "constant_operator"

- fitness_function (String)
  - Purpose: Function used to evaluate fitness of solutions ("error", "const").
  - Default: "error"

- mutation_rate (f64)
  - Purpose: Rate at which mutations occur in the genetic algorithm.
  - Default: 0.3

- crossover_rate (f64)
  - Purpose: Rate at which crossover occurs in the genetic algorithm.
  - Default: 0.5

- operator_mutation_rate (f64)
  - Purpose: Rate of mutation for operators in the genetic algorithm.
  - Default: 0.1

- runtime_mutation_rate (f64)
  - Purpose: Specifies the mutation rate applied during runtime mutation processes.
  - Default: 0.3

- num_eliminated_individuals (usize)
  - Purpose: The number of individuals with poor fitness eliminated in each generation.
  - Default: 5

- max_num_mutation_points (usize)
  - Purpose: The maximum number of mutation points allowed in the symbolic trace.
  - Default: 10

- input_update_interval (usize)
  - Purpose: Interval at which inputs are updated.
  - Default: 1

- input_generation_max_iteration (usize)
  - Purpose: Maximum number of iterations for input generation.
  - Default: 30

- input_generation_crossover_rate (f64)
  - Purpose: Crossover rate for input generation.
  - Default: 0.66

- input_generation_mutation_rate (f64)
  - Purpose: Mutation rate for input generation.
  - Default: 0.5

- input_generation_singlepoint_mutation_rate (f64)
  - Purpose: Single-point mutation rate for input generation.
  - Default: 0.5

- random_value_ranges (Array of Arrays)
  - Purpose: Specifies ranges for random value generation. Each range is defined as a pair of big integers (provided as strings) representing the lower and upper bounds.
  - Default: [["0", "2"], ["2", "11"], ["11", "21888242871839275222246405745257275088548364400416034343698204186575808495517"], ["21888242871839275222246405745257275088548364400416034343698204186575808495517", "21888242871839275222246405745257275088548364400416034343698204186575808495617"]]

- random_value_probs (Array of f64)
  - Purpose: Probabilities associated with each range in `random_value_ranges`.
  - Default: [0.15, 0.34, 0.01, 0.5]

- binary_mode_prob (f64)
  - Purpose: Probability of restricting random input to only 0 or 1.
  - Default: 0.0

- binary_mode_search_level (usize)
  - Purpose: Search depth for the binary pattern (x * (1 - x) === 0) check.
  - Default: 1

- binary_mode_warmup_round (f64)
  - Purpose: Ratio of warmup rounds where binary_mode_prob is temporarily set to 1 upon detecting the binary pattern.
  - Default: 0.0

- zero_div_attempt_prob (f64)
  - Purpose: Probability of invoking the quadratic equation solver to analytically determine solutions for zero-division patterns.
  - Default: 0.2

- statement_deletion_prob (f64)
  - Purpose: Probability of deleting a statement during mutation.
  - Default: 0.2

- add_random_const_prob (f64)
  - Purpose: Probability of adding a random constant during mutation.
  - Default: 0.2

- dissable_runtime_mutation_for_hash_check (bool)
  - Purpose: When enabled, disables runtime mutation for hash checks.
  - Default: false

- dissable_heuristic_for_invalid_array_subscript (bool)
  - Purpose: When enabled, disables heuristics that handle invalid array subscripts.
  - Default: false

- save_fitness_scores (bool)
  - Purpose: Flag indicating whether fitness scores should be saved.
  - Default: false
```

</details>

## 💡 Tips & Advanced Features

### 💾 Saving Output

When the `--save_output` option is enabled, the counterexample is saved to the directory when found.

**Example Command with `--save_output`**

```bash
./target/release/zkfuzz ./tests/sample/test_vuln_iszero.circom --search_mode="ga" --save_output
```

The output filename will follow the pattern `<TARGET_FILE_NAME>_<RANDOM_SUFFIX>_counterexample.json`.

**Example Output:**

```json
{
  "0_target_path": "./tests/sample/test_vuln_iszero.circom",
  "1_main_template": "VulnerableIsZero",
  "2_search_mode": "ga",
  "3_execution_time": "36.3001ms",
  "4_git_hash_of_zkfuzz": "106b20ddad6431d0eee3cd73f9aac0153af4bbd9",
  "5_flag": {
    "1_type": "UnderConstrained-NonDeterministic",
    "2_expected_output": {
      "name": "main.out",
      "value": "0"
    }
  },
  "6_target_output": "main.out",
  "7_assignment": {
    "main.in": "21888242871839275222246405745257275088548364400416034343698204186575808495524",
    "main.inv": "0",
    "main.out": "1"
  },
  "8_auxiliary_result": {
    "mutation_test_config": {
      "crossover_rate": 0.5,
      "fitness_function": "error",
      "input_generation_crossover_rate": 0.66,
      "input_generation_max_iteration": 30,
      "input_generation_mutation_rate": 0.5,
      "input_generation_singlepoint_mutation_rate": 0.5,
      "input_initialization_method": "random",
      "input_population_size": 30,
      "input_update_interval": 1,
      "max_generations": 300,
      "mutation_rate": 0.3,
      "operator_mutation_rate": 0.2,
      "program_population_size": 30,
      "random_value_probs": [
        0.5,
        0.5
      ],
      "random_value_ranges": [
        [
          "-10",
          "10"
        ],
        [
          "21888242871839275222246405745257275088548364400416034343698204186575808495517",
          "21888242871839275222246405745257275088548364400416034343698204186575808495617"
        ]
      ],
      "save_fitness_scores": false,
      "seed": 0,
      "trace_mutation_method": "constant"
    },
    "mutation_test_log": {
      "fitness_score_log": [],
      "generation": 7,
      "random_seed": 13057132941229430025
    }
  }
}
```

### 🧪 Logging

zkFuzz offers multiple verbosity levels for detailed analysis with the environmental variable `RUST_LOG`:

- `warn`: Outputs warnings and errors.
- `info`: Includes everything from `warn` and adds the basic statistics about the trace and constraints.
- `debug`: Includes everything from `info` and adds the trace of the final state.
- `trace`: Includes everything from `debug` and outputs all intermediate trace states during execution.

**Example Command with Verbosity:**

```bash
RUST_LOG=trace ./target/debug/zkfuzz ../sample/lessthan3.circom --print_ast --print_stats
```

**Example Output:**

<div style="display: flex; align-items: flex-start; justify-content: space-around;">
  <img src="img/ast.png" alt="AST" style="width: 20%; margin-right: 5px;">
  <img src="img/se.png" alt="Traces" style="width: 50%; margin-right: 5px;">
  <img src="img/result.png" alt="Summary Reports" style="width: 20%;">
</div>

## 🏆 Trophies

Here are some of the most notable vulnerabilities uncovered using zkfuzz, confirmed by developers.
If you’ve discovered a significant issue with our tool, we’d love to hear about it. Please submit a pull request with the relevant details.

- https://github.com/zkemail/zk-regex/pull/83 (awarded a bug bounty)
- https://github.com/rarimo/passport-zk-circuits/pull/60
- https://github.com/banyancomputer/hot-proofs-blake3-circom/pull/11
- https://github.com/tokamak-network/zvm
- https://github.com/inference-labs-inc/omron-circom/pull/2 (awarded a bug bounty)
- https://github.com/siv-org/secretly-cancellable-votes/pull/13
- https://github.com/SpiralOutDotEu/zk_whitelist/pull/54
- https://github.com/numtel/ntru-circom/issues/1

## 🧩 Complementary Tools

There are several excellent tools for bug detection and verification of ZK circuits.

- [Circomspect](https://github.com/trailofbits/circomspect): A linter and static analysis tool for ZK circuits
- [ZKAP](https://github.com/whbjzzwjxq/ZKAP): A collection of static analysis tools designed to detect bugs in ZK circuits
- [Picus](https://github.com/Veridise/Picus): A tool for automatically verifying non-deterministic behaviors in ZK circuits
- [ConsCS](https://github.com/jinan789/ConsCS): Enables fast verification of non-deterministic properties in ZK circuits
- [CIVER](https://github.com/costa-group/circom_civer): Verification tool for weak safety properties, pre-/post-conditions and tag-based specifications
- [CODA](https://github.com/Veridise/Coda): A statically-typed programming language for constructing and certifying ZK circuits

Using zkFuzz in conjunction with these tools can significantly enhance the likelihood of detecting subtle and critical bugs early in the development cycle.

## 📃 Cite

```
@misc{takahashi2025zkfuzzfoundationframeworkeffective,
      title={zkFuzz: Foundation and Framework for Effective Fuzzing of Zero-Knowledge Circuits}, 
      author={Hideaki Takahashi and Jihwan Kim and Suman Jana and Junfeng Yang},
      year={2025},
      eprint={2504.11961},
      archivePrefix={arXiv},
      primaryClass={cs.CR},
      url={https://arxiv.org/abs/2504.11961}, 
}
```

