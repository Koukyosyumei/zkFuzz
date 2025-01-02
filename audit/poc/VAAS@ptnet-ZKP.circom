include "../circomlib/circuits/comparators.circom";
// https://github.com/FactomProject/ptnet-ZKP/blob/master/vass/circuit.circom

/*
 Encode the Petri-Net equivilant VASS as a cuicuit.

 see: https://en.wikipedia.org/wiki/Vector_addition_system

 This example show a game of tic-tac-toe

 State Machine is generated from this pflow file
 https://raw.githubusercontent.com/FactomProject/ptnet-eventstore/master/examples/octoe.pflow

 "initial": [0,1,1,1,1,1,1,1,1,1,1,0,0,1],
 
 // "transitions":{
 
 0 "ON":{"delta":[1,0,0,0,0,0,0,0,0,0,0,0,0,-1]}, 
 1 "OFF":{"delta":[-1,0,0,0,0,0,0,0,0,0,0,0,0,1]},
 2 "ON":{"delta":[1,0,0,0,0,0,0,0,0,0,0,0,0,-1]},
 3 "O00":{"delta":[0,-1,0,0,0,0,0,0,0,0,0,1,-1,0]},
 4 "O01":{"delta":[0,0,-1,0,0,0,0,0,0,0,0,1,-1,0]},
 5 "O02":{"delta":[0,0,0,-1,0,0,0,0,0,0,0,1,-1,0]},
 6 "O10":{"delta":[0,0,0,0,-1,0,0,0,0,0,0,1,-1,0]},
 7 "O11":{"delta":[0,0,0,0,0,-1,0,0,0,0,0,1,-1,0]},
 8 "O12":{"delta":[0,0,0,0,0,0,-1,0,0,0,0,1,-1,0]},
 9 "O20":{"delta":[0,0,0,0,0,0,0,-1,0,0,0,1,-1,0]},
 10 "O21":{"delta":[0,0,0,0,0,0,0,0,-1,0,0,1,-1,0]},
 11 "O22":{"delta":[0,0,0,0,0,0,0,0,0,-1,0,1,-1,0]},
 12 "X00":{"delta":[0,-1,0,0,0,0,0,0,0,0,0,-1,1,0]},
 13 "X01":{"delta":[0,0,-1,0,0,0,0,0,0,0,0,-1,1,0]},
 14 "X02":{"delta":[0,0,0,-1,0,0,0,0,0,0,0,-1,1,0]},
 15 "X10":{"delta":[0,0,0,0,-1,0,0,0,0,0,0,-1,1,0]},
 16 "X11":{"delta":[0,0,0,0,0,-1,0,0,0,0,0,-1,1,0]},
 17 "X12":{"delta":[0,0,0,0,0,0,-1,0,0,0,0,-1,0,0]},
 18 "X20":{"delta":[0,0,0,0,0,0,0,-1,0,0,0,-1,1,0]},
 19 "X21":{"delta":[0,0,0,0,0,0,0,0,-1,0,0,-1,1,0]},
 22 "X22":{"delta":[0,0,0,0,0,0,0,0,0,-1,0,-1,1,0]}}
     }
*/

template VAAS(n) {
    signal input state[n];
    signal input sub[n]; // REVIEW: circom to disallow negative values for signal,
    signal input action;
                                 // so we break the action into two parts
    signal input add[n]; // action 
    signal input result[n];
    signal output yield[n];

    component eq[n];

    
    for (var i=0; i<n; i++) {

        eq[i] = IsEqual();
        eq[i].in[0] <-- result[i];
        eq[i].in[1] <-- state[i] - sub[i] + add[i];

        yield[i] <== eq[i].out;
    }

}

// NOTE: to make use of this with another size petri-net
// update this to match the n-dimention of the Petri-Net you are working with
component main = VAAS(3);