use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::interpret_fn::interpret_func;
use crate::scope_information::ScopeInformation;
use crate::{environment::GlobalInformation, type_infer::construct_array_ty};
use inkwell::types::{ArrayType, BasicType};
use inkwell::values::ArrayValue;
use num_bigint_dig::BigInt;
use num_traits::{FromPrimitive, Pow, Signed, ToPrimitive};
use program_structure::ast::{
    Access, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, Statement,
};

#[derive(Clone, PartialEq, Eq)]
pub enum ConcreteValue {
    Int(i128),
    Array(Box<Vec<ConcreteValue>>),
    Unknown,
}

impl ConcreteValue {
    pub fn as_int(&self) -> i128 {
        match self {
            ConcreteValue::Int(i) => *i,
            ConcreteValue::Array(..) => unreachable!(),
            ConcreteValue::Unknown => unreachable!(),
        }
    }

    pub fn as_array(&self) -> &Box<Vec<ConcreteValue>> {
        match self {
            ConcreteValue::Int(..) => unreachable!(),
            ConcreteValue::Array(v) => v,
            ConcreteValue::Unknown => unreachable!(),
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            ConcreteValue::Int(..) => true,
            ConcreteValue::Array(..) => false,
            ConcreteValue::Unknown => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            ConcreteValue::Int(..) => false,
            ConcreteValue::Array(..) => true,
            ConcreteValue::Unknown => false,
        }
    }

    pub fn is_unknown(&self) -> bool {
        match self {
            ConcreteValue::Int(..) => false,
            ConcreteValue::Array(..) => false,
            ConcreteValue::Unknown => true,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ConcreteValue::Int(i) => i.to_string(),
            ConcreteValue::Array(v) => {
                format!("array{}", v.len())
            }
            ConcreteValue::Unknown => "unknown".to_string(),
        }
    }

    pub fn get_from_access(&self, mut access: Vec<usize>) -> ConcreteValue {
        match self {
            ConcreteValue::Array(arr) => {
                let a = access.pop();
                match a {
                    None => self.clone(),
                    Some(idx) => arr.as_ref()[idx].get_from_access(access),
                }
            }
            _ => self.clone(),
        }
    }

    pub fn init_array(ints: Vec<i128>) -> ConcreteValue {
        let vals = ints.into_iter().map(|i| ConcreteValue::Int(i)).collect();
        ConcreteValue::Array(Box::new(vals))
    }
}

impl Hash for ConcreteValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

pub type ArgValues = Vec<ConcreteValue>;
pub type ArgTable = HashMap<String, ConcreteValue>;
pub type Instantiation = (ArgTable, Statement);

pub struct SymbolValueManager {
    pub var2val: HashMap<String, Expression>,
    pub var2dims: HashMap<String, Vec<usize>>,
}

impl SymbolValueManager {
    fn compute_idx(&self, var_name: &String, access: &Vec<usize>) -> usize {
        match self.var2dims.get(var_name) {
            Some(dims) => {
                assert!(dims.len() == access.len());
                if dims.len() == 0 {
                    0
                } else {
                    let mut res = 0;
                    for i in 0..dims.len() {
                        let acc = &dims[(i + 1)..].iter().fold(1, |a, b| a * b);
                        res += access[i] * acc;
                    }
                    res
                }
            },
            None => unreachable!(),
        }
    }
    fn compute_key(&self, var_name: &String, access: &Vec<usize>) -> String {
        match self.var2dims.get(var_name) {
            Some(dims) => {
                assert!(dims.len() == access.len());
                if dims.len() == 0 {
                    var_name.clone()
                } else {
                    let res = self.compute_idx(var_name, access);
                    format!("{}_{}", var_name, res.to_string())
                }
            },
            None => unreachable!(),
        }
    }

    pub fn get_expr(&self, var_name: &String, access: &Vec<usize>) -> Option<&Expression> {
        match self.var2dims.get(var_name) {
            Some(dims) => {
                if dims.len() != access.len() {
                    None
                } else {
                    let key = self.compute_key(var_name, access);
                    self.var2val.get(&key)
                }
            },
            None => unreachable!(),
        }
    }

