mod utils;

use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;
use num_traits::identities::Zero;
use num_traits::One;

use program_structure::ast::{Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode};

use zkfuzz::executor::debug_ast::{
    DebuggableExpressionInfixOpcode, DebuggableExpressionPrefixOpcode,
};
use zkfuzz::executor::symbolic_execution::SymbolicExecutor;
use zkfuzz::executor::symbolic_setting::get_default_setting_for_symbolic_execution;
use zkfuzz::executor::symbolic_value::{OwnerName, SymbolicAccess, SymbolicName, SymbolicValue};
use zkfuzz::mutator::unused_outputs::check_unused_outputs;
use zkfuzz::mutator::utils::BaseVerificationConfig;

use crate::utils::{execute, prepare_symbolic_library};

#[test]
fn test_if_else() {
    let path = "./tests/sample/test_if_else.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    assert_eq!(sexe.symbolic_library.id2name.len(), 5);
    assert!(sexe.symbolic_library.name2id.contains_key("IsZero"));
    assert!(sexe.symbolic_library.name2id.contains_key("in"));
    assert!(sexe.symbolic_library.name2id.contains_key("inv"));
    assert!(sexe.symbolic_library.name2id.contains_key("out"));
    assert!(sexe.symbolic_library.name2id.contains_key("main"));

    let ground_truth_symbolic_trace_if_branch = vec![
        SymbolicValue::Assign(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["inv"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
            Rc::new(SymbolicValue::Conditional(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::Variable(SymbolicName::new(
                        sexe.symbolic_library.name2id["in"],
                        Rc::new(vec![OwnerName {
                            id: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        }]),
                        None,
                    ))),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::NotEq),
                    Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
                )),
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Div),
                    Rc::new(SymbolicValue::Variable(SymbolicName::new(
                        sexe.symbolic_library.name2id["in"],
                        Rc::new(vec![OwnerName {
                            id: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        }]),
                        None,
                    ))),
                )),
                Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
            )),
            false,
            None,
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["out"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::UnaryOp(
                        DebuggableExpressionPrefixOpcode(ExpressionPrefixOpcode::Sub),
                        Rc::new(SymbolicValue::Variable(SymbolicName::new(
                            sexe.symbolic_library.name2id["in"],
                            Rc::new(vec![OwnerName {
                                id: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            }]),
                            None,
                        ))),
                    )),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                    Rc::new(SymbolicValue::Variable(SymbolicName::new(
                        sexe.symbolic_library.name2id["inv"],
                        Rc::new(vec![OwnerName {
                            id: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        }]),
                        None,
                    ))),
                )),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
            )),
        ),
        SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["in"],
                    Rc::new(vec![OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    }]),
                    None,
                ))),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["out"],
                    Rc::new(vec![OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    }]),
                    None,
                ))),
            )),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
        ),
    ];

    assert_eq!(sexe.cur_state.symbolic_trace.len(), 3);
    for i in 0..3 {
        assert_eq!(
            ground_truth_symbolic_trace_if_branch[i],
            *sexe.cur_state.symbolic_trace[i].clone()
        );
    }
}

