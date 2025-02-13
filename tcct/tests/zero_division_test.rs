use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;
use num_traits::{One, Zero};

use program_structure::ast::ExpressionInfixOpcode;

use tcct::executor::debug_ast::DebuggableExpressionInfixOpcode;
use tcct::executor::symbolic_value::{
    enumerate_array, evaluate_binary_op, initialize_symbolic_nested_array_with_name, OwnerName,
    SymbolicAccess, SymbolicName, SymbolicValue,
};
use tcct::mutator::utils::get_coefficient_of_polynomials;

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
fn test_get_coefficient_constant() {
    // For a constant integer, the coefficients should be:
    // [ <constant>, 0, 0 ]
    let expr = SymbolicValue::ConstantInt(BigInt::from(5));
    let target = make_symbolic_name(100); // target name is irrelevant here

    let result = get_coefficient_of_polynomials(&expr, &target);
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

    let result = get_coefficient_of_polynomials(&expr, &target);
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

    let result = get_coefficient_of_polynomials(&expr, &target);
    let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));

    let expected = [zero.clone(), zero.clone(), zero.clone()];
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

    let result = get_coefficient_of_polynomials(&expr, &target);
    let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));

    let expected_const = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::from(3))),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
    ));
    let expected_linear = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
    ));
    let expected_quadratic = Rc::new(SymbolicValue::BinaryOp(
        zero.clone(),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        zero.clone(),
    ));

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

    let result = get_coefficient_of_polynomials(&expr, &target);
    let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));

    let expected_const = Rc::new(SymbolicValue::BinaryOp(
        zero.clone(),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
        Rc::new(SymbolicValue::ConstantInt(BigInt::from(2))),
    ));
    let expected_linear = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
        zero.clone(),
    ));
    let expected_quadratic = Rc::new(SymbolicValue::BinaryOp(
        zero.clone(),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
        zero.clone(),
    ));

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

    let result = get_coefficient_of_polynomials(&expr, &target);

    // To build our expected values we re-create the coefficients for (3+x) and (4+x):
    let l_const = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::from(3))),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
    ));
    let l_lin = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
    ));
    let l_quad = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
    ));

    let r_const = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::from(4))),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
    ));
    let r_lin = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
    ));
    let r_quad = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
    ));

    // Now, following the multiplication branch:
    let expected_c0 = Rc::new(SymbolicValue::BinaryOp(
        l_const.clone(),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        r_const.clone(),
    ));

    let c1 = SymbolicValue::BinaryOp(
        l_const.clone(),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        r_lin.clone(),
    );
    let c2 = SymbolicValue::BinaryOp(
        l_lin.clone(),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        r_const.clone(),
    );
    let expected_c1 = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(c1.clone()),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        Rc::new(c2.clone()),
    ));

    let c3 = SymbolicValue::BinaryOp(
        l_const.clone(),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        r_quad.clone(),
    );
    let c4 = SymbolicValue::BinaryOp(
        l_const.clone(),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        r_quad.clone(),
    );
    let c5 = SymbolicValue::BinaryOp(
        l_lin.clone(),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        r_lin.clone(),
    );
    let expected_c2 = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(c3.clone()),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
            Rc::new(c4.clone()),
        )),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        Rc::new(c5.clone()),
    ));

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
    let result = get_coefficient_of_polynomials(&expr, &target);
    let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));

    let expected = [Rc::new(expr.clone()), zero.clone(), zero.clone()];
    assert_eq!(result, expected);
}
