pragma circom 2.0.0;

template Callee() {
    signal input x[2];
    signal output y1[2];
    signal output y2[2];

    y1[0] <== x[0] + x[1];
    y1[1] <== x[0] - x[1];
    y2[0] <== x[1] - x[0];
    y2[1] <== x[0] * x[1];
}

template Caller(N) {
    signal input a;
    signal input b;
    signal output out1[2];
    signal output out2[2];

    (out1, out2) <== Callee()([a, b]);
}

component main = Caller(2);