#[test]
fn test_lessthan() {
    let path = "./tests/sample/test_lessthan.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_symbolic_trace = vec![
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["lt"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["a"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["lt"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["b"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
        ),
    ];

    let owner_name = Rc::new(vec![
        OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        },
        OwnerName {
            id: sexe.symbolic_library.name2id["lt"],
            access: None,
            counter: 0,
        },
    ]);
    let in_0 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["in"],
        owner_name.clone(),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::zero()),
        )]),
    )));
    let in_1 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["in"],
        owner_name.clone(),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::one()),
        )]),
    )));
    let lessthan_out = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["out"],
        owner_name.clone(),
        None,
    )));
    let cond_1 = SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            lessthan_out.clone(),
        )),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BoolAnd),
        Rc::new(SymbolicValue::AuxBinaryOp(
            in_0.clone(),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Lesser),
            in_1.clone(),
        )),
    );
    let cond_0 = SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            lessthan_out.clone(),
        )),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BoolAnd),
        Rc::new(SymbolicValue::AuxBinaryOp(
            in_0,
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::GreaterEq),
            in_1,
        )),
    );
    let cond = SymbolicValue::BinaryOp(
        Rc::new(cond_1),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BoolOr),
        Rc::new(cond_0),
    );

    // (BoolOr (BoolAnd (Eq 1 main.lt.out) (Lt main.lt.in[0] main.lt.in[1])) (BoolAnd (Eq 0 main.lt.out) (GEq main.lt.in[0] main.lt.in[1]))),

    for i in 0..ground_truth_symbolic_trace.len() {
        assert_eq!(
            ground_truth_symbolic_trace[i],
            *sexe.cur_state.symbolic_trace[i].clone()
        );
    }

    let n = sexe.cur_state.symbolic_trace.len();
    assert_eq!(cond, *sexe.cur_state.symbolic_trace[n - 2].clone());
}

#[test]
fn test_1d_array_component() {
    let path = "./tests/sample/test_1d_array_component.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_symbolic_trace = vec![
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["x"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: Some(vec![SymbolicAccess::ArrayAccess(
                            SymbolicValue::ConstantInt(BigInt::zero()),
                        )]),
                        counter: 0,
                    },
                ]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["a"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["x"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: Some(vec![SymbolicAccess::ArrayAccess(
                            SymbolicValue::ConstantInt(BigInt::zero()),
                        )]),
                        counter: 0,
                    },
                ]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["b"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
        ),
        SymbolicValue::Assign(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["y"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: Some(vec![SymbolicAccess::ArrayAccess(
                            SymbolicValue::ConstantInt(BigInt::zero()),
                        )]),
                        counter: 0,
                    },
                ]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            ))),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["x"],
                    Rc::new(vec![
                        OwnerName {
                            id: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            id: sexe.symbolic_library.name2id["c"],
                            access: Some(vec![SymbolicAccess::ArrayAccess(
                                SymbolicValue::ConstantInt(BigInt::zero()),
                            )]),
                            counter: 0,
                        },
                    ]),
                    Some(vec![SymbolicAccess::ArrayAccess(
                        SymbolicValue::ConstantInt(BigInt::zero()),
                    )]),
                ))),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Div),
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["x"],
                    Rc::new(vec![
                        OwnerName {
                            id: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            id: sexe.symbolic_library.name2id["c"],
                            access: Some(vec![SymbolicAccess::ArrayAccess(
                                SymbolicValue::ConstantInt(BigInt::zero()),
                            )]),
                            counter: 0,
                        },
                    ]),
                    Some(vec![SymbolicAccess::ArrayAccess(
                        SymbolicValue::ConstantInt(BigInt::one()),
                    )]),
                ))),
            )),
            false,
            None,
        ),
    ];

    let mut results = Vec::new();
    for s in sexe.cur_state.symbolic_trace {
        if let SymbolicValue::Assign(a, b, c, Some(_)) = s.as_ref() {
            results.push(Rc::new(SymbolicValue::Assign(
                a.clone(),
                b.clone(),
                c.clone(),
                None,
            )));
        } else {
            results.push(s.clone());
        }
    }

    for i in 0..ground_truth_symbolic_trace.len() {
        assert_eq!(ground_truth_symbolic_trace[i], *results[i + 1].clone());
    }

    // main.c[0].x[0] = main.a;
    assert_eq!(
        *sexe.cur_state.symbol_binding_map[&SymbolicName::new(
            sexe.symbolic_library.name2id["x"],
            Rc::new(vec![
                OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                },
                OwnerName {
                    id: sexe.symbolic_library.name2id["c"],
                    access: Some(vec![SymbolicAccess::ArrayAccess(
                        SymbolicValue::ConstantInt(BigInt::zero()),
                    )]),
                    counter: 0,
                },
            ]),
            Some(vec![SymbolicAccess::ArrayAccess(
                SymbolicValue::ConstantInt(BigInt::zero()),
            )]),
        )]
            .clone(),
        SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["a"],
            Rc::new(vec![OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            None,
        ))
    );
}

