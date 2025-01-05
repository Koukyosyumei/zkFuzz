pragma circom 2.0.6;

include "../../circomlib/circuits/bitify.circom";

// out = remainder of the (in + 16*p) by p
template GLNorm() {
    signal input in;
    signal output out;

    var p=0xFFFFFFFF00000001;
    signal k <-- (in + 16*p)\p;
    out <-- (in+16*p) - k*p;

    component n2bK = Num2Bits(10);
    component n2bO = Num2Bits(64);

    n2bK.in <== k;
    n2bO.in <== out;

    (in+16*p) === k*p + out;
}

component main = GLNorm();