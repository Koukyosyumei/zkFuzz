pragma circom 2.0.0;

include "function.circom";

template Foo() {
    signal input in;
    signal output out;

    out <== isNegative(in);
}
