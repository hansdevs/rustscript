//! Tree-walking interpreter for RustScript.
//! Used in `--run` mode to execute programs directly in the terminal.

use std::collections::HashMap;
use std::fmt;
use std::io::Write;

use crate::ast::*;
use crate::turbo::RsBigInt;

// ── runtime values ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    Dict(Vec<(Value, Value)>),
    BigInt(RsBigInt),
    Lambda {
        params: Vec<String>,
        body: Expr,
    },
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::List(items) => {
                let parts: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", parts.join(", "))
            }
            Value::Dict(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{{}}}", parts.join(", "))
            }
            Value::BigInt(n) => write!(f, "{}", n),
            Value::Lambda { params, .. } => {
                write!(f, "<lambda({})>", params.join(", "))
            }
            Value::Null => write!(f, "none"),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Dict(d) => !d.is_empty(),
            Value::BigInt(n) => !n.is_zero(),
            Value::Lambda { .. } => true,
            Value::Null => false,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Str(_) => "str",
            Value::Bool(_) => "bool",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::BigInt(_) => "bigint",
            Value::Lambda { .. } => "lambda",
            Value::Null => "none",
        }
    }

    pub fn to_float(&self) -> f64 {
        match self {
            Value::Int(n) => *n as f64,
            Value::Float(n) => *n,
            Value::Str(s) => s.parse().unwrap_or(0.0),
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::BigInt(n) => n.to_f64(),
            _ => 0.0,
        }
    }

    /// Compute a hash-like key for dict lookups.
    fn dict_key_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        }
    }
}

// ── control-flow signals ─────────────────────────────────────

enum Signal {
    None,
    Return(Value),
    Break,
    Continue,
}

// ── interpreter ──────────────────────────────────────────────

