pragma circom 2.1.4;
// https://github.com/alex-lindenbaum/battlesnark/blob/main/circuits/utils.circom

include "../../include/battlesnark-8aeb530/hitShip.circom";

component main { public [boardID, attack_i, attack_j] } = HitShip(3);