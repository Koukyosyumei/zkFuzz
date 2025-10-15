pragma circom 2.1.4;

include "utils.circom";

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

// component main { public [boardID, attack_i, attack_j] } = HitShip(3);

/* INPUT = {
    "board": [[0,0,1], [1,0,0], [0,0,1]],
    "salt": "354729834751283645",
    "boardID": "11156158942301171942571071266390699230298862109118883142074973974812136979122",
    "attack_i": "2",
    "attack_j": "0"
} */