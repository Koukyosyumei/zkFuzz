pragma circom 2.0.3;

include "../libs/darkforest-eth-9033eaf-fixed/range_proof.circom";

template RPTester() {
    signal input a;
    component rp = RangeProof(10);
    rp.in <== a;
    rp.max_abs_value <== 100;
}

component main = RPTester();