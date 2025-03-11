use std::rc::Rc;

use num_bigint_dig::BigInt;
use num_traits::{One, Zero};

use program_structure::ast::ExpressionInfixOpcode;

use zkfuzz::executor::debug_ast::DebuggableExpressionInfixOpcode;
use zkfuzz::executor::symbolic_value::{get_coefficient_of_polynomials, get_degree_polynomial};
use zkfuzz::executor::symbolic_value::{OwnerName, SymbolicName, SymbolicValue};
use zkfuzz::executor::utils::solve_quadratic_modulus_equation;

// A dummy owner to use for creating SymbolicNames.
fn dummy_owner() -> OwnerName {
    OwnerName {
        id: 1,
        access: None,
        counter: 0,
    }
}

// Helper to construct a SymbolicName with a given id.
// (Use different ids to simulate different variable names.)
fn make_symbolic_name(id: usize) -> SymbolicName {
    SymbolicName::new(id, Rc::new(vec![dummy_owner()]), None)
}

#[test]
fn test_get_degree_constant() {
    // A constant integer should have degree 0.
    let expr = SymbolicValue::ConstantInt(BigInt::from(42));
    let target = make_symbolic_name(1);
    let degree = get_degree_polynomial(&expr, &target);
    assert_eq!(degree, 0);
}

#[test]
fn test_get_degree_variable_match() {
    // A variable that matches the target has degree 1.
    let target = make_symbolic_name(1);
    let expr = SymbolicValue::Variable(target.clone());
    let degree = get_degree_polynomial(&expr, &target);
    assert_eq!(degree, 1);
}

#[test]
fn test_get_degree_addition() {
    // For addition the degree is the max of the degrees of the operands.
    // Expression: (x + 5) where x is the target variable (degree 1).
    // Expected degree is max(1, 0) = 1.
    let target = make_symbolic_name(1);
    let expr_left = SymbolicValue::Variable(target.clone()); // degree 1
    let expr_right = SymbolicValue::ConstantInt(BigInt::from(5)); // degree 0
    let expr = SymbolicValue::BinaryOp(
        Rc::new(expr_left),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(expr_right),
    );
    let degree = get_degree_polynomial(&expr, &target);
    assert_eq!(degree, 1);
}

#[test]
fn test_get_degree_subtraction() {
    // For subtraction the degree is the max of the degrees of the operands.
    // Expression: (5 - x) where x is the target variable (degree 1).
    // Expected degree is max(0, 1) = 1.
    let target = make_symbolic_name(1);
    let expr_left = SymbolicValue::ConstantInt(BigInt::from(5)); // degree 0
    let expr_right = SymbolicValue::Variable(target.clone()); // degree 1
    let expr = SymbolicValue::BinaryOp(
        Rc::new(expr_left),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
        Rc::new(expr_right),
    );
    let degree = get_degree_polynomial(&expr, &target);
    assert_eq!(degree, 1);
}

#[test]
fn test_get_degree_multiplication() {
    // For multiplication the degree is the sum of the degrees of the operands.
    // Expression: x * x where each x has degree 1 → total degree = 1 + 1 = 2.
    let target = make_symbolic_name(1);
    let expr_left = SymbolicValue::Variable(target.clone());
    let expr_right = SymbolicValue::Variable(target.clone());
    let expr = SymbolicValue::BinaryOp(
        Rc::new(expr_left),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        Rc::new(expr_right),
    );
    let degree = get_degree_polynomial(&expr, &target);
    assert_eq!(degree, 2);
}

#[test]
fn test_get_degree_unknown_operator_non_zero() {
    // For an operator not handled (e.g., Div) if any operand has nonzero degree,
    // the result should be std::usize::MAX.
    // Expression: (x / 5) where x (degree 1) / constant (degree 0)
    let target = make_symbolic_name(1);
    let expr_left = SymbolicValue::Variable(target.clone()); // degree 1
    let expr_right = SymbolicValue::ConstantInt(BigInt::from(5)); // degree 0
    let expr = SymbolicValue::BinaryOp(
        Rc::new(expr_left),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Div),
        Rc::new(expr_right),
    );
    let degree = get_degree_polynomial(&expr, &target);
    assert_eq!(degree, std::usize::MAX);
}

#[test]
fn test_get_degree_variable_no_match() {
    // A variable that does not match the target is considered degree 0.
    let target = make_symbolic_name(1);
    let other = make_symbolic_name(2);
    let expr = SymbolicValue::Variable(other);
    let degree = get_degree_polynomial(&expr, &target);
    assert_eq!(degree, 0);
}

#[test]
fn test_get_coefficient_constant() {
    // For a constant integer, the coefficients should be:
    // [ <constant>, 0, 0 ]
    let expr = SymbolicValue::ConstantInt(BigInt::from(5));
    let target = make_symbolic_name(100); // target name is irrelevant here

    let result = get_coefficient_of_polynomials(&expr, &target, &BigInt::from(7));
    let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));

    let expected = [
        Rc::new(SymbolicValue::ConstantInt(BigInt::from(5))),
        zero.clone(),
        zero.clone(),
    ];
    assert_eq!(result, expected);
}

