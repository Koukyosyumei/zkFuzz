pragma circom 2.0.0;

template Callee() {
    signal input a[3];
    signal output b[3];

    b[0] <== a[0] + 1;
    b[1] <== a[1] + 2;
    b[2] <== a[2] + 3;
}

template Main() {
    signal input x[3];
    signal input y[3];
    signal output out[2][3];

    out[0] <== Callee()(x);
    out[1] <== y;
}

component main = Main();