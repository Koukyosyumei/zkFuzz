pragma circom 2.0.0;

function nbits(a) {
    var n = 1;
    var r = 0;
    while (n-1<a) {
        r++;
        n *= 2;
    }
    return r;
}

template NBits(C) {
    signal input in;
    signal output out;

    out <== nbits(C) + in;
}

component main = NBits(5);
