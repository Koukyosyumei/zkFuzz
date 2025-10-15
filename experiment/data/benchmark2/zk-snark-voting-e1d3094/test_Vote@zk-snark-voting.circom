pragma circom 2.1.6;

// https://github.com/BjernoFolkvardsen/zk-snark-voting/blob/main/circuits/VoteCircuit/Vote.circom

include "../../include/zk-snark-voting-e1d3094/Vote.circom";

component main {public [pk_t, g, e_v]} = Vote();