pragma circom 2.0.0;

include "../include/circomlib/circuits/comparators.circom";

template CheckIsEuql() {
    signal input x;
    signal input y;
    signal flag;

    component c = IsEqual();
    c.in[0] <== x;
    c.in[1] <== y;

    flag <-- c.out;
    flag === 1;
}

component main = CheckIsEuql();