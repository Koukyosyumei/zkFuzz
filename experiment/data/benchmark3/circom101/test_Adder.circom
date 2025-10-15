pragma circom 2.1.0;

// Finds the sum of an array of signals.
//
// Parameters:
// - n: length of `in`
//
// Inputs:
// - in: an array of `n` values
//
// Outputs:
// - out: sum of all values in `in`
template Adder(n) {
  signal input in[n];
  signal output out;

  var lc = 0;
  for (var i = 0; i < n; i++) {
    lc += in[i];
  }
  out <== lc;
}

component main = Adder(2);