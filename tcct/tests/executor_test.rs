const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use std::rc::Rc;
use std::str::FromStr;

use num_bigint_dig::BigInt;
use num_traits::identities::Zero;
use num_traits::One;
use rustc_hash::FxHashMap;

use program_structure::ast::{Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode};
use program_structure::constants::UsefulConstants;
use program_structure::error_definition::Report;
use program_structure::program_archive::ProgramArchive;

use tcct::executor::debug_ast::{
    simplify_statement, DebugExpressionInfixOpcode, DebugExpressionPrefixOpcode,
};
use tcct::executor::symbolic_execution::{SymbolicExecutor, SymbolicExecutorSetting};
use tcct::executor::symbolic_value::{
    OwnerName, SymbolicAccess, SymbolicLibrary, SymbolicName, SymbolicValue,
};
use tcct::input_user::Input;
use tcct::solver::unused_outputs::check_unused_outputs;
use tcct::solver::utils::VerificationSetting;
use tcct::type_analysis_user::analyse_project;

pub fn parse_project(initial_file: String, prime: BigInt) -> Result<ProgramArchive, ()> {
    let result_program_archive = parser::run_parser(initial_file, VERSION, Vec::new(), &prime);
    match result_program_archive {
        Result::Err((file_library, report_collection)) => {
            Report::print_reports(&report_collection, &file_library);
            Result::Err(())
        }
        Result::Ok((program_archive, warnings)) => {
            Report::print_reports(&warnings, &program_archive.file_library);
            Result::Ok(program_archive)
        }
    }
}

pub fn prepare_symbolic_library(
    initial_file: String,
    prime: BigInt,
) -> (SymbolicLibrary, ProgramArchive) {
    let mut program_archive = parse_project(initial_file, prime.clone()).unwrap();
    let _ = analyse_project(&mut program_archive);

    let mut symbolic_library = SymbolicLibrary {
        template_library: FxHashMap::default(),
        name2id: FxHashMap::default(),
        id2name: FxHashMap::default(),
        function_library: FxHashMap::default(),
        function_counter: FxHashMap::default(),
    };

    for (k, v) in program_archive.templates.clone().into_iter() {
        let body = simplify_statement(&v.get_body().clone());
        symbolic_library.register_template(k.clone(), &body.clone(), v.get_name_of_params());
    }

    for (k, v) in program_archive.functions.clone().into_iter() {
        let body = simplify_statement(&v.get_body().clone());
        symbolic_library.register_function(k.clone(), body.clone(), v.get_name_of_params());
    }

    (symbolic_library, program_archive)
}

pub fn execute(sexe: &mut SymbolicExecutor, program_archive: &ProgramArchive) {
    match &program_archive.initial_template_call {
        Expression::Call { id, args, .. } => {
            let template = program_archive.templates[id].clone();

            sexe.symbolic_library
                .name2id
                .insert("main".to_string(), sexe.symbolic_library.name2id.len());
            sexe.symbolic_library
                .id2name
                .insert(sexe.symbolic_library.name2id["main"], "main".to_string());

            sexe.cur_state.add_owner(&OwnerName {
                name: sexe.symbolic_library.name2id["main"],
                counter: 0,
                access: None,
            });
            sexe.cur_state
                .set_template_id(sexe.symbolic_library.name2id[id]);

            sexe.feed_arguments(template.get_name_of_params(), args);

            let body = sexe.symbolic_library.template_library[&sexe.symbolic_library.name2id[id]]
                .body
                .clone();
            sexe.execute(&body, 0);
        }
        _ => {
            panic!("Cannot Find Main Call");
        }
    }
}

pub fn get_setting(prime: &BigInt) -> SymbolicExecutorSetting {
    SymbolicExecutorSetting {
        prime: prime.clone(),
        propagate_substitution: false,
        skip_initialization_blocks: false,
        only_initialization_blocks: false,
        off_trace: false,
        keep_track_constraints: true,
        substitute_output: false,
    }
}