#[test]
fn test_array_signal_initialization() {
    let path = "./tests/sample/test_array_signal_initialization.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_symbolic_trace = vec![
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            ))),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["in"],
                    Rc::new(vec![OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    }]),
                    None,
                ))),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                Rc::new(SymbolicValue::ConstantInt(BigInt::from(1))),
            )),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            ))),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["in"],
                    Rc::new(vec![OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    }]),
                    None,
                ))),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                Rc::new(SymbolicValue::ConstantInt(BigInt::from(2))),
            )),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["out"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                None,
            ))),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    Rc::new(SymbolicValue::Variable(SymbolicName::new(
                        sexe.symbolic_library.name2id["in"],
                        Rc::new(vec![
                            OwnerName {
                                id: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                id: sexe.symbolic_library.name2id["c"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        Some(vec![SymbolicAccess::ArrayAccess(
                            SymbolicValue::ConstantInt(BigInt::zero()),
                        )]),
                    ))),
                )),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["in"],
                    Rc::new(vec![
                        OwnerName {
                            id: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            id: sexe.symbolic_library.name2id["c"],
                            access: None,
                            counter: 0,
                        },
                    ]),
                    Some(vec![SymbolicAccess::ArrayAccess(
                        SymbolicValue::ConstantInt(BigInt::one()),
                    )]),
                ))),
            )),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["out"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["out"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                None,
            ))),
        ),
    ];

    for i in 0..ground_truth_symbolic_trace.len() {
        assert_eq!(
            ground_truth_symbolic_trace[i],
            *sexe.cur_state.symbolic_trace[i].clone()
        );
    }
}

#[test]
fn test_2d_array_var() {
    let path = "./tests/sample/test_2d_array_var.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_symbolic_trace = vec![SymbolicValue::Assign(
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["out"],
            Rc::new(vec![OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            None,
        ))),
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::BinaryOp(
                        Rc::new(SymbolicValue::Variable(SymbolicName::new(
                            sexe.symbolic_library.name2id["in"],
                            Rc::new(vec![OwnerName {
                                id: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            }]),
                            None,
                        ))),
                        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                        Rc::new(SymbolicValue::ConstantInt(BigInt::from(1))),
                    )),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    Rc::new(SymbolicValue::ConstantInt(BigInt::from(2))),
                )),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Div),
                Rc::new(SymbolicValue::ConstantInt(BigInt::from(3))),
            )),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
            Rc::new(SymbolicValue::ConstantInt(BigInt::from(4))),
        )),
        false,
        None,
    )];

    for i in 0..ground_truth_symbolic_trace.len() {
        assert_eq!(
            ground_truth_symbolic_trace[i],
            *sexe.cur_state.symbolic_trace[i].clone()
        );
    }
}

#[test]
fn test_multidimensional_array_function() {
    let path = "./tests/sample/test_multidimensional_array_function.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_symbolic_trace = vec![SymbolicValue::AssignEq(
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["out"],
            Rc::new(vec![OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            None,
        ))),
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
            Rc::new(SymbolicValue::ConstantInt(BigInt::from(8))),
        )),
    )];

    for i in 0..ground_truth_symbolic_trace.len() {
        assert_eq!(
            ground_truth_symbolic_trace[i],
            *sexe.cur_state.symbolic_trace[i].clone()
        );
    }
}

