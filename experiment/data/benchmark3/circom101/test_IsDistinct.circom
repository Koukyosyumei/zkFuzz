pragma circom 2.1.0;

include "../../include/circomlib-cff5ab6/comparators.circom";

// Returns 1 if all values are distinct in a given array.
//
// Parameters:
// - n: length of `in`
//
// Inputs:
// - in: an array of `n` values
//
// Outputs:
// - out: 1 if all values are distinct
template IsDistinct(n) {
  signal input in[n];
  signal output out;

  var acc = 0;
  for (var i = 0; i < n-1; i++){
    for (var j = i+1; j < n; j++){
      var eq = IsEqual()([in[i], in[j]]);
      acc += eq;
    }
  }

  // note that technically it is possible for `acc` to overflow
  // and wrap back to 0, however, that is unlikely to happen given
  // how large the prime-field is and we would need that many components
  // to be able to overflow
  signal outs <== acc;
  out <== IsZero()(outs);
}


component main = IsDistinct(2);