include "../../../benchmark/circomlib/circuits/bitify.circom";
include "../../../benchmark/circomlib/circuits/comparators.circom";

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

/*
╔══════════════════════════════════════════════════════════════╗
║🚨 Counter Example:                                           ║
║    🔥 UnderConstrained (Non-Deterministic) 🔥
║    🔍 Assignment Details:
║           ➡️ main.x = 3
║           ➡️ main.q = 1
║           ➡️ main.y = 21888242871839275222246405745257275088548364400416034343698204186575808495615
║           ➡️ main.ltQ.in[0] = 3
║           ➡️ main.ltQ.n2b.out[2] = 0
║           ➡️ main.ltQ.n2b.out[3] = 0
║           ➡️ main.ltQ.in[1] = 1
║           ➡️ main.ltQ.n2b.out[0] = 0
║           ➡️ main.ltQ.n2b.out[5] = 0
║           ➡️ main.ltQ.n2b.out[1] = 1
║           ➡️ main.ltQ.n2b.in = 66
║           ➡️ main.ltQ.n2b.out[6] = 1
║           ➡️ main.ltQ.out = 0
║           ➡️ main.ltQ.n2b.out[4] = 0
╚══════════════════════════════════════════════════════════════╝
*/