#[test]
fn test_2d_array_component() {
    let path = "./tests/sample/test_2d_array_component.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_symbolic_trace = vec![
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["x"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                ]),
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            ))),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["x"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                ]),
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            ))),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["x"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                ]),
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::from_str("2").unwrap()),
                )]),
            ))),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["x"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                ]),
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::from_str("3").unwrap()),
                )]),
            ))),
        ),
        SymbolicValue::Assign(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["y"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            ))),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::Variable(SymbolicName::new(
                        sexe.symbolic_library.name2id["x"],
                        Rc::new(vec![
                            OwnerName {
                                id: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                id: sexe.symbolic_library.name2id["c"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        Some(vec![
                            SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                            SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                        ]),
                    ))),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    Rc::new(SymbolicValue::Variable(SymbolicName::new(
                        sexe.symbolic_library.name2id["x"],
                        Rc::new(vec![
                            OwnerName {
                                id: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                id: sexe.symbolic_library.name2id["c"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        Some(vec![
                            SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                            SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                        ]),
                    ))),
                )),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Div),
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["x"],
                    Rc::new(vec![
                        OwnerName {
                            id: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            id: sexe.symbolic_library.name2id["c"],
                            access: None,
                            counter: 0,
                        },
                    ]),
                    Some(vec![
                        SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                        SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                    ]),
                ))),
            )),
            false,
            None,
        ),
    ];

    let mut results = Vec::new();
    for s in sexe.cur_state.symbolic_trace {
        if let SymbolicValue::Assign(a, b, c, Some(_)) = s.as_ref() {
            results.push(Rc::new(SymbolicValue::Assign(
                a.clone(),
                b.clone(),
                c.clone(),
                None,
            )));
        } else {
            results.push(s.clone());
        }
    }

    for i in 0..ground_truth_symbolic_trace.len() {
        assert_eq!(ground_truth_symbolic_trace[i], *results[i].clone());
    }
}

#[test]
fn test_recursive_function() {
    let path = "./tests/sample/test_recursive_function.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_symbolic_trace = vec![SymbolicValue::AssignEq(
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["out"],
            Rc::new(vec![OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            None,
        ))),
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
            Rc::new(SymbolicValue::ConstantInt(BigInt::from(15))),
        )),
    )];

    for i in 0..ground_truth_symbolic_trace.len() {
        assert_eq!(
            ground_truth_symbolic_trace[i],
            *sexe.cur_state.symbolic_trace[i].clone()
        );
    }
}

#[test]
fn test_bulk_assignment() {
    let path = "./tests/sample/test_bulk_assignment.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_trace_constraint_1 = SymbolicValue::AssignEq(
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["in"],
            Rc::new(vec![
                OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                },
                OwnerName {
                    id: sexe.symbolic_library.name2id["c"],
                    access: None,
                    counter: 0,
                },
            ]),
            Some(vec![
                SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
            ]),
        ))),
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["in"],
            Rc::new(vec![OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            Some(vec![
                SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
            ]),
        ))),
    );

    let ground_truth_trace_constraint_2 = SymbolicValue::AssignEq(
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["out"],
            Rc::new(vec![
                OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                },
                OwnerName {
                    id: sexe.symbolic_library.name2id["c"],
                    access: None,
                    counter: 0,
                },
            ]),
            None,
        ))),
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    Rc::new(SymbolicValue::Variable(SymbolicName::new(
                        sexe.symbolic_library.name2id["in"],
                        Rc::new(vec![
                            OwnerName {
                                id: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                id: sexe.symbolic_library.name2id["c"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        Some(vec![
                            SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                            SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                        ]),
                    ))),
                )),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["in"],
                    Rc::new(vec![
                        OwnerName {
                            id: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            id: sexe.symbolic_library.name2id["c"],
                            access: None,
                            counter: 0,
                        },
                    ]),
                    Some(vec![
                        SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                        SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                    ]),
                ))),
            )),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::from(2))),
                ]),
            ))),
        )),
    );

    assert_eq!(
        ground_truth_trace_constraint_1,
        *sexe.cur_state.symbolic_trace[1].clone()
    );
    assert_eq!(
        ground_truth_trace_constraint_2,
        *sexe.cur_state.symbolic_trace[7].clone()
    );
}

