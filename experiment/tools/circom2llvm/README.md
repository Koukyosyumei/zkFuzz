# Circom2LLVM
Compile circom code to LLVM IR.

# Dependencies
## LLVM Installation
**NOTICE** Use `Ninja` as the build tool default, so you need to install one before building LLVM.
If you don't have, please follow the offcial guide from LLVM at: https://github.com/llvm/llvm-project/tree/release/13.x .

**NOTICE** The building command used by us is different from the standard one.

**NOTICE** The path/to/llvm-project should be an abstract path, such as /User/You/app/llvm-project.

**NOTICE** DLLVM_TARGETS_TO_BUILD depends on your architecture.

```bash
git clone --depth 1 --branch release/13.x git@github.com:llvm/llvm-project.git
cd ./llvm-project
cmake -S llvm -B build -G Ninja \
-DLLVM_TARGETS_TO_BUILD="{X86||ARM||RISCV}" \
-DCMAKE_BUILD_TYPE=Release \
-DLLVM_PARALLEL_LINK_JOBS=1 \
-DLLVM_OPTIMIZED_TABLEGEN=ON \
-DCMAKE_EXPORT_COMPILE_COMMANDS=ON
cmake --build build/
export LLVM_PATH=path/to/llvm-project/build
export PATH=$PATH:$LLVM_PATH/bin
export LLVM_SYS_130_PREFIX=$LLVM_PATH
```

## Rust & Cargo
This repo works on `rustc & cargo 1.64.0`, and all of the crates dependencies will be installed automatically.

## Compiler
### Compilation Rule
1. One circom file will => one LLVM `.ll` file, with a module named `main`.
2. All functions and templates are concluded into the `.ll` file.
3. If the main component isn't provided:
    One circom template => two LLVM functions:
    Name the parameters of the circom template as `p`
    1. `Build(p)` => Returns an LLVM struct represents the template, named as `s`.
    2. `Init(p, s)` => Executes the logic inside the template.
4. Else, all possible instantiations will be collected, and for every instantiation, two functions mentioned before will be generated under that instantiation.
5. One circom Function => one LLVM function.

### Compilation Stage
1. Collect and resolve dependencies.
2. Infer types and collect all possible instantiations (if provided).
3. Unroll all of the `while` loop and `if-else` statements if the instantiation is provided.
4. Generate LLVM functions and instructions of circom functions.
5. Generate LLVM functions of circom templates.
6. Generate LLVM instructions of circom templates.

### Build
```bash
git clone https://github.com/Veridise/circom2llvm path/to/circom2llvm
cd path/to/circom2llvm
cargo build --bin=circom2llvm --package=circom2llvm
```

### Usage
```bash
./target/debug/circom2llvm --input path/to/circomfile_or_dir --output path/to/output
```

### Hints
1. Critical variables and functions are named in `ir_generator/src/namer.rs`.
2. The IR is Load-Store format, you could transfer it to SSA format by mem2reg pass.
3. The size of every dimension of the array is limited to a constant, the default is 256 and could be provided by the command line.
4. Don't define a component variable without initializing it.

### TODO
1. Support the function interpretation during the instantiation.