#[test]
fn test_if_else() {
    let path = "./tests/sample/iszero_safe.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_setting(&prime);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    assert_eq!(sexe.symbolic_store.final_states.len(), 2);
    assert_eq!(sexe.symbolic_library.id2name.len(), 5);
    assert!(sexe.symbolic_library.name2id.contains_key("IsZero"));
    assert!(sexe.symbolic_library.name2id.contains_key("in"));
    assert!(sexe.symbolic_library.name2id.contains_key("inv"));
    assert!(sexe.symbolic_library.name2id.contains_key("out"));
    assert!(sexe.symbolic_library.name2id.contains_key("main"));

    let ground_truth_trace_constraints_if_branch = vec![
        SymbolicValue::UnaryOp(
            DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["in"],
                    owner: Rc::new(vec![OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    }]),
                    access: None,
                })),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::NotEq),
                Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
            )),
        ),
        SymbolicValue::Assign(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["inv"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
            Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["out"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::UnaryOp(
                        DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::Sub),
                        Rc::new(SymbolicValue::Variable(SymbolicName {
                            name: sexe.symbolic_library.name2id["in"],
                            owner: Rc::new(vec![OwnerName {
                                name: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            }]),
                            access: None,
                        })),
                    )),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                    Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
                )),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
            )),
        ),
        SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["in"],
                    owner: Rc::new(vec![OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    }]),
                    access: None,
                })),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["out"],
                    owner: Rc::new(vec![OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    }]),
                    access: None,
                })),
            )),
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
        ),
    ];

    assert_eq!(
        sexe.symbolic_store.final_states[0].trace_constraints.len(),
        4
    );
    for i in 0..4 {
        assert_eq!(
            ground_truth_trace_constraints_if_branch[i],
            *sexe.symbolic_store.final_states[0].trace_constraints[i].clone()
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
    let setting = get_setting(&prime);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_trace_constraints = vec![
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["lt"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["a"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["lt"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["b"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
        ),
    ];

    let owner_name = Rc::new(vec![
        OwnerName {
            name: sexe.symbolic_library.name2id["main"],
            access: None,
            counter: 0,
        },
        OwnerName {
            name: sexe.symbolic_library.name2id["lt"],
            access: None,
            counter: 0,
        },
    ]);
    let in_0 = Rc::new(SymbolicValue::Variable(SymbolicName {
        name: sexe.symbolic_library.name2id["in"],
        owner: owner_name.clone(),
        access: Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::zero()),
        )]),
    }));
    let in_1 = Rc::new(SymbolicValue::Variable(SymbolicName {
        name: sexe.symbolic_library.name2id["in"],
        owner: owner_name.clone(),
        access: Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::one()),
        )]),
    }));
    let lessthan_out = Rc::new(SymbolicValue::Variable(SymbolicName {
        name: sexe.symbolic_library.name2id["out"],
        owner: owner_name.clone(),
        access: None,
    }));
    let cond_1 = SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            lessthan_out.clone(),
        )),
        DebugExpressionInfixOpcode(ExpressionInfixOpcode::BoolAnd),
        Rc::new(SymbolicValue::BinaryOp(
            in_0.clone(),
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Lesser),
            in_1.clone(),
        )),
    );
    let cond_0 = SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            lessthan_out.clone(),
        )),
        DebugExpressionInfixOpcode(ExpressionInfixOpcode::BoolAnd),
        Rc::new(SymbolicValue::BinaryOp(
            in_0,
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::GreaterEq),
            in_1,
        )),
    );
    let cond = SymbolicValue::BinaryOp(
        Rc::new(cond_1),
        DebugExpressionInfixOpcode(ExpressionInfixOpcode::BoolOr),
        Rc::new(cond_0),
    );

    // (BoolOr (BoolAnd (Eq 1 main.lt.out) (Lt main.lt.in[0] main.lt.in[1])) (BoolAnd (Eq 0 main.lt.out) (GEq main.lt.in[0] main.lt.in[1]))),

    for i in 0..ground_truth_trace_constraints.len() {
        assert_eq!(
            ground_truth_trace_constraints[i],
            *sexe.symbolic_store.final_states[0].trace_constraints[i].clone()
        );
    }

    let n = sexe.symbolic_store.final_states[0].trace_constraints.len();
    assert_eq!(
        cond,
        *sexe.symbolic_store.final_states[0].trace_constraints[n - 2].clone()
    );
}

