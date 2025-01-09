pragma circom 2.1.6;
        
include "../include/circomlib/circuits/comparators.circom";

template Main() {
    signal input x;
    signal input y;
    signal output z;

    component c = LessThan(8);
    c.in[0] <== x;
    c.in[1] <== 255;

    z <== c.out;
}

component main = Main();