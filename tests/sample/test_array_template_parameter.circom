pragma circom 2.0.0;

template Main(N, C) {
    signal input x;
    signal output y;
    y <-- x + N * C[0] + C[1];
}

component main = Main(3, [1, 2]);