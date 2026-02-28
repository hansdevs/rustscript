/// Abstract Syntax Tree types for RustScript.

// ── Program ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

// ── Statements ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let name = expr`
    Let {
        name: String,
        value: Expr,
    },
    /// `name = expr`  or  `name += expr`  etc.
    Assign {
        name: String,
        value: Expr,
    },
    /// Index assignment: `list[idx] = expr`
    IndexAssign {
        list: String,
        index: Expr,
        value: Expr,
    },
    /// `fn name(params) { body }`
    FnDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    /// `return expr?`
    Return(Option<Expr>),
    /// `if cond { then } else { else }`
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    /// `while cond { body }`
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    /// `for var in iter { body }`
    For {
        var: String,
        iter: Expr,
        body: Vec<Stmt>,
    },
    /// `page { elements }`
    Page {
        elements: Vec<Element>,
    },
    /// Expression used as a statement (e.g. function call)
    Expr(Expr),
}

// ── Expressions ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Ident(String),
    List(Vec<Expr>),

    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    /// Function call: `name(args)`
    Call {
        name: String,
        args: Vec<Expr>,
    },
    /// Method call: `expr.method(args)`
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    /// Index: `expr[index]`
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    /// Member access: `expr.field`
    Member {
        object: Box<Expr>,
        field: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

// ── Page elements ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Element {
    /// An HTML tag element.
    Tag {
        tag: String,
        text: Option<Expr>,
        attrs: Vec<Attribute>,
        style: Vec<StyleProp>,
        events: Vec<Event>,
        children: Vec<Element>,
    },
    /// Conditional rendering inside a page.
    If {
        cond: Expr,
        then_els: Vec<Element>,
        else_els: Option<Vec<Element>>,
    },
    /// Loop rendering inside a page.
    For {
        var: String,
        iter: Expr,
        body: Vec<Element>,
    },
}

#[derive(Debug, Clone)]
pub struct StyleProp {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,  // e.g. "click", "input"
    pub body: Vec<Stmt>,
}
