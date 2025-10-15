pragma circom 2.2.1;
// https://github.com/Dyslex7c/zk-Election/blob/bddc5c24315bf3dd7aeeb6611fa9936f1e23f733/circuits/voting_circuit.circom
include "../../include/zk-Election-0667ff7/voting_circuit.circom";

// Main component instantiation
component main = VotingCircuit(5, 256);