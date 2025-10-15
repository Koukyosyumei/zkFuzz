pragma circom 2.0.0;

template Reward() {
    signal input inp;
    signal output out;
    var gwei = 10 ** 6;
    out <-- inp \ gwei;
    out * gwei === inp;
}

component main = Reward();
