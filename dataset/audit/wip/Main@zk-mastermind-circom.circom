pragma circom 2.0.0; 

include "../../include/circomlib/circuits/comparators.circom";
include "../../include/circomlib/circuits/bitify.circom";
include "../../include/circomlib/circuits/sha256/sha256.circom";

template ConcatGuesses() {

    signal input privSolnA;
    signal input privSolnB;
    signal input privSolnC;
    signal input privSolnD;

    signal input privSalt;

    var guesses[4] = [privSolnA, privSolnB, privSolnC, privSolnD];
    component bitConverter[4];

    component concatenatedBits = Bits2Num(160);

    for (var i = 0; i < 4; i++) {
        bitConverter[i] = Num2Bits(8);
        bitConverter[i].in <== guesses[i];
        for (var j = 7; j >= 0 ;j--) {
            concatenatedBits.in[i*8+j] <== bitConverter[i].out[7 - j];
        }
    }

    component bitConverterSalt = Num2Bits(128);
    bitConverterSalt.in <== privSalt;
    for (var i = 0; i < 128; i++) {
        concatenatedBits.in[32+i] <== bitConverterSalt.out[i];
    }

    signal output out[160] <== concatenatedBits.in;
}

template MastermindSha256() {

    signal input privSolnA;
    signal input privSolnB;
    signal input privSolnC;
    signal input privSolnD;

    signal input pubSolnHash;

    signal input privSalt;

    signal output hash;

    component concat = ConcatGuesses();

    concat.privSolnA <== privSolnA;
    concat.privSolnB <== privSolnB;
    concat.privSolnC <== privSolnC;
    concat.privSolnD <== privSolnD;
    concat.privSalt <== privSalt;

    component sha256 = Sha256(160);

    sha256.in <== concat.out;

    component decoder = Bits2Num(248);

    for (var i = 255; i > 7; i--) {
        decoder.in[255 - i] <== sha256.out[i];
    }

    hash <== decoder.out;

    hash === pubSolnHash;

}

template Main() {
    // Public inputs
    signal input pubGuessA;
    signal input pubGuessB;
    signal input pubGuessC;
    signal input pubGuessD;
    signal input pubNumBlacks;
    signal input pubNumWhites;
    signal input pubSolnHash;

    // Private inputs: the solution to the puzzle
    signal input privSolnA;
    signal input privSolnB;
    signal input privSolnC;
    signal input privSolnD;

    signal input privSalt;

    // Output
    signal output solnHashOut;

    var nb = 0;

    var guess[4] = [pubGuessA, pubGuessB, pubGuessC, pubGuessD];
    var soln[4] =  [privSolnA, privSolnB, privSolnC, privSolnD];

    component eqB[4];

    // Count black pegs
    for (var i=0; i<4; i++) {
        
        eqB[i] = IsEqual();
        
        eqB[i].in[0] <== guess[i];
        eqB[i].in[1] <== soln[i];

        nb += eqB[i].out;

        var r = eqB[i].out;

        guess[i] = guess[i] * (-1 * r + 1 );
        soln[i] = soln[i] * (-1 * r + 1 );
    }
    
    var nw = 0;

    // Count white pegs
    // block scope isn't respected, so k and j have to be declared outside
    var k = 0;
    var j = 0;
    component eqW[16];
    component isZ[16];
    component isG[16];
    for (j=0; j<4; j++) {
        for (k=0; k<4; k++) {
            // the && operator doesn't work
            if (j != k) {
                var indexW = j * 4 + k;  

                isZ[indexW] = IsZero();
                isZ[indexW].in <-- guess[j];

                var z = isZ[indexW].out;

                eqW[indexW] = IsEqual();
                eqW[indexW].in[0] <-- guess[j];
                eqW[indexW].in[1] <-- soln[k];
        
                var eq = eqW[indexW].out;

                isG[indexW] = GreaterThan(1);
                isG[indexW].in[0] <== eq;
                isG[indexW].in[1] <== z;

                var r = isG[indexW].out;

                nw += r;

                guess[j] = guess[j] * (-1 * r + 1 );
                soln[k] = soln[k] * (-1 * r + 1 );

            }
        }
    }

    // Create a constraint around the number of black pegs
    nb === pubNumBlacks;

    // Create a constraint around the number of white pegs
    nw  ===  pubNumWhites;


    // Verify that the hash of the private solution matches pubSolnHash
    // via a constraint that the publicly declared solution hash matches the
    // private solution witness

   component mastermindSha256 = MastermindSha256();

   mastermindSha256.privSolnA <== privSolnA;
   mastermindSha256.privSolnB <== privSolnB;
   mastermindSha256.privSolnC <== privSolnC;
   mastermindSha256.privSolnD <== privSolnD;

   mastermindSha256.privSalt <== privSalt;
   mastermindSha256.pubSolnHash <== pubSolnHash;

   solnHashOut <== mastermindSha256.hash;

   log("Solution hash: ", solnHashOut);
}

component main {public [pubGuessA, pubGuessB, pubGuessC, pubGuessD, pubNumBlacks, pubNumWhites, pubSolnHash]} = Main();