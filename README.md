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

# 
```