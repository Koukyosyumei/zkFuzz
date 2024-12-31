pragma circom 2.1.4;
// https://github.com/alex-lindenbaum/battlesnark/blob/main/circuits/utils.circom

include "../../circomlib/circuits/mimcsponge.circom";

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

template HitShip (n) {
    // private
    signal input board[n][n];
    signal input salt;

    // public
    signal input boardID; // certify that player is using the same init board
    signal input attack_i;
    signal input attack_j;
    signal output hitShip;

    // 1. match hash of board to boardID
    component boardToHash = BoardToHash(n);
    boardToHash.board <== board;
    boardToHash.salt <== salt;

    boardID === boardToHash.hash;

    // 2. did verifier hit a ship
    signal hit <-- board[attack_i][attack_j];
    hitShip <== hit;
}

component main { public [boardID, attack_i, attack_j] } = HitShip(3);