# ProoFuzz

- [Doc](./doc/)
- [Meeting Notes](./NOTE.md)

## Build

- circom2llvm

```bash
cargo build --bin=circom2llvm --package=circom2llvm --release
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

```bash
# compile circom to llvm ir
circom2llvm --input ./benchmark/sample/iszero_safe.circom --output ./benchmark/sample/
```

- Visualization

```bash
opt -enable-new-pm=0 -load ./proofuzz/build/libProoFuzzPass.so --ExtendedPrintGraphviz -S ./benchmark/sample/iszero_safe.ll -o /dev/null 2> ./benchmark/sample/iszero_safe.dot
```

<img src="./benchmark/sample/iszero_safe_graphviz.svg" width=900>


- Execution

```bash
# modify .ll file 
opt -enable-new-pm=0 -load ./proofuzz/build/libProoFuzzPass.so  --InitializeConstraintPass --MainAdderPass -S ./benchmark/sample/iszero_safe.ll -o ./benchmark/sample/iszero_safe_modified.ll
llvm-link ./benchmark/sample/iszero_safe_modified.ll ../circom2llvm/utils/field_operations.ll -o ./benchmark/sample/iszero_safe_linked.ll

# execute .ll file
lli ./benchmark/sample/iszero_safe_linked.ll
```