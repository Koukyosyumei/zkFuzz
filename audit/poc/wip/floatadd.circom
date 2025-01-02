// https://github.com/iden3/distanceProver/blob/5740f4e16c862542e301864e602914d3a814f430/circuits/floatadd.circom#L29

template FloatAdd(n, e, nops, offset) {
    signal input in[nops][n];
    signal output out[n];

    var e2;
    var NIntermediate = n - 1;

    signal aux[NIntermediate];

    signal fin[nops];
    signal fout;

    var lcout = offset;
    for (var i=0; i<nops; i++) {

        var lc = 0;
        e2 = 1;
        for (var k=0; k<n-1; k++) {
            lc += in[i][k]*e2;

            e2 = e2+e2;
        }

        fin[i] <== lc*( 1 - 2 * in[i][n-1]);  // Invert if negative
        lcout += fin[i];
    }


    out[n-1] <-- lcout < 0;
    out[n-1] * (out[n-1] - 1) === 0;

    fout <== lcout* (1 - 2*out[n-1]);

    // Convert fout to binary
    var lcout2 = 0;
    e2 = 1;
    for (var k=0; k<NIntermediate; k++) {
        aux[k] <-- (fout >> k) & 1;

        aux[k] * (aux[k] - 1) === 0;
        lcout2 += aux[k] * e2;

        e2 = e2+e2;
    }

    lcout2 === fout;

    for (var k=0; k<n-1; k++) {
        out[k] <== aux[  k ];
    }

}

component main = FloatAdd(56, 32, 3, 0);