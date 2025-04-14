pragma circom 2.1.6;

include "../circomlib-cff5ab6/poseidon.circom";
include "../circomlib-cff5ab6/mux1.circom";

template MerkleTreeInclusionProof(nLevels) {
    signal input leaf;
    signal input pathIndices[nLevels];
    signal input siblings[nLevels];
    signal input root;
    signal output out;

    component poseidons[nLevels];
    component mux[nLevels];

    component poseidon = Poseidon(2);
    poseidon.inputs[0] <== leaf;
    poseidon.inputs[1] <== 0;
    // log("hashed leaf", poseidon.out);

    signal hashes[nLevels + 1];
    hashes[0] <== poseidon.out;

    for (var i = 0; i < nLevels; i++) {
        pathIndices[i] * (1 - pathIndices[i]) === 0;

        poseidons[i] = Poseidon(2);
        mux[i] = MultiMux1(2);

        mux[i].c[0][0] <== hashes[i];
        mux[i].c[0][1] <== siblings[i];

        mux[i].c[1][0] <== siblings[i];
        mux[i].c[1][1] <== hashes[i];

        mux[i].s <== pathIndices[i];

        poseidons[i].inputs[0] <== mux[i].out[0];
        poseidons[i].inputs[1] <== mux[i].out[1];

        // log("hashed level", i);
        // log(poseidons[i].out);
        hashes[i + 1] <== poseidons[i].out;
    }

    var root_assert;
    // log("root",root);
    // log("hashed root",hashes[nLevels]);
    root_assert = root == hashes[nLevels] ? 1 : 0;

    out <-- root_assert;

    signal dummy;
    dummy <== leaf * root;

}
