pragma circom 2.0.0;

template Reward() {
    signal input inp;
    signal output out;
    var gwei = 2;
    log(gwei);
    out <-- inp \ gwei;
    out * gwei === inp;
}

component main = Reward();