#[test]
fn test_1d_array_component() {
    let path = "./tests/sample/test_1d_array_component.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_setting(&prime);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_trace_constraints = vec![
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["x"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: Some(vec![SymbolicAccess::ArrayAccess(
                            SymbolicValue::ConstantInt(BigInt::zero()),
                        )]),
                        counter: 0,
                    },
                ]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["a"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["x"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: Some(vec![SymbolicAccess::ArrayAccess(
                            SymbolicValue::ConstantInt(BigInt::zero()),
                        )]),
                        counter: 0,
                    },
                ]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["b"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
        ),
        SymbolicValue::Assign(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["y"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: Some(vec![SymbolicAccess::ArrayAccess(
                            SymbolicValue::ConstantInt(BigInt::zero()),
                        )]),
                        counter: 0,
                    },
                ]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            })),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["x"],
                    owner: Rc::new(vec![
                        OwnerName {
                            name: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            name: sexe.symbolic_library.name2id["c"],
                            access: Some(vec![SymbolicAccess::ArrayAccess(
                                SymbolicValue::ConstantInt(BigInt::zero()),
                            )]),
                            counter: 0,
                        },
                    ]),
                    access: Some(vec![SymbolicAccess::ArrayAccess(
                        SymbolicValue::ConstantInt(BigInt::zero()),
                    )]),
                })),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Div),
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["x"],
                    owner: Rc::new(vec![
                        OwnerName {
                            name: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            name: sexe.symbolic_library.name2id["c"],
                            access: Some(vec![SymbolicAccess::ArrayAccess(
                                SymbolicValue::ConstantInt(BigInt::zero()),
                            )]),
                            counter: 0,
                        },
                    ]),
                    access: Some(vec![SymbolicAccess::ArrayAccess(
                        SymbolicValue::ConstantInt(BigInt::one()),
                    )]),
                })),
            )),
        ),
    ];

    for i in 0..ground_truth_trace_constraints.len() {
        assert_eq!(
            ground_truth_trace_constraints[i],
            *sexe.symbolic_store.final_states[0].trace_constraints[i].clone()
        );
    }

    // main.c[0].x[0] = main.a;
    assert_eq!(
        *sexe.symbolic_store.final_states[0].values[&SymbolicName {
            name: sexe.symbolic_library.name2id["x"],
            owner: Rc::new(vec![
                OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                },
                OwnerName {
                    name: sexe.symbolic_library.name2id["c"],
                    access: Some(vec![SymbolicAccess::ArrayAccess(
                        SymbolicValue::ConstantInt(BigInt::zero()),
                    )]),
                    counter: 0,
                },
            ]),
            access: Some(vec![SymbolicAccess::ArrayAccess(
                SymbolicValue::ConstantInt(BigInt::zero()),
            )]),
        }]
            .clone(),
        SymbolicValue::Variable(SymbolicName {
            name: sexe.symbolic_library.name2id["a"],
            owner: Rc::new(vec![OwnerName {
                name: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            access: None,
        })
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
    let setting = get_setting(&prime);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_trace_constraints = vec![
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            })),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["in"],
                    owner: Rc::new(vec![OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    }]),
                    access: None,
                })),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                Rc::new(SymbolicValue::ConstantInt(BigInt::from(1))),
            )),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            })),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["in"],
                    owner: Rc::new(vec![OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    }]),
                    access: None,
                })),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                Rc::new(SymbolicValue::ConstantInt(BigInt::from(2))),
            )),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["out"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: None,
            })),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    Rc::new(SymbolicValue::Variable(SymbolicName {
                        name: sexe.symbolic_library.name2id["in"],
                        owner: Rc::new(vec![
                            OwnerName {
                                name: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                name: sexe.symbolic_library.name2id["c"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        access: Some(vec![SymbolicAccess::ArrayAccess(
                            SymbolicValue::ConstantInt(BigInt::zero()),
                        )]),
                    })),
                )),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["in"],
                    owner: Rc::new(vec![
                        OwnerName {
                            name: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            name: sexe.symbolic_library.name2id["c"],
                            access: None,
                            counter: 0,
                        },
                    ]),
                    access: Some(vec![SymbolicAccess::ArrayAccess(
                        SymbolicValue::ConstantInt(BigInt::one()),
                    )]),
                })),
            )),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["out"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["out"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: None,
            })),
        ),
    ];

    for i in 0..ground_truth_trace_constraints.len() {
        assert_eq!(
            ground_truth_trace_constraints[i],
            *sexe.symbolic_store.final_states[0].trace_constraints[i].clone()
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
    let setting = get_setting(&prime);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_trace_constraints = vec![SymbolicValue::Assign(
        Rc::new(SymbolicValue::Variable(SymbolicName {
            name: sexe.symbolic_library.name2id["out"],
            owner: Rc::new(vec![OwnerName {
                name: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            access: None,
        })),
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::BinaryOp(
                        Rc::new(SymbolicValue::Variable(SymbolicName {
                            name: sexe.symbolic_library.name2id["in"],
                            owner: Rc::new(vec![OwnerName {
                                name: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            }]),
                            access: None,
                        })),
                        DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                        Rc::new(SymbolicValue::ConstantInt(BigInt::from(1))),
                    )),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    Rc::new(SymbolicValue::ConstantInt(BigInt::from(2))),
                )),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Div),
                Rc::new(SymbolicValue::ConstantInt(BigInt::from(3))),
            )),
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add),
            Rc::new(SymbolicValue::ConstantInt(BigInt::from(4))),
        )),
    )];

    for i in 0..ground_truth_trace_constraints.len() {
        assert_eq!(
            ground_truth_trace_constraints[i],
            *sexe.symbolic_store.final_states[0].trace_constraints[i].clone()
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
    let setting = SymbolicExecutorSetting {
        prime: prime.clone(),
        propagate_substitution: false,
        skip_initialization_blocks: false,
        only_initialization_blocks: false,
        off_trace: false,
        keep_track_constraints: true,
        substitute_output: false,
    };

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_trace_constraints = vec![SymbolicValue::AssignEq(
        Rc::new(SymbolicValue::Variable(SymbolicName {
            name: sexe.symbolic_library.name2id["out"],
            owner: Rc::new(vec![OwnerName {
                name: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            access: None,
        })),
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add),
            Rc::new(SymbolicValue::ConstantInt(BigInt::from(8))),
        )),
    )];

    for i in 0..ground_truth_trace_constraints.len() {
        assert_eq!(
            ground_truth_trace_constraints[i],
            *sexe.symbolic_store.final_states[0].trace_constraints[i].clone()
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
    let setting = get_setting(&prime);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_trace_constraints = vec![
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["x"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: Some(vec![
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                ]),
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            })),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["x"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: Some(vec![
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                ]),
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            })),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["x"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: Some(vec![
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                ]),
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::from_str("2").unwrap()),
                )]),
            })),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["x"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: Some(vec![
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                ]),
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::from_str("3").unwrap()),
                )]),
            })),
        ),
        SymbolicValue::Assign(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["y"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["c"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            })),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::Variable(SymbolicName {
                        name: sexe.symbolic_library.name2id["x"],
                        owner: Rc::new(vec![
                            OwnerName {
                                name: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                name: sexe.symbolic_library.name2id["c"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        access: Some(vec![
                            SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                            SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                        ]),
                    })),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    Rc::new(SymbolicValue::Variable(SymbolicName {
                        name: sexe.symbolic_library.name2id["x"],
                        owner: Rc::new(vec![
                            OwnerName {
                                name: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                name: sexe.symbolic_library.name2id["c"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        access: Some(vec![
                            SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                            SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::one())),
                        ]),
                    })),
                )),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Div),
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["x"],
                    owner: Rc::new(vec![
                        OwnerName {
                            name: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            name: sexe.symbolic_library.name2id["c"],
                            access: None,
                            counter: 0,
                        },
                    ]),
                    access: Some(vec![
                        SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                        SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(BigInt::zero())),
                    ]),
                })),
            )),
        ),
    ];

    for i in 0..ground_truth_trace_constraints.len() {
        assert_eq!(
            ground_truth_trace_constraints[i],
            *sexe.symbolic_store.final_states[0].trace_constraints[i].clone()
        );
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
    let setting = get_setting(&prime);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_trace_constraints = vec![SymbolicValue::AssignEq(
        Rc::new(SymbolicValue::Variable(SymbolicName {
            name: sexe.symbolic_library.name2id["out"],
            owner: Rc::new(vec![OwnerName {
                name: sexe.symbolic_library.name2id["main"],
                access: None,
                counter: 0,
            }]),
            access: None,
        })),
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Add),
            Rc::new(SymbolicValue::ConstantInt(BigInt::from(15))),
        )),
    )];

    for i in 0..ground_truth_trace_constraints.len() {
        assert_eq!(
            ground_truth_trace_constraints[i],
            *sexe.symbolic_store.final_states[0].trace_constraints[i].clone()
        );
    }
}

