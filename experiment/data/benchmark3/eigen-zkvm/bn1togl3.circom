pragma circom 2.0.6;

include "../../include/circomlib-cff5ab6/bitify.circom";

template BN1toGL3() {
    signal input in;
    signal output out[3];

    component n2b = Num2Bits(255);

    n2b.in <== in;

    component b2n[3];

    for (var i=0; i<3; i++) {
        b2n[i] = Bits2Num(64);
        for (var j=0; j<64; j++) {
            b2n[i].in[j] <== n2b.out[64*i+j];
        }
        log(b2n[i].out);
        out[i] <== b2n[i].out;
    }
}

component main = BN1toGL3();