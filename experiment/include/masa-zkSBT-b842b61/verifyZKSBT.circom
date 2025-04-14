pragma circom 2.0.0;

include "../circomlib-cff5ab6/poseidon.circom";
include "./compare.circom";

// length = length of the data array
template verifyZKSBT(length) {
    // public
    signal input index;     // index of the value in the data array
    signal input root;      // root of the merkle tree
    signal input owner;     // address of the owner of the soulbound token
    signal input threshold; // threshold to compare with the value
    signal input operator;  // operator to compare the value with the threshold
    // 000: ==
    // 001: !=
    // 010: >
    // 011: >=
    // 100: <
    // 101: <=

    // private
    signal input value;
    signal input data[length];

    // true/false
    signal output out;

    // check that the owner is equal to the owner in the data
    assert(owner == data[0]);
    // check that the value is equal to the data at the index
    assert(value == data[index]);

    // check merckle tree root of data to be equal to root public input
    component merkleTree = Poseidon(length);
    for (var i = 0; i < length; i++) {
        merkleTree.inputs[i] <== data[i];
    }
    root === merkleTree.out;

    // Solution from
    // https://github.com/enricobottazzi/ZK-SBT/blob/b158f1a6ddcd098ee24b5674e0505f99586ad742/iden3-circuits/lib/query/query.circom

    // Verifies if the criterion met the field value
    component cmp = Compare();
    cmp.a <== value;
    cmp.b <== threshold;
    cmp.op <== operator;
    cmp.out === 1;

    // output
    out <== cmp.out;
}