#[test]
fn test_array_template_argument() {
    let path = "./tests/sample/test_array_template_argument.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let thrid_cond = SymbolicValue::AssignEq(
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["out"],
            Rc::new(vec![
                OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                },
                OwnerName {
                    id: sexe.symbolic_library.name2id["A"],
                    access: None,
                    counter: 0,
                },
            ]),
            Some(vec![SymbolicAccess::ArrayAccess(
                SymbolicValue::ConstantInt(BigInt::zero()),
            )]),
        ))),
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["A"],
                        access: None,
                        counter: 0,
                    },
                ]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            ))),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
            Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
        )),
    );

    assert_eq!(thrid_cond, *sexe.cur_state.symbolic_trace[2].clone());
}

#[test]
fn test_anonymous_component() {
    let path = "./tests/sample/test_anonymous_component.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_symbolic_trace = vec![
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["a"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["A_11_163"],
                        access: None,
                        counter: 0,
                    },
                ]),
                None,
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            ))),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["b"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["A_11_163"],
                        access: None,
                        counter: 0,
                    },
                ]),
                None,
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            ))),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["c"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["A_11_163"],
                        access: None,
                        counter: 0,
                    },
                ]),
                None,
            ))),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::ConstantInt(BigInt::from(2))),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                    Rc::new(SymbolicValue::Variable(SymbolicName::new(
                        sexe.symbolic_library.name2id["a"],
                        Rc::new(vec![
                            OwnerName {
                                id: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                id: sexe.symbolic_library.name2id["A_11_163"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        None,
                    ))),
                )),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["b"],
                    Rc::new(vec![
                        OwnerName {
                            id: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            id: sexe.symbolic_library.name2id["A_11_163"],
                            access: None,
                            counter: 0,
                        },
                    ]),
                    None,
                ))),
            )),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["out_1"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["c"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["A_11_163"],
                        access: None,
                        counter: 0,
                    },
                ]),
                None,
            ))),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["a"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["A_12_210"],
                        access: None,
                        counter: 0,
                    },
                ]),
                None,
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            ))),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["b"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["A_12_210"],
                        access: None,
                        counter: 0,
                    },
                ]),
                None,
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            ))),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["c"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["A_12_210"],
                        access: None,
                        counter: 0,
                    },
                ]),
                None,
            ))),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::ConstantInt(BigInt::from(3))),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                    Rc::new(SymbolicValue::Variable(SymbolicName::new(
                        sexe.symbolic_library.name2id["a"],
                        Rc::new(vec![
                            OwnerName {
                                id: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                id: sexe.symbolic_library.name2id["A_12_210"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        None,
                    ))),
                )),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["b"],
                    Rc::new(vec![
                        OwnerName {
                            id: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            id: sexe.symbolic_library.name2id["A_12_210"],
                            access: None,
                            counter: 0,
                        },
                    ]),
                    None,
                ))),
            )),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["out_2"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                None,
            ))),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["c"],
                Rc::new(vec![
                    OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        id: sexe.symbolic_library.name2id["A_12_210"],
                        access: None,
                        counter: 0,
                    },
                ]),
                None,
            ))),
        ),
    ];

    for i in 0..ground_truth_symbolic_trace.len() {
        assert_eq!(
            ground_truth_symbolic_trace[i],
            *sexe.cur_state.symbolic_trace[i + 1].clone()
        );
    }
}

#[test]
fn test_branch_within_callee() {
    let path = "./tests/sample/test_branch_within_callee.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    assert_eq!(sexe.cur_state.symbolic_trace.len(), 5)
}

#[test]
fn test_array_dimension_calculation_within_callee() {
    let path = "./tests/sample/test_array_dimension_calculation_within_callee.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    assert_eq!(sexe.cur_state.symbolic_trace.len(), 4);
    assert_eq!(sexe.cur_state.side_constraints.len(), 4)
}

#[test]
fn test_multi_substitution() {
    let path = "./tests/sample/test_multi_substitution.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    assert_eq!(sexe.cur_state.symbolic_trace.len(), 11);
    assert_eq!(sexe.cur_state.side_constraints.len(), 11)
}

