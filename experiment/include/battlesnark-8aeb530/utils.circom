pragma circom 2.1.4;

include "../circomlib-cff5ab6/mimcsponge.circom";

template BoardSerialization (n) {
    signal input board[n][n];
    signal output serialID;

    var s = 0;
    for (var k = 0; k < n * n; k++) {
        s += (2 ** k) * board[k \ 3][k % 3];
    }

    serialID <== s;
}

template BoardToHash (n) {
    signal input board[n][n];
    signal input salt;
    signal output hash;

    component computeSerial = BoardSerialization(n);
    computeSerial.board <== board;
    
    component hashfn = MiMCSponge(2, 220, 1);
    hashfn.ins[0] <== computeSerial.serialID;
    hashfn.ins[1] <== salt;
    hashfn.k <== 0;

    hash <== hashfn.outs[0];
}