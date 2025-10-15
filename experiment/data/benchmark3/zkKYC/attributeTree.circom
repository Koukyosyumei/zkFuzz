pragma circom 2.1.8;

include "../../include/zkKYC/hasher.circom";
include "../../include/zkKYC/merkleTree.circom";

template AttributeTree(levels) {
    signal input root;
    signal input value;
    signal input pathElements[levels];
    signal input pathIndices[levels];
 
    component hasher = Hasher();
    hasher.val <== value;

    component tree = MerkleTreeChecker(levels);
    tree.leaf <== hasher.hashedValue; 
    tree.root <== root;
    for (var i = 0; i < levels; i++) {
        tree.pathElements[i] <== pathElements[i];
        tree.pathIndices[i] <== pathIndices[i];
    }
}

component main {public [root, value]} = AttributeTree(1);