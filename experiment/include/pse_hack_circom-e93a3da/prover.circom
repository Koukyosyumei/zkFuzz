pragma circom 2.1.4;

include "../circomlib-cff5ab6/poseidon.circom";
include "../circomlib-cff5ab6/comparators.circom";

template get_commitment(n, m) {
  signal input ch1_a[n];
  signal input ch1_b[m];
  signal input ch2_a[n];
  signal input ch2_b[m];
  signal input ch3_a[n];
  signal input ch3_b[m];
  signal output hash1;
  signal output hash2;
  signal output hash3; 

  component hashed1 = Poseidon(n+m);
  component hashed2 = Poseidon(n+m);
  component hashed3 = Poseidon(n+m);
  // First Hash Calculation
  for (var i = 0; i < n; i++) {
    hashed1.inputs[i] <== ch1_a[i];
    hashed2.inputs[i] <== ch2_a[i];
    hashed3.inputs[i] <== ch3_a[i];
  }
  for (var i = 0; i < m; i++) {
    hashed1.inputs[n+i] <== ch1_b[i];
    hashed2.inputs[n+i] <== ch2_b[i];
    hashed3.inputs[n+i] <== ch3_b[i];
  }
  hash1 <== hashed1.out;
  hash2 <== hashed2.out;
  hash3 <== hashed3.out;

}

// component main = get_commitment(2, 3);