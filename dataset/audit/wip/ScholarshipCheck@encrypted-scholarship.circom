pragma circom 2.1.6;

template CheckBalance(n) {
    signal input balance[n];
    signal input threshold;
    signal output isBalanceLow[n];
    
    for (var i = 0; i < n; i++) {
        isBalanceLow[i] <-- balance[i] < threshold ? 1 : 0;
    }
}

template CheckHighestGPA(n) {
    signal input gpa[n];
    signal output isHighestGPA[n];
    
    for (var i = 0; i < n; i++) {
        var isHighest = 1;
        
        for (var j = 0; j < n; j++) {
            if (i != j) {
                isHighest *= (gpa[i] >= gpa[j] ? 1 : 0);
            }
        }
        
        isHighestGPA[i] <-- isHighest;
    }
}

// template FindEligibleStudent(n) {
//     signal input isBalanceLow[n];
//     signal input isHighestGPA[n];
    
//     signal output eligibleStudentIndex;
//     signal output eligibleCount;

//     signal isEligible[n];
//     signal computedCount;
//     signal computedIndex;

//     var count = 0;
//     var index = n;

//     for (var i = 0; i < n; i++) {
//         isEligible[i] <-- isBalanceLow[i] * isHighestGPA[i];
        
//         computedCount <-- (isEligible[i] == 1) ? count + 1 : count;
//         computedIndex <-- (isEligible[i] == 1) ? i : index;

//         if (isEligible[i] == 1){
//             computedCount <== count + 1;
//             computedIndex <== i;
//             count = i + 1;
//             index = i;
//         }else{
//             computedCount <== count;
//             computedIndex <== index;
//         }
//     }

//     eligibleCount <-- computedCount;
//     eligibleStudentIndex <-- computedIndex;
    
// }

template ScholarshipCheck() {
    signal input balance[4];
    signal input gpa[4];
    signal input threshold;

    // signal output eligibleStudentIndex;
    signal output out;

    component checkBalance = CheckBalance(4);
    checkBalance.balance <== balance;
    checkBalance.threshold <== threshold;

    //component checkGPA = CheckHighestGPA(4);
    //checkGPA.gpa <== gpa;

    // component findEligibleStudent = FindEligibleStudent(4);
    // findEligibleStudent.isBalanceLow <== checkBalance.isBalanceLow;
    // findEligibleStudent.isHighestGPA <== checkGPA.isHighestGPA;

    // eligibleStudentIndex <== findEligibleStudent.eligibleStudentIndex;
    out <== checkBalance.isBalanceLow[3];
}

component main = ScholarshipCheck();