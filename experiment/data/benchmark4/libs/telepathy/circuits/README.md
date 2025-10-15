# telepathy-circuits

All zkSNARKS for Telepathy are written in [Circom](https://docs.circom.io/), a battle-tested circuit compiler for Groth16 (used in Tornado Cash, etc).

This repository implements two major circuits which are used downstream in our lightclient implementations on chain:

-   **circuits/step.circom**: used for incrementing the light client to the next finalized header
-   **circuits/rotate.circom**: used for rotating the validator set to the next sync committee (every 27 hours)

## Requirements

You will need a machine with an Intel processor, lots of RAM and a large hard drive with swap enabled. To run the circuits, we recommend:

-   300+GB of memory
-   500+GB of space
-   32+ CPUs

In particular, instances such as AWS r5.8xlarge and r6a.8xlarge have worked well historically.

## Install

We follow the installation instructions described [here](https://hackmd.io/V-7Aal05Tiy-ozmzTGBYPA).

Circom witness generation can be buggy for large circuits--we recommend using the Circom 2.0.5 compiler or installing our own branch of [Circom](https://github.com/jtguibas/circom-stable) which resolves some of these issues.

## Build

To build the circuits, install a ptau file large enough (>128M) for our circuits at `/shared/powersOfTau28_hez_final_27.ptau`. You can use the existing ptau files from the [Hermez trusted setup ceremony](https://github.com/iden3/snarkjs#7-prepare-phase-2).

To build and run the circuits, observe the build scripts available in the `scripts` folder. Make sure to configure the enviroment variables at the top of the scripts correctly:

```
PHASE1=`realpath /shared/powersOfTau28_hez_final_27.ptau`
CIRCUITS_DIR=`realpath ../../circuits`
BUILD_DIR=`realpath ../../build`
OUTPUT_DIR=`realpath "$BUILD_DIR"/"$CIRCUIT_NAME"_cpp`
NODE_PATH=`realpath ~/node/node`
NODE_CMD="$NODE_PATH --trace-gc --trace-gc-ignore-scavenger --max-old-space-size=2048000 --initial-old-space-size=2048000 --no-global-gc-scheduling --no-incremental-marking --max-semi-space-size=1024 --initial-heap-size=2048000 --expose-gc"
SNARKJS_PATH=`realpath ~/snarkjs/cli.js`
RAPIDSNARK_PATH=`realpath ~/rapidsnark/build/prover`
```

To execute the build, simply run `bash build.sh` in the `step` or `rotate` folder. The build can take between a few hours and a day based on your machine.

## Benchmarks

These are old benchmarks, but should give a rough idea of how long it takes for the circuits to run.

|                    | Rotate      | Step        |
| ------------------ | ----------- | ----------- |
| # of Constraints   | 70M         | 21M         |
| Witness Generation | 124 seconds | 180 seconds |
| Proving Time       | 118 seconds | 60 seconds  |

## Acknowledgements

We rely on previous circuits implemented by close collaborators at Iden3 and 0xPARC in [circomlib](https://github.com/iden3/circomlib) and [circom-pairing](https://github.com/yi-sun/circom-pairing).

## TODO

Because we are using `jest` for tests in another project in this workspace, and `mocha` (which is used in this workspace) conflicts, we temporarily removed
`"@types/mocha": "^9.1.1"` from the dev dependencies and `"mocha": "^10.0.0"` from the dependencies to get our workspace to compile. We should either migrate our tests in this repo to jest or figure out how to get mocha and jest to play nice.
