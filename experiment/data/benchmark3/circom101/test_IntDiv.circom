pragma circom 2.1.0;

include "../../include/circomlib-cff5ab6/comparators.circom";

// Integer division.
//
// Parameters:
// - n: number of bits for the numbers
//
// Inputs:
// - in: 2 numbers as [dividend, divisor]
//
// Outputs:
// - out: `in[0] / in[1]`
template IntDiv(n) {
  signal input in[2];
  signal output out;

  // divisor must be non-zero
  signal is_non_zero <== IsZero()(in[1]);
  0 === is_non_zero;

  // compute the quotient and remainder outside the circuit
  var quot_hint = in[0] \ in[1];
  var rem_hint = in[0] % in[1];
  signal quot <-- quot_hint;
  signal rem <-- rem_hint;

  // contrain the division operation
  // in[0] / in[1] is defined as the unique pair (q, r) s.t.
  // in[0] = in[1] * q + r and 0 <= r < |in[1]|
  in[0] === quot * in[1] + rem;

  // OPTIONAL: quot edge case is when `rem = 0` and `in[1] = 1`
  // signal quot_is_valid <== LessEqThan(n)([quot, in[0]]);
  // 1 === quot_is_valid;

  signal rem_is_valid <== LessThan(n)([rem, in[1]]);
  1 === rem_is_valid;

  out <== quot;
}

component main = IntDiv(16);