    pub fn set_expr(&mut self, var_name: &String, access: &Vec<usize>, expr: &Expression) {
        match self.var2dims.get(var_name) {
            Some(dims) => {
                if dims.len() != access.len() {
                    match expr {
                        Expression::ArrayInLine { meta: _, values } => {
                            for (i, expr) in values.iter().enumerate() {
                                let mut new_access = access.clone();
                                new_access.push(i);
                                self.set_expr(var_name, access, expr);
                            }
                        },
                        _ => (),
                    }
                } else {
                    let key = self.compute_key(var_name, access);
                    self.var2val.insert(key, expr.clone());
                }
            },
            None => unreachable!(),
        }
    }

    pub fn add_dims(&mut self, var_name: &String, dimensions: Vec<usize>) {
        self.var2dims.insert(var_name.clone(), dimensions);
    }

    pub fn init() -> SymbolValueManager {
        SymbolValueManager {
            var2val: HashMap::new(),
            var2dims: HashMap::new(),
        }
    }
}

pub fn resolve_expr_static<'ctx>(
    env: &GlobalInformation<'ctx>,
    scope_info: &ScopeInformation,
    arg2val: &ArgTable,
    expr: &Expression,
) -> ConcreteValue {
    use Expression::*;
    match expr {
        InfixOp {
            meta: _,
            lhe,
            infix_op,
            rhe,
        } => {
            let lval = resolve_expr_static(env, scope_info, arg2val, lhe.as_ref());
            let rval = resolve_expr_static(env, scope_info, arg2val, rhe.as_ref());
            if lval.is_unknown() || rval.is_unknown() {
                ConcreteValue::Unknown
            } else {
                let i = resolve_infix_op_static(env, infix_op, lval.as_int(), rval.as_int());
                ConcreteValue::Int(i)
            }
        }
        InlineSwitchOp {
            meta: _,
            cond,
            if_true,
            if_false,
        } => {
            let cond = resolve_expr_static(env, scope_info, arg2val, cond.as_ref());
            let lval = resolve_expr_static(env, scope_info, arg2val, if_true.as_ref());
            let rval = resolve_expr_static(env, scope_info, arg2val, if_false.as_ref());
            match cond {
                ConcreteValue::Int(v) => {
                    let i = if v == 1 { lval } else { rval };
                    i
                }
                ConcreteValue::Array(..) => unreachable!(),
                ConcreteValue::Unknown => ConcreteValue::Unknown,
            }
        }
        PrefixOp {
            meta: _,
            prefix_op,
            rhe,
        } => {
            let rval = resolve_expr_static(env, scope_info, arg2val, rhe.as_ref());
            match rval {
                ConcreteValue::Int(i) => {
                    let r = resolve_prefix_op_static(env, prefix_op, i);
                    ConcreteValue::Int(r)
                }
                ConcreteValue::Array(..) => unreachable!(),
                ConcreteValue::Unknown => ConcreteValue::Unknown,
            }
        }
        Variable {
            meta: _,
            name,
            access,
        } => {
            if arg2val.contains_key(name) {
                let v = &arg2val[name];
                let access_idxes: Vec<usize> = access
                    .iter()
                    .rev()
                    .map(|a| match a {
                        Access::ArrayAccess(a) => {
                            resolve_expr_static(env, scope_info, arg2val, a).as_int() as usize
                        }
                        Access::ComponentAccess(..) => unreachable!(),
                    })
                    .collect();
                v.get_from_access(access_idxes)
            } else {
                ConcreteValue::Unknown
            }
        }
        Number { .. } => ConcreteValue::Int(resolve_number_static(expr)),
        Call { meta: _, id, args } => {
            if scope_info.is_component(id) {
                ConcreteValue::Unknown
            } else {
                interpret_func(env, scope_info, arg2val, id, args)
            }
        }
        ArrayInLine { meta: _, values } => {
            let mut arr = Vec::new();
            for expr in values {
                let v = resolve_expr_static(env, scope_info, arg2val, expr);
                arr.push(v);
            }
            ConcreteValue::Array(Box::new(arr))
        }
        _ => ConcreteValue::Unknown,
    }
}

fn resolve_prefix_op_static<'ctx>(
    _env: &GlobalInformation<'ctx>,
    prefix_op: &ExpressionPrefixOpcode,
    rval: i128,
) -> i128 {
    use ExpressionPrefixOpcode::*;
    let res = match prefix_op {
        Sub => !rval,
        BoolNot => !rval,
        Complement => {
            println!("Error: Complement isn't supported now.");
            unreachable!();
        }
    };
    res
}

