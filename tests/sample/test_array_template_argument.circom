pragma circom 2.0.0;

function CONSTANT_C() {
    return [0, 1, 2, 3, 4];
}

template Ark(t, C, r) {
    signal input in[t];
    signal output out[t];

    for (var i=0; i<t; i++) {
        out[i] <== in[i] + C[i + r];
    }
}

template Main() {
    signal input x[2];
    signal output y[2];

    var C[5] = CONSTANT_C();
    component A = Ark(2, C, 1);

    for (var i=0; i<2; i++) {
        A.in[i] <== x[i];
    }

    for (var i=0; i<2; i++) {
        y[i] <== A.out[i];
    }
}

component main = Main();