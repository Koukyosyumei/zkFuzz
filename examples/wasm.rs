use std::collections::HashMap;

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

fn executor(
    wasm_path: String,
    r1cs_path: String,
    inputs: HashMap<String, BigInt>,
) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = CircomConfig::<Fr>::new(wasm_path, r1cs_path)?;

    let mut builder = CircomBuilder::new(cfg);
    for (k, v) in inputs.iter() {
        builder.push_input(k, v.clone());
    }

    let mut circuit = builder.build()?;
    let mut mutated_circuit = circuit.clone();
    let mut mutated_witness = circuit.witness.clone().unwrap();
    mutated_witness[1] = 1.into();

    println!("{:?}", circuit.witness);
    println!("{:?}", mutated_witness);

    let cs = ConstraintSystem::<Fr>::new_ref();
    circuit.generate_constraints(cs.clone()).unwrap();
    println!("{:}", cs.is_satisfied().unwrap());

    let mutated_cs = ConstraintSystem::<Fr>::new_ref();
    mutated_circuit.witness = Some(mutated_witness);
    mutated_circuit
        .generate_constraints(mutated_cs.clone())
        .unwrap();
    println!("{:}", mutated_cs.is_satisfied().unwrap());

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut inputs = HashMap::new();
    inputs.insert("in".to_string(), BigInt::from(3));

    executor(
        "./examples/test_vuln_iszero.wasm".to_string(),
        "./examples/test_vuln_iszero.r1cs".to_string(),
        inputs,
    );
}