fn resolve_infix_op_static<'ctx>(
    _env: &GlobalInformation<'ctx>,
    infix_op: &ExpressionInfixOpcode,
    lval: i128,
    rval: i128,
) -> i128 {
    let _lval = BigInt::from_i128(lval).unwrap();
    let _rval = BigInt::from_i128(rval).unwrap();
    use ExpressionInfixOpcode::*;
    let res = match infix_op {
        Add => _lval + _rval,
        BitAnd => _lval & _rval,
        BitOr => _lval | _rval,
        BitXor => _lval ^ _rval,
        BoolAnd => _lval & _rval,
        BoolOr => _lval | _rval,
        Div => _lval / _rval,
        IntDiv => _lval / _rval,
        Mod => _lval % _rval,
        Mul => _lval * _rval,
        Pow => _lval.pow(_rval.to_u32().unwrap()),
        ShiftL => _lval << _rval.abs().to_usize().unwrap(),
        ShiftR => _lval >> _rval.abs().to_usize().unwrap(),
        Sub => _lval - _rval,

        // Comparison
        Eq => BigInt::from_u8((_lval == _rval) as u8).unwrap(),
        Greater => BigInt::from_u8((_lval > _rval) as u8).unwrap(),
        GreaterEq => BigInt::from_u8((_lval >= _rval) as u8).unwrap(),
        NotEq => BigInt::from_u8((_lval != _rval) as u8).unwrap(),
        Lesser => BigInt::from_u8((_lval < _rval) as u8).unwrap(),
        LesserEq => BigInt::from_u8((_lval <= _rval) as u8).unwrap(),
    };
    (res % i128::MAX).to_i128().unwrap()
}

pub fn resolve_uniform_array_static<'ctx>(
    env: &GlobalInformation<'ctx>,
    scope_info: &ScopeInformation,
    expr: &Expression,
) -> ArrayValue<'ctx> {
    let mut dims = 0;
    let mut end = false;
    let mut current_expr = expr;
    let mut element_val: u64 = 0;
    while !end {
        match current_expr {
            Expression::UniformArray {
                meta: _,
                value,
                dimension: _,
            } => {
                dims += 1;
                current_expr = value.as_ref();
            }
            _ => {
                match resolve_expr_static(env, scope_info, &HashMap::new(), expr) {
                    ConcreteValue::Int(i) => element_val = i as u64,
                    ConcreteValue::Array(..) => (),
                    ConcreteValue::Unknown => (),
                }
                end = true;
            }
        }
    }
    let val = env.val_ty.const_int(element_val, true);
    let mut res = env.val_ty.const_array(&vec![val; env.arraysize as usize]);
    for _ in 1..dims {
        res = res
            .get_type()
            .const_array(&vec![res; env.arraysize as usize]);
    }
    res
}

pub fn resolve_inline_array_static<'ctx>(
    env: &GlobalInformation<'ctx>,
    expr: &Expression,
) -> (ArrayType<'ctx>, Vec<u32>) {
    let mut dims = 0;
    let mut end = false;
    let mut current_expr = expr;
    let mut dims_record = Vec::new();
    while !end {
        match current_expr {
            Expression::ArrayInLine { meta: _, values } => {
                dims += 1;
                dims_record.push(values.len() as u32);
                current_expr = &values[0];
            }
            _ => {
                end = true;
            }
        }
    }
    let ty = construct_array_ty(env, env.val_ty.as_basic_type_enum(), dims);
    (ty, dims_record)
}

pub fn resolve_number_static<'ctx>(expr: &Expression) -> i128 {
    use Expression::*;
    match expr {
        Number(_, bigint) => match (bigint % i128::MAX).to_i128() {
            Some(i) => i,
            None => {
                println!("Error: Unknown bigint: {}", bigint.to_string());
                unreachable!()
            }
        },
        _ => {
            println!("Error: Unknown number expression: {}", print_expr(&expr));
            unreachable!()
        }
    }
}

pub fn print_expr(expr: &Expression) -> &'static str {
    use Expression::*;
    match expr {
        AnonymousComp { .. } => "AnonymousComp",
        ArrayInLine { .. } => "ArrayInLine",
        Call { .. } => "Call",
        InfixOp { .. } => "InfixOp",
        InlineSwitchOp { .. } => "InlineSwitchOp",
        Number(..) => "Number",
        ParallelOp { .. } => "ParallelOp",
        PrefixOp { .. } => "PrefixOp",
        Tuple { .. } => "Tuple",
        UniformArray { .. } => "UniformArray",
        Variable { .. } => "Variable",
    }
}
