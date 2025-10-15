pragma circom 2.1.6;

include "./constants.circom";
include "./goldilocks.circom";
include "./utils.circom";

// TODO: Use optimized monolith_mds_12 for goldilocks
template concrete(round) {
    signal input in[SPONGE_WIDTH()];
    signal output out[SPONGE_WIDTH()];

    for (var row = 0; row < SPONGE_WIDTH(); row++) {
        var acc = ROUND_CONSTANTS(round, row);
        for (var column = 0; column < SPONGE_WIDTH(); column++) {
            acc += in[column] * MAT_12(row, column);
        }
        out[row] <== GlReduce(64)(acc);
    }
}

template bar() {
    signal input limbIn; // 64-bit input
    signal output limbOut;

    if (LOOKUP_BITS() == 8) {
        signal limbInBitArrays[8][8] <== Num2BitArrays(8, 8)(limbIn);
        signal limbInBitArraysNeg[8][8] <== NegateBitArrays(8, 8)(limbInBitArrays);

        signal limbInNegRot1[8][8] <== RotateBitArraysLeft(8, 8, 1)(limbInBitArraysNeg);
        signal limbInRot2[8][8] <== RotateBitArraysLeft(8, 8, 2)(limbInBitArrays);
        signal limbInRot3[8][8] <== RotateBitArraysLeft(8, 8, 3)(limbInBitArrays);

        signal tmp1[8][8] <== AND3BitArrays(8, 8)(limbInNegRot1, limbInRot2, limbInRot3);
        signal tmp2[8][8] <== XORBitArrays(8, 8)(limbInBitArrays, tmp1);

        signal limbOutBitArrays[8][8] <== RotateBitArraysLeft(8, 8, 1)(tmp2);
        limbOut <== BitArrays2Num(8, 8)(limbOutBitArrays);
    } else if (LOOKUP_BITS() == 16) {
        signal limbInBitArrays[4][16] <== Num2BitArrays(4, 16)(limbIn);
        signal limbInBitArraysNeg[4][16] <== NegateBitArrays(4, 16)(limbInBitArrays);

        signal limbInNegRot1[4][16] <== RotateBitArraysLeft(4, 16, 1)(limbInBitArraysNeg);
        signal limbInRot2[4][16] <== RotateBitArraysLeft(4, 16, 2)(limbInBitArrays);
        signal limbInRot3[4][16] <== RotateBitArraysLeft(4, 16, 3)(limbInBitArrays);

        signal tmp1[4][16] <== AND3BitArrays(4, 16)(limbInNegRot1, limbInRot2, limbInRot3);
        signal tmp2[4][16] <== XORBitArrays(4, 16)(limbInBitArrays, tmp1);

        signal limbOutBitArrays[4][16] <== RotateBitArraysLeft(4, 16, 1)(tmp2);
        limbOut <== BitArrays2Num(4, 16)(limbOutBitArrays);
    } else {
        assert(0);
    }
}

template bars() {
    signal input in[SPONGE_WIDTH()];
    signal output out[SPONGE_WIDTH()];

    for (var row = 0; row < SPONGE_WIDTH(); row++) {
        if (row < N_BARS()) {
            out[row] <== bar()(in[row]);
        } else {
            out[row] <== in[row];
        }
    }
}

template bricks() {
    signal input in[SPONGE_WIDTH()];
    signal output out[SPONGE_WIDTH()];

    out[0] <== in[0];
    for (var i = 1; i < SPONGE_WIDTH(); i++) {
        var tmp = GlMul()(in[i - 1], in[i - 1]);
        out[i] <== in[i] + tmp;
    }
}

template Monolith() {
    signal input in[SPONGE_WIDTH()];
    signal output out[SPONGE_WIDTH()];

    signal tmp[N_ROUNDS() + 1][3][SPONGE_WIDTH()];
    tmp[0][2] <== concrete(0)(in);
    for (var rc = 1; rc < N_ROUNDS() + 1; rc++) {
        tmp[rc][0] <== bars()(tmp[rc-1][2]);
        tmp[rc][1] <== bricks()(tmp[rc][0]);
        tmp[rc][2] <== concrete(rc)(tmp[rc][1]);
    }

    out <== tmp[N_ROUNDS()][2];
}

template HashNToMNoPad(nInputs, nOutputs) {
    assert(nOutputs <= SPONGE_WIDTH());

    signal input in[nInputs];
    signal output out[nOutputs];

    var nHash = (nInputs + SPONGE_RATE() - 1) \ SPONGE_RATE();
    component cMonolith[nHash];
    component tmpHash[nHash][SPONGE_WIDTH()];

    for (var i = 0; i < nHash; i++) {
        cMonolith[i] = Monolith();
    }

    // Capacity
    for (var j = 0; j < SPONGE_CAPACITY(); j++) {
        cMonolith[0].in[SPONGE_RATE() + j] <== 0;
    }

    for (var i = 0; i < nHash; i++) {
        for (var j = 0; j < SPONGE_RATE(); j++) {
            var index = i * SPONGE_RATE() + j;
            if (index >= nInputs) {
                if (i > 0) {
                  cMonolith[i].in[j] <== cMonolith[i-1].out[j];
                } else {
                  cMonolith[i].in[j] <== 0;
                }
            } else {
                cMonolith[i].in[j] <== in[index];
            }
        }
        if (i > 0) {
            // Capacity
            for (var j = 0; j < SPONGE_CAPACITY(); j++) {
                cMonolith[i].in[SPONGE_RATE() + j] <== cMonolith[i - 1].out[SPONGE_RATE() + j];
            }
        }
    }

    for (var i = 0; i < nOutputs; i++) {
        out[i] <== cMonolith[nHash - 1].out[i];
    }
}

template HashNoPad(nInputs) {
    signal input in[nInputs];
    signal output out[NUM_HASH_OUT_ELTS()];

    out <== HashNToMNoPad(nInputs, NUM_HASH_OUT_ELTS())(in);
}

template TwoToOne() {
    assert(SPONGE_RATE() == 2*NUM_HASH_OUT_ELTS());

    signal input left[NUM_HASH_OUT_ELTS()];
    signal input right[NUM_HASH_OUT_ELTS()];
    signal output out[NUM_HASH_OUT_ELTS()];

    component monolith = Monolith();
    for (var i = 0; i < SPONGE_WIDTH(); i++) {
        if (i < NUM_HASH_OUT_ELTS()) {
            monolith.in[i] <== left[i];
        } else if (i < 2*NUM_HASH_OUT_ELTS()) {
            monolith.in[i] <== right[i - NUM_HASH_OUT_ELTS()];
        } else {
            monolith.in[i] <== 0;
        }
    }

    for (var i = 0; i < NUM_HASH_OUT_ELTS(); i++) {
        out[i] <== monolith.out[i];
    }
}
