pragma circom 2.0.3;

include "../circomlib-cff5ab6/poseidon.circom";
include "../circomlib-cff5ab6/comparators.circom";

template Notebook() {

    signal input address;
    signal input notebookId;
    signal input notePointer;
    signal output hashedOutput;

    // create a constraint that the notebookId is equal to 4
    component isEqual = IsEqual();
    isEqual.in[0] <== notebookId;
    isEqual.in[1] <== 4;

    // create a poseidon hash of the address and notebookId
    component poseidon = Poseidon(2);
    poseidon.inputs[0] <== address;
    poseidon.inputs[1] <== notebookId;
    hashedOutput <== poseidon.out;

    // verify that the hashed output matches the notePointer for the given noteId
    notePointer === hashedOutput;

    log(notePointer);

}

/* INPUT = {
    "address": "0xd89350284c7732163765b23338f2ff27449E0Bf5",
    "notebookId": "2030",
    "notePointer": "20953151860789434919091619613839473050169182927933255172895792454938448763431"
} */
