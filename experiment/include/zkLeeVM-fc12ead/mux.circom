pragma circom 2.0.0;

template EscalarProduct(w) {
    signal input in1[w];
    signal input in2[w];
    signal output out;
    signal aux[w];
    var lc = 0;
    for (var i=0; i<w; i++) {
        aux[i] <== in1[i]*in2[i];
        lc = lc + aux[i];
    }
    out <== lc;
}

template Decoder(w) {
    signal input inp;
    signal output out[w];
    signal output success;
    var lc=0;

    for (var i=0; i<w; i++) {
        out[i] <-- (inp == i) ? 1 : 0;
        out[i] * (inp-i) === 0;
        lc = lc + out[i];
    }

    lc ==> success;
    success * (success -1) === 0;
}


template MultiMux(nIn, wIn) {
    signal input inp[nIn][wIn];
    signal input sel;
    signal output out[wIn];
    component dec = Decoder(nIn);
    component ep[wIn];

    for (var k=0; k<wIn; k++) {
        ep[k] = EscalarProduct(nIn);
    }

    sel ==> dec.inp;
    for (var j=0; j<wIn; j++) {
        for (var k=0; k<nIn; k++) {
            inp[k][j] ==> ep[j].in1[k];
            dec.out[k] ==> ep[j].in2[k];
        }
        ep[j].out ==> out[j];
    }
    dec.success === 1;
}

template SinglMux(nIn) {
    signal input inp[nIn];
    signal input sel;
    signal output out;
    component dec = Decoder(nIn);
    component ep;

    ep = EscalarProduct(nIn);

    sel ==> dec.inp;
    for (var k=0; k<nIn; k++) {
        inp[k] ==> ep.in1[k];
        dec.out[k] ==> ep.in2[k];
    }
    ep.out ==> out;
    dec.success === 1;
}
