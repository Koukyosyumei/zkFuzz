pragma circom 2.0.5;

include "../libs/telepathy/circuits/circuits/rotate.circom";

component main {public [finalizedHeaderRoot, syncCommitteeSSZ, syncCommitteePoseidon]} = Rotate();
