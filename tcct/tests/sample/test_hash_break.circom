pragma circom 2.0.0;

template Main() {
    signal input x;
    signal input y;
    signal input z;
    signal output out;

    out <== x * y;
    y === 987654321;
    z === 123456789;
}

component main = Main();
