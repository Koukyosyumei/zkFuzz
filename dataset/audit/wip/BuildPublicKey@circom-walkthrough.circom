// LICENSE: MIT
pragma circom 2.1.6;
// https://github.com/ZKDL-Camp/circom-walkthrough/blob/main/circuits/templates/BuildPublicKey.circom

include "../../include/circomlib/circuits/poseidon.circom";

template BuildPublicKey() {
    signal output publicKey;

    signal input sk_i;

    component hasher1 = Poseidon(1);

    sk_i ==> hasher1.inputs[0];

    publicKey <-- hasher1.out;
}

component main = BuildPublicKey();