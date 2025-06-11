# zkFuzz for Noir

`zkfuzz-noir` is a fuzzing tool designed to work with the Noir ecosystem. It is compatible with the following versions of `nargo` and `noirc`:

```
nargo version = 1.0.0-beta.6
noirc version = 1.0.0-beta.6+29d137ae036e41dc093202c8a68bf080069ccd98
(git version hash: 29d137ae036e41dc093202c8a68bf080069ccd98, is dirty: true)
```

- Usage

```
cd examples
nargo compile
cd ..

./target/release/zkfuzz-noir --artifact-path ./examples/iszero/target/iszero.json --prover-file ./examples/iszero/Prover.toml 2>/dev/null
```