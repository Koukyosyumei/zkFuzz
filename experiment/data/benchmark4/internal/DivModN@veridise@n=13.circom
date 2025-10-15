pragma circom 2.0.0;

template DivModN(n) {
    signal input inp;
    signal output div;
    signal output mod;
    div <-- inp \ n;
    mod <-- inp % n;
    inp === div * n + mod;
}

component main = DivModN(13);
