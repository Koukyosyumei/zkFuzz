# Dataset

## Benchmark

You can run experiments on either ./benchmark1 or ./benchmark2 using one of the following tools: zkfuzz, conscs, zkap, picus-z3, picus-cvc5, or circomspect.

Use the following script to iterate through each benchmark directory:

```
for dir in $(find <BENCHMARK> -mindepth 1 -maxdepth 1 -type d); do
    if [ -d "$dir" ]; then
        echo "Processing: $dir"
        ./experiment.sh -m <TOOL_NAME> -d ${dir} -t 120
    fi
done
```

Replace <BENCHMARK> and <TOOL_NAME> accordingly.

## Install Tools

### Circomspect

**Install**

```bash
sh ./install_circomspect.sh
```

**Example**

```bash
circomspect ../tests/sample/test_vuln_iszero.circom
```

### ZKAP

**Install**

```bash
sh ./install_zkap.sh
```

**Example**

```bash
zkap ../tests/sample/test_vuln_iszero.circom
```

### Picus

**Install**

```bash
sh ./install_picus.sh
sh ./install_picus_dependencies.sh
```

**Example**

```bash
./tools/Picus/run-picus --solver z3 ../tests/sample/test_vuln_iszero.circom
```

### ConsCS

**Install**

```bash
pip install z3-solver
```

**Example**

```bash
circom ../tests/sample/test_vuln_iszero.circom --r1cs
python3 ./ConsCS/analyze_circuit.py test_vuln_iszero.r1cs ./ConsCS/logs/test_log.log ./ConsCS/logs/test_log_with_contributions.log 111 4
```