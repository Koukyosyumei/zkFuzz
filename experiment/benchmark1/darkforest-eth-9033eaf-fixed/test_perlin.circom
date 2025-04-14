/*
    Largely from DarkForest/circuits
    Changelog:
    - added GetWeight() for circom2 compatibility
    - changed modulo() for circom2 compatibility
*/

pragma circom 2.0.3;

include "../libs/darkforest-eth-9033eaf-fixed/perlin.circom";

component main = MultiScalePerlin(); // if you change this n, you also need to recompute DENOMINATOR with JS.