#[test]
fn test_array_processing_and_nested_output() {
    let path = "./tests/sample//test_array_processing_and_nested_output.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let first_cond = Rc::new(SymbolicValue::AssignEq(
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["a"],
            Rc::new(vec![
                OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                },
                OwnerName {
                    id: sexe.symbolic_library.name2id["Callee_17_271"],
                    access: None,
                    counter: 0,
                },
            ]),
            Some(vec![SymbolicAccess::ArrayAccess(
                SymbolicValue::ConstantInt(BigInt::zero()),
            )]),
        ))),
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["x"],
            Rc::new(vec![OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            Some(vec![SymbolicAccess::ArrayAccess(
                SymbolicValue::ConstantInt(BigInt::zero()),
            )]),
        ))),
    ));

    assert_eq!(sexe.cur_state.symbolic_trace[0], first_cond);
    assert_eq!(sexe.cur_state.side_constraints[0], first_cond);

    assert_eq!(sexe.cur_state.symbolic_trace.len(), 12);
    assert_eq!(sexe.cur_state.side_constraints.len(), 12)
}

#[test]
fn test_nested_inline_calls_and_multidim_returns() {
    let path = "./tests/sample//test_nested_inline_calls_and_multidim_returns.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    assert_eq!(sexe.cur_state.symbolic_trace.len(), 22);
    assert_eq!(sexe.cur_state.side_constraints.len(), 22)
}

#[test]
fn test_inline_call_bulk_return_assignment() {
    let path = "./tests/sample/test_inline_call_bulk_return_assignment.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let x_0 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["x"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::zero()),
        )]),
    )));

    let x_1 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["x"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::one()),
        )]),
    )));

    let x0_0 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["x0"],
        Rc::new(vec![
            OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            },
            OwnerName {
                id: sexe.symbolic_library.name2id["Callee_17_283"],
                access: None,
                counter: 0,
            },
        ]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::zero()),
        )]),
    )));

    let x0_1 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["x0"],
        Rc::new(vec![
            OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            },
            OwnerName {
                id: sexe.symbolic_library.name2id["Callee_17_283"],
                access: None,
                counter: 0,
            },
        ]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::one()),
        )]),
    )));

    let out0_0 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["out0"],
        Rc::new(vec![
            OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            },
            OwnerName {
                id: sexe.symbolic_library.name2id["Callee_17_283"],
                access: None,
                counter: 0,
            },
        ]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::zero()),
        )]),
    )));

    let out0_1 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["out0"],
        Rc::new(vec![
            OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            },
            OwnerName {
                id: sexe.symbolic_library.name2id["Callee_17_283"],
                access: None,
                counter: 0,
            },
        ]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::one()),
        )]),
    )));

    let out_0 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["out"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::zero()),
        )]),
    )));

    let out_1 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["out"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::one()),
        )]),
    )));

    let ground_truth_constraints = vec![
        SymbolicValue::AssignEq(x0_0.clone(), x_0.clone()),
        SymbolicValue::AssignEq(x0_1.clone(), x_1.clone()),
        SymbolicValue::AssignEq(
            out0_0.clone(),
            Rc::new(SymbolicValue::BinaryOp(
                x0_0.clone(),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                x0_1.clone(),
            )),
        ),
        SymbolicValue::AssignEq(
            out0_1.clone(),
            Rc::new(SymbolicValue::BinaryOp(
                x0_0.clone(),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
                x0_1.clone(),
            )),
        ),
        SymbolicValue::AssignEq(out_0.clone(), out0_0.clone()),
        SymbolicValue::AssignEq(out_1.clone(), out0_1.clone()),
    ];

    assert_eq!(sexe.cur_state.symbolic_trace.len(), 6);
    assert_eq!(sexe.cur_state.side_constraints.len(), 6);

    for i in 0..ground_truth_constraints.len() {
        assert_eq!(
            ground_truth_constraints[i],
            *sexe.cur_state.symbolic_trace[i].clone()
        );
        assert_eq!(
            ground_truth_constraints[i],
            *sexe.cur_state.side_constraints[i].clone()
        );
    }
}

