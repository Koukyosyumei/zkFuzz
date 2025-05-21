use std::collections::HashMap;

use ark_bn254::{Bn254, Fr};
use ark_circom::{CircomBuilder, CircomConfig};
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

    let circuit = builder.build()?;
    println!("{:?}", circuit.witness);
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
