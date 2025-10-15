pragma circom 2.1.0;

include "../../include/circomlib-cff5ab6/comparators.circom";

// Returns a boolean indicating that an array is sorted (ascending).
//
// Parameters:
// - n: length of `in`
// - b: max number of bits in the values of `in`
//
// Inputs:
// - in: an array of `n` `b`-bit values
//
// Outputs:
// - out: a boolean indicating whether the array is sorted
template IsSorted(n, b) {
  signal input in[n];
  signal output out;

  // accumulator for in[i-1] < in[1] checks
  var acc = 0;
  for (var i = 1; i < n; i++) {
    var isLessThan = LessEqThan(b)([in[i-1], in[i]]);
    acc += isLessThan;
  }

  // note that technically it is possible for `acc` to overflow
  // and wrap back to 0, however, that is unlikely to happen given
  // how large the prime-field is and we would need that many components
  // to be able to overflow
  signal outs <== acc;
  var outsIsZero = IsZero()(outs);
  out <== 1 - outsIsZero;
}

component main = IsSorted(2, 16);