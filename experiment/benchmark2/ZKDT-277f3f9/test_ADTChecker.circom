pragma circom 2.1.6;

include "../../include/ZKDT-277f3f9/AuthenticatedDT.circom";

component main { public [ leaf, root] } =  ADTChecker(3);