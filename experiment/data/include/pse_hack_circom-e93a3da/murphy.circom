pragma circom 2.1.4;

include "../circomlib-cff5ab6/mux2.circom";
include "../circomlib-cff5ab6/bitify.circom";
include "../circomlib-cff5ab6/poseidon.circom";
include "../circomlib-cff5ab6/comparators.circom";

template ThreePassVerification(n, m) {
  signal input a[n];
  signal input b[m];
  signal input c[n];
  signal input d[m];
  signal input ch;
  signal input c1;
  signal input c2;
  signal input c3;

  // Outputs
  signal output isValid;       // Verification result (1 for valid, 0 for invalid)

  // Internal signals
  signal G_result1;
  signal G_result2;

  // Hash components
  component hash1 = Poseidon(n+m);
  component hash2 = Poseidon(n+m);

  // First Hash Calculation
  for (var i = 0; i < n; i++) {
      hash1.inputs[i] <== a[i];
      hash2.inputs[i] <== c[i];
  }
  for (var i = 0; i < m; i++) {
      hash1.inputs[n+i] <== b[i];
      hash2.inputs[n+i] <== d[i];
  }

  G_result1 <== hash1.out;
  G_result2 <== hash2.out;

  // Comparison using EqualComparator
  component isEqual = IsEqual();
  isEqual.in[0] <== G_result1;
  isEqual.in[1] <== G_result2;

  // Set isValid based on comparison result

  signal compare[3][2];
  component commit_check[3][2];
  commit_check[0][0]=IsEqual();
  commit_check[0][0].in[0] <== c2;
  commit_check[0][0].in[1] <== G_result1;
  compare[0][0] <== commit_check[0][0].out;
  commit_check[0][1]=IsEqual();
  commit_check[0][1].in[0] <== c3;
  commit_check[0][1].in[1] <== G_result2;  
    compare[0][1] <== commit_check[0][1].out;
    commit_check[1][0]=IsEqual();
    commit_check[1][0].in[0] <== c1;
    commit_check[1][0].in[1] <== G_result1;
    compare[1][0] <== commit_check[1][0].out;
    commit_check[1][1]=IsEqual();
    commit_check[1][1].in[0] <== c3;
    commit_check[1][1].in[1] <== G_result2;
    compare[1][1] <== commit_check[1][1].out;
    commit_check[2][0]=IsEqual();
    commit_check[2][0].in[0] <== c1;
    commit_check[2][0].in[1] <== G_result1;
    compare[2][0] <== commit_check[2][0].out;
    commit_check[2][1]=IsEqual();
    commit_check[2][1].in[0] <== c2;
    commit_check[2][1].in[1] <== G_result2;
    compare[2][1] <== commit_check[2][1].out;

  //isValid <== isEqual.out;


  signal mux_inputs[4];
    mux_inputs[0] <== compare[0][0]*compare[0][1];
    mux_inputs[1] <== compare[1][0]*compare[1][1];
    mux_inputs[2] <== compare[2][0]*compare[2][1];
    mux_inputs[3] <== 0;

  component n2b=Num2Bits(2);
  n2b.in <== ch;
  signal ch_bits[2];
  ch_bits<== n2b.out;

  component mux=Mux2();
    mux.c <== mux_inputs;
    mux.s <== ch_bits;
    isValid <== mux.out;
}
    
// component main = ThreePassVerification(2, 3);


/* INPUT = {
  "a": [1, 2, 2],
  "b": [4, 5],
  "c": [1, 2, 3],
  "d": [4, 5],
  "ch": 1,
  "c1": 1,
    "c2": 2,
    "c3": 3
} */