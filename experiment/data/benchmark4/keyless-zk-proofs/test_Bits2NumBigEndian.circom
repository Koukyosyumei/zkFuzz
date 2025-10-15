pragma circom 2.1.3;
// https://github.com/alex-lindenbaum/battlesnark/blob/main/circuits/utils.circom

include "../../include/keyless-zk-proofs/packing.circom";

component main = Bits2NumBigEndian(128);