pragma circom 2.0.0;

function get_two() {
    return 2;
}

template Callee2() {
    signal input x2[get_two()];
    signal input y2;
    signal output out2;

    out2 <== (x2[0] + x2[1]) * y2;
}

template Callee1() {
    signal input x1[2];
    signal output out1;
    component c = Callee0();

    c.x0 <== x1;
    out1 <== c.out0[1][0];
}

template Callee0() {
    signal input x0[2];
    signal output out0[get_two()][get_two()];

    out0[0] <== Callee()(x0[0], x0[1]);
    out0[1] <== Callee()(x0[1], x0[0]);
}

template Callee() {
    signal input a;
    signal input b;
    signal output c[get_two()];

    c[0] <== a + b;
    c[1] <== a * b;
}

template Main() {
    signal input x[2][2];
    signal output out;

    out <== Callee2()(x[0], Callee1()(x[1]));
}

component main = Main();