include "../../circomlib/circuits/comparators.circom";
include "../../circomlib/circuits/gates.circom";
// https://github.com/kevinz917/zksnarks-library/blob/fe2a3b265d89e0a3a28e461547707f37eaf07f68/src/circuits/arrayContains/arrayContains.circom#L17

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

component main = BFS();