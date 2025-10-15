pragma circom 2.0.0;

include "./util.circom";

// circuit specific for 256x256x3 image

template zkPhoto() {
    signal input in[256][256][3];
    signal output out[16*16\4]; // 16x16 image bundled every 4 pixels
    signal output hash;

    component bilinear[4];

    // first layer
    bilinear[0] = BilinearInterpolation(256);

    for (var i=0; i<256; i++) {
        for (var j=0; j<256; j++) {
            bilinear[0].in[i][j][0] <== in[i][j][0];
            bilinear[0].in[i][j][1] <== in[i][j][1];
            bilinear[0].in[i][j][2] <== in[i][j][2];
        }
    }

    // second to fourth layer
    for (var k=1; k<4; k++) {
        bilinear[k] = BilinearInterpolation(256>>k);
        for (var i=0; i<(256>>k); i++) {
            for (var j=0; j<(256>>k); j++) {
                bilinear[k].in[i][j][0] <== bilinear[k-1].out[i][j][0];
                bilinear[k].in[i][j][1] <== bilinear[k-1].out[i][j][1];
                bilinear[k].in[i][j][2] <== bilinear[k-1].out[i][j][2];
            }
        }
    }

    var idx = 0;

    for (var i=0; i<16; i+=2) {
        for (var j=0; j<16; j+=2) {
            out[idx] <--
                (bilinear[3].out[i][j][0] << 112) +
                (bilinear[3].out[i][j][1] << 104) + 
                (bilinear[3].out[i][j][2] << 96) +

                (bilinear[3].out[i][j+1][0] << 80) + 
                (bilinear[3].out[i][j+1][1] << 72) + 
                (bilinear[3].out[i][j+1][2] << 64) +

                (bilinear[3].out[i+1][j][0] << 48) + 
                (bilinear[3].out[i+1][j][1] << 40) + 
                (bilinear[3].out[i+1][j][2] << 32) +

                (bilinear[3].out[i+1][j+1][0] << 16) + 
                (bilinear[3].out[i+1][j+1][1] << 8) + 
                bilinear[3].out[i+1][j+1][2];
            idx++;
        }
    }

    // hash
    component poseidon = PoseidonTree(16**2); // i.e. 32*32/4

    idx = 0;

    for (var i=0; i<32; i+=2) {
        for (var j=0; j<32; j+=2) {
            poseidon.in[idx] <--
                (bilinear[2].out[i][j][0] << 112) +
                (bilinear[2].out[i][j][1] << 104) + 
                (bilinear[2].out[i][j][2] << 96) +

                (bilinear[2].out[i][j+1][0] << 80) + 
                (bilinear[2].out[i][j+1][1] << 72) + 
                (bilinear[2].out[i][j+1][2] << 64) +

                (bilinear[2].out[i+1][j][0] << 48) + 
                (bilinear[2].out[i+1][j][1] << 40) + 
                (bilinear[2].out[i+1][j][2] << 32) +

                (bilinear[2].out[i+1][j+1][0] << 16) + 
                (bilinear[2].out[i+1][j+1][1] << 8) + 
                bilinear[2].out[i+1][j+1][2];
            idx++;
        }
    }

    hash <== poseidon.out;
}