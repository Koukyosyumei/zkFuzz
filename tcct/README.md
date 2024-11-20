# Trace-Constraint Consistency Test (TCCT)

- build

```
cargo build
```

- example

```bash
RUST_LOG=debug ./target/debug/tcct ../sample/ifelse.circom --O0
```

```bash
[2024-11-20T20:49:09Z DEBUG tcct]  body: Block { statements: [InitializationBlock { initializations: [] }, InitializationBlock { initializations: [] }, InitializationBlock { initializations: [Declaration { name: "a", dimensions: [], is_constant: true }] }, InitializationBlock { initializations: [Declaration { name: "b", dimensions: [], is_constant: true }] }, InitializationBlock { initializations: [Declaration { name: "out", dimensions: [], is_constant: true }] }, IfThenElse { condition: InfixOp { infix_op: Greater, lhe: Variable { name: "C", access: [] }, rhe: Number { value: BigInt { sign: Plus, data: BigUint { data: [3] } } } }, if_case: Block { statements: [Substitution { variable: "out", access: [], operation: AssignSignal, rhe: InfixOp { infix_op: Div, lhe: Variable { name: "a", access: [] }, rhe: Variable { name: "b", access: [] } } }, ConstraintEquality { lhs_expression: InfixOp { infix_op: Mul, lhe: Variable { name: "b", access: [] }, rhe: Variable { name: "out", access: [] } }, rhs_expression: Variable { name: "a", access: [] } }] }, else_case: Block { statements: [Substitution { variable: "out", access: [], operation: AssignSignal, rhe: InfixOp { infix_op: Div, lhe: InfixOp { infix_op: Mul, lhe: Number { value: BigInt { sign: Plus, data: BigUint { data: [2] } } }, rhe: Variable { name: "a", access: [] } }, rhe: Variable { name: "b", access: [] } } }, ConstraintEquality { lhs_expression: InfixOp { infix_op: Mul, lhe: Variable { name: "b", access: [] }, rhe: Variable { name: "out", access: [] } }, rhs_expression: InfixOp { infix_op: Mul, lhe: Number { value: BigInt { sign: Plus, data: BigUint { data: [2] } } }, rhe: Variable { name: "a", access: [] } } }] } }] }
[2024-11-20T20:49:09Z INFO  tcct] final_state: SymbolicState { values: {"a": a, "out": (Div a b), "b": b}, trace_constraints: [(Greater C 3), (Eq out (Div a b)), (Eq (Mul b (Div a b)) a)], side_constraints: [(Eq (Mul b out) a)], depth: 0 }
[2024-11-20T20:49:09Z INFO  tcct] final_state: SymbolicState { values: {"a": a, "out": (Div (Mul 2 a) b), "b": b}, trace_constraints: [(BoolNot (Greater C 3)), (Eq out (Div (Mul 2 a) b)), (Eq (Mul b (Div (Mul 2 a) b)) (Mul 2 a))], side_constraints: [(Eq (Mul b out) (Mul 2 a))], depth: 0 }
Everything went okay
```

YOu can also check the traces during the symbolic execution by setting `RUST_LOG=trace`:

```bash
RUST_LOG=trace ./target/debug/tcct ../sample/ifelse.circom --O0
```

