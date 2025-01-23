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

component main = StringCheck(3);