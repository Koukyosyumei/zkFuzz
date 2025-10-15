pragma circom 2.1.4;

include "../circomlib-cff5ab6/comparators.circom";
include "../circomlib-cff5ab6/gates.circom";

// searches whether a grid can be traversed from point A to point B
// applications: prove that a player has traversed from point 1 to point b without revealing location
// TODO: Copy code from main.circom to here
// TODO: Figure out better compilation workflow

template BFS() {
   signal input cards[5]; // Each 2..14
   signal input number; // 1 or 0
   signal output out; // 1 or 0

   var sum; // signals are immutable

   for(var i=0; i<5; i++){
     sum = sum + cards[i];
   }

   component eq = IsEqual();
   eq.in[0] <-- sum;
   eq.in[1] <-- number;

   out <-- eq.out;
   out === 1;
}

// component main = BFS();