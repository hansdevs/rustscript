/// Tree-walking interpreter for RustScript.
/// Used in `--run` mode to execute programs directly in the terminal.

use std::collections::HashMap;
use std::fmt;

use crate::ast::*;

// ── runtime values ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
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
            Value::Null => write!(f, "null"),
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
            Value::Null => "null",
        }
    }

    pub fn to_float(&self) -> f64 {
        match self {
            Value::Int(n) => *n as f64,
            Value::Float(n) => *n,
            Value::Str(s) => s.parse().unwrap_or(0.0),
            Value::Bool(b) => if *b { 1.0 } else { 0.0 },
            _ => 0.0,
        }
    }
}

// ── control-flow signals ─────────────────────────────────────

enum Signal {
    None,
    Return(Value),
}

// ── interpreter ──────────────────────────────────────────────

pub struct Interpreter {
    /// Global variable scope
    globals: HashMap<String, Value>,
    /// Function declarations (name → (params, body))
    functions: HashMap<String, (Vec<String>, Vec<Stmt>)>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            globals: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Run a full program.
    pub fn run(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.stmts {
            match self.exec_stmt(stmt, &mut None)? {
                Signal::Return(_) => {
                    return Err("'return' outside of function".into());
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
                let val = self.eval_expr(value, local)?;
                // Check local first, then global
                if let Some(scope) = local {
                    if scope.contains_key(name) {
                        scope.insert(name.clone(), val);
                        return Ok(Signal::None);
                    }
                }
                self.globals.insert(name.clone(), val);
                Ok(Signal::None)
            }
            Stmt::IndexAssign { list, index, value } => {
                let idx = self.eval_expr(index, local)?;
                let val = self.eval_expr(value, local)?;
                let idx = match idx {
                    Value::Int(i) => i as usize,
                    _ => return Err("Index must be an integer".into()),
                };
                // Find the list in scope
                let list_val = if let Some(scope) = local {
                    if scope.contains_key(list) {
                        scope.get_mut(list)
                    } else {
                        self.globals.get_mut(list)
                    }
                } else {
                    self.globals.get_mut(list)
                };
                if let Some(Value::List(items)) = list_val {
                    if idx < items.len() {
                        items[idx] = val;
                    } else {
                        return Err(format!("Index {} out of bounds (len {})", idx, items.len()));
                    }
                } else {
                    return Err(format!("'{}' is not a list", list));
                }
                Ok(Signal::None)
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
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                let c = self.eval_expr(cond, local)?;
                if c.is_truthy() {
                    for s in then_body {
                        let sig = self.exec_stmt(s, local)?;
                        if let Signal::Return(_) = sig {
                            return Ok(sig);
                        }
                    }
                } else if let Some(eb) = else_body {
                    for s in eb {
                        let sig = self.exec_stmt(s, local)?;
                        if let Signal::Return(_) = sig {
                            return Ok(sig);
                        }
                    }
                }
                Ok(Signal::None)
            }
            Stmt::While { cond, body } => {
                loop {
                    let c = self.eval_expr(cond, local)?;
                    if !c.is_truthy() {
                        break;
                    }
                    for s in body {
                        let sig = self.exec_stmt(s, local)?;
                        if let Signal::Return(_) = sig {
                            return Ok(sig);
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
                    other => {
                        return Err(format!("Cannot iterate over {}", other.type_name()));
                    }
                };
                for item in items {
                    if let Some(scope) = local {
                        scope.insert(var.clone(), item);
                    } else {
                        self.globals.insert(var.clone(), item);
                    }
                    for s in body {
                        let sig = self.exec_stmt(s, local)?;
                        if let Signal::Return(_) = sig {
                            return Ok(sig);
                        }
                    }
                }
                Ok(Signal::None)
            }
            Stmt::Page { .. } => {
                // In interpreter mode, skip page blocks (they're for HTML output)
                println!("[info] Page block skipped in --run mode. Use build mode to generate HTML.");
                Ok(Signal::None)
            }
            Stmt::Import { .. } => {
                // Imports are resolved before interpretation; should not reach here.
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
            Expr::Ident(name) => {
                if let Some(scope) = &*local {
                    if let Some(v) = scope.get(name) {
                        return Ok(v.clone());
                    }
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
                self.call_fn(name, arg_vals)
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
                self.call_method(obj, method, arg_vals)
            }
            Expr::Index { object, index } => {
                let obj = self.eval_expr(object, local)?;
                let idx = self.eval_expr(index, local)?;
                match (&obj, &idx) {
                    (Value::List(items), Value::Int(i)) => {
                        let i = *i as usize;
                        items
                            .get(i)
                            .cloned()
                            .ok_or_else(|| format!("Index {} out of bounds (len {})", i, items.len()))
                    }
                    (Value::Str(s), Value::Int(i)) => {
                        let i = *i as usize;
                        s.chars()
                            .nth(i)
                            .map(|c| Value::Str(c.to_string()))
                            .ok_or_else(|| format!("Index {} out of bounds", i))
                    }
                    _ => Err("Invalid index operation".into()),
                }
            }
            Expr::Member { object, field } => {
                let obj = self.eval_expr(object, local)?;
                match (&obj, field.as_str()) {
                    (Value::List(items), "length") => Ok(Value::Int(items.len() as i64)),
                    (Value::Str(s), "length") => Ok(Value::Int(s.len() as i64)),
                    _ => Err(format!("No field '{}' on {}", field, obj.type_name())),
                }
            }
        }
    }

    fn eval_binop(&self, left: &Value, op: BinOp, right: &Value) -> Result<Value, String> {
        match op {
            BinOp::Add => match (left, right) {
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
                _ => Err(format!(
                    "Cannot add {} and {}",
                    left.type_name(),
                    right.type_name()
                )),
            },
            BinOp::Sub => self.numeric_op(left, right, |a, b| a - b, |a, b| a - b),
            BinOp::Mul => match (left, right) {
                (Value::Str(s), Value::Int(n)) | (Value::Int(n), Value::Str(s)) => {
                    Ok(Value::Str(s.repeat(*n as usize)))
                }
                _ => self.numeric_op(left, right, |a, b| a * b, |a, b| a * b),
            },
            BinOp::Div => self.numeric_op(
                left,
                right,
                |a, b| {
                    if b == 0 {
                        0 // prevent panic, or we could error
                    } else {
                        a / b
                    }
                },
                |a, b| a / b,
            ),
            BinOp::Mod => self.numeric_op(
                left,
                right,
                |a, b| {
                    if b == 0 {
                        0
                    } else {
                        a % b
                    }
                },
                |a, b| a % b,
            ),
            BinOp::Eq => Ok(Value::Bool(self.values_equal(left, right))),
            BinOp::NotEq => Ok(Value::Bool(!self.values_equal(left, right))),
            BinOp::Lt => Ok(Value::Bool(left.to_float() < right.to_float())),
            BinOp::Gt => Ok(Value::Bool(left.to_float() > right.to_float())),
            BinOp::LtEq => Ok(Value::Bool(left.to_float() <= right.to_float())),
            BinOp::GtEq => Ok(Value::Bool(left.to_float() >= right.to_float())),
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
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    // ── function calls ───────────────────────────────────────

    fn call_fn(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        // Built-in functions
        match name {
            "print" => {
                let parts: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
                println!("{}", parts.join(" "));
                return Ok(Value::Null);
            }
            "len" => {
                return match args.first() {
                    Some(Value::List(l)) => Ok(Value::Int(l.len() as i64)),
                    Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
                    _ => Err("len() expects a list or string".into()),
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
                        .parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| format!("Cannot convert '{}' to float", s)),
                    _ => Ok(Value::Float(0.0)),
                };
            }
            "push" => {
                if args.len() >= 2 {
                    // We need to push to a list in scope – but we have the value, not the name.
                    // This is a limitation of the current design; push modifies in place.
                    // For now, return a new list.
                    if let Value::List(mut l) = args[0].clone() {
                        l.push(args[1].clone());
                        return Ok(Value::List(l));
                    }
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
                    [Value::Int(n)] => {
                        Ok(Value::List((0..*n).map(Value::Int).collect()))
                    }
                    [Value::Int(start), Value::Int(end)] => {
                        Ok(Value::List((*start..*end).map(Value::Int).collect()))
                    }
                    _ => Err("range() expects 1 or 2 integer arguments".into()),
                };
            }
            "type" => {
                return match args.first() {
                    Some(v) => Ok(Value::Str(v.type_name().to_string())),
                    None => Ok(Value::Str("null".into())),
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
                if args.len() >= 2 {
                    let a = args[0].to_float();
                    let b = args[1].to_float();
                    return Ok(Value::Float(a.min(b)));
                }
                return Err("min() expects 2 arguments".into());
            }
            "max" => {
                if args.len() >= 2 {
                    let a = args[0].to_float();
                    let b = args[1].to_float();
                    return Ok(Value::Float(a.max(b)));
                }
                return Err("max() expects 2 arguments".into());
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
            }
        }
        Ok(Value::Null)
    }

    fn call_method(
        &self,
        object: Value,
        method: &str,
        _args: Vec<Value>,
    ) -> Result<Value, String> {
        match (&object, method) {
            (Value::Str(s), "upper") => Ok(Value::Str(s.to_uppercase())),
            (Value::Str(s), "lower") => Ok(Value::Str(s.to_lowercase())),
            (Value::Str(s), "trim") => Ok(Value::Str(s.trim().to_string())),
            (Value::Str(s), "length") => Ok(Value::Int(s.len() as i64)),
            (Value::List(l), "length") => Ok(Value::Int(l.len() as i64)),
            (Value::Str(s), "contains") => {
                if let Some(Value::Str(sub)) = _args.first() {
                    Ok(Value::Bool(s.contains(sub.as_str())))
                } else {
                    Err("contains() expects a string argument".into())
                }
            }
            (Value::Str(s), "split") => {
                if let Some(Value::Str(delim)) = _args.first() {
                    Ok(Value::List(
                        s.split(delim.as_str())
                            .map(|p| Value::Str(p.to_string()))
                            .collect(),
                    ))
                } else {
                    Err("split() expects a string argument".into())
                }
            }
            (Value::List(l), "join") => {
                if let Some(Value::Str(delim)) = _args.first() {
                    let parts: Vec<String> = l.iter().map(|v| format!("{}", v)).collect();
                    Ok(Value::Str(parts.join(delim)))
                } else {
                    Err("join() expects a string argument".into())
                }
            }
            _ => Err(format!(
                "No method '{}' on {}",
                method,
                object.type_name()
            )),
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
                if let Some(scope) = &*local {
                    if let Some(v) = scope.get(trimmed) {
                        result.push_str(&format!("{}", v));
                        continue;
                    }
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
}
