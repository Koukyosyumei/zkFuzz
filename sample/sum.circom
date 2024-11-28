pragma circom 2.0.0;

template Sum() {
    signal input in[3];
    signal output out;
    var tmp = 0;
    for (var i = 0; i < 3; i++) {
        tmp += in[i];
    }
    out <== tmp;
}

component main = Sum();