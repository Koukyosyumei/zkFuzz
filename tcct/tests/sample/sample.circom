pragma circom 2.0.0;

include "../../../benchmark/circomlib/circuits/bitify.circom";
include "../../../benchmark/circomlib/circuits/comparators.circom";
include "../../../benchmark/circomlib/circuits/mimcsponge.circom";

function GetGCD(x, y){
    var temp;

    // Pass by reference
    var X;
    var Y;
    X = x;
    Y = y;

    while (Y != 0){
        temp = X % Y;
        X = Y;
        Y = temp;
    }
    return X;
}

template GCD() {
    signal input in[2];
    signal output out;

    out <-- GetGCD(in[0], in[1]);
}

template Main() {
    signal input x;
    signal input y;
    signal output z;

    component gcd = GCD();
    gcd.in[0] <== x;
    gcd.in[1] <== y;
    z <== gcd.out;

    component gcdGreater = GreaterThan(64);
    gcd.out ==> gcdGreater.in[0];
    gcdGreater.in[1] <== 1;
    gcdGreater.out === 1;
}

component main = Main();