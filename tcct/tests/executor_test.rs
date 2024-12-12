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

use tcct::executor::debug_ast::DebugExpressionPrefixOpcode;
use tcct::executor::debug_ast::{simplify_statement, DebugExpressionInfixOpcode};
use tcct::executor::symbolic_execution::SymbolicExecutor;
use tcct::executor::symbolic_execution::SymbolicExecutorSetting;
use tcct::executor::symbolic_value::{OwnerName, SymbolicLibrary, SymbolicName, SymbolicValue};
use tcct::input_user::Input;
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

#[test]
fn test_if_else() {
    let path = "../sample/iszero_safe.circom".to_string();
    let prime = BigInt::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let (mut symbolic_library, program_archive) = prepare_symbolic_library(path, prime.clone());
    let setting = SymbolicExecutorSetting {
        prime: prime.clone(),
        propagate_substitution: false,
        skip_initialization_blocks: false,
        off_trace: false,
        keep_track_constraints: true,
        substitute_output: false,
    };

    let mut sexe = SymbolicExecutor::new(&mut symbolic_library, &setting);
    execute(&mut sexe, &program_archive);

    assert_eq!(sexe.symbolic_store.final_states.len(), 2);
    assert_eq!(sexe.symbolic_library.id2name.len(), 5);
    assert!(sexe.symbolic_library.name2id.contains_key("IsZero"));
    assert!(sexe.symbolic_library.name2id.contains_key("in"));
    assert!(sexe.symbolic_library.name2id.contains_key("inv"));
    assert!(sexe.symbolic_library.name2id.contains_key("out"));
    assert!(sexe.symbolic_library.name2id.contains_key("main"));

    let trace_constraints_if_branch = vec![
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
            trace_constraints_if_branch[i],
            *sexe.symbolic_store.final_states[0].trace_constraints[i].clone()
        );
    }
}