#[test]
fn test_one_line_call() {
    let path = "./tests/sample/test_one_line_call.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let main_a_11_152_a = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["a"],
        Rc::new(vec![
            OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            },
            OwnerName {
                id: sexe.symbolic_library.name2id["A_11_152"],
                access: None,
                counter: 0,
            },
        ]),
        None,
    )));

    let main_a_11_152_b = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["b"],
        Rc::new(vec![
            OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            },
            OwnerName {
                id: sexe.symbolic_library.name2id["A_11_152"],
                access: None,
                counter: 0,
            },
        ]),
        None,
    )));

    let main_a_11_152_c = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["c"],
        Rc::new(vec![
            OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            },
            OwnerName {
                id: sexe.symbolic_library.name2id["A_11_152"],
                access: None,
                counter: 0,
            },
        ]),
        None,
    )));

    let main_n = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["n"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    )));

    let main_in_0 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["in"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::zero()),
        )]),
    )));

    let main_in_1 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["in"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::one()),
        )]),
    )));

    let main_out = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["out"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    )));

    let ground_truth_constraints = vec![
        SymbolicValue::AssignTemplParam(
            main_n.clone(),
            Rc::new(SymbolicValue::ConstantInt(BigInt::from_str("2").unwrap())),
        ),
        SymbolicValue::AssignEq(main_a_11_152_a.clone(), main_in_0.clone()),
        SymbolicValue::AssignEq(main_a_11_152_b.clone(), main_in_1.clone()),
        SymbolicValue::AssignEq(
            main_a_11_152_c.clone(),
            Rc::new(SymbolicValue::BinaryOp(
                main_a_11_152_a,
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                main_a_11_152_b,
            )),
        ),
        SymbolicValue::AssignEq(main_out, main_a_11_152_c),
    ];

    for i in 0..ground_truth_constraints.len() {
        assert_eq!(
            ground_truth_constraints[i],
            *sexe.cur_state.symbolic_trace[i].clone()
        );
        assert_eq!(
            ground_truth_constraints[i],
            *sexe.cur_state.side_constraints[i].clone()
        );
    }
}

#[test]
fn test_one_line_call_array() {
    let path = "./tests/sample/test_one_line_call_array.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let main_callee_13_217_in_0 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["in"],
        Rc::new(vec![
            OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            },
            OwnerName {
                id: sexe.symbolic_library.name2id["Callee_13_217"],
                access: None,
                counter: 0,
            },
        ]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::zero()),
        )]),
    )));

    let main_callee_13_217_in_1 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["in"],
        Rc::new(vec![
            OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            },
            OwnerName {
                id: sexe.symbolic_library.name2id["Callee_13_217"],
                access: None,
                counter: 0,
            },
        ]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::one()),
        )]),
    )));

    let main_callee_13_217_out = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["out"],
        Rc::new(vec![
            OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            },
            OwnerName {
                id: sexe.symbolic_library.name2id["Callee_13_217"],
                access: None,
                counter: 0,
            },
        ]),
        None,
    )));

    let main_a = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["a"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    )));

    let main_b = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["b"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    )));

    let main_c = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["c"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        None,
    )));

    let ground_truth_constraints = vec![
        SymbolicValue::AssignEq(main_callee_13_217_in_0.clone(), main_a),
        SymbolicValue::AssignEq(main_callee_13_217_in_1.clone(), main_b),
        SymbolicValue::AssignEq(
            main_callee_13_217_out.clone(),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::ConstantInt(BigInt::from_str("3").unwrap())),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                Rc::new(SymbolicValue::BinaryOp(
                    main_callee_13_217_in_0,
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    main_callee_13_217_in_1,
                )),
            )),
        ),
        SymbolicValue::AssignEq(main_c, main_callee_13_217_out),
    ];

    for i in 0..ground_truth_constraints.len() {
        assert_eq!(
            ground_truth_constraints[i],
            *sexe.cur_state.symbolic_trace[i].clone()
        );
        assert_eq!(
            ground_truth_constraints[i],
            *sexe.cur_state.side_constraints[i].clone()
        );
    }
}

