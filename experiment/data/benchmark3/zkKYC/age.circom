pragma circom 2.1.8;

include "../../include/circomlib-cff5ab6/comparators.circom";
include "../../include/zkKYC/hasher.circom";
include "../../include/zkKYC/merkleTree.circom";

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

component main {public [nowTimestamp, root]} = Age(48, 1);//Age(48, 3);
