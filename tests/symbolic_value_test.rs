use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;

use program_structure::ast::ExpressionInfixOpcode;

use zkfuzz::executor::debug_ast::DebuggableExpressionInfixOpcode;
use zkfuzz::executor::symbolic_value::{enumerate_array, evaluate_binary_op, SymbolicValue};

#[test]
fn test_arithmetic_operations() {
    let prime = BigInt::from(17);

    // Addition
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(7)),
            &prime,
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add)
        ),
        SymbolicValue::ConstantInt(BigInt::from(12))
    );

    // Subtraction
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(10)),
            &SymbolicValue::ConstantInt(BigInt::from(7)),
            &prime,
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub)
        ),
        SymbolicValue::ConstantInt(BigInt::from(3))
    );

    // Multiplication
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(7)),
            &prime,
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul)
        ),
        SymbolicValue::ConstantInt(BigInt::from(1)) // (5 * 7) % 17 = 35 % 17 = 1
    );

    // Division
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(8)),
            &SymbolicValue::ConstantInt(BigInt::from(2)),
            &prime,
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Div)
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
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Lesser)
        ),
        SymbolicValue::ConstantBool(true)
    );

    // Greater than or equal
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(7)),
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &prime,
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::GreaterEq)
        ),
        SymbolicValue::ConstantBool(true)
    );

    // Equal
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &prime,
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq)
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
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BitOr)
        ),
        SymbolicValue::ConstantInt(BigInt::from(7))
    );

    // Bitwise AND
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(3)),
            &prime,
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BitAnd)
        ),
        SymbolicValue::ConstantInt(BigInt::from(1))
    );

    // Bitwise XOR
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantInt(BigInt::from(5)),
            &SymbolicValue::ConstantInt(BigInt::from(3)),
            &prime,
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BitXor)
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
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BoolAnd)
        ),
        SymbolicValue::ConstantBool(false)
    );

    // Boolean OR
    assert_eq!(
        evaluate_binary_op(
            &SymbolicValue::ConstantBool(true),
            &SymbolicValue::ConstantBool(false),
            &prime,
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BoolOr)
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
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Div)
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
            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul)
        ),
        SymbolicValue::ConstantInt(BigInt::from(15)) // (1000000000000000000000000 * 2) % 17 = 15
    );
}

#[test]
fn test_enumerate_flat_array() {
    let array = SymbolicValue::Array(vec![
        Rc::new(SymbolicValue::ConstantInt(BigInt::from(1))),
        Rc::new(SymbolicValue::ConstantInt(BigInt::from(2))),
        Rc::new(SymbolicValue::ConstantInt(BigInt::from(3))),
    ]);

    let result = enumerate_array(&array);

    assert_eq!(result.len(), 3);
    assert_eq!(
        result[0],
        (vec![0], &SymbolicValue::ConstantInt(BigInt::from(1)))
    );
    assert_eq!(
        result[1],
        (vec![1], &SymbolicValue::ConstantInt(BigInt::from(2)))
    );
    assert_eq!(
        result[2],
        (vec![2], &SymbolicValue::ConstantInt(BigInt::from(3)))
    );
}

#[test]
fn test_enumerate_nested_array() {
    let nested_array = SymbolicValue::Array(vec![
        Rc::new(SymbolicValue::Array(vec![
            Rc::new(SymbolicValue::ConstantInt(BigInt::from(1))),
            Rc::new(SymbolicValue::ConstantInt(BigInt::from(2))),
        ])),
        Rc::new(SymbolicValue::Array(vec![
            Rc::new(SymbolicValue::ConstantInt(BigInt::from(3))),
            Rc::new(SymbolicValue::ConstantInt(BigInt::from(4))),
        ])),
    ]);

    let result = enumerate_array(&nested_array);

    assert_eq!(result.len(), 4);
    assert_eq!(
        result[0],
        (vec![0, 0], &SymbolicValue::ConstantInt(BigInt::from(1)))
    );
    assert_eq!(
        result[1],
        (vec![0, 1], &SymbolicValue::ConstantInt(BigInt::from(2)))
    );
    assert_eq!(
        result[2],
        (vec![1, 0], &SymbolicValue::ConstantInt(BigInt::from(3)))
    );
    assert_eq!(
        result[3],
        (vec![1, 1], &SymbolicValue::ConstantInt(BigInt::from(4)))
    );
}

#[test]
fn test_enumerate_deeply_nested_array() {
    let deeply_nested_array =
        SymbolicValue::Array(vec![Rc::new(SymbolicValue::Array(vec![Rc::new(
            SymbolicValue::Array(vec![
                Rc::new(SymbolicValue::ConstantInt(BigInt::from(2))),
                Rc::new(SymbolicValue::ConstantInt(BigInt::from(3))),
            ]),
        )]))]);

    let result = enumerate_array(&deeply_nested_array);

    assert_eq!(result.len(), 2);
    assert_eq!(
        result[0],
        (vec![0, 0, 0], &SymbolicValue::ConstantInt(BigInt::from(2)))
    );
    assert_eq!(
        result[1],
        (vec![0, 0, 1], &SymbolicValue::ConstantInt(BigInt::from(3)))
    );
}

#[test]
fn test_enumerate_non_array() {
    let non_array = SymbolicValue::ConstantInt(BigInt::from(42));

    let result = enumerate_array(&non_array);

    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0],
        (vec![], &SymbolicValue::ConstantInt(BigInt::from(42)))
    );
}

#[test]
fn test_enumerate_empty_array() {
    let empty_array = SymbolicValue::Array(vec![]);

    let result = enumerate_array(&empty_array);

    assert_eq!(result.len(), 0);
}