#[test]
fn test_unused_outputs() {
    let path = "./tests/sample/test_unused_output.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();
    let range = BigInt::from(100);

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime.clone(), false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let (main_template_name, template_param_names, template_param_values) =
        match &program_archive.initial_template_call {
            Expression::Call { id, args, .. } => {
                let template = &program_archive.templates[id];
                (id, template.get_name_of_params().clone(), args.clone())
            }
            _ => unimplemented!(),
        };

    let verification_setting = BaseVerificationConfig {
        target_template_name: main_template_name.to_string(),
        prime: prime.clone(),
        range: range.clone(),
        quick_mode: false,
        heuristics_mode: false,
        progress_interval: 10000,
        template_param_names: template_param_names,
        template_param_values: template_param_values,
    };

    assert!(check_unused_outputs(&mut sexe, &verification_setting).is_some());
}

#[test]
fn test_assign_nested_array() {
    let path = "./tests/sample/test_assign_nested_array.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let last_cond = SymbolicValue::AssignEq(
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["out"],
            Rc::new(vec![OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            Some(vec![
                SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::from(2))),
            ]),
        ))),
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["y"],
            Rc::new(vec![
                OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                },
                OwnerName {
                    id: sexe.symbolic_library.name2id["c4"],
                    access: None,
                    counter: 0,
                },
            ]),
            Some(vec![SymbolicAccess::ArrayAccess(
                SymbolicValue::ConstantInt(BigInt::from(2)),
            )]),
        ))),
    );

    assert_eq!(**sexe.cur_state.symbolic_trace.last().unwrap(), last_cond);
    assert_eq!(**sexe.cur_state.side_constraints.last().unwrap(), last_cond);
}

#[test]
fn test_signal_var_constraint() {
    let path = "./tests/sample/test_signal_var_constraint.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let pzbb_0 = SymbolicValue::Variable(SymbolicName::new(
        sexe.symbolic_library.name2id["pzbb"],
        Rc::new(vec![OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        }]),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::zero()),
        )]),
    ));

    let sub_1_pzbb_0 = Rc::new(SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
        Rc::new(pzbb_0.clone()),
    ));
    let mul_sub_1_pzbb_0_pzbb_0 = Rc::new(SymbolicValue::BinaryOp(
        sub_1_pzbb_0,
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        Rc::new(pzbb_0),
    ));

    let cond = SymbolicValue::BinaryOp(
        mul_sub_1_pzbb_0_pzbb_0,
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
        Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
    );

    assert_eq!(*sexe.cur_state.side_constraints[0], cond);
}

#[test]
fn test_var_bulk_assignment() {
    let path = "./tests/sample/test_var_bulk_assignment.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_default_setting_for_symbolic_execution(prime, false);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let first_cond = SymbolicValue::AssignEq(
        Rc::new(SymbolicValue::Variable(SymbolicName::new(
            sexe.symbolic_library.name2id["out"],
            Rc::new(vec![OwnerName {
                id: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            None,
        ))),
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                Rc::new(SymbolicValue::Variable(SymbolicName::new(
                    sexe.symbolic_library.name2id["in"],
                    Rc::new(vec![OwnerName {
                        id: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    }]),
                    Some(vec![
                        SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                        SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                    ]),
                ))),
            )),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
            Rc::new(SymbolicValue::Variable(SymbolicName::new(
                sexe.symbolic_library.name2id["in"],
                Rc::new(vec![OwnerName {
                    id: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                Some(vec![
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                ]),
            ))),
        )),
    );

    assert_eq!(*sexe.cur_state.symbolic_trace[0], first_cond);
    assert_eq!(*sexe.cur_state.side_constraints[0], first_cond);
}
