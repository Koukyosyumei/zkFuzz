pragma circom 2.0.6;

include "../../include/circomlib-cff5ab6/poseidon.circom";
include "../../include/circomlib-cff5ab6/comparators.circom";
include "../../include/circomlib-cff5ab6/mux1.circom";

template MerkleTreeInclusionProof(nLevels) {
    signal input leaf;
    signal input pathIndices[nLevels];
    signal input siblings[nLevels];

    signal output root;

    component poseidons[nLevels];
    component mux[nLevels];

    signal hashes[nLevels + 1];
    hashes[0] <== leaf;

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

        hashes[i + 1] <== poseidons[i].out;
    }

    root <== hashes[nLevels];
}

template VerifyTalent(numOfFields, nLevels) {
    // Public inputs
    signal input commits[numOfFields];
    signal input age;

    // Private inputs: in the order of name, age, countryCode 
    signal input values[numOfFields];
    signal input nonces[numOfFields];

    // For inclusion check
    signal input pathIndices[nLevels];
    signal input siblings[nLevels];
    signal input leaf;
    signal input root;
    
    component hashers[numOfFields];
    for (var i = 0; i < numOfFields; i++) {
        hashers[i] = Poseidon(2);
        hashers[i].inputs[0] <== values[i];
        hashers[i].inputs[1] <== nonces[i];
        commits[i] === hashers[i].out;
    }

    // CountryCode eligibility check
    component inclusionProof = MerkleTreeInclusionProof(nLevels);
    inclusionProof.leaf <== leaf;

    for (var i = 0; i < nLevels; i++) {
        inclusionProof.siblings[i] <== siblings[i];
        inclusionProof.pathIndices[i] <== pathIndices[i];
    }
    root === inclusionProof.root;

    // Age confirmation
    component ageGreaterEqThan = GreaterEqThan(32);
    ageGreaterEqThan.in[0] <== values[1];
    ageGreaterEqThan.in[1] <== age;

    ageGreaterEqThan.out === 1;

}

// We verify name, age and country eligibility
component main { public [ commits, age ] } = VerifyTalent(2, 1);//VerifyTalent(3, 8);

/*
║           ➡️ main.values[0] = 4
║           ➡️ main.siblings[0] = 21888242871839275222246405745257275088548364400416034343698204186575808495613
║           ➡️ main.root = 7276830708316375434465886777008938421833911662389086012012326314961634162769
║           ➡️ main.commits[0] = 1785831526463508266528679234189027565829465053240759806866653519448943759350
║           ➡️ main.age = 21888242871839275222246405745257275088548364400416034343698204186575808495601
║           ➡️ main.nonces[1] = 21888242871839275222246405745257275088548364400416034343698204186575808495545
║           ➡️ main.leaf = 21888242871839275222246405745257275088548364400416034343698204186575808495574
║           ➡️ main.nonces[0] = 21888242871839275222246405745257275088548364400416034343698204186575808495559
║           ➡️ main.values[1] = 1
║           ➡️ main.numOfFields = 2
║           ➡️ main.pathIndices[0] = 0
║           ➡️ main.commits[1] = 5047226990689914582574316590323994120736095986568334767478482783923265716750
║           ➡️ main.nLevels = 1
*/