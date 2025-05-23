use std::{collections::HashMap, fs, path::PathBuf};

use ark_bn254::Fr;
use ark_circom::circom::R1CS;
use ark_circom::{CircomBuilder, CircomCircuit, CircomConfig};
use ark_ff::PrimeField;
use ark_relations::r1cs::ConstraintSystem;
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, SynthesisError, Variable,
};
use color_eyre::Result;
use num_bigint::BigInt;
use rand::{rngs::StdRng, Rng, SeedableRng};
use wasm_mutate::{ErrorKind, WasmMutate};
//use wasmparser::Validator;

/// Maximum number of mutation generations per fuzz loop
const MAX_GENERATIONS: usize = 10;

/// Runs the original and mutated circuits on input `x` and returns
/// (witness, cs.is_satisfied)
fn run_circuit(
    wasm_bytes: &[u8],
    r1cs_path: &str,
    inputs: &HashMap<String, BigInt>,
) -> Result<(Vec<Fr>, bool)> {
    // write mutated wasm to temp file so CircomBuilder can pick it up
    let tmp_wasm = tempfile::NamedTempFile::new()?;
    fs::write(tmp_wasm.path(), wasm_bytes)?;
    let cfg = CircomConfig::<Fr>::new(
        tmp_wasm.path().to_string_lossy().into_owned(),
        r1cs_path.to_string(),
    )?;
    let mut builder = CircomBuilder::new(cfg);
    for (k, v) in inputs {
        builder.push_input(k, v.clone());
    }
    let mut circuit = builder.build()?;
    let witness: Vec<Fr> = circuit
        .witness
        .clone()
        .ok_or("No witness generated")
        .unwrap()
        .into_iter()
        //.map(|bi| Fr::from_bigint(bi.to_bigint().unwrap()).unwrap())
        .collect();

    let cs: ConstraintSystemRef<Fr> = ConstraintSystem::new_ref();
    circuit.generate_constraints(cs.clone())?;
    let sat = cs.is_satisfied()?;
    Ok((witness, sat))
}

/// The core fuzz loop for TCCT detection
fn fuzz_tcct(
    orig_wasm_path: &str,
    r1cs_path: &str,
    mut inputs: HashMap<String, BigInt>,
) -> Result<()> {
    // Load original wasm
    let orig_wasm_bytes = fs::read(orig_wasm_path)?;
    // Initialize RNG (deterministic seed for reproducibility)
    let mut rng = StdRng::seed_from_u64(0xDEADBEEF);

    for gen in 1..=MAX_GENERATIONS {
        // 1. Generate input x (could be random; here we reuse or mutate `inputs`)
        //    e.g. randomly flip a bit of one of the BigInts
        if gen > 1 {
            // example mutation: randomize the one named "in"
            if let Some(v) = inputs.get_mut("in") {
                let flip: u64 = rng.gen();
                *v += BigInt::from(flip % 5);
            }
        }

        let mut mutator = WasmMutate::default();
        mutator.fuel(10);
        mutator.seed(gen.try_into().unwrap());

        match mutator.run(&orig_wasm_bytes) {
            Ok(it) => {
                for mutated in it.into_iter().take(1000) {
                    // Down here is the validation for the correct mutation
                    let mutated_wasm_bytes = mutated.unwrap();

                    // 3. Execute both programs
                    let (z, y_sat) = run_circuit(&orig_wasm_bytes, r1cs_path, &inputs)?;
                    let (z_p, y_p_sat) = run_circuit(&mutated_wasm_bytes, r1cs_path, &inputs)?;

                    // 4. Check for over-constrained: original rejects a valid-looking witness
                    if y_sat && !y_sat {
                        println!("Generation {}: Over-Constrained Problem detected.", gen);
                        break;
                    }

                    // 5. Check for under-constrained:
                    //    both satisfy, but outputs differ, and mutated one satisfies C
                    if y_sat && y_p_sat && z != z_p {
                        println!("Generation {}: Under-Constrained Problem detected.", gen);
                        break;
                    }
                }
            }
            Err(e) => {}
        }
        println!("pass");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // initial inputs
    let mut inputs = HashMap::new();
    inputs.insert("in".to_string(), BigInt::from(3));

    fuzz_tcct(
        "./examples/test_vuln_iszero.wasm",
        "./examples/test_vuln_iszero.r1cs",
        inputs,
    )?;
    Ok(())
}
