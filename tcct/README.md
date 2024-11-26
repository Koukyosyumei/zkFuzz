# Trace-Constraint Consistency Test (TCCT)

This tool is designed to extract and analyze the trace constraints ($`\mathcal{T}(\mathcal{P})`$) and side constraints ($`\mathcal{S}(\mathcal{C})`$) from ZKP circuits written in Circom.

## Build

To compile the tool, run:

```bash
cargo build
```

## Usage

```
USAGE:
    tcct [FLAGS] [OPTIONS] [--] [input]

FLAGS:
        --print_ast                            (TCCT) Prints AST
        --print_stats                          (TCCT) Prints the stats of constraints
        --symbolic_template_params             (TCCT) Treats the template parameters of the main template as symbolic values
        --propagate_substitution               (TCCT) Propagate variable substitution as much as possible
        --search_counter_example               (TCCT) Search counter examples to check whether the given circuit is well-constrained or not
OPTIONS:
        --debug_prime <debug_prime>          (TCCT) Prime number for TCCT debugging [default:
                                     21888242871839275222246405745257275088548364400416034343698204186575808495617]
ARGS:
    <input>    Path to a circuit with a main component [default: ./circuit.circom]
```

**Example command:**

```bash
./target/debug/tcct ../sample/iszero_vuln.circom --debug_prime 3 --search_counter_example
```

**Example output:**

```bash
🧩 Parsing Templates...
⚙️ Parsing Function...
🛒 Gathering Trace/Side Constraints...
===========================================================
===========================================================
🩺 Scanning TCCT Instances...
   🚨 Counter Example:
      🔥 UnderConstrained 🔥
      🔍 Assignment Details:
           ➡️ main.out = 1
           ➡️ main.inv = 0
           ➡️ main.in = 1

======================= TCCT Report =======================
📊 Execution Summary:
  - Prime Number        : 3
  - Total Paths Explored: 2
  - Compression Rate    : 50.00% (4/8)
  - Verification        : 💥 NOT SAFE 💥
  - Execution Time      : 3.5248ms
===========================================================
Everything went okay
```

This tool also provides multiple verbosity levels for detailed analysis:

- `warn`: Outputs warnings and basic statistics about the trace and side constraints.
- `info`: Includes everything from `warn` and adds details about all possible finite states.
- `debug`: Includes everything from `info` and adds the full AST (Abstract Syntax Tree).
- `trace`: Includes everything from `debug` and outputs all intermediate trace states during execution.

**Example Command with Verbosity:**

```bash
RUST_LOG=trace ./target/debug/tcct ../sample/lessthan3.circom --print_ast --print_stats
```

**Example output:**

<div style="display: flex; align-items: flex-start; justify-content: space-around;">
  <img src="img/ast.png" alt="AST" style="width: 20%; margin-right: 5px;">
  <img src="img/se.png" alt="Traces" style="width: 50%; margin-right: 5px;">
  <img src="img/result.png" alt="Summary Reports" style="width: 20%;">
</div>


