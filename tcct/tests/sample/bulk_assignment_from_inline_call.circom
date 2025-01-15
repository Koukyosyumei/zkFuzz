pragma circom 2.0.0;

function get_two() {
    return 2;
}

template Callee() {
    signal input x0[2];
    signal output out0[get_two()];
    out0[0] <== x0[0] + x0[1];
    out0[1] <== x0[0] - x0[1];
}

template Main() {
    signal input x[2];
    signal output out[2];
    out <== Callee()(x);
}

component main = Main();