```
[2024-11-20T20:51:25Z DEBUG tcct]  body: Block { statements: [InitializationBlock { initializations: [] }, InitializationBlock { initializations: [] }, InitializationBlock { initializations: [Declaration { name: "a", dimensions: [], is_constant: true }] }, InitializationBlock { initializations: [Declaration { name: "b", dimensions: [], is_constant: true }] }, InitializationBlock { initializations: [Declaration { name: "out", dimensions: [], is_constant: true }] }, IfThenElse { condition: InfixOp { infix_op: Greater, lhe: Variable { name: "C", access: [] }, rhe: Number { value: BigInt { sign: Plus, data: BigUint { data: [3] } } } }, if_case: Block { statements: [Substitution { variable: "out", access: [], operation: AssignSignal, rhe: InfixOp { infix_op: Div, lhe: Variable { name: "a", access: [] }, rhe: Variable { name: "b", access: [] } } }, ConstraintEquality { lhs_expression: InfixOp { infix_op: Mul, lhe: Variable { name: "b", access: [] }, rhe: Variable { name: "out", access: [] } }, rhs_expression: Variable { name: "a", access: [] } }] }, else_case: Block { statements: [Substitution { variable: "out", access: [], operation: AssignSignal, rhe: InfixOp { infix_op: Div, lhe: InfixOp { infix_op: Mul, lhe: Number { value: BigInt { sign: Plus, data: BigUint { data: [2] } } }, rhe: Variable { name: "a", access: [] } }, rhe: Variable { name: "b", access: [] } } }, ConstraintEquality { lhs_expression: InfixOp { infix_op: Mul, lhe: Variable { name: "b", access: [] }, rhe: Variable { name: "out", access: [] } }, rhs_expression: InfixOp { infix_op: Mul, lhe: Number { value: BigInt { sign: Plus, data: BigUint { data: [2] } } }, rhe: Variable { name: "a", access: [] } } }] } }] }
[2024-11-20T20:51:25Z TRACE tcct::symbolic_execution] cur_bid=0: Block { statements: [InitializationBlock { initializations: [] }, InitializationBlock { initializations: [] }, InitializationBlock { initializations: [Declaration { name: "a", dimensions: [], is_constant: true }] }, InitializationBlock { initializations: [Declaration { name: "b", dimensions: [], is_constant: true }] }, InitializationBlock { initializations: [Declaration { name: "out", dimensions: [], is_constant: true }] }, IfThenElse { condition: InfixOp { infix_op: Greater, lhe: Variable { name: "C", access: [] }, rhe: Number { value: BigInt { sign: Plus, data: BigUint { data: [3] } } } }, if_case: Block { statements: [Substitution { variable: "out", access: [], operation: AssignSignal, rhe: InfixOp { infix_op: Div, lhe: Variable { name: "a", access: [] }, rhe: Variable { name: "b", access: [] } } }, ConstraintEquality { lhs_expression: InfixOp { infix_op: Mul, lhe: Variable { name: "b", access: [] }, rhe: Variable { name: "out", access: [] } }, rhs_expression: Variable { name: "a", access: [] } }] }, else_case: Block { statements: [Substitution { variable: "out", access: [], operation: AssignSignal, rhe: InfixOp { infix_op: Div, lhe: InfixOp { infix_op: Mul, lhe: Number { value: BigInt { sign: Plus, data: BigUint { data: [2] } } }, rhe: Variable { name: "a", access: [] } }, rhe: Variable { name: "b", access: [] } } }, ConstraintEquality { lhs_expression: InfixOp { infix_op: Mul, lhe: Variable { name: "b", access: [] }, rhe: Variable { name: "out", access: [] } }, rhs_expression: InfixOp { infix_op: Mul, lhe: Number { value: BigInt { sign: Plus, data: BigUint { data: [2] } } }, rhe: Variable { name: "a", access: [] } } }] } }] }
[2024-11-20T20:51:25Z TRACE tcct::symbolic_execution] cur_bid=0: InitializationBlock { initializations: [] }
[2024-11-20T20:51:25Z TRACE tcct::symbolic_execution] cur_bid=1: InitializationBlock { initializations: [] }
[2024-11-20T20:51:25Z TRACE tcct::symbolic_execution] cur_bid=2: InitializationBlock { initializations: [Declaration { name: "a", dimensions: [], is_constant: true }] }
[2024-11-20T20:51:25Z TRACE tcct::symbolic_execution] cur_bid=0: Declaration { name: "a", dimensions: [], is_constant: true }
[2024-11-20T20:51:25Z TRACE tcct::symbolic_execution] cur_bid=3: InitializationBlock { initializations: [Declaration { name: "b", dimensions: [], is_constant: true }] }
[2024-11-20T20:51:25Z TRACE tcct::symbolic_execution] cur_bid=0: Declaration { name: "b", dimensions: [], is_constant: true }
[2024-11-20T20:51:25Z TRACE tcct::symbolic_execution] cur_bid=4: InitializationBlock { initializations: [Declaration { name: "out", dimensions: [], is_constant: true }] }
.
.
.
```

