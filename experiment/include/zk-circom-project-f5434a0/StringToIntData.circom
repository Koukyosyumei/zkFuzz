// pragma circom 2.0.0;

// template StringToIntData(n) {
//     signal input string[n];
//     signal output integer;

//     signal temp[n];
//     signal mult_factor[n];
//     signal resultInt[n+1];

//     resultInt[0] <== 0;
//     mult_factor[0] <== 1;
//     for (var i = 1; i < n; i++) {
//         mult_factor[i] <== mult_factor[i-1] * 10;
//     }

//     component partial_sum[n];
//     for (var i = 0; i < n; i++) {
//         partial_sum[i] = calculateTheInt();
//         temp[i] <== string[i] - 64;
//         partial_sum[i].in1 <== temp[i] * mult_factor[n-1-i]; 
//         resultInt[i+1] <== resultInt[i] + partial_sum[i].out;
//     }

//     integer <== resultInt[n];
//     log(integer);
// }

// template calculateTheInt() {
//     signal input in1;
//     signal output out;

//     signal temp <== in1 * in1;
//     out <== temp - temp + in1;
// }

// component main {public [string]} = StringToIntData(5);

pragma circom 2.0.0;

template StringCheck(n) {
    signal input str[n];  // Array of ASCII values
    signal output valid;  // 1 if valid, 0 if not

    var sum = 0;
    signal isLowerCase[n];
    for (var i = 0; i < n; i++) {
        // Check if character is a-z (ASCII 97-122)
        
        isLowerCase[i] <-- (str[i] >= 97) && (str[i] <= 122);
        isLowerCase[i] * (1 - isLowerCase[i]) === 0;  // Constrain to 0 or 1

        sum += isLowerCase[i];
    }

    valid <-- (sum == n);
    valid * (1 - valid) === 0;  // Constrain to 0 or 1
}

// component main = StringCheck(5);