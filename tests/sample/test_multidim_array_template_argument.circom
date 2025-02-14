pragma circom 2.0.0;

function CONSTANT_M() {
    return [[0, 1], [2, 3]];
}

template Ark(t, C) {
    signal input in;
    signal output out;
    out <== C[t][t] + in;
}

template Main() {
    signal input x;
    signal output y;

    var M[2][2] = CONSTANT_M();
    component A = Ark(1, M);

    A.in <== x;
    y <== A.out;
}

component main = Main();