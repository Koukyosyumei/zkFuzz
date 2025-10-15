pragma circom 2.1.0;

// Multiply `n` numbers.
//
// Note that the multiplication result, if greater than the prime modulus, will overflow and wrap back around!
//
// Parameters:
// - n: how many numbers there are
//
// Inputs:
// - in: `n` numbers
//
// Outputs:
// - out: `in[0] * in[1] * ... * in[n-1]`
template Multiplier(n) {
  assert(n > 1);
  signal input in[n];
  signal output out;

  signal inner[n-1];
  inner[0] <== in[0] * in[1];
  for (var i = 0; i < n - 2; i++) {
    inner[i+1] <== inner[i] * in[i+2];
  }

  out <== inner[n-2]; 
}

component main = Multiplier(2);