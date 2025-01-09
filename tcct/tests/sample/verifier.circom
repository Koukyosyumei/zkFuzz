pragma circom 2.0.0;

template Verifier() {
    signal input inp;
    //signal intermediate;
    //intermediate <== inp + 1;
    //intermediate * 2 === 1;
    inp * 2 === 1;
}

component main = Verifier();