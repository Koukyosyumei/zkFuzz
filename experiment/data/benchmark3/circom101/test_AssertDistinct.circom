pragma circom 2.1.0;

include "../../include/circomlib-cff5ab6/comparators.circom";

// Asserts that values in an array are unique.
//
// Parameters:
// - n: length of `in`
//
// Inputs:
// - in: an array of `n` values
template AssertDistinct(n) {
  signal input in[n];

  var acc = 0;
  for (var i = 0; i < n-1; i++){
    for (var j = i+1; j < n; j++){
      var eq = IsEqual()([in[i], in[j]]);
      acc += 1 - eq;
    }
  }

  acc === n * (n - 1) / 2;
}

component main = AssertDistinct(2);