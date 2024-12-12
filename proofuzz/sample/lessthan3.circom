pragma circom 2.0.0;

include "../benchmark/circomlib/circuits/comparators.circom";

template VulnerableLessThan() {
    signal input a;
    signal input b;
    signal output out;

    // Assume a and b are 3-bit integers
    component lt = LessThan(3) ;
    lt.in[0] <== a;
    lt.in[1] <== b;
    out <== lt.out;
}

component main = VulnerableLessThan();
