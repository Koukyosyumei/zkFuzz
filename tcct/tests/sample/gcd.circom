pragma circom 2.0.0;

include "../../../benchmark/circomlib/circuits/bitify.circom";
include "../../../benchmark/circomlib/circuits/comparators.circom";
include "../../../benchmark/circomlib/circuits/mimcsponge.circom";

function isPrime(n)
{
    // Corner case
    if (n <= 1) {return 0;}
  
    // Check from 2 to n \ 2
    for (var i = 2; i < (n >> 1); i++) {
        if ((n % i == 0)&(n != i)) {return 0;}
    }
    return 1;
}

template Prime() {
    signal input in;
    signal output out;

    out <-- isPrime(in);
}

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

template Spawn() {
    signal input x;
    signal input y;
    signal output pub;

    /* check abs(x), abs(y) <= 2^31 */
    component n2bx = Num2Bits(32);
    n2bx.in <== x + (1 << 31);
    component n2by = Num2Bits(32);
    n2by.in <== y + (1 << 31);

    // It has to be within a Euclidean distance of 64 to the origin (0, 0)
    component compUpper = LessThan(64);
    signal xSq;
    signal ySq;
    xSq <== x * x;
    ySq <== y * y;
    compUpper.in[0] <== xSq + ySq;
    compUpper.in[1] <== 64*64;
    compUpper.out === 1;

    // Its Euclidean distance to the origin (0,0) has to be more than 32.
    component compLower = GreaterThan(64);
    compLower.in[0] <== xSq + ySq;
    compLower.in[1] <== 32*32;
    compLower.out === 1;

    // GCD(x,y) must be greater than 1 
    component gcd = GCD();
    gcd.in[0] <== x;
    gcd.in[1] <== y;

    component gcdGreater = GreaterThan(64);
    gcd.out ==> gcdGreater.in[0];
    gcdGreater.in[1] <== 1;

    gcdGreater.out === 1;

    //and must not be a prime number.
    component prime = Prime();
    gcd.out ==> prime.in;

    prime.out === 0;

    //generate hash output
    component mimc = MiMCSponge(2, 220, 1);

    mimc.ins[0] <== x;
    mimc.ins[1] <== y;
    mimc.k <== 0;

    pub <== mimc.outs[0];
}

component main = Spawn();