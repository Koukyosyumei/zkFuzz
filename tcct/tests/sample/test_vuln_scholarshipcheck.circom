pragma circom 2.1.6;

template CheckBalance(n) {
    signal input balance[n];
    signal input threshold;
    signal output isBalanceLow[n];
    
    for (var i = 0; i < n; i++) {
        isBalanceLow[i] <-- balance[i] < threshold ? 1 : 0;
    }
}

template ScholarshipCheck() {
    signal input balance[4];
    signal input gpa[4];
    signal input threshold;

    // signal output eligibleStudentIndex;
    signal output out;

    component checkBalance = CheckBalance(4);
    checkBalance.balance <== balance;
    checkBalance.threshold <== threshold;

    out <== checkBalance.isBalanceLow[3];
}

component main = ScholarshipCheck();