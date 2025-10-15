pragma circom 2.0.0;
include "./subtractTensor.circom";
include "./clampTensor.circom";
include "./integerDivision.circom";

template CalculatedScoreFraction(b){
    signal input response_time_reward_metric;
    signal input proof_size_reward_metric;
    signal input competition_weighted;
    signal input scaling;
    signal output out;

    signal total;
    total <== response_time_reward_metric + proof_size_reward_metric + competition_weighted;

    component clamp = Clamp(b);
    clamp.val <== total;
    clamp.min <== 0;
    clamp.max <== scaling;

    out <== clamp.out;
}
