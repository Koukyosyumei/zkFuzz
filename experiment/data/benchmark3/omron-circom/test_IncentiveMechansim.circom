pragma circom 2.0.0;

include "../../include/omron-circom/circuit.circom";

component main {public [
    RATE_OF_DECAY,
    RATE_OF_RECOVERY,
    FLATTENING_COEFFICIENT,
    PROOF_SIZE_THRESHOLD,
    PROOF_SIZE_WEIGHT,
    RESPONSE_TIME_WEIGHT,
    COMPETITION_WEIGHT,
    MAXIMUM_RESPONSE_TIME_DECIMAL,
    maximum_score,
    previous_score,
    verified,
    proof_size,
    response_time,
    competition,
    maximum_response_time,
    minimum_response_time,
    block_number,
    validator_uid,
    miner_uid,
    scaling
]} = IncentiveMechansim(1,40);