#[test]
fn test_get_coefficient_variable_match() {
    // When the variable matches the target, we expect:
    // [ 0, 1, 0 ]
    let target = make_symbolic_name(1);
    let expr = SymbolicValue::Variable(target.clone());

    let result = get_coefficient_of_polynomials(&expr, &target, &BigInt::from(7));
    let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));
    let one = Rc::new(SymbolicValue::ConstantInt(BigInt::one()));

    let expected = [zero.clone(), one, zero.clone()];
    assert_eq!(result, expected);
}

#[test]
fn test_get_coefficient_variable_no_match() {
    // When the variable does not match the target, all coefficients are 0.
    let target = make_symbolic_name(1);
    let other = make_symbolic_name(2);
    let expr = SymbolicValue::Variable(other);

    let result = get_coefficient_of_polynomials(&expr, &target, &BigInt::from(7));
    let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));
    let expected = [Rc::new(expr), zero.clone(), zero.clone()];
    assert_eq!(result, expected);
}

#[test]
fn test_get_coefficient_addition() {
    // For an addition expression the coefficients are the sum (as BinaryOp nodes)
    // of the coefficients from each operand.
    //
    // For the left operand: a constant 3 → coefficients: [3, 0, 0]
    // For the right operand: variable x (which matches target) → [0, 1, 0]
    // So the overall expected coefficients are:
    // constant: BinaryOp(3, Add, 0)
    // linear:   BinaryOp(0, Add, 1)
    // quadratic: BinaryOp(0, Add, 0)
    let target = make_symbolic_name(1);
    let expr_left = SymbolicValue::ConstantInt(BigInt::from(3));
    let expr_right = SymbolicValue::Variable(target.clone());

    let expr = SymbolicValue::BinaryOp(
        Rc::new(expr_left),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(expr_right),
    );

    let result = get_coefficient_of_polynomials(&expr, &target, &BigInt::from(7));

    let expected_const = Rc::new(SymbolicValue::ConstantInt(BigInt::from(3)));
    let expected_linear = Rc::new(SymbolicValue::ConstantInt(BigInt::one()));
    let expected_quadratic = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));

    let expected = [expected_const, expected_linear, expected_quadratic];
    assert_eq!(result, expected);
}

#[test]
fn test_get_coefficient_subtraction() {
    // For subtraction, the coefficients are the difference (wrapped as BinaryOp nodes)
    // of the coefficients from each operand.
    //
    // Left: variable x → [0, 1, 0]
    // Right: constant 2 → [2, 0, 0]
    // Expected:
    // constant: BinaryOp(0, Sub, 2)
    // linear:   BinaryOp(1, Sub, 0)
    // quadratic: BinaryOp(0, Sub, 0)
    let target = make_symbolic_name(1);
    let expr_left = SymbolicValue::Variable(target.clone());
    let expr_right = SymbolicValue::ConstantInt(BigInt::from(2));

    let expr = SymbolicValue::BinaryOp(
        Rc::new(expr_left),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
        Rc::new(expr_right),
    );

    let result = get_coefficient_of_polynomials(&expr, &target, &BigInt::from(7));
    let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));

    let expected_const = Rc::new(SymbolicValue::ConstantInt(BigInt::from(5)));
    let expected_linear = Rc::new(SymbolicValue::ConstantInt(BigInt::one()));
    let expected_quadratic = zero.clone();

    let expected = [expected_const, expected_linear, expected_quadratic];
    assert_eq!(result, expected);
}

#[test]
fn test_get_coefficient_multiplication() {
    // For multiplication the function combines the coefficients of the two factors.
    // For example, let’s use the expression: (3 + x) * (4 + x)
    //
    // For (3 + x):
    //   constant: BinaryOp(3, Add, 0)
    //   linear:   BinaryOp(0, Add, 1)
    //   quadratic: BinaryOp(0, Add, 0)
    //
    // For (4 + x):
    //   constant: BinaryOp(4, Add, 0)
    //   linear:   BinaryOp(0, Add, 1)
    //   quadratic: BinaryOp(0, Add, 0)
    //
    // According to the multiplication branch:
    //   coefficient0 = L0 * R0
    //   coefficient1 = (L0 * R1) * (L1 * R0)
    //   coefficient2 = ((L0 * R2) * (L0 * R2)) * (L1 * R1)
    //
    // We build the expected trees accordingly.
    let target = make_symbolic_name(1);
    let expr_left = SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::from(3))),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(SymbolicValue::Variable(target.clone())),
    );
    let expr_right = SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::from(4))),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(SymbolicValue::Variable(target.clone())),
    );
    let expr = SymbolicValue::BinaryOp(
        Rc::new(expr_left),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        Rc::new(expr_right),
    );

    let result = get_coefficient_of_polynomials(&expr, &target, &BigInt::from(7));

    // Now, following the multiplication branch:
    let expected_c0 = Rc::new(SymbolicValue::ConstantInt(BigInt::from(5)));
    let expected_c1 = Rc::new(SymbolicValue::ConstantInt(BigInt::from(0)));
    let expected_c2 = Rc::new(SymbolicValue::ConstantInt(BigInt::from(1)));

    let expected = [expected_c0, expected_c1, expected_c2];
    assert_eq!(result, expected);
}

