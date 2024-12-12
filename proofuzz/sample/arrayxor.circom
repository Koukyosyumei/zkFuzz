pragma circom 2.0.0;

template ArrayXOR() {
    signal input a[2];
    signal input b[2];
    signal output out[2];
    for (var i = 0; i < 2; i++) {
        out[i] <-- a[i] ^ b[i];
    }
}

component main = ArrayXOR();