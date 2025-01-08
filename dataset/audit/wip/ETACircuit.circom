pragma circom 2.0.0;

include "../../include/circomlib/circuits/comparators.circom";
include "../../include/circomlib/circuits/mux1.circom";
include "../../include/circomlib/circuits/bitify.circom";

template ETAVerifier() {
    // Public inputs
    signal input claimedETA;
    signal input actualETA;
    signal input tolerance;

    // Ensure inputs are within valid ranges (0-100 for ETAs, 1-20 for tolerance)
    component claimedRange = Num2Bits(8); // 8 bits can represent 0-255
    component actualRange = Num2Bits(8);
    component toleranceRange = Num2Bits(5); // 5 bits can represent 0-31

    claimedRange.in <== claimedETA;
    actualRange.in <== actualETA;
    toleranceRange.in <== tolerance;

    // Calculate absolute difference using a comparator
    signal difference;
    component isGreaterComparator = GreaterEqThan(8); // Changed from 32 to 8 bits
    isGreaterComparator.in[0] <== claimedETA;
    isGreaterComparator.in[1] <== actualETA;
    signal isGreater <== isGreaterComparator.out;

    // Calculate the difference
    signal diff1, diff2;
    diff1 <== claimedETA - actualETA;
    diff2 <== actualETA - claimedETA;

    // Use Mux1 to select the correct difference
    component mux = Mux1();
    mux.s <== isGreater;
    mux.c[0] <== diff2;
    mux.c[1] <== diff1;
    difference <== mux.out;
    
    // Check if within tolerance
    component lessThan = LessThan(8); // Changed from 32 to 8 bits
    lessThan.in[0] <== difference;
    lessThan.in[1] <== tolerance + 1;
    
    signal output isValid <== lessThan.out;
}

component main {public [claimedETA, actualETA, tolerance]} = ETAVerifier();