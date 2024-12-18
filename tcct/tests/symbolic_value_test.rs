use num_bigint_dig::BigInt;
use std::str::FromStr;

use program_structure::ast::ExpressionInfixOpcode;

use tcct::executor::debug_ast::DebugExpressionInfixOpcode;
use tcct::executor::symbolic_value::{evaluate_binary_op, SymbolicValue};

#[test]
fn test_arithmetic_operations() {
    let prime = BigInt::from(17);

    // Addition
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(7)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add)
        ),
        SymbolicValue::ConstantInt(BigInt::from(12))
    );

    // Subtraction
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(10)),
            &SymbolicValue::ConstantInt(BigInt::from(7)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::Sub)
        ),
        SymbolicValue::ConstantInt(BigInt::from(3))
    );

    // Multiplication
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(7)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::Mul)
        ),
        SymbolicValue::ConstantInt(BigInt::from(1)) // (5 * 7) % 17 = 35 % 17 = 1
    );

    // Division
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(8)),
            &SymbolicValue::ConstantInt(BigInt::from(2)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::Div)
        ),
        SymbolicValue::ConstantInt(BigInt::from(4))
    );
}

#[test]
fn test_comparison_operations() {
    let prime = BigInt::from(17);

    // Less than
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(7)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::Lesser)
        ),
        SymbolicValue::ConstantBool(true)
    );

    // Greater than or equal
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(7)),
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::GreaterEq)
        ),
        SymbolicValue::ConstantBool(true)
    );

    // Equal
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq)
        ),
        SymbolicValue::ConstantBool(true)
    );
}

#[test]
fn test_bitwise_operations() {
    let prime = BigInt::from(17);

    // Bitwise OR
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(3)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::BitOr)
        ),
        SymbolicValue::ConstantInt(BigInt::from(7))
    );

    // Bitwise AND
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(3)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::BitAnd)
        ),
        SymbolicValue::ConstantInt(BigInt::from(1))
    );

    // Bitwise XOR
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(3)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::BitXor)
        ),
        SymbolicValue::ConstantInt(BigInt::from(6))
    );
}

#[test]
fn test_boolean_operations() {
    let prime = BigInt::from(17);

    // Boolean AND
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantBool(true),
            &SymbolicValue::ConstantBool(false),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::BoolAnd)
        ),
        SymbolicValue::ConstantBool(false)
    );

    // Boolean OR
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantBool(true),
            &SymbolicValue::ConstantBool(false),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::BoolOr)
        ),
        SymbolicValue::ConstantBool(true)
    );
}

#[test]
fn test_edge_cases() {
    let prime = BigInt::from(17);

    // Division by zero
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(0)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::Div)
        ),
        SymbolicValue::ConstantInt(BigInt::from(0))
    );

    // Large numbers
    let large_num = BigInt::from_str("1000000000000000000000000").unwrap();
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(large_num.clone()),
            &SymbolicValue::ConstantInt(BigInt::from(2)),
            &prime,
            &DebugExpressionInfixOpcode(ExpressionInfixOpcode::Mul)
        ),
        SymbolicValue::ConstantInt(BigInt::from(15)) // (1000000000000000000000000 * 2) % 17 = 15
    );
}
