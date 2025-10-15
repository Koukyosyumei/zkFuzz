pragma circom 2.0.0;

include "../circomlib-cff5ab6/poseidon.circom";

template PoseidonTree(nLeafs) {
    signal input in[nLeafs];
    signal output out;

    component poseidon[nLeafs\15];

    var idx = 0;

    // level 1
    for (var i=0; i<nLeafs; i+=16) {
        poseidon[idx] = Poseidon(16);
        for (var j=0; j<16; j++) {
            poseidon[idx].inputs[j] <== in[i+j];
        }
        idx++;
    }
    
    // higher levels
    
    for (var level=2; (nLeafs>>(level*4)) > 0; level++) {
        for (var i=0; i<(nLeafs>>(level*4)); i++) {
            poseidon[idx] = Poseidon(16);
            for (var j=0; j<16; j++) {
                poseidon[idx].inputs[j] <== poseidon[idx-(nLeafs>>((level-1)*4))+i+j].out;     
            }
            idx++;
        }
    }
    
    out <== poseidon[idx-1].out;
}

template BilinearInterpolation(d) {
    signal input in[d][d][3];
    signal output out[d\2][d\2][3];

    for (var i=0; i<d; i+=2) {
        for (var j=0; j<d; j+=2) {
            out[i\2][j\2][0] <== (in[i][j][0] + in[i][j+1][0] + in[i+1][j][0] + in[i+1][j+1][0])\4; 
            out[i\2][j\2][1] <== (in[i][j][1] + in[i][j+1][1] + in[i+1][j][1] + in[i+1][j+1][1])\4;
            out[i\2][j\2][2] <== (in[i][j][2] + in[i][j+1][2] + in[i+1][j][2] + in[i+1][j+1][2])\4;
        }
    }
}