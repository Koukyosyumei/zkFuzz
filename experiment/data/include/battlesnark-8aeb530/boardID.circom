pragma circom 2.1.4;

include "utils.circom";

template BoardID (n, m) {
    // n = size of board
    // m = # ships per player
    signal input board[n][n];
    signal input salt;
    signal output boardID;


    var sum = 0;
    for (var i = 0; i < n; i++) {
        for (var j = 0; j < n; j++) {
            // board[i][j] should be 0/1 valued
            board[i][j] * (1 - board[i][j]) === 0;

            // if it is 0/1, compute the sum and assert that it is equal to m
            sum += board[i][j];
        }
    }

    sum === m;

    // Compute hash of unique serialization
    component boardToHash = BoardToHash(n);
    boardToHash.board <== board;
    boardToHash.salt <== salt;

    boardID <== boardToHash.hash;
}