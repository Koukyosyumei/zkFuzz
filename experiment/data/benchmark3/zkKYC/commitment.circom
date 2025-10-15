pragma circom 2.1.8;

include "../../include/circomlib-cff5ab6/mimcsponge.circom";
include "../../include/zkKYC/merkleTree.circom";

template CommitmentHasher() {
    signal input addr;
    signal input secret;

    signal output commitment;
   
    component hasher = MiMCSponge(2, 220, 1);
    hasher.ins[0] <== addr;
    hasher.ins[1] <== secret;
    hasher.k <== 0;
    commitment <== hasher.outs[0];
}

// Verifies that commitment that corresponds to given secret and nullifier is included in the merkle tree of deposits
template Commitment(levels) {
    signal input root;
    signal input secret;
    signal input addr;
    signal input pathElements[levels];
    signal input pathIndices[levels];
 
    component hasher = CommitmentHasher();
    hasher.addr <== addr;
    hasher.secret <== secret;

    component tree = MerkleTreeChecker(levels);
    tree.leaf <== hasher.commitment; 
    tree.root <== root;
    for (var i = 0; i < levels; i++) {
        tree.pathElements[i] <== pathElements[i];
        tree.pathIndices[i] <== pathIndices[i];
    }
}

component main {public [root]} = Commitment(1);