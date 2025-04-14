pragma circom 2.1.6;

template logBytes() {
    signal input in[8];
    log(in[0], in[1], in[2], in[3], in[4], in[5], in[6], in[7]);
}

template logState() {
    signal input in[12];
    log(in[0], in[1], in[2], in[3], in[4], in[5], in[6], in[7], in[8], in[9], in[10], in[11]);
}

template Num2BitArrays(nLimbs, nBits) {
    signal input in;
    signal output out[nLimbs][nBits];
    var lc1 = 0;
    var e2 = 1;
    for (var i = 0; i < nLimbs; i++) {
        for (var j = 0; j < nBits; j++) {
            out[i][j] <-- (in >> (nBits*i + j)) & 1;
            out[i][j] * (1 - out[i][j]) === 0;
            lc1 += out[i][j] * e2;
            e2 = e2 + e2;
        }
    }

    lc1 === in;
}

template BitArrays2Num(nLimbs, nBits) {
    signal input in[nLimbs][nBits];
    signal output out;
    var lc1 = 0;
    var e2 = 1;
    for (var i = 0; i < nLimbs; i++) {
        for (var j = 0; j < nBits; j++) {
            lc1 += in[i][j] * e2;
            e2 = e2 + e2;
        }
    }

    out <== lc1;
}

template NegateBitArrays(nLimbs, nBits) {
    signal input in[nLimbs][nBits];
    signal output out[nLimbs][nBits];
    for (var i = 0; i < nLimbs; i++) {
        for (var j = 0; j < nBits; j++) {
            out[i][j] <== 1 - in[i][j];
        }
    }
}

template RotateBitArrayLeft(nBits, k) {
    signal input in[nBits];
    signal output out[nBits];
    for (var i = 0; i < nBits; i++) {
        out[i] <== in[(i - k + nBits) % nBits];
    }
}

template RotateBitArraysLeft(nLimbs,nBits, k) {
    signal input in[nLimbs][nBits];
    signal output out[nLimbs][nBits];
    for (var i = 0; i < nLimbs; i++) {
        out[i] <== RotateBitArrayLeft(nBits, k)(in[i]);
    }
}

template AND3BitArrays(nLimbs, nBits) {
    // Assumes all are bit-arrays
    signal input in1[nLimbs][nBits];
    signal input in2[nLimbs][nBits];
    signal input in3[nLimbs][nBits];
    signal output out[nLimbs][nBits];

    signal tmp[nLimbs][nBits];
    for (var i = 0; i < nLimbs; i++) {
        for (var j = 0; j < nBits; j++) {
            tmp[i][j] <== in1[i][j] * in2[i][j];
            out[i][j] <== tmp[i][j] * in3[i][j];
        }
    }
}

template XORBitArrays(nLimbs, nBits) {
    // Assumes both are bit-arrays
    signal input in1[nLimbs][nBits];
    signal input in2[nLimbs][nBits];
    signal output out[nLimbs][nBits];
    for (var i = 0; i < nLimbs; i++) {
        for (var j = 0; j < nBits; j++) {
            out[i][j] <== in1[i][j] + in2[i][j] - 2 * in1[i][j] * in2[i][j];
        }
    }
}
