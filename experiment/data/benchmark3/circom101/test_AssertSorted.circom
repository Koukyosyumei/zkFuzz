pragma circom 2.1.0;

include "../../include/circomlib-cff5ab6/comparators.circom";

// Asserts that an array is sorted (ascending).
//
// Parameters:
// - n: length of `in`
// - b: max number of bits in the values of `in`
//
// Inputs:
// - in: an array of `n` `b`-bit values
template AssertSorted(n, b) {
  signal input in[n];

  // accumulator for in[i-1] < in[1] checks
  var acc = 0;
  for (var i = 1; i < n; i++) {
    var isLessThan = LessEqThan(b)([in[i-1], in[i]]);
    acc += isLessThan;
  }

  acc === n - 1;
}

component main = AssertSorted(2, 16);