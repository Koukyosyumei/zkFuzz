pragma circom 2.0.0;

function get_dim() {
    return 3;
}

template Callee() {
    signal input x[2];
    signal output y[3];
    y[0] <== x[0] + x[1];
    y[1] <== x[0] - x[1];
    y[2] <== x[0] * x[1];
}

template Caller(N) {
    signal input a;
    signal input b;

    component c1 = Callee();
    c1.x <== [a, a];
    component c2 = Callee();
    c2.x <== [a, b];
    component c3 = Callee();
    c3.x <== [b, a];
    component c4 = Callee();
    c4.x <== [b, b];

    signal output out[2][2][get_dim()] <== [[c1.y, c2.y], [c3.y, c4.y]];
}

component main = Caller(2);
