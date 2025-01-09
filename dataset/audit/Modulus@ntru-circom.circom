include "../../include/circomlib/circuits/bitify.circom";
include "../../include/circomlib/circuits/comparators.circom";

//https://github.com/numtel/ntru-circom/blob/886418f47a7f34580e4446a7bdc6d6ef58f72e17/circuits/ntru.circom

// From https://pps-lab.com/blog/fhe_arithmetization/
template Modulus(p, n) {
  signal input x;
  signal q;
  signal output y;

  y <-- x % p;
  q <-- x \ p;
  x === q * p + y;

//   component ltP = LessThan(n);
//   ltP.in[0] <== p;
//   ltP.in[1] <== y;
//   ltP.out === 0;

  component ltQ = LessThan(n);
  ltQ.in[0] <== x;
  ltQ.in[1] <== q;
  ltQ.out === 0;
}

component main = Modulus(5, 6);