#[test]
fn test_get_coefficient_unknown_operator() {
    // For a binary operator that is not Add, Sub, or Mul
    // (for example, Div), the function returns [ expr, 0, 0 ].
    let target = make_symbolic_name(1);
    let expr_left = SymbolicValue::ConstantInt(BigInt::from(7));
    let expr_right = SymbolicValue::ConstantInt(BigInt::from(3));

    let expr = SymbolicValue::BinaryOp(
        Rc::new(expr_left),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Div),
        Rc::new(expr_right),
    );
    let result = get_coefficient_of_polynomials(&expr, &target, &BigInt::from(7));
    let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));

    let expected = [Rc::new(expr.clone()), zero.clone(), zero.clone()];
    assert_eq!(result, expected);
}

#[test]
fn test_degenerate() {
    // Equation: 5 ≡ 0 (mod 7)  i.e. coefficients: c = 5, b = 0, a = 0.
    let coeffs = [BigInt::from(5), BigInt::zero(), BigInt::zero()];
    let modulus = BigInt::from(7);
    assert_eq!(solve_quadratic_modulus_equation(&coeffs, &modulus), None);
}

#[test]
fn test_linear_equation() {
    // Equation: 2*x + 3 ≡ 0 (mod 7)
    // Coeffs: [c, b, a] = [3, 2, 0].
    // In mod 7: -3 mod 7 is 4. Since the inverse of 2 mod 7 is 4 (because 2×4 = 8 ≡ 1 mod 7),
    // we expect x ≡ 4×4 = 16 ≡ 2 mod 7.
    let coeffs = [BigInt::from(3), BigInt::from(2), BigInt::zero()];
    let modulus = BigInt::from(7);
    assert_eq!(
        solve_quadratic_modulus_equation(&coeffs, &modulus),
        Some(BigInt::from(2))
    );
}

/// Test a linear equation with zero constant term.
/// For example: 2*x ≡ 0 (mod 7) should have the unique solution x ≡ 0.
#[test]
fn test_linear_zero_constant() {
    // Equation: 2*x ≡ 0 (mod 7)
    // Coeffs: [0, 2, 0].
    let coeffs = [BigInt::zero(), BigInt::from(2), BigInt::zero()];
    let modulus = BigInt::from(7);
    assert_eq!(
        solve_quadratic_modulus_equation(&coeffs, &modulus),
        Some(BigInt::zero())
    );
}

/// Test a quadratic equation that has a solution.
/// For example, consider the equation: x² – 1 ≡ 0 (mod 17).
/// Written as: 1·x² + 0·x + (-1) ≡ 0 (mod 17), i.e. coeffs = [-1, 0, 1].
/// The discriminant is D = 0² – 4×1×(-1) = 4, and √4 ≡ 2 mod 17.
/// Thus, one candidate solution is x ≡ (–0 + 2) / 2 ≡ 1 (mod 17).
#[test]
fn test_quadratic_with_solution_mod17() {
    let coeffs = [BigInt::from(-1), BigInt::zero(), BigInt::one()];
    let modulus = BigInt::from(17);
    assert_eq!(
        solve_quadratic_modulus_equation(&coeffs, &modulus),
        Some(BigInt::one())
    );
}

/// Test another quadratic equation with a solution.
/// For example, the equation: x² + x + 1 ≡ 0 (mod 7).
/// Here, coeffs = [1, 1, 1]. The discriminant is D = 1 – 4 = -3 ≡ 4 mod 7,
/// and √4 ≡ 2 mod 7. Hence, the candidate solution is:
/// x ≡ (–1 + 2) / 2 ≡ 1/2 mod 7, and since the inverse of 2 mod 7 is 4,
/// we get x ≡ 1×4 = 4 mod 7.
#[test]
fn test_quadratic_with_solution_mod7() {
    let coeffs = [BigInt::one(), BigInt::one(), BigInt::one()];
    let modulus = BigInt::from(7);
    assert_eq!(
        solve_quadratic_modulus_equation(&coeffs, &modulus),
        Some(BigInt::from(4))
    );
}

/// Test a quadratic equation for which no solution exists.
/// For example, consider: 2*x² + 3*x + 4 ≡ 0 (mod 11).
/// Here, coeffs = [4, 3, 2] and the discriminant is
/// D = 3² – 4×2×4 = 9 – 32 = -23 ≡ 10 mod 11.
/// Since 10 is not a quadratic residue modulo 11, no solution exists.
#[test]
fn test_quadratic_no_solution() {
    let coeffs = [BigInt::from(4), BigInt::from(3), BigInt::from(2)];
    let modulus = BigInt::from(11);
    assert_eq!(solve_quadratic_modulus_equation(&coeffs, &modulus), None);
}