#[test]
fn test_anonymous_component() {
    let path = "./tests/sample/test_anonymous_component.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_setting(&prime);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let ground_truth_trace_constraints = vec![
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["a"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["A_11_163"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: None,
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            })),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["b"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["A_11_163"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: None,
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            })),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["c"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["A_11_163"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: None,
            })),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::ConstantInt(BigInt::from(2))),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                    Rc::new(SymbolicValue::Variable(SymbolicName {
                        name: sexe.symbolic_library.name2id["a"],
                        owner: Rc::new(vec![
                            OwnerName {
                                name: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                name: sexe.symbolic_library.name2id["A_11_163"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        access: None,
                    })),
                )),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["b"],
                    owner: Rc::new(vec![
                        OwnerName {
                            name: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            name: sexe.symbolic_library.name2id["A_11_163"],
                            access: None,
                            counter: 0,
                        },
                    ]),
                    access: None,
                })),
            )),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["out_1"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["c"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["A_11_163"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: None,
            })),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["a"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["A_12_210"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: None,
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::one()),
                )]),
            })),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["b"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["A_12_210"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: None,
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["in"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: Some(vec![SymbolicAccess::ArrayAccess(
                    SymbolicValue::ConstantInt(BigInt::zero()),
                )]),
            })),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["c"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["A_12_210"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: None,
            })),
            Rc::new(SymbolicValue::BinaryOp(
                Rc::new(SymbolicValue::BinaryOp(
                    Rc::new(SymbolicValue::ConstantInt(BigInt::from(3))),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                    Rc::new(SymbolicValue::Variable(SymbolicName {
                        name: sexe.symbolic_library.name2id["a"],
                        owner: Rc::new(vec![
                            OwnerName {
                                name: sexe.symbolic_library.name2id["main"],
                                access: None,
                                counter: 0,
                            },
                            OwnerName {
                                name: sexe.symbolic_library.name2id["A_12_210"],
                                access: None,
                                counter: 0,
                            },
                        ]),
                        access: None,
                    })),
                )),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                Rc::new(SymbolicValue::Variable(SymbolicName {
                    name: sexe.symbolic_library.name2id["b"],
                    owner: Rc::new(vec![
                        OwnerName {
                            name: sexe.symbolic_library.name2id["main"],
                            access: None,
                            counter: 0,
                        },
                        OwnerName {
                            name: sexe.symbolic_library.name2id["A_12_210"],
                            access: None,
                            counter: 0,
                        },
                    ]),
                    access: None,
                })),
            )),
        ),
        SymbolicValue::AssignEq(
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["out_2"],
                owner: Rc::new(vec![OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    access: None,
                    counter: 0,
                }]),
                access: None,
            })),
            Rc::new(SymbolicValue::Variable(SymbolicName {
                name: sexe.symbolic_library.name2id["c"],
                owner: Rc::new(vec![
                    OwnerName {
                        name: sexe.symbolic_library.name2id["main"],
                        access: None,
                        counter: 0,
                    },
                    OwnerName {
                        name: sexe.symbolic_library.name2id["A_12_210"],
                        access: None,
                        counter: 0,
                    },
                ]),
                access: None,
            })),
        ),
    ];

    for i in 0..ground_truth_trace_constraints.len() {
        assert_eq!(
            ground_truth_trace_constraints[i],
            *sexe.symbolic_store.final_states[0].trace_constraints[i].clone()
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

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = get_setting(&prime);

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    let mut main_template_id = "";
    let mut template_param_names = Vec::new();
    let mut template_param_values = Vec::new();
    match &program_archive.initial_template_call {
        Expression::Call { id, args, .. } => {
            main_template_id = id;
            let template = program_archive.templates[id].clone();
            template_param_names = template.get_name_of_params().clone();
            template_param_values = args.clone();
        }
        _ => unimplemented!(),
    }

    let verification_setting = VerificationSetting {
        id: main_template_id.to_string(),
        prime: prime.clone(),
        quick_mode: false,
        progress_interval: 10000,
        template_param_names: template_param_names,
        template_param_values: template_param_values,
    };

    assert!(check_unused_outputs(&mut sexe, &verification_setting).is_some());
}
