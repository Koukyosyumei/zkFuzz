pragma circom 2.1.8;

include "../../include/circomlib/circuits/comparators.circom";

template Main() {
    signal input x;
    component lt1 = LessThan(8);
    component lt2 = LessThan(8);

    //lt1.in[0] <== 47;
    //lt1.in[1] <== x;
    //lt1.out === 1;

    //lt2.in[0] <== x;
    //lt2.in[1] <== 58;
    //lt2.out === 1;
}

component main = Main();