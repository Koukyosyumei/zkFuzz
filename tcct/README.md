# Trace-Constraint Consistency Test (TCCT)

This tool is designed to extract and analyze the trace constraints ($\mathcal{T}(\mathcal{P})$) and side constraints ($\mathcal{S}(\mathcal{C})$) from ZKP circuits written in Circom.

## Bbuild

To compile the tool, run:

```bash
cargo build
```

## Usage Example

TCCT provides multiple verbosity levels for detailed analysis:

`warn`: Outputs warnings and basic statistics about the trace and side constraints.
`info`: Includes everything from `warn` and adds details about all possible finite states.
`debug`: Includes everything from `info` and adds the full AST (Abstract Syntax Tree).
`trace`: Includes everything from `debug` and outputs all intermediate trace states during execution.

To analyze a sample Circom circuit, use the following command:

```bash
RUST_LOG=trace ./target/debug/tcct ../sample/iszero_safe.circom
```

Example output:

```bash
[2024-11-21T15:59:54Z DEBUG tcct] body:
    Block (elem_id=0):
        -------------------------------
        InitializationBlock (elem_id=0):
          Initializations::
        -------------------------------
        InitializationBlock (elem_id=0):
          Initializations::
        -------------------------------
        InitializationBlock (elem_id=1):
          Initializations::
            Declaration (elem_id=2):
              Name: in
              Dimensions::
              Is Constant: true
        -------------------------------
        InitializationBlock (elem_id=3):
          Initializations::
            Declaration (elem_id=4):
              Name: out
              Dimensions::
              Is Constant: true
        -------------------------------
        InitializationBlock (elem_id=5):
          Initializations::
            Declaration (elem_id=6):
              Name: inv
              Dimensions::
              Is Constant: true
        -------------------------------
        IfThenElse (elem_id=8):
          Condition::
            InfixOp:
              Operator: NotEq
              Left-Hand Expression:
                Variable:
                  Name: in
                  Access: []
              Right-Hand Expression:
                Number: 0
          If Case::
            Substitution (elem_id=18446744073709551599):
              Variable: inv
              Access: []
              Operation: AssignSignal
              Right-Hand Expression::
                InfixOp:
                  Operator: Div
                  Left-Hand Expression:
                    Number: 1
                  Right-Hand Expression:
                    Variable:
                      Name: in
                      Access: []
          Else Case::
            Substitution (elem_id=18446744073709551598):
              Variable: inv
              Access: []
              Operation: AssignSignal
              Right-Hand Expression::
                Number: 0
        -------------------------------
        Substitution (elem_id=16):
          Variable: out
          Access: []
          Operation: AssignConstraintSignal
          Right-Hand Expression::
            InfixOp:
              Operator: Add
              Left-Hand Expression:
                InfixOp:
                  Operator: Mul
                  Left-Hand Expression:
                    PrefixOp:
                      Operator: Minus
                      Right-Hand Expression:
                        Variable:
                          Name: in
                          Access: []
                  Right-Hand Expression:
                    Variable:
                      Name: inv
                      Access: []
              Right-Hand Expression:
                Number: 1
        -------------------------------
        ConstraintEquality (elem_id=23):
          Left-Hand Expression::
            InfixOp:
              Operator: Mul
              Left-Hand Expression:
                Variable:
                  Name: in
                  Access: []
              Right-Hand Expression:
                Variable:
                  Name: out
                  Access: []
          Right-Hand Expression::
            Number: 0
        -------------------------------

[2024-11-21T15:59:54Z TRACE tcct::symbolic_execution] (elem_id=0) SymbolicState [
      values: {}
      trace_constraints: []
      sidec_constraints: []
      depth: 0
    ]
[2024-11-21T15:59:54Z TRACE tcct::symbolic_execution] (elem_id=8) SymbolicState [
      values: {"out": out, "inv": inv, "in": in}
      trace_constraints: []
      sidec_constraints: []
      depth: 0
    ]
[2024-11-21T15:59:54Z TRACE tcct::symbolic_execution] (elem_id=18446744073709551599) SymbolicState [
      values: {"out": out, "inv": inv, "in": in}
      trace_constraints: [(NotEq in 0)]
      sidec_constraints: []
      depth: 1
    ]
[2024-11-21T15:59:54Z TRACE tcct::symbolic_execution] (elem_id=16) SymbolicState [
      values: {"inv": (Div 1 in), "in": in, "out": out}
      trace_constraints: [(NotEq in 0), (Eq inv (Div 1 in))]
      sidec_constraints: []
      depth: 0
    ]
[2024-11-21T15:59:54Z TRACE tcct::symbolic_execution] (elem_id=23) SymbolicState [
      values: {"inv": (Div 1 in), "in": in, "out": (Add (Mul (Minus in) (Div 1 in)) 1)}
      trace_constraints: [(NotEq in 0), (Eq inv (Div 1 in)), (Eq out (Add (Mul (Minus in) (Div 1 in)) 1))]
      sidec_constraints: [(Eq out (Add (Mul (Minus in) inv) 1))]
      depth: 0
    ]
[2024-11-21T15:59:54Z TRACE tcct::symbolic_execution] (elem_id=18446744073709551598) SymbolicState [
      values: {"out": out, "inv": inv, "in": in}
      trace_constraints: [(BoolNot (NotEq in 0))]
      sidec_constraints: []
      depth: 1
    ]
[2024-11-21T15:59:54Z TRACE tcct::symbolic_execution] (elem_id=16) SymbolicState [
      values: {"inv": 0, "in": in, "out": out}
      trace_constraints: [(BoolNot (NotEq in 0)), (Eq inv 0)]
      sidec_constraints: []
      depth: 0
    ]
[2024-11-21T15:59:54Z TRACE tcct::symbolic_execution] (elem_id=23) SymbolicState [
      values: {"inv": 0, "in": in, "out": (Add (Mul (Minus in) 0) 1)}
      trace_constraints: [(BoolNot (NotEq in 0)), (Eq inv 0), (Eq out (Add (Mul (Minus in) 0) 1))]
      sidec_constraints: [(Eq out (Add (Mul (Minus in) inv) 1))]
      depth: 0
    ]
[2024-11-21T15:59:54Z INFO  tcct] final_state: SymbolicState [
      values: {"inv": 0, "in": in, "out": (Add (Mul (Minus in) 0) 1)}
      trace_constraints: [(BoolNot (NotEq in 0)), (Eq inv 0), (Eq out (Add (Mul (Minus in) 0) 1)), (Eq (Mul in (Add (Mul (Minus in) 0) 1)) 0)]
      sidec_constraints: [(Eq out (Add (Mul (Minus in) inv) 1)), (Eq (Mul in out) 0)]
      depth: 0
    ]
[2024-11-21T15:59:54Z INFO  tcct] final_state: SymbolicState [
      values: {"inv": (Div 1 in), "in": in, "out": (Add (Mul (Minus in) (Div 1 in)) 1)}
      trace_constraints: [(NotEq in 0), (Eq inv (Div 1 in)), (Eq out (Add (Mul (Minus in) (Div 1 in)) 1)), (Eq (Mul in (Add (Mul (Minus in) (Div 1 in)) 1)) 0)]
      sidec_constraints: [(Eq out (Add (Mul (Minus in) inv) 1)), (Eq (Mul in out) 0)]
      depth: 0
    ]
template_name,num_of_params,max_depth
IsZero,0,1
Total_Constraints,Constant_Counts,Conditional_Counts,Array_Counts,Tuple_Counts,Avg_Depth,Max_Depth,Count_Mul,Count_Div,Count_Add,Count_Sub,Count_Pow,Count_IntDiv,Count_Mod,Count_ShiftL,Count_ShiftR,Count_LesserEq,Count_GreaterEq,Count_Lesser,Count_Greater,Count_Eq,Count_NotEq,Count_BoolOr,Count_BoolAnd,Count_BitOr,Count_BitAnd,Count_BitXor,Variable_Avg_Count,Variable_Max_Count,Function_Avg_Count,Function_Max_Count
7,13,0,0,0,8.50,13,6,3,4,0,0,0,0,0,0,0,0,0,0,6,1,0,0,0,0,0,4.67,10,0.00,0
Total_Constraints,Constant_Counts,Conditional_Counts,Array_Counts,Tuple_Counts,Avg_Depth,Max_Depth,Count_Mul,Count_Div,Count_Add,Count_Sub,Count_Pow,Count_IntDiv,Count_Mod,Count_ShiftL,Count_ShiftR,Count_LesserEq,Count_GreaterEq,Count_Lesser,Count_Greater,Count_Eq,Count_NotEq,Count_BoolOr,Count_BoolAnd,Count_BitOr,Count_BitAnd,Count_BitXor,Variable_Avg_Count,Variable_Max_Count,Function_Avg_Count,Function_Max_Count
4,4,0,0,0,5.20,9,4,0,2,0,0,0,0,0,0,0,0,0,0,4,0,0,0,0,0,0,3.33,4,0.00,0
Everything went okay
```

