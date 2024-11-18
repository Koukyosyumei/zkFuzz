# ProoFuzz

- [Documentation](./doc/)
- [Meeting Notes](./NOTE.md)

## Build

- circom2llvm

```bash
$ cargo build --bin=circom2llvm --package=circom2llvm --release
# sudo cp ./target/release/circom2llvm /usr/local/bin/circom2llvm
```

- zkap

```bash
cd zkap/detectors
sh ./build.sh
```

- proofuzz

```bash
cd proofuzz
sh ./build.sh
```


## Example

1. Compile Circom to LLVM IR

```bash
$ circom2llvm --input ./sample/iszero_safe.circom --output ./sample/
```

2. Visualization

```bash
$ opt -enable-new-pm=0 -load ./proofuzz/build/libProoFuzzPass.so --ExtendedPrintGraphviz -S ./sample/iszero_safe.ll -o /dev/null 2> ./sample/iszero_safe.dot
```

<img src="./sample/iszero_safe_graphviz.svg" width=900>


3. Execution

- Compile to LLVM IR

```bash
$ circom2llvm --input ./sample/iszero_vuln.circom --output ./sample/
```

- Modify LLVM IR file

```bash
$ opt -enable-new-pm=0 -load ./proofuzz/build/libProoFuzzPass.so  --InitializeConstraintPass --MainAdderPass --enable-overwrite-free-variables --printout-outputs --printout-constraints -S ./sample/iszero_vuln.ll -o ./sample/iszero_vuln_overwritten.ll
```

- Link LLVM IR files

```bash
$ llvm-link ./sample/iszero_vuln_overwritten.ll ../circom2llvm/utils/field_operations.ll -o ./sample/iszero_vuln_overwritten_linked.ll
```

- Execute the LLVM IR file

```bash
# execute .ll file
$ lli ./sample/iszero_vuln_overwritten_linked.ll
```

Input:

```makefile
1     # Lower bits of input `in`
0     # Higher bits of input `in`
0     # Lower bits of intermediate variable `inv`
0     # Higher bits of intermediate variable `inv`
```

Output:

```makefile
1     # Lower bits of the modified circuit's output `out`
0     # Higher bits of the modified circuit's output `out`
1     # Whether all constraints are met in the modified circuit
0     # Lower bits of the original circuit's output `out`
0     # Higher bits of the original circuit's output `out`
1     # Whether all constraints are met in the modified circuit
Error: Under-Constraint-Condition Met. Terminating program.
```

4. AFL++

```bash
$ afl-clang-fast -S -emit-llvm ./sample/iszero_vuln_overwritten_linked.ll -o ./sample/iszero_vuln_overwritten_linked_instrumented.ll
$ llc -filetype=obj ./sample/iszero_vuln_overwritten_linked_instrumented.ll -o ./sample/iszero_vuln_overwritten_linked_instrumented.o
$ afl-clang-fast ./sample/iszero_vuln_overwritten_linked_instrumented.o -o ./sample/iszero_vuln_overwritten_linked_instrumented.out
$ afl-fuzz -i ./benchmark/data/ -o benchmark/output_dir/ -- ./sample/iszero_vuln_overwritten_linked_instrumented.out
```
