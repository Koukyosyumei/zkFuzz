pragma circom 2.0.0;

// https://github.com/tinaaliakbarpour/Circom-examples/blob/d71c19c0b346c21038b5ae455642f9accd306843/secretsharing/secretsharing.circom

include "../../include/Circom-examples-d71c19c/secretsharing.circom";

component main {public [p, s2, s3]} = SecretSharingForParty1(9);