pragma circom 2.0.6;

include "../../circomlib-cff5ab6/comparators.circom";
include "../../circomlib-cff5ab6/switcher.circom";

// 无符号转有符号
template Sign() {
    signal input in;
    signal output out;
    signal output sign;
    component lt = LessThan(252);
    component sw = Switcher();

    lt.in[0] <== in;
    lt.in[1] <== 0;

    sw.sel <== lt.out;
    sw.L <== in;
    sw.R <== -in;

    out <== sw.outL;
    sign <== lt.out;
}

// 有符号转无符号
template UnSign() {
    signal input in;
    signal input sign;
    signal output out;

    out <== (1-2*sign) * in;
}