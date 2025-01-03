pragma circom 2.1.6;
// https://github.com/RAKESH9494/Zkp-voting/blob/82f1cacb1cab6093f8aada16fd66e35acbd0fd4a/backend/zkp_files/circuit.circom#L4

include "../../include/circomlib/circuits/poseidon.circom"; // Poseidon hash function
include "../../include/circomlib/circuits/comparators.circom"; // Include IsZero comparator

template voting () {
    signal input ToWhomVote;
    signal input Age;

    signal output EligibleToVote;
    signal output Vote;


    component AgeAbove18 = GreaterEqThan(8);
    AgeAbove18.in[0] <== Age;
    AgeAbove18.in[1] <== 18;

    // 0 - Nota
    // 1 - A , 2- B, 3-C, 4- D
    component VoteIsMinRange = GreaterEqThan(8);
    VoteIsMinRange.in[0] <== ToWhomVote;
    VoteIsMinRange.in[1] <==  0;

    component VoteInMaxRange = LessEqThan(8);
    VoteInMaxRange.in[0] <== ToWhomVote;
    VoteInMaxRange.in[1] <== 4;


    signal VoteIsValid <== VoteInMaxRange.out * VoteIsMinRange.out;
    component VoteHash = Poseidon(1);
    VoteHash.inputs[0] <== ToWhomVote;

    EligibleToVote <== AgeAbove18.out * VoteIsValid;
    Vote <== VoteHash.out;

}

component main = voting();