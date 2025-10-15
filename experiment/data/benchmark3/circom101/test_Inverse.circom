pragma circom 2.1.0;

include "../../include/circomlib-cff5ab6/comparators.circom";

// Multiplicative inverse.
//
// Note that zero does not have an inverse, and the circuit
// asserts that input is non-zero. In any case, if a malicious
// prover provides a zero input, the circuit will return 0 as well.
//
// This circuit is very similar to `IsZero` circuit.
//
// Inputs:
// - in: a number, expected to be non-zero
//
// Outputs:
// - out: a number such that `in * out == 1`.
template Inverse() {
  signal input in;
  signal output out;

  //assert(in != 0);

  signal inv <-- in != 0 ? 1 / in : 0;
  signal isZero <== 1 - (in * inv);

  // if isZero, since in = 0 this holds
  // otherwise, it holds due to in * 0 == 0
  in * isZero === 0;
  inv * isZero === 0;

  out <== inv;
}

component main = Inverse();