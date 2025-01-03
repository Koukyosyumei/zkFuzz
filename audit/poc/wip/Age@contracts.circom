pragma circom 2.1.8;
// https://github.com/zkKYC/contracts/blob/main/circuits/age/age.circom

include "../../circomlib/circuits/comparators.circom";
include "../../circomlib/circuits/mimcsponge.circom";

template Hasher() {
    signal input val;

    signal output hashedValue;
   
    component hasher = MiMCSponge(1, 220, 1);
    hasher.ins[0] <== val;
    hasher.k <== 0;
    hashedValue <== hasher.outs[0];
}

// Computes MiMC([left, right])
template HashLeftRight() {
    signal input left;
    signal input right;
    signal output hash;

    component hasher = MiMCSponge(2, 220, 1);
    hasher.ins[0] <== left;
    hasher.ins[1] <== right;
    hasher.k <== 0;
    hash <== hasher.outs[0];
}

// if s == 0 returns [in[0], in[1]]
// if s == 1 returns [in[1], in[0]]
template DualMux() {
    signal input in[2];
    signal input s;
    signal output out[2];

    s * (1 - s) === 0;
    out[0] <== (in[1] - in[0])*s + in[0];
    out[1] <== (in[0] - in[1])*s + in[1];
}

// Verifies that merkle proof is correct for given merkle root and a leaf
// pathIndices input is an array of 0/1 selectors telling whether given pathElement is on the left or right side of merkle path
template MerkleTreeChecker(levels) {
    signal input leaf;
    signal input root;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    component selectors[levels];
    component hashers[levels];

    for (var i = 0; i < levels; i++) {
        selectors[i] = DualMux();
        selectors[i].in[0] <== i == 0 ? leaf : hashers[i - 1].hash;
        selectors[i].in[1] <== pathElements[i];
        selectors[i].s <== pathIndices[i];

        hashers[i] = HashLeftRight();
        hashers[i].left <== selectors[i].out[0];
        hashers[i].right <== selectors[i].out[1];
    }

    root === hashers[levels - 1].hash;
}

template Age(n, levels) {  
    signal input birthdayTimestamp;
    signal input root;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    signal input nowTimestamp;

    signal output out;

    signal adult <== 568036800;

    // possible vulnerability - fix later
    signal difference <== nowTimestamp - birthdayTimestamp;

    // 1. If in[0] < in[1]
    // 0. If in[0] > in[1]
    component lessThan = LessThan(n);
    lessThan.in[0] <== adult;
    lessThan.in[1] <== difference;

    // Checking that the difference is greater than adult
    lessThan.out === 1;

    component hasher = Hasher();
    hasher.val <== birthdayTimestamp;

    component tree = MerkleTreeChecker(levels);
    tree.leaf <== hasher.hashedValue; 
    tree.root <== root;
    for (var i = 0; i < levels; i++) {
        tree.pathElements[i] <== pathElements[i];
        tree.pathIndices[i] <== pathIndices[i];
    }

    out <== lessThan.out;
} 

component main {public [nowTimestamp, root]} = Age(48, 3);