pragma circom 2.0.0;

template Reward() {
    signal input x;
    signal input y;
    signal output z;

    z <-- x \ y;
    z * y === x;
}

component main = Reward();