pub struct Interpreter {
    /// Global variable scope
    globals: HashMap<String, Value>,
    /// Function declarations (name → (params, body))
    functions: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    /// Turbo mode: enables BigInt auto-promotion, timestamps, JSON I/O
    turbo: bool,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            globals: HashMap::new(),
            functions: HashMap::new(),
            turbo: false,
        }
    }

    /// Run a full program.
    pub fn run(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.stmts {
            match self.exec_stmt(stmt, &mut None)? {
                Signal::Return(_) => {
                    return Err("'return' outside of function".into());
                }
                Signal::Break => {
                    return Err("'break' outside of loop".into());
                }
                Signal::Continue => {
                    return Err("'continue' outside of loop".into());
                }
                Signal::None => {}
            }
        }
        Ok(())
    }

    /// Execute a statement.
    /// `local_scope` is Some when inside a function call.
    fn exec_stmt(
        &mut self,
        stmt: &Stmt,
        local: &mut Option<HashMap<String, Value>>,
    ) -> Result<Signal, String> {
        match stmt {
            Stmt::Let { name, value } => {
                let val = self.eval_expr(value, local)?;
                if let Some(scope) = local {
                    scope.insert(name.clone(), val);
                } else {
                    self.globals.insert(name.clone(), val);
                }
                Ok(Signal::None)
            }
            Stmt::Assign { name, value } => {
                // ── Fast path: in-place BigInt scalar multiply ────────
                // Detects `name = name * expr` or `name = expr * name`
                // and uses mul_scalar_inplace (zero allocation) instead
                // of clone → mul → store (2 huge allocations).
                if self.turbo
                    && let Expr::BinOp { left, op: BinOp::Mul, right } = value {
                        let (var_side, scalar_side) = match (left.as_ref(), right.as_ref()) {
                            (Expr::Ident(lname), _) if lname == name => (true, right.as_ref()),
                            (_, Expr::Ident(rname)) if rname == name => (true, left.as_ref()),
                            _ => (false, value), // not our pattern
                        };
                        if var_side {
                            // Evaluate the scalar side
                            let scalar_val = self.eval_expr(scalar_side, local)?;
                            if let Value::Int(n) = scalar_val {
                                // Find the scope that holds `name`
                                let target = if let Some(scope) = local.as_mut() {
                                    if scope.contains_key(name) {
                                        scope.get_mut(name)
                                    } else {
                                        self.globals.get_mut(name)
                                    }
                                } else {
                                    self.globals.get_mut(name)
                                };
                                if let Some(Value::BigInt(big)) = target {
                                    big.mul_scalar_inplace_i64(n);
                                    return Ok(Signal::None);
                                }
                            }
                        }
                    }
                    // ── Fast path: in-place BigInt add/sub ──────────
                    // (future: in-place BigInt add/sub optimization)
                // ── Normal path ──────────────────────────────────
                let val = self.eval_expr(value, local)?;
                // Check local first, then global
                if let Some(scope) = local
                    && scope.contains_key(name)
                {
                    scope.insert(name.clone(), val);
                    return Ok(Signal::None);
                }
                self.globals.insert(name.clone(), val);
                Ok(Signal::None)
            }
            Stmt::IndexAssign { list, index, value } => {
                let idx = self.eval_expr(index, local)?;
                let val = self.eval_expr(value, local)?;
                // Find the list/dict in scope
                let target = if let Some(scope) = local {
                    if scope.contains_key(list) {
                        scope.get_mut(list)
                    } else {
                        self.globals.get_mut(list)
                    }
                } else {
                    self.globals.get_mut(list)
                };
                match target {
                    Some(Value::List(items)) => {
                        let i = match idx {
                            Value::Int(i) => {
                                if i < 0 {
                                    (items.len() as i64 + i) as usize
                                } else {
                                    i as usize
                                }
                            }
                            _ => return Err("List index must be an integer".into()),
                        };
                        if i < items.len() {
                            items[i] = val;
                        } else {
                            return Err(format!(
                                "Index {} out of bounds (len {})",
                                i,
                                items.len()
                            ));
                        }
                    }
                    Some(Value::Dict(pairs)) => {
                        // Update existing or insert new
                        for pair in pairs.iter_mut() {
                            if pair.0.dict_key_eq(&idx) {
                                pair.1 = val.clone();
                                return Ok(Signal::None);
                            }
                        }
                        pairs.push((idx, val));
                    }
                    _ => {
                        return Err(format!("'{}' is not a list or dict", list));
                    }
                }
                Ok(Signal::None)
            }
            Stmt::MemberAssign {
                object,
                field,
                value,
            } => {
                let val = self.eval_expr(value, local)?;
                let key = Value::Str(field.clone());
                let target = if let Some(scope) = local {
                    if scope.contains_key(object) {
                        scope.get_mut(object)
                    } else {
                        self.globals.get_mut(object)
                    }
                } else {
                    self.globals.get_mut(object)
                };
                if let Some(Value::Dict(pairs)) = target {
                    for pair in pairs.iter_mut() {
                        if pair.0.dict_key_eq(&key) {
                            pair.1 = val.clone();
                            return Ok(Signal::None);
                        }
                    }
                    pairs.push((key, val));
                    Ok(Signal::None)
                } else {
                    Err(format!("'{}' is not a dict", object))
                }
            }
            Stmt::FnDecl { name, params, body } => {
                self.functions
                    .insert(name.clone(), (params.clone(), body.clone()));
                Ok(Signal::None)
            }
            Stmt::Return(val) => {
                let v = if let Some(expr) = val {
                    self.eval_expr(expr, local)?
                } else {
                    Value::Null
                };
                Ok(Signal::Return(v))
            }
            Stmt::Break => Ok(Signal::Break),
            Stmt::Continue => Ok(Signal::Continue),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                let c = self.eval_expr(cond, local)?;
                if c.is_truthy() {
                    for s in then_body {
                        let sig = self.exec_stmt(s, local)?;
                        match sig {
                            Signal::None => {}
                            _ => return Ok(sig),
                        }
                    }
                } else if let Some(eb) = else_body {
                    for s in eb {
                        let sig = self.exec_stmt(s, local)?;
                        match sig {
                            Signal::None => {}
                            _ => return Ok(sig),
                        }
                    }
                }
                Ok(Signal::None)
            }
            Stmt::While { cond, body } => {
                'outer: loop {
                    let c = self.eval_expr(cond, local)?;
                    if !c.is_truthy() {
                        break;
                    }
                    for s in body {
                        let sig = self.exec_stmt(s, local)?;
                        match sig {
                            Signal::Break => break 'outer,
                            Signal::Continue => continue 'outer,
                            Signal::Return(_) => return Ok(sig),
                            Signal::None => {}
                        }
                    }
                }
                Ok(Signal::None)
            }
            Stmt::For { var, iter, body } => {
                let iter_val = self.eval_expr(iter, local)?;
                let items = match iter_val {
                    Value::List(items) => items,
                    Value::Str(s) => s.chars().map(|c| Value::Str(c.to_string())).collect(),
                    Value::Dict(pairs) => {
                        // Iterating a dict yields its keys
                        pairs.into_iter().map(|(k, _)| k).collect()
                    }
                    other => {
                        return Err(format!("Cannot iterate over {}", other.type_name()));
                    }
                };
                'outer: for item in items {
                    if let Some(scope) = local {
                        scope.insert(var.clone(), item);
                    } else {
                        self.globals.insert(var.clone(), item);
                    }
                    for s in body {
                        let sig = self.exec_stmt(s, local)?;
                        match sig {
                            Signal::Break => break 'outer,
                            Signal::Continue => continue 'outer,
                            Signal::Return(_) => return Ok(sig),
                            Signal::None => {}
                        }
                    }
                }
                Ok(Signal::None)
            }
            Stmt::Page { .. } => {
                // In interpreter mode, skip page blocks (they're for HTML output)
                println!(
                    "[info] Page block skipped in --run mode. Use build mode to generate HTML."
                );
                Ok(Signal::None)
            }
            Stmt::Import { path } => {
                // Module imports: activate the module
                match path.as_str() {
                    "turbo" => {
                        self.turbo = true;
                    }
                    _ => {
                        // File imports are resolved before interpretation
                    }
                }
                Ok(Signal::None)
            }
            Stmt::Expr(expr) => {
                self.eval_expr(expr, local)?;
                Ok(Signal::None)
            }
        }
    }

    // ── expression evaluation ────────────────────────────────

    fn eval_expr(
        &mut self,
        expr: &Expr,
        local: &mut Option<HashMap<String, Value>>,
    ) -> Result<Value, String> {
        match expr {
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Float(n) => Ok(Value::Float(*n)),
            Expr::Str(s) => Ok(Value::Str(self.interpolate(s, local)?)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::None => Ok(Value::Null),
            Expr::Ident(name) => {
                if let Some(scope) = &*local
                    && let Some(v) = scope.get(name)
                {
                    return Ok(v.clone());
                }
                self.globals
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: '{}'", name))
            }
            Expr::List(items) => {
                let vals: Result<Vec<Value>, _> =
                    items.iter().map(|e| self.eval_expr(e, local)).collect();
                Ok(Value::List(vals?))
            }
            Expr::Dict(pairs) => {
                let mut result = Vec::new();
                for (k, v) in pairs {
                    let key = self.eval_expr(k, local)?;
                    let val = self.eval_expr(v, local)?;
                    result.push((key, val));
                }
                Ok(Value::Dict(result))
            }
            Expr::Lambda { params, body } => Ok(Value::Lambda {
                params: params.clone(),
                body: *body.clone(),
            }),
            Expr::PipeCall {
                value,
                func,
                extra_args,
            } => {
                let val = self.eval_expr(value, local)?;
                let mut args = vec![val];
                for a in extra_args {
                    args.push(self.eval_expr(a, local)?);
                }
                self.call_fn(func, args, local)
            }
            Expr::BinOp { left, op, right } => {
                let l = self.eval_expr(left, local)?;
                let r = self.eval_expr(right, local)?;
                self.eval_binop(&l, *op, &r)
            }
            Expr::UnaryOp { op, expr } => {
                let v = self.eval_expr(expr, local)?;
                match op {
                    UnaryOp::Neg => match v {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        Value::Float(n) => Ok(Value::Float(-n)),
                        _ => Err("Cannot negate non-numeric value".into()),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!v.is_truthy())),
                }
            }
            Expr::Call { name, args } => {
                let arg_vals: Result<Vec<Value>, _> =
                    args.iter().map(|a| self.eval_expr(a, local)).collect();
                let arg_vals = arg_vals?;
                self.call_fn(name, arg_vals, local)
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let obj = self.eval_expr(object, local)?;
                let arg_vals: Result<Vec<Value>, _> =
                    args.iter().map(|a| self.eval_expr(a, local)).collect();
                let arg_vals = arg_vals?;
                self.call_method(obj, method, arg_vals, local)
            }
            Expr::Index { object, index } => {
                let obj = self.eval_expr(object, local)?;
                let idx = self.eval_expr(index, local)?;
                match (&obj, &idx) {
                    (Value::List(items), Value::Int(i)) => {
                        let i = if *i < 0 {
                            (items.len() as i64 + *i) as usize
                        } else {
                            *i as usize
                        };
                        items.get(i).cloned().ok_or_else(|| {
                            format!("Index {} out of bounds (len {})", i, items.len())
                        })
                    }
                    (Value::Str(s), Value::Int(i)) => {
                        let len = s.chars().count() as i64;
                        let i = if *i < 0 { (len + *i) as usize } else { *i as usize };
                        s.chars()
                            .nth(i)
                            .map(|c| Value::Str(c.to_string()))
                            .ok_or_else(|| format!("Index {} out of bounds", i))
                    }
                    (Value::Dict(pairs), _) => {
                        for (k, v) in pairs {
                            if k.dict_key_eq(&idx) {
                                return Ok(v.clone());
                            }
                        }
                        Err(format!("Key {} not found in dict", idx))
                    }
                    _ => Err("Invalid index operation".into()),
                }
            }
            Expr::Member { object, field } => {
                let obj = self.eval_expr(object, local)?;
                match (&obj, field.as_str()) {
                    (Value::List(items), "length") => Ok(Value::Int(items.len() as i64)),
                    (Value::Str(s), "length") => Ok(Value::Int(s.chars().count() as i64)),
                    (Value::Dict(pairs), "length") => Ok(Value::Int(pairs.len() as i64)),
                    (Value::Dict(pairs), _) => {
                        // Dict member access: obj.field == obj["field"]
                        let key = Value::Str(field.clone());
                        for (k, v) in pairs {
                            if k.dict_key_eq(&key) {
                                return Ok(v.clone());
                            }
                        }
                        Err(format!("Key '{}' not found in dict", field))
                    }
                    _ => Err(format!("No field '{}' on {}", field, obj.type_name())),
                }
            }
        }
    }

    fn eval_binop(&self, left: &Value, op: BinOp, right: &Value) -> Result<Value, String> {
        match op {
            BinOp::Add => match (left, right) {
                // BigInt cases
                (Value::BigInt(a), Value::BigInt(b)) => Ok(Value::BigInt(a.add(b))),
                (Value::BigInt(a), Value::Int(b)) => Ok(Value::BigInt(a.add(&RsBigInt::from_i64(*b)))),
                (Value::Int(a), Value::BigInt(b)) => Ok(Value::BigInt(RsBigInt::from_i64(*a).add(b))),
                // Turbo: auto-promote on overflow
                (Value::Int(a), Value::Int(b)) if self.turbo => {
                    match a.checked_add(*b) {
                        Some(r) => Ok(Value::Int(r)),
                        None => Ok(Value::BigInt(RsBigInt::from_i64(*a).add(&RsBigInt::from_i64(*b)))),
                    }
                }
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                (Value::Str(a), other) => Ok(Value::Str(format!("{}{}", a, other))),
                (other, Value::Str(b)) => Ok(Value::Str(format!("{}{}", other, b))),
                (Value::List(a), Value::List(b)) => {
                    let mut combined = a.clone();
                    combined.extend(b.clone());
                    Ok(Value::List(combined))
                }
                (Value::Dict(a), Value::Dict(b)) => {
                    let mut combined = a.clone();
                    for pair in b {
                        let mut found = false;
                        for existing in combined.iter_mut() {
                            if existing.0.dict_key_eq(&pair.0) {
                                existing.1 = pair.1.clone();
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            combined.push(pair.clone());
                        }
                    }
                    Ok(Value::Dict(combined))
                }
                _ => Err(format!(
                    "Cannot add {} and {}",
                    left.type_name(),
                    right.type_name()
                )),
            },
            BinOp::Sub => match (left, right) {
                (Value::BigInt(a), Value::BigInt(b)) => Ok(Value::BigInt(a.sub(b))),
                (Value::BigInt(a), Value::Int(b)) => Ok(Value::BigInt(a.sub(&RsBigInt::from_i64(*b)))),
                (Value::Int(a), Value::BigInt(b)) => Ok(Value::BigInt(RsBigInt::from_i64(*a).sub(b))),
                (Value::Int(a), Value::Int(b)) if self.turbo => {
                    match a.checked_sub(*b) {
                        Some(r) => Ok(Value::Int(r)),
                        None => Ok(Value::BigInt(RsBigInt::from_i64(*a).sub(&RsBigInt::from_i64(*b)))),
                    }
                }
                _ => self.numeric_op(left, right, |a, b| a - b, |a, b| a - b),
            },
            BinOp::Mul => match (left, right) {
                // BigInt × BigInt
                (Value::BigInt(a), Value::BigInt(b)) => Ok(Value::BigInt(a.mul(b))),
                // BigInt × scalar (fast path)
                (Value::BigInt(a), Value::Int(b)) => Ok(Value::BigInt(a.mul_i64(*b))),
                (Value::Int(a), Value::BigInt(b)) => Ok(Value::BigInt(b.mul_i64(*a))),
                // String/list repetition
                (Value::Str(s), Value::Int(n)) | (Value::Int(n), Value::Str(s)) => {
                    if *n <= 0 { return Ok(Value::Str(String::new())); }
                    Ok(Value::Str(s.repeat(*n as usize)))
                }
                (Value::List(l), Value::Int(n)) => {
                    if *n <= 0 { return Ok(Value::List(Vec::new())); }
                    let mut result = Vec::new();
                    for _ in 0..*n {
                        result.extend(l.clone());
                    }
                    Ok(Value::List(result))
                }
                // Turbo: auto-promote on overflow
                (Value::Int(a), Value::Int(b)) if self.turbo => {
                    match a.checked_mul(*b) {
                        Some(r) => Ok(Value::Int(r)),
                        None => Ok(Value::BigInt(RsBigInt::from_i64(*a).mul_i64(*b))),
                    }
                }
                _ => self.numeric_op(left, right, |a, b| a * b, |a, b| a * b),
            },
            BinOp::Div => match (left, right) {
                (Value::BigInt(a), Value::Int(b)) if *b != 0 => {
                    let (mut q, _) = a.div_mod_scalar(b.unsigned_abs());
                    // Sign: negative if exactly one operand is negative
                    if a.negative != (*b < 0) && !q.is_zero() {
                        q.negative = true;
                    }
                    Ok(Value::BigInt(q))
                }
                _ => self.numeric_op(
                    left,
                    right,
                    |a, b| {
                        if b == 0 {
                            0
                        } else {
                            a / b
                        }
                    },
                    |a, b| a / b,
                ),
            },
            BinOp::Mod => match (left, right) {
                (Value::BigInt(a), Value::Int(b)) if *b != 0 => {
                    let (_, rem) = a.div_mod_scalar(b.unsigned_abs());
                    // Match sign of the dividend (like Rust's % operator)
                    let rem_i64 = rem as i64;
                    Ok(Value::Int(if a.negative && rem_i64 != 0 { -rem_i64 } else { rem_i64 }))
                }
                _ => self.numeric_op(
                    left,
                    right,
                    |a, b| {
                        if b == 0 { 0 } else { a % b }
                    },
                    |a, b| a % b,
                ),
            },
            BinOp::Pow => {
                match (left, right) {
                    (Value::BigInt(a), Value::Int(b)) if *b >= 0 => {
                        Ok(Value::BigInt(a.pow_u32(*b as u32)))
                    }
                    (Value::Int(a), Value::Int(b)) => {
                        if self.turbo && *b >= 0 {
                            // Use BigInt for potentially large results
                            Ok(Value::BigInt(RsBigInt::from_i64(*a).pow_u32(*b as u32)))
                        } else if *b >= 0 {
                            match a.checked_pow(*b as u32) {
                                Some(r) => Ok(Value::Int(r)),
                                None => Ok(Value::Float((*a as f64).powf(*b as f64))),
                            }
                        } else {
                            Ok(Value::Float((*a as f64).powf(*b as f64)))
                        }
                    }
                    _ => {
                        let a = left.to_float();
                        let b = right.to_float();
                        Ok(Value::Float(a.powf(b)))
                    }
                }
            }
            BinOp::FloorDiv => {
                match (left, right) {
                    (Value::BigInt(a), Value::Int(b)) if *b != 0 => {
                        let (mut q, rem) = a.div_mod_scalar(b.unsigned_abs());
                        let signs_differ = a.negative != (*b < 0);
                        if signs_differ && !q.is_zero() {
                            q.negative = true;
                        }
                        // Floor division: if remainder != 0 and signs differ, subtract 1
                        if signs_differ && rem != 0 {
                            q = q.sub(&RsBigInt::one());
                        }
                        Ok(Value::BigInt(q))
                    }
                    _ => {
                        let a = left.to_float();
                        let b = right.to_float();
                        if b == 0.0 {
                            Err("Floor division by zero".into())
                        } else {
                            Ok(Value::Int((a / b).floor() as i64))
                        }
                    }
                }
            }
            BinOp::Eq => Ok(Value::Bool(self.values_equal(left, right))),
            BinOp::NotEq => Ok(Value::Bool(!self.values_equal(left, right))),
            BinOp::Lt => {
                let result = match (left, right) {
                    (Value::BigInt(a), Value::BigInt(b)) => a.cmp(b) == std::cmp::Ordering::Less,
                    (Value::BigInt(a), Value::Int(b)) => a.cmp(&RsBigInt::from_i64(*b)) == std::cmp::Ordering::Less,
                    (Value::Int(a), Value::BigInt(b)) => RsBigInt::from_i64(*a).cmp(b) == std::cmp::Ordering::Less,
                    _ => left.to_float() < right.to_float(),
                };
                Ok(Value::Bool(result))
            }
            BinOp::Gt => {
                let result = match (left, right) {
                    (Value::BigInt(a), Value::BigInt(b)) => a.cmp(b) == std::cmp::Ordering::Greater,
                    (Value::BigInt(a), Value::Int(b)) => a.cmp(&RsBigInt::from_i64(*b)) == std::cmp::Ordering::Greater,
                    (Value::Int(a), Value::BigInt(b)) => RsBigInt::from_i64(*a).cmp(b) == std::cmp::Ordering::Greater,
                    _ => left.to_float() > right.to_float(),
                };
                Ok(Value::Bool(result))
            }
            BinOp::LtEq => {
                let result = match (left, right) {
                    (Value::BigInt(a), Value::BigInt(b)) => a.cmp(b) != std::cmp::Ordering::Greater,
                    (Value::BigInt(a), Value::Int(b)) => a.cmp(&RsBigInt::from_i64(*b)) != std::cmp::Ordering::Greater,
                    (Value::Int(a), Value::BigInt(b)) => RsBigInt::from_i64(*a).cmp(b) != std::cmp::Ordering::Greater,
                    _ => left.to_float() <= right.to_float(),
                };
                Ok(Value::Bool(result))
            }
            BinOp::GtEq => {
                let result = match (left, right) {
                    (Value::BigInt(a), Value::BigInt(b)) => a.cmp(b) != std::cmp::Ordering::Less,
                    (Value::BigInt(a), Value::Int(b)) => a.cmp(&RsBigInt::from_i64(*b)) != std::cmp::Ordering::Less,
                    (Value::Int(a), Value::BigInt(b)) => RsBigInt::from_i64(*a).cmp(b) != std::cmp::Ordering::Less,
                    _ => left.to_float() >= right.to_float(),
                };
                Ok(Value::Bool(result))
            }
            BinOp::And => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
            BinOp::Or => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),
        }
    }

    fn numeric_op<F1, F2>(
        &self,
        left: &Value,
        right: &Value,
        int_op: F1,
        float_op: F2,
    ) -> Result<Value, String>
    where
        F1: Fn(i64, i64) -> i64,
        F2: Fn(f64, f64) -> f64,
    {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_op(*a, *b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(*a, *b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(float_op(*a as f64, *b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(float_op(*a, *b as f64))),
            _ => Err(format!(
                "Cannot perform arithmetic on {} and {}",
                left.type_name(),
                right.type_name()
            )),
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::BigInt(a), Value::BigInt(b)) => a == b,
            (Value::BigInt(a), Value::Int(b)) => *a == RsBigInt::from_i64(*b),
            (Value::Int(a), Value::BigInt(b)) => RsBigInt::from_i64(*a) == *b,
            _ => false,
        }
    }

    // ── function calls ───────────────────────────────────────

    #[allow(clippy::only_used_in_recursion)]
    fn call_fn(
        &mut self,
        name: &str,
        args: Vec<Value>,
        caller_local: &mut Option<HashMap<String, Value>>,
    ) -> Result<Value, String> {
        // Built-in functions
        match name {
            "print" => {
                let parts: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
                let text = parts.join(" ");
                print!("{}", text);
                let _ = std::io::stdout().flush();
                return Ok(Value::Null);
            }
            "println" => {
                let parts: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
                println!("{}", parts.join(" "));
                return Ok(Value::Null);
            }
            "input" => {
                // input() or input("prompt")
                if let Some(Value::Str(prompt)) = args.first() {
                    eprint!("{}", prompt);
                }
                let mut buf = String::new();
                std::io::stdin()
                    .read_line(&mut buf)
                    .map_err(|e| format!("input() error: {}", e))?;
                return Ok(Value::Str(buf.trim_end_matches('\n').to_string()));
            }
            "len" => {
                return match args.first() {
                    Some(Value::List(l)) => Ok(Value::Int(l.len() as i64)),
                    Some(Value::Str(s)) => Ok(Value::Int(s.chars().count() as i64)),
                    Some(Value::Dict(d)) => Ok(Value::Int(d.len() as i64)),
                    _ => Err("len() expects a list, string, or dict".into()),
                };
            }
            "str" => {
                return match args.first() {
                    Some(v) => Ok(Value::Str(format!("{}", v))),
                    None => Ok(Value::Str(String::new())),
                };
            }
            "int" => {
                return match args.first() {
                    Some(Value::Int(n)) => Ok(Value::Int(*n)),
                    Some(Value::Float(n)) => Ok(Value::Int(*n as i64)),
                    Some(Value::Str(s)) => s
                        .trim()
                        .parse::<i64>()
                        .map(Value::Int)
                        .map_err(|_| format!("Cannot convert '{}' to int", s)),
                    Some(Value::Bool(b)) => Ok(Value::Int(if *b { 1 } else { 0 })),
                    _ => Ok(Value::Int(0)),
                };
            }
            "float" => {
                return match args.first() {
                    Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
                    Some(Value::Float(n)) => Ok(Value::Float(*n)),
                    Some(Value::Str(s)) => s
                        .trim()
                        .parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| format!("Cannot convert '{}' to float", s)),
                    _ => Ok(Value::Float(0.0)),
                };
            }
            "bool" => {
                return match args.first() {
                    Some(v) => Ok(Value::Bool(v.is_truthy())),
                    None => Ok(Value::Bool(false)),
                };
            }
            "push" => {
                // push(list, value) — modifies list in place if possible
                if args.len() >= 2
                    && let Value::List(mut l) = args[0].clone() {
                        l.push(args[1].clone());
                        return Ok(Value::List(l));
                    }
                return Err("push() expects (list, value)".into());
            }
            "pop" => {
                if let Some(Value::List(mut l)) = args.into_iter().next() {
                    let popped = l.pop().unwrap_or(Value::Null);
                    return Ok(popped);
                }
                return Err("pop() expects a list".into());
            }
            "range" => {
                return match args.as_slice() {
                    [Value::Int(n)] => Ok(Value::List((0..*n).map(Value::Int).collect())),
                    [Value::Int(start), Value::Int(end)] => {
                        Ok(Value::List((*start..*end).map(Value::Int).collect()))
                    }
                    [Value::Int(start), Value::Int(end), Value::Int(step)] => {
                        let mut result = Vec::new();
                        let mut i = *start;
                        if *step > 0 {
                            while i < *end {
                                result.push(Value::Int(i));
                                i += step;
                            }
                        } else if *step < 0 {
                            while i > *end {
                                result.push(Value::Int(i));
                                i += step;
                            }
                        }
                        Ok(Value::List(result))
                    }
                    _ => Err("range() expects 1, 2, or 3 integer arguments".into()),
                };
            }
            "type" => {
                return match args.first() {
                    Some(v) => Ok(Value::Str(v.type_name().to_string())),
                    None => Ok(Value::Str("none".into())),
                };
            }
            "abs" => {
                return match args.first() {
                    Some(Value::Int(n)) => Ok(Value::Int(n.abs())),
                    Some(Value::Float(n)) => Ok(Value::Float(n.abs())),
                    _ => Err("abs() expects a number".into()),
                };
            }
            "min" => {
                if args.len() == 1
                    && let Value::List(items) = &args[0] {
                        if items.is_empty() {
                            return Err("min() of empty list".into());
                        }
                        let mut m = items[0].to_float();
                        for item in &items[1..] {
                            let v = item.to_float();
                            if v < m {
                                m = v;
                            }
                        }
                        return Ok(Value::Float(m));
                    }
                if args.len() >= 2 {
                    let a = args[0].to_float();
                    let b = args[1].to_float();
                    return Ok(Value::Float(a.min(b)));
                }
                return Err("min() expects 2 arguments or a list".into());
            }
            "max" => {
                if args.len() == 1
                    && let Value::List(items) = &args[0] {
                        if items.is_empty() {
                            return Err("max() of empty list".into());
                        }
                        let mut m = items[0].to_float();
                        for item in &items[1..] {
                            let v = item.to_float();
                            if v > m {
                                m = v;
                            }
                        }
                        return Ok(Value::Float(m));
                    }
                if args.len() >= 2 {
                    let a = args[0].to_float();
                    let b = args[1].to_float();
                    return Ok(Value::Float(a.max(b)));
                }
                return Err("max() expects 2 arguments or a list".into());
            }
            "sum" => {
                if let Some(Value::List(items)) = args.first() {
                    let mut total = 0.0_f64;
                    let mut all_int = true;
                    let mut int_total = 0_i64;
                    for item in items {
                        match item {
                            Value::Int(n) => {
                                int_total += n;
                                total += *n as f64;
                            }
                            Value::Float(n) => {
                                all_int = false;
                                total += n;
                            }
                            _ => {
                                all_int = false;
                                total += item.to_float();
                            }
                        }
                    }
                    return if all_int {
                        Ok(Value::Int(int_total))
                    } else {
                        Ok(Value::Float(total))
                    };
                }
                return Err("sum() expects a list".into());
            }
            "round" => {
                return match args.as_slice() {
                    [Value::Float(n)] => Ok(Value::Int(n.round() as i64)),
                    [Value::Int(n)] => Ok(Value::Int(*n)),
                    [Value::Float(n), Value::Int(places)] => {
                        let factor = 10f64.powi(*places as i32);
                        Ok(Value::Float((n * factor).round() / factor))
                    }
                    _ => Err("round() expects a number and optional places".into()),
                };
            }
            "chr" => {
                if let Some(Value::Int(n)) = args.first()
                    && let Some(c) = char::from_u32(*n as u32) {
                        return Ok(Value::Str(c.to_string()));
                    }
                return Err("chr() expects an integer code point".into());
            }
            "ord" => {
                if let Some(Value::Str(s)) = args.first()
                    && let Some(c) = s.chars().next() {
                        return Ok(Value::Int(c as i64));
                    }
                return Err("ord() expects a non-empty string".into());
            }
            "sorted" => {
                if let Some(Value::List(items)) = args.first() {
                    let mut sorted = items.clone();
                    sorted.sort_by(|a, b| a.to_float().partial_cmp(&b.to_float()).unwrap_or(std::cmp::Ordering::Equal));
                    return Ok(Value::List(sorted));
                }
                return Err("sorted() expects a list".into());
            }
            "reversed" => {
                return match args.first() {
                    Some(Value::List(items)) => {
                        let mut rev = items.clone();
                        rev.reverse();
                        Ok(Value::List(rev))
                    }
                    Some(Value::Str(s)) => Ok(Value::Str(s.chars().rev().collect())),
                    _ => Err("reversed() expects a list or string".into()),
                };
            }
            "enumerate" => {
                if let Some(Value::List(items)) = args.first() {
                    let result: Vec<Value> = items
                        .iter()
                        .enumerate()
                        .map(|(i, v)| Value::List(vec![Value::Int(i as i64), v.clone()]))
                        .collect();
                    return Ok(Value::List(result));
                }
                return Err("enumerate() expects a list".into());
            }
            "zip" => {
                if args.len() >= 2
                    && let (Value::List(a), Value::List(b)) = (&args[0], &args[1]) {
                        let result: Vec<Value> = a
                            .iter()
                            .zip(b.iter())
                            .map(|(x, y)| Value::List(vec![x.clone(), y.clone()]))
                            .collect();
                        return Ok(Value::List(result));
                    }
                return Err("zip() expects two lists".into());
            }
            "any" => {
                if let Some(Value::List(items)) = args.first() {
                    return Ok(Value::Bool(items.iter().any(|v| v.is_truthy())));
                }
                return Err("any() expects a list".into());
            }
            "all" => {
                if let Some(Value::List(items)) = args.first() {
                    return Ok(Value::Bool(items.iter().all(|v| v.is_truthy())));
                }
                return Err("all() expects a list".into());
            }
            "keys" => {
                if let Some(Value::Dict(pairs)) = args.first() {
                    return Ok(Value::List(pairs.iter().map(|(k, _)| k.clone()).collect()));
                }
                return Err("keys() expects a dict".into());
            }
            "values" => {
                if let Some(Value::Dict(pairs)) = args.first() {
                    return Ok(Value::List(pairs.iter().map(|(_, v)| v.clone()).collect()));
                }
                return Err("values() expects a dict".into());
            }
            "items" => {
                if let Some(Value::Dict(pairs)) = args.first() {
                    let result: Vec<Value> = pairs
                        .iter()
                        .map(|(k, v)| Value::List(vec![k.clone(), v.clone()]))
                        .collect();
                    return Ok(Value::List(result));
                }
                return Err("items() expects a dict".into());
            }
            "has" => {
                // has(dict, key) or has(list, value)
                if args.len() >= 2 {
                    match &args[0] {
                        Value::Dict(pairs) => {
                            let found = pairs.iter().any(|(k, _)| k.dict_key_eq(&args[1]));
                            return Ok(Value::Bool(found));
                        }
                        Value::List(items) => {
                            let found = items.iter().any(|v| {
                                self.values_equal(v, &args[1])
                            });
                            return Ok(Value::Bool(found));
                        }
                        _ => {}
                    }
                }
                return Err("has() expects (dict, key) or (list, value)".into());
            }
            "map" => {
                // map(list, fn) or map(list, lambda)
                if args.len() >= 2
                    && let Value::List(items) = &args[0] {
                        let mut result = Vec::new();
                        match &args[1] {
                            Value::Lambda { params, body } => {
                                if params.len() != 1 {
                                    return Err("map() lambda must take exactly 1 parameter".into());
                                }
                                for item in items {
                                    let mut scope: HashMap<String, Value> = HashMap::new();
                                    scope.insert(params[0].clone(), item.clone());
                                    let mut local_scope = Some(scope);
                                    let val = self.eval_expr(body, &mut local_scope)?;
                                    result.push(val);
                                }
                            }
                            Value::Str(fn_name) => {
                                for item in items {
                                    let val = self.call_fn(fn_name, vec![item.clone()], caller_local)?;
                                    result.push(val);
                                }
                            }
                            _ => return Err("map() second argument must be a lambda or function name".into()),
                        }
                        return Ok(Value::List(result));
                    }
                return Err("map() expects (list, fn)".into());
            }
            "filter" => {
                // filter(list, fn) or filter(list, lambda)
                if args.len() >= 2
                    && let Value::List(items) = &args[0] {
                        let mut result = Vec::new();
                        match &args[1] {
                            Value::Lambda { params, body } => {
                                if params.len() != 1 {
                                    return Err("filter() lambda must take exactly 1 parameter".into());
                                }
                                for item in items {
                                    let mut scope: HashMap<String, Value> = HashMap::new();
                                    scope.insert(params[0].clone(), item.clone());
                                    let mut local_scope = Some(scope);
                                    let val = self.eval_expr(body, &mut local_scope)?;
                                    if val.is_truthy() {
                                        result.push(item.clone());
                                    }
                                }
                            }
                            Value::Str(fn_name) => {
                                for item in items {
                                    let val = self.call_fn(fn_name, vec![item.clone()], caller_local)?;
                                    if val.is_truthy() {
                                        result.push(item.clone());
                                    }
                                }
                            }
                            _ => return Err("filter() second argument must be a lambda or function name".into()),
                        }
                        return Ok(Value::List(result));
                    }
                return Err("filter() expects (list, fn)".into());
            }
            "reduce" => {
                // reduce(list, initial, lambda)
                if args.len() >= 3
                    && let Value::List(items) = &args[0] {
                        let mut acc = args[1].clone();
                        if let Value::Lambda { params, body } = &args[2] {
                            if params.len() != 2 {
                                return Err("reduce() lambda must take exactly 2 parameters (acc, item)".into());
                            }
                            for item in items {
                                let mut scope: HashMap<String, Value> = HashMap::new();
                                scope.insert(params[0].clone(), acc.clone());
                                scope.insert(params[1].clone(), item.clone());
                                let mut local_scope = Some(scope);
                                acc = self.eval_expr(body, &mut local_scope)?;
                            }
                            return Ok(acc);
                        }
                    }
                return Err("reduce() expects (list, initial, lambda)".into());
            }
            "assert" => {
                if let Some(v) = args.first()
                    && !v.is_truthy() {
                        let msg = args
                            .get(1)
                            .map(|m| format!("{}", m))
                            .unwrap_or_else(|| "Assertion failed".into());
                        return Err(msg);
                    }
                return Ok(Value::Null);
            }
            // ── Terminal formatting built-ins ─────────────
            "color" => {
                // color("red", text) or color("bold", text)
                if args.len() >= 2 {
                    let code = match &args[0] {
                        Value::Str(c) => match c.as_str() {
                            "red"     => "\x1b[31m",
                            "green"   => "\x1b[32m",
                            "yellow"  => "\x1b[33m",
                            "blue"    => "\x1b[34m",
                            "magenta" => "\x1b[35m",
                            "cyan"    => "\x1b[36m",
                            "white"   => "\x1b[37m",
                            "gray" | "grey" => "\x1b[90m",
                            "bold"    => "\x1b[1m",
                            "dim"     => "\x1b[2m",
                            "italic"  => "\x1b[3m",
                            "underline" => "\x1b[4m",
                            "reset"   => "\x1b[0m",
                            _ => "",
                        },
                        _ => "",
                    };
                    let text = format!("{}", args[1]);
                    return Ok(Value::Str(format!("{}{}\x1b[0m", code, text)));
                }
                return Err("color(name, text) expects 2 arguments".into());
            }
            "pad_left" => {
                // pad_left(text, width) or pad_left(text, width, char)
                if let Some(Value::Str(s)) = args.first() {
                    let width = match args.get(1) {
                        Some(Value::Int(w)) => *w as usize,
                        _ => return Err("pad_left() expects (str, int)".into()),
                    };
                    let pad_char = match args.get(2) {
                        Some(Value::Str(c)) => c.chars().next().unwrap_or(' '),
                        _ => ' ',
                    };
                    let current = s.chars().count();
                    if current >= width {
                        return Ok(Value::Str(s.clone()));
                    }
                    let padding: String = std::iter::repeat(pad_char).take(width - current).collect();
                    return Ok(Value::Str(format!("{}{}", padding, s)));
                }
                return Err("pad_left() expects (str, int)".into());
            }
            "pad_right" => {
                // pad_right(text, width) or pad_right(text, width, char)
                if let Some(Value::Str(s)) = args.first() {
                    let width = match args.get(1) {
                        Some(Value::Int(w)) => *w as usize,
                        _ => return Err("pad_right() expects (str, int)".into()),
                    };
                    let pad_char = match args.get(2) {
                        Some(Value::Str(c)) => c.chars().next().unwrap_or(' '),
                        _ => ' ',
                    };
                    let current = s.chars().count();
                    if current >= width {
                        return Ok(Value::Str(s.clone()));
                    }
                    let padding: String = std::iter::repeat(pad_char).take(width - current).collect();
                    return Ok(Value::Str(format!("{}{}", s, padding)));
                }
                return Err("pad_right() expects (str, int)".into());
            }
            "format_number" => {
                // format_number(12345) → "12,345"
                if let Some(v) = args.first() {
                    let s = format!("{}", v);
                    // Add commas to integer portion
                    let (neg, digits) = if let Some(stripped) = s.strip_prefix('-') {
                        ("-", stripped)
                    } else {
                        ("", s.as_str())
                    };
                    let parts: Vec<&str> = digits.splitn(2, '.').collect();
                    let int_part = parts[0];
                    let mut result = String::new();
                    for (i, ch) in int_part.chars().rev().enumerate() {
                        if i > 0 && i % 3 == 0 {
                            result.push(',');
                        }
                        result.push(ch);
                    }
                    let formatted: String = result.chars().rev().collect();
                    if parts.len() > 1 {
                        return Ok(Value::Str(format!("{}{}.{}", neg, formatted, parts[1])));
                    }
                    return Ok(Value::Str(format!("{}{}", neg, formatted)));
                }
                return Err("format_number() expects a number".into());
            }
            "repeat_str" => {
                // repeat_str("─", 40) → "────────..."
                if let (Some(Value::Str(s)), Some(Value::Int(n))) = (args.first(), args.get(1)) {
                    if *n <= 0 { return Ok(Value::Str(String::new())); }
                    return Ok(Value::Str(s.repeat(*n as usize)));
                }
                return Err("repeat_str(str, count) expects (str, int)".into());
            }
            "clear_screen" => {
                print!("\x1b[2J\x1b[H");
                let _ = std::io::stdout().flush();
                return Ok(Value::Null);
            }
            // ── Turbo built-ins ──────────────────────────────
            "timestamp" if self.turbo => {
                return Ok(Value::Int(crate::turbo::timestamp_ms()));
            }
            "timestamp_ns" if self.turbo => {
                return Ok(Value::Int(crate::turbo::timestamp_ns()));
            }
            "bigint" if self.turbo => {
                return match args.first() {
                    Some(Value::Int(n)) => Ok(Value::BigInt(RsBigInt::from_i64(*n))),
                    Some(Value::Float(n)) => Ok(Value::BigInt(RsBigInt::from_i64(*n as i64))),
                    Some(Value::Str(s)) => {
                        RsBigInt::from_str(s)
                            .map(Value::BigInt)
                            .ok_or_else(|| format!("Cannot convert '{}' to bigint", s))
                    }
                    Some(Value::BigInt(n)) => Ok(Value::BigInt(n.clone())),
                    Some(Value::Bool(b)) => Ok(Value::BigInt(RsBigInt::from_i64(if *b { 1 } else { 0 }))),
                    _ => Ok(Value::BigInt(RsBigInt::zero())),
                };
            }
            "digit_count" if self.turbo => {
                return match args.first() {
                    Some(Value::BigInt(n)) => Ok(Value::Int(n.digit_count() as i64)),
                    Some(Value::Int(n)) => Ok(Value::Int(RsBigInt::from_i64(*n).digit_count() as i64)),
                    _ => Err("digit_count() expects a bigint or int".into()),
                };
            }
            "factorial" if self.turbo => {
                return match args.first() {
                    Some(Value::Int(n)) if *n >= 0 => {
                        Ok(Value::BigInt(crate::turbo::factorial(*n as u64)))
                    }
                    Some(Value::BigInt(n)) => {
                        match n.to_i64() {
                            Some(v) if v >= 0 => Ok(Value::BigInt(crate::turbo::factorial(v as u64))),
                            _ => Err("factorial() argument must be a non-negative integer".into()),
                        }
                    }
                    _ => Err("factorial() expects a non-negative integer".into()),
                };
            }
            "product_range" if self.turbo => {
                let lo = match args.first() {
                    Some(Value::Int(n)) if *n >= 0 => *n as u64,
                    _ => return Err("product_range() expects two non-negative integers".into()),
                };
                let hi = match args.get(1) {
                    Some(Value::Int(n)) if *n >= 0 => *n as u64,
                    _ => return Err("product_range() expects two non-negative integers".into()),
                };
                return Ok(Value::BigInt(crate::turbo::product_range(lo, hi)));
            }
            "mem_usage" if self.turbo => {
                return match args.first() {
                    Some(Value::BigInt(n)) => Ok(Value::Int(n.mem_bytes() as i64)),
                    Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
                    Some(Value::List(l)) => {
                        let base = std::mem::size_of::<Vec<Value>>() + l.capacity() * std::mem::size_of::<Value>();
                        Ok(Value::Int(base as i64))
                    }
                    _ => Ok(Value::Int(std::mem::size_of::<Value>() as i64)),
                };
            }

            "to_json" if self.turbo => {
                return match args.first() {
                    Some(v) => Ok(Value::Str(Self::value_to_json(v))),
                    None => Ok(Value::Str("null".to_string())),
                };
            }
            "write_json" if self.turbo => {
                if args.len() >= 2
                    && let Value::Str(path) = &args[0] {
                        let json = Self::value_to_json_pretty(&args[1], 0);
                        return std::fs::write(path, &json)
                            .map(|_| {
                                Value::Bool(true)
                            })
                            .map_err(|e| format!("write_json error: {}", e));
                    }
                return Err("write_json() expects (path, value)".into());
            }
            "read_json" if self.turbo => {
                if let Some(Value::Str(path)) = args.first() {
                    let content = std::fs::read_to_string(path)
                        .map_err(|e| format!("read_json error: {}", e))?;
                    return Ok(Value::Str(content));
                }
                return Err("read_json() expects a file path".into());
            }
            "gc" if self.turbo => {
                // Rust has no GC — memory is freed deterministically.
                // This is a no-op that returns confirmation.
                return Ok(Value::Str("turbo: zero-cost memory — no GC needed".into()));
            }
            "sleep" if self.turbo => {
                if let Some(Value::Int(ms)) = args.first() {
                    std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                    return Ok(Value::Null);
                }
                return Err("sleep() expects milliseconds (int)".into());
            }
            "exit" => {
                let code = match args.first() {
                    Some(Value::Int(n)) => *n as i32,
                    _ => 0,
                };
                std::process::exit(code);
            }
            _ => {}
        }

        // User-defined function
        let func = self
            .functions
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined function: '{}'", name))?;
        let (params, body) = func;

        if args.len() != params.len() {
            return Err(format!(
                "{}() expects {} arguments, got {}",
                name,
                params.len(),
                args.len()
            ));
        }

        let mut scope: HashMap<String, Value> = HashMap::new();
        for (param, arg) in params.iter().zip(args) {
            scope.insert(param.clone(), arg);
        }

        let mut local = Some(scope);
        for stmt in &body {
            match self.exec_stmt(stmt, &mut local)? {
                Signal::Return(v) => return Ok(v),
                Signal::None => {}
                Signal::Break => return Err("'break' outside of loop".into()),
                Signal::Continue => return Err("'continue' outside of loop".into()),
            }
        }
        Ok(Value::Null)
    }

    fn call_method(
        &mut self,
        object: Value,
        method: &str,
        args: Vec<Value>,
        _caller_local: &mut Option<HashMap<String, Value>>,
    ) -> Result<Value, String> {
        match (&object, method) {
            // ── String methods ───────────────────────────────
            (Value::Str(s), "upper") => Ok(Value::Str(s.to_uppercase())),
            (Value::Str(s), "lower") => Ok(Value::Str(s.to_lowercase())),
            (Value::Str(s), "trim") => Ok(Value::Str(s.trim().to_string())),
            (Value::Str(s), "strip") => Ok(Value::Str(s.trim().to_string())),
            (Value::Str(s), "lstrip") => Ok(Value::Str(s.trim_start().to_string())),
            (Value::Str(s), "rstrip") => Ok(Value::Str(s.trim_end().to_string())),
            (Value::Str(s), "length") => Ok(Value::Int(s.chars().count() as i64)),
            (Value::Str(s), "chars") => Ok(Value::List(
                s.chars().map(|c| Value::Str(c.to_string())).collect(),
            )),
            (Value::Str(s), "contains") => {
                if let Some(Value::Str(sub)) = args.first() {
                    Ok(Value::Bool(s.contains(sub.as_str())))
                } else {
                    Err("contains() expects a string argument".into())
                }
            }
            (Value::Str(s), "starts_with") => {
                if let Some(Value::Str(prefix)) = args.first() {
                    Ok(Value::Bool(s.starts_with(prefix.as_str())))
                } else {
                    Err("starts_with() expects a string argument".into())
                }
            }
            (Value::Str(s), "ends_with") => {
                if let Some(Value::Str(suffix)) = args.first() {
                    Ok(Value::Bool(s.ends_with(suffix.as_str())))
                } else {
                    Err("ends_with() expects a string argument".into())
                }
            }
            (Value::Str(s), "replace") => {
                if args.len() >= 2
                    && let (Value::Str(from), Value::Str(to)) = (&args[0], &args[1]) {
                        return Ok(Value::Str(s.replace(from.as_str(), to.as_str())));
                    }
                Err("replace() expects (old, new) string arguments".into())
            }
            (Value::Str(s), "find") => {
                if let Some(Value::Str(sub)) = args.first() {
                    match s.find(sub.as_str()) {
                        Some(pos) => Ok(Value::Int(pos as i64)),
                        None => Ok(Value::Int(-1)),
                    }
                } else {
                    Err("find() expects a string argument".into())
                }
            }
            (Value::Str(s), "count") => {
                if let Some(Value::Str(sub)) = args.first() {
                    Ok(Value::Int(s.matches(sub.as_str()).count() as i64))
                } else {
                    Err("count() expects a string argument".into())
                }
            }
            (Value::Str(s), "split") => {
                if let Some(Value::Str(delim)) = args.first() {
                    Ok(Value::List(
                        s.split(delim.as_str())
                            .map(|p| Value::Str(p.to_string()))
                            .collect(),
                    ))
                } else {
                    // Split on whitespace if no delimiter
                    Ok(Value::List(
                        s.split_whitespace()
                            .map(|p| Value::Str(p.to_string()))
                            .collect(),
                    ))
                }
            }
            (Value::Str(s), "repeat") => {
                if let Some(Value::Int(n)) = args.first() {
                    Ok(Value::Str(s.repeat(*n as usize)))
                } else {
                    Err("repeat() expects an integer argument".into())
                }
            }
            (Value::Str(s), "is_empty") => Ok(Value::Bool(s.is_empty())),
            (Value::Str(s), "is_digit") => Ok(Value::Bool(s.chars().all(|c| c.is_ascii_digit()))),
            (Value::Str(s), "is_alpha") => Ok(Value::Bool(s.chars().all(|c| c.is_alphabetic()))),
            (Value::Str(s), "slice") => {
                let len = s.chars().count() as i64;
                let start = match args.first() {
                    Some(Value::Int(n)) => {
                        if *n < 0 { (len + n).max(0) as usize } else { *n as usize }
                    }
                    _ => 0,
                };
                let end = match args.get(1) {
                    Some(Value::Int(n)) => {
                        if *n < 0 { (len + n).max(0) as usize } else { *n as usize }
                    }
                    _ => len as usize,
                };
                let sliced: String = s.chars().skip(start).take(end.saturating_sub(start)).collect();
                Ok(Value::Str(sliced))
            }

            // ── List methods ─────────────────────────────────
            (Value::List(l), "length") => Ok(Value::Int(l.len() as i64)),
            (Value::List(l), "is_empty") => Ok(Value::Bool(l.is_empty())),
            (Value::List(l), "contains") => {
                if let Some(val) = args.first() {
                    let found = l.iter().any(|v| self.values_equal(v, val));
                    Ok(Value::Bool(found))
                } else {
                    Err("contains() expects an argument".into())
                }
            }
            (Value::List(l), "index") => {
                if let Some(val) = args.first() {
                    for (i, v) in l.iter().enumerate() {
                        if self.values_equal(v, val) {
                            return Ok(Value::Int(i as i64));
                        }
                    }
                    Ok(Value::Int(-1))
                } else {
                    Err("index() expects an argument".into())
                }
            }
            (Value::List(l), "count") => {
                if let Some(val) = args.first() {
                    let count = l.iter().filter(|v| self.values_equal(v, val)).count();
                    Ok(Value::Int(count as i64))
                } else {
                    Err("count() expects an argument".into())
                }
            }
            (Value::List(l), "join") => {
                let delim = match args.first() {
                    Some(Value::Str(d)) => d.as_str(),
                    _ => "",
                };
                let parts: Vec<String> = l.iter().map(|v| format!("{}", v)).collect();
                Ok(Value::Str(parts.join(delim)))
            }
            (Value::List(l), "reverse") => {
                let mut rev = l.clone();
                rev.reverse();
                Ok(Value::List(rev))
            }
            (Value::List(l), "sort") => {
                let mut sorted = l.clone();
                sorted.sort_by(|a, b| {
                    a.to_float()
                        .partial_cmp(&b.to_float())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                Ok(Value::List(sorted))
            }
            (Value::List(l), "slice") => {
                let len = l.len() as i64;
                let start = match args.first() {
                    Some(Value::Int(n)) => {
                        if *n < 0 { (len + n).max(0) as usize } else { *n as usize }
                    }
                    _ => 0,
                };
                let end = match args.get(1) {
                    Some(Value::Int(n)) => {
                        if *n < 0 { (len + n).max(0) as usize } else { (*n as usize).min(l.len()) }
                    }
                    _ => l.len(),
                };
                Ok(Value::List(l[start..end].to_vec()))
            }
            (Value::List(l), "first") => {
                Ok(l.first().cloned().unwrap_or(Value::Null))
            }
            (Value::List(l), "last") => {
                Ok(l.last().cloned().unwrap_or(Value::Null))
            }
            (Value::List(l), "flat") => {
                // Flatten one level
                let mut result = Vec::new();
                for item in l {
                    match item {
                        Value::List(inner) => result.extend(inner.clone()),
                        other => result.push(other.clone()),
                    }
                }
                Ok(Value::List(result))
            }
            (Value::List(l), "unique") => {
                let mut result = Vec::new();
                for item in l {
                    let already = result.iter().any(|v: &Value| self.values_equal(v, item));
                    if !already {
                        result.push(item.clone());
                    }
                }
                Ok(Value::List(result))
            }
            (Value::List(l), "map") => {
                if let Some(Value::Lambda { params, body }) = args.first() {
                    if params.len() != 1 {
                        return Err("map() lambda must take exactly 1 parameter".into());
                    }
                    let mut result = Vec::new();
                    for item in l {
                        let mut scope: HashMap<String, Value> = HashMap::new();
                        scope.insert(params[0].clone(), item.clone());
                        let mut local_scope = Some(scope);
                        let val = self.eval_expr(body, &mut local_scope)?;
                        result.push(val);
                    }
                    Ok(Value::List(result))
                } else {
                    Err("map() expects a lambda argument".into())
                }
            }
            (Value::List(l), "filter") => {
                if let Some(Value::Lambda { params, body }) = args.first() {
                    if params.len() != 1 {
                        return Err("filter() lambda must take exactly 1 parameter".into());
                    }
                    let mut result = Vec::new();
                    for item in l {
                        let mut scope: HashMap<String, Value> = HashMap::new();
                        scope.insert(params[0].clone(), item.clone());
                        let mut local_scope = Some(scope);
                        let val = self.eval_expr(body, &mut local_scope)?;
                        if val.is_truthy() {
                            result.push(item.clone());
                        }
                    }
                    Ok(Value::List(result))
                } else {
                    Err("filter() expects a lambda argument".into())
                }
            }
            (Value::List(l), "each") => {
                // .each(lambda) — like forEach, runs side effects
                if let Some(Value::Lambda { params, body }) = args.first() {
                    for item in l {
                        let mut scope: HashMap<String, Value> = HashMap::new();
                        if !params.is_empty() {
                            scope.insert(params[0].clone(), item.clone());
                        }
                        let mut local_scope = Some(scope);
                        self.eval_expr(body, &mut local_scope)?;
                    }
                    Ok(Value::Null)
                } else {
                    Err("each() expects a lambda argument".into())
                }
            }

            // ── Dict methods ─────────────────────────────────
            (Value::Dict(d), "keys") => {
                Ok(Value::List(d.iter().map(|(k, _)| k.clone()).collect()))
            }
            (Value::Dict(d), "values") => {
                Ok(Value::List(d.iter().map(|(_, v)| v.clone()).collect()))
            }
            (Value::Dict(d), "items") => {
                let result: Vec<Value> = d
                    .iter()
                    .map(|(k, v)| Value::List(vec![k.clone(), v.clone()]))
                    .collect();
                Ok(Value::List(result))
            }
            (Value::Dict(d), "has") | (Value::Dict(d), "contains") => {
                if let Some(key) = args.first() {
                    let found = d.iter().any(|(k, _)| k.dict_key_eq(key));
                    Ok(Value::Bool(found))
                } else {
                    Err("has() expects a key argument".into())
                }
            }
            (Value::Dict(d), "get") => {
                if let Some(key) = args.first() {
                    for (k, v) in d {
                        if k.dict_key_eq(key) {
                            return Ok(v.clone());
                        }
                    }
                    // Return default if provided, otherwise null
                    Ok(args.get(1).cloned().unwrap_or(Value::Null))
                } else {
                    Err("get() expects a key argument".into())
                }
            }
            (Value::Dict(d), "length") => Ok(Value::Int(d.len() as i64)),
            (Value::Dict(d), "is_empty") => Ok(Value::Bool(d.is_empty())),
            (Value::Dict(d), "merge") => {
                if let Some(Value::Dict(other)) = args.first() {
                    let mut merged = d.clone();
                    for pair in other {
                        let mut found = false;
                        for existing in merged.iter_mut() {
                            if existing.0.dict_key_eq(&pair.0) {
                                existing.1 = pair.1.clone();
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            merged.push(pair.clone());
                        }
                    }
                    Ok(Value::Dict(merged))
                } else {
                    Err("merge() expects a dict argument".into())
                }
            }

            _ => Err(format!("No method '{}' on {}", method, object.type_name())),
        }
    }

    // ── string interpolation ─────────────────────────────────

    fn interpolate(
        &mut self,
        s: &str,
        local: &mut Option<HashMap<String, Value>>,
    ) -> Result<String, String> {
        if !s.contains('{') {
            // Restore escaped braces: \u{E000} → {  \u{E001} → }
            let restored = s.replace('\u{E000}', "{").replace('\u{E001}', "}");
            return Ok(restored);
        }

        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '\u{E000}' {
                // Escaped brace → literal {
                result.push('{');
                i += 1;
            } else if chars[i] == '\u{E001}' {
                // Escaped brace → literal }
                result.push('}');
                i += 1;
            } else if chars[i] == '{' {
                i += 1;
                let mut depth = 1;
                let mut expr_str = String::new();
                while i < chars.len() && depth > 0 {
                    if chars[i] == '{' {
                        depth += 1;
                    }
                    if chars[i] == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    expr_str.push(chars[i]);
                    i += 1;
                }
                i += 1; // skip }

                // Simple: try to look up as identifier, or evaluate
                let trimmed = expr_str.trim();
                if let Some(scope) = &*local
                    && let Some(v) = scope.get(trimmed)
                {
                    result.push_str(&format!("{}", v));
                    continue;
                }
                if let Some(v) = self.globals.get(trimmed) {
                    result.push_str(&format!("{}", v));
                    continue;
                }

                // Try to tokenize & parse & evaluate the expression
                let mut lexer = crate::lexer::Lexer::new(trimmed);
                match lexer.tokenize() {
                    Ok(tokens) => {
                        let mut parser = crate::parser::Parser::new(tokens);
                        match parser.parse_program() {
                            Ok(prog) => {
                                if let Some(Stmt::Expr(expr)) = prog.stmts.first() {
                                    match self.eval_expr(expr, local) {
                                        Ok(val) => result.push_str(&format!("{}", val)),
                                        Err(_) => result.push_str(trimmed),
                                    }
                                } else {
                                    result.push_str(trimmed);
                                }
                            }
                            Err(_) => result.push_str(trimmed),
                        }
                    }
                    Err(_) => result.push_str(trimmed),
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        Ok(result)
    }

    // ── JSON serialization (turbo) ───────────────────────────

    fn value_to_json(val: &Value) -> String {
        match val {
            Value::Int(n) => n.to_string(),
            Value::Float(n) => {
                if n.is_infinite() || n.is_nan() {
                    "null".to_string()
                } else {
                    n.to_string()
                }
            }
            Value::Str(s) => {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                format!("\"{}\"", escaped)
            }
            Value::Bool(b) => b.to_string(),
            Value::List(items) => {
                let parts: Vec<String> = items.iter().map(Self::value_to_json).collect();
                format!("[{}]", parts.join(","))
            }
            Value::Dict(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| {
                        let key = match k {
                            Value::Str(s) => {
                                format!("\"{}\"", s.replace('"', "\\\""))
                            }
                            other => Self::value_to_json(other),
                        };
                        format!("{}:{}", key, Self::value_to_json(v))
                    })
                    .collect();
                format!("{{{}}}", parts.join(","))
            }
            Value::BigInt(n) => format!("\"{}\"", n),
            Value::Lambda { .. } => "null".to_string(),
            Value::Null => "null".to_string(),
        }
    }

    fn value_to_json_pretty(val: &Value, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        let inner_pad = "  ".repeat(indent + 1);
        match val {
            Value::List(items) if !items.is_empty() => {
                let parts: Vec<String> = items
                    .iter()
                    .map(|v| format!("{}{}", inner_pad, Self::value_to_json_pretty(v, indent + 1)))
                    .collect();
                format!("[\n{}\n{}]", parts.join(",\n"), pad)
            }
            Value::Dict(pairs) if !pairs.is_empty() => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| {
                        let key = match k {
                            Value::Str(s) => format!("\"{}\"", s.replace('"', "\\\"")),
                            other => Self::value_to_json(other),
                        };
                        format!(
                            "{}{}: {}",
                            inner_pad,
                            key,
                            Self::value_to_json_pretty(v, indent + 1)
                        )
                    })
                    .collect();
                format!("{{\n{}\n{}}}", parts.join(",\n"), pad)
            }
            _ => Self::value_to_json(val),
        }
    }
}
