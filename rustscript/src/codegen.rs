//! Code generator: transforms a RustScript AST into a self-contained HTML file
//! with embedded CSS and JavaScript.

use crate::ast::*;

// ── Custom CSS shorthand system ──────────────────────────────────────────────
// Maps RustScript style shorthands → one or more CSS property-value pairs.
// Standard CSS property names pass through unchanged.
fn map_style_prop(name: &str, value: &str) -> Vec<(String, String)> {
    let v = value.to_string();
    let flag = value.is_empty(); // flag-style prop (no value)

    match name {
        // ── Typography ──────────────────────────────────────
        "size" => vec![("font-size".into(), v)],
        "font" => vec![("font-family".into(), v)],
        "weight" => vec![("font-weight".into(), v)],
        "bold" => vec![("font-weight".into(), if flag { "bold".into() } else { v })],
        "italic" => vec![("font-style".into(), if flag { "italic".into() } else { v })],
        "underline" => vec![(
            "text-decoration".into(),
            if flag { "underline".into() } else { v },
        )],
        "strike" => vec![(
            "text-decoration".into(),
            if flag { "line-through".into() } else { v },
        )],
        "uppercase" => vec![(
            "text-transform".into(),
            if flag { "uppercase".into() } else { v },
        )],
        "lowercase" => vec![(
            "text-transform".into(),
            if flag { "lowercase".into() } else { v },
        )],
        "capitalize" => vec![(
            "text-transform".into(),
            if flag { "capitalize".into() } else { v },
        )],
        "spacing" => vec![("letter-spacing".into(), v)],
        "lh" => vec![("line-height".into(), v)],
        "align" => vec![("text-align".into(), v)],
        "indent" => vec![("text-indent".into(), v)],

        // ── Colors & Background ─────────────────────────────
        "bg" => vec![("background".into(), v)],
        "fg" => vec![("color".into(), v)],

        // ── Spacing ─────────────────────────────────────────
        "pad" => vec![("padding".into(), v)],
        "pt" => vec![("padding-top".into(), v)],
        "pb" => vec![("padding-bottom".into(), v)],
        "pl" => vec![("padding-left".into(), v)],
        "pr" => vec![("padding-right".into(), v)],
        "px" => vec![
            ("padding-left".into(), v.clone()),
            ("padding-right".into(), v),
        ],
        "py" => vec![
            ("padding-top".into(), v.clone()),
            ("padding-bottom".into(), v),
        ],
        "m" => vec![("margin".into(), v)],
        "mt" => vec![("margin-top".into(), v)],
        "mb" => vec![("margin-bottom".into(), v)],
        "ml" => vec![("margin-left".into(), v)],
        "mr" => vec![("margin-right".into(), v)],
        "mx" => vec![
            ("margin-left".into(), v.clone()),
            ("margin-right".into(), v),
        ],
        "my" => vec![
            ("margin-top".into(), v.clone()),
            ("margin-bottom".into(), v),
        ],

        // ── Sizing ──────────────────────────────────────────
        "w" => vec![("width".into(), v)],
        "h" => vec![("height".into(), v)],
        "minw" => vec![("min-width".into(), v)],
        "maxw" => vec![("max-width".into(), v)],
        "minh" => vec![("min-height".into(), v)],
        "maxh" => vec![("max-height".into(), v)],

        // ── Border & Shape ──────────────────────────────────
        "radius" => vec![("border-radius".into(), v)],
        "shadow" => vec![("box-shadow".into(), v)],
        "outline" => vec![("outline".into(), v)],

        // ── Layout (flag-style compound properties) ─────────
        "row" if flag => vec![
            ("display".into(), "flex".into()),
            ("flex-direction".into(), "row".into()),
        ],
        "col" if flag => vec![
            ("display".into(), "flex".into()),
            ("flex-direction".into(), "column".into()),
        ],
        "center" if flag => vec![
            ("display".into(), "flex".into()),
            ("justify-content".into(), "center".into()),
            ("align-items".into(), "center".into()),
        ],
        "hidden" if flag => vec![("display".into(), "none".into())],
        "pointer" if flag => vec![("cursor".into(), "pointer".into())],
        "nowrap" if flag => vec![("white-space".into(), "nowrap".into())],
        "clip" if flag => vec![("overflow".into(), "hidden".into())],
        "scroll" if flag => vec![("overflow".into(), "auto".into())],
        "fixed" if flag => vec![("position".into(), "fixed".into())],
        "absolute" if flag => vec![("position".into(), "absolute".into())],
        "relative" if flag => vec![("position".into(), "relative".into())],
        "sticky" if flag => vec![("position".into(), "sticky".into())],
        "inline" if flag => vec![("display".into(), "inline".into())],
        "block" if flag => vec![("display".into(), "block".into())],
        "grid" if flag => vec![("display".into(), "grid".into())],

        // ── Flex / Grid ─────────────────────────────────────
        "items" => vec![("align-items".into(), v)],
        "justify" => vec![("justify-content".into(), v)],
        "self-align" => vec![("align-self".into(), v)],
        "grow" => vec![("flex-grow".into(), v)],
        "shrink" => vec![("flex-shrink".into(), v)],
        "basis" => vec![("flex-basis".into(), v)],
        "wrap" => vec![("flex-wrap".into(), v)],
        "gap" => vec![("gap".into(), v)],
        "cols" => vec![("grid-template-columns".into(), v)],
        "rows" => vec![("grid-template-rows".into(), v)],

        // ── Position ────────────────────────────────────────
        "z" => vec![("z-index".into(), v)],
        "pos" => vec![("position".into(), v)],

        // ── Effects ─────────────────────────────────────────
        "opacity" => vec![("opacity".into(), v)],
        "transition" => vec![("transition".into(), v)],
        "transform" => vec![("transform".into(), v)],
        "filter" => vec![("filter".into(), v)],
        "backdrop" => vec![("backdrop-filter".into(), v)],

        // ── Passthrough: standard CSS property names ────────
        _ => vec![(name.to_string(), v)],
    }
}

pub struct Codegen {
    /// Accumulated JavaScript for variable declarations
    js_vars: String,
    /// Accumulated JavaScript for function declarations
    js_fns: String,
    /// Accumulated JavaScript for top-level (imperative) code
    js_top: String,
    /// Accumulated CSS for page-level styles
    css: String,
    /// Accumulated HTML render body
    js_render: String,
    /// Event handlers  (id → js code)
    events: Vec<(String, String)>,
    /// Counter for unique event IDs
    evt_counter: usize,
    /// Whether a page block was found
    has_page: bool,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            js_vars: String::new(),
            js_fns: String::new(),
            js_top: String::new(),
            css: String::new(),
            js_render: String::new(),
            events: Vec::new(),
            evt_counter: 0,
            has_page: false,
        }
    }

    /// Generate a full HTML document from the program AST.
    pub fn generate(&mut self, program: &Program) -> String {
        // Process all statements
        for stmt in &program.stmts {
            self.gen_stmt(stmt, false);
        }

        self.build_html()
    }

    // ── statement codegen ────────────────────────────────────

    fn gen_stmt(&mut self, stmt: &Stmt, in_fn: bool) -> String {
        match stmt {
            Stmt::Let { name, value } => {
                let val_js = self.gen_expr(value);
                let line = format!("var {} = {};\n", name, val_js);
                if in_fn {
                    line
                } else {
                    self.js_vars.push_str(&line);
                    String::new()
                }
            }
            Stmt::Assign { name, value } => {
                let val_js = self.gen_expr(value);
                let line = format!("{} = {};\n", name, val_js);
                if in_fn {
                    line
                } else {
                    self.js_top.push_str(&line);
                    String::new()
                }
            }
            Stmt::IndexAssign { list, index, value } => {
                let idx = self.gen_expr(index);
                let val = self.gen_expr(value);
                let line = format!("{}[{}] = {};\n", list, idx, val);
                if in_fn {
                    line
                } else {
                    self.js_top.push_str(&line);
                    String::new()
                }
            }
            Stmt::FnDecl { name, params, body } => {
                let params_js = params.join(", ");
                let body_js = self.gen_body(body);
                let line = format!("function {}({}) {{\n{}}}\n", name, params_js, body_js);
                if in_fn {
                    line
                } else {
                    self.js_fns.push_str(&line);
                    String::new()
                }
            }
            Stmt::Return(val) => {
                if let Some(expr) = val {
                    format!("return {};\n", self.gen_expr(expr))
                } else {
                    "return;\n".to_string()
                }
            }
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                let cond_js = self.gen_expr(cond);
                let then_js = self.gen_body(then_body);
                let mut s = format!("if ({}) {{\n{}}}", cond_js, then_js);
                if let Some(eb) = else_body {
                    let else_js = self.gen_body(eb);
                    s.push_str(&format!(" else {{\n{}}}", else_js));
                }
                s.push('\n');
                if in_fn {
                    s
                } else {
                    self.js_top.push_str(&s);
                    String::new()
                }
            }
            Stmt::While { cond, body } => {
                let cond_js = self.gen_expr(cond);
                let body_js = self.gen_body(body);
                let s = format!("while ({}) {{\n{}}}\n", cond_js, body_js);
                if in_fn {
                    s
                } else {
                    self.js_top.push_str(&s);
                    String::new()
                }
            }
            Stmt::For { var, iter, body } => {
                let iter_js = self.gen_expr(iter);
                let body_js = self.gen_body(body);
                let s = format!(
                    "for (var __i = 0; __i < {iter}.length; __i++) {{\nvar {v} = {iter}[__i];\n{body}}}\n",
                    iter = iter_js,
                    v = var,
                    body = body_js
                );
                if in_fn {
                    s
                } else {
                    self.js_top.push_str(&s);
                    String::new()
                }
            }
            Stmt::Page { elements } => {
                self.has_page = true;
                self.gen_page_elements(elements);
                String::new()
            }
            Stmt::Expr(expr) => {
                let e = self.gen_expr(expr);
                let line = format!("{};\n", e);
                if in_fn {
                    line
                } else {
                    self.js_top.push_str(&line);
                    String::new()
                }
            }
            Stmt::Import { .. } => {
                // Imports are resolved before codegen; should not reach here.
                String::new()
            }
            Stmt::Break => {
                let s = "break;\n".to_string();
                if in_fn { s } else { self.js_top.push_str(&s); String::new() }
            }
            Stmt::Continue => {
                let s = "continue;\n".to_string();
                if in_fn { s } else { self.js_top.push_str(&s); String::new() }
            }
            Stmt::MemberAssign { object, field, value } => {
                let val_js = self.gen_expr(value);
                let line = format!("{}.{} = {};\n", object, field, val_js);
                if in_fn {
                    line
                } else {
                    self.js_top.push_str(&line);
                    String::new()
                }
            }
        }
    }

    fn gen_body(&mut self, stmts: &[Stmt]) -> String {
        let mut s = String::new();
        for stmt in stmts {
            s.push_str(&self.gen_stmt(stmt, true));
        }
        s
    }

    // ── expression codegen ───────────────────────────────────

    fn gen_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(n) => n.to_string(),
            Expr::Float(n) => format!("{}", n),
            Expr::Str(s) => self.gen_interpolated_string(s),
            Expr::Bool(b) => b.to_string(),
            Expr::None => "null".to_string(),
            Expr::Ident(name) => name.clone(),
            Expr::List(items) => {
                let inner: Vec<String> = items.iter().map(|e| self.gen_expr(e)).collect();
                format!("[{}]", inner.join(", "))
            }
            Expr::Dict(pairs) => {
                let inner: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", self.gen_expr(k), self.gen_expr(v)))
                    .collect();
                format!("{{{}}}", inner.join(", "))
            }
            Expr::Lambda { params, body } => {
                let params_js = params.join(", ");
                let body_js = self.gen_expr(body);
                format!("(function({}) {{ return {}; }})", params_js, body_js)
            }
            Expr::PipeCall { value, func, extra_args } => {
                let val_js = self.gen_expr(value);
                let mut all_args = vec![val_js];
                for a in extra_args {
                    all_args.push(self.gen_expr(a));
                }
                format!("{}({})", func, all_args.join(", "))
            }
            Expr::BinOp { left, op, right } => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Pow => "**",
                    BinOp::FloorDiv => return format!("Math.floor(({}) / ({}))", l, r),
                    BinOp::Eq => "===",
                    BinOp::NotEq => "!==",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::LtEq => "<=",
                    BinOp::GtEq => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                };
                format!("({} {} {})", l, op_str, r)
            }
            Expr::UnaryOp { op, expr } => {
                let e = self.gen_expr(expr);
                match op {
                    UnaryOp::Neg => format!("(-{})", e),
                    UnaryOp::Not => format!("(!{})", e),
                }
            }
            Expr::Call { name, args } => {
                let args_js: Vec<String> = args.iter().map(|a| self.gen_expr(a)).collect();
                // Map built-in functions
                match name.as_str() {
                    "print" => format!("__print({})", args_js.join(", ")),
                    "len" => {
                        if let Some(a) = args_js.first() {
                            format!("({}).length", a)
                        } else {
                            "0".to_string()
                        }
                    }
                    "str" => {
                        if let Some(a) = args_js.first() {
                            format!("String({})", a)
                        } else {
                            "\"\"".to_string()
                        }
                    }
                    "int" => {
                        if let Some(a) = args_js.first() {
                            format!("parseInt({})", a)
                        } else {
                            "0".to_string()
                        }
                    }
                    "float" => {
                        if let Some(a) = args_js.first() {
                            format!("parseFloat({})", a)
                        } else {
                            "0.0".to_string()
                        }
                    }
                    "push" => {
                        if args_js.len() >= 2 {
                            format!("{}.push({})", args_js[0], args_js[1])
                        } else {
                            "undefined".to_string()
                        }
                    }
                    "pop" => {
                        if let Some(a) = args_js.first() {
                            format!("{}.pop()", a)
                        } else {
                            "undefined".to_string()
                        }
                    }
                    "range" => {
                        if args_js.len() == 1 {
                            format!(
                                "Array.from({{length: {}}}, function(_, i) {{ return i; }})",
                                args_js[0]
                            )
                        } else if args_js.len() == 2 {
                            format!(
                                "Array.from({{length: {} - {}}}, function(_, i) {{ return {} + i; }})",
                                args_js[1], args_js[0], args_js[0]
                            )
                        } else if args_js.len() >= 3 {
                            format!(
                                "(function(){{ var r=[]; for(var i={};({s}>0?i<{e}:i>{e});i+={s})r.push(i); return r; }})()",
                                args_js[0], s = args_js[2], e = args_js[1]
                            )
                        } else {
                            "[]".to_string()
                        }
                    }
                    "type" => {
                        if let Some(a) = args_js.first() {
                            format!("typeof {}", a)
                        } else {
                            "\"undefined\"".to_string()
                        }
                    }
                    "abs" => {
                        if let Some(a) = args_js.first() {
                            format!("Math.abs({})", a)
                        } else {
                            "0".to_string()
                        }
                    }
                    "min" => format!("Math.min({})", args_js.join(", ")),
                    "max" => format!("Math.max({})", args_js.join(", ")),
                    "round" => {
                        if args_js.len() >= 2 {
                            format!(
                                "(Math.round({} * Math.pow(10, {})) / Math.pow(10, {}))",
                                args_js[0], args_js[1], args_js[1]
                            )
                        } else if let Some(a) = args_js.first() {
                            format!("Math.round({})", a)
                        } else {
                            "0".to_string()
                        }
                    }
                    "sum" => {
                        if let Some(a) = args_js.first() {
                            format!("{}.reduce(function(a,b){{ return a+b; }}, 0)", a)
                        } else {
                            "0".to_string()
                        }
                    }
                    "sorted" => {
                        if let Some(a) = args_js.first() {
                            format!("{}.slice().sort(function(a,b){{ return a-b; }})", a)
                        } else {
                            "[]".to_string()
                        }
                    }
                    "reversed" => {
                        if let Some(a) = args_js.first() {
                            format!("{}.slice().reverse()", a)
                        } else {
                            "[]".to_string()
                        }
                    }
                    "enumerate" => {
                        if let Some(a) = args_js.first() {
                            format!("{}.map(function(v,i){{ return [i,v]; }})", a)
                        } else {
                            "[]".to_string()
                        }
                    }
                    "zip" => {
                        if args_js.len() >= 2 {
                            format!(
                                "{}.map(function(v,i){{ return [v, {}[i]]; }})",
                                args_js[0], args_js[1]
                            )
                        } else {
                            "[]".to_string()
                        }
                    }
                    "any" => {
                        if let Some(a) = args_js.first() {
                            format!("{}.some(function(v){{ return !!v; }})", a)
                        } else {
                            "false".to_string()
                        }
                    }
                    "all" => {
                        if let Some(a) = args_js.first() {
                            format!("{}.every(function(v){{ return !!v; }})", a)
                        } else {
                            "true".to_string()
                        }
                    }
                    "keys" => {
                        if let Some(a) = args_js.first() {
                            format!("Object.keys({})", a)
                        } else {
                            "[]".to_string()
                        }
                    }
                    "values" => {
                        if let Some(a) = args_js.first() {
                            format!("Object.values({})", a)
                        } else {
                            "[]".to_string()
                        }
                    }
                    "items" => {
                        if let Some(a) = args_js.first() {
                            format!("Object.entries({})", a)
                        } else {
                            "[]".to_string()
                        }
                    }
                    "has" => {
                        if args_js.len() >= 2 {
                            format!("(({}).hasOwnProperty ? ({}).hasOwnProperty({}) : ({}).includes({}))", args_js[0], args_js[0], args_js[1], args_js[0], args_js[1])
                        } else {
                            "false".to_string()
                        }
                    }
                    "chr" => {
                        if let Some(a) = args_js.first() {
                            format!("String.fromCharCode({})", a)
                        } else {
                            "\"\"".to_string()
                        }
                    }
                    "ord" => {
                        if let Some(a) = args_js.first() {
                            format!("({}).charCodeAt(0)", a)
                        } else {
                            "0".to_string()
                        }
                    }
                    "bool" => {
                        if let Some(a) = args_js.first() {
                            format!("Boolean({})", a)
                        } else {
                            "false".to_string()
                        }
                    }
                    "map" => {
                        if args_js.len() >= 2 {
                            format!("{}.map({})", args_js[0], args_js[1])
                        } else {
                            "[]".to_string()
                        }
                    }
                    "filter" => {
                        if args_js.len() >= 2 {
                            format!("{}.filter({})", args_js[0], args_js[1])
                        } else {
                            "[]".to_string()
                        }
                    }
                    "reduce" => {
                        if args_js.len() >= 3 {
                            format!("{}.reduce({}, {})", args_js[0], args_js[2], args_js[1])
                        } else {
                            "undefined".to_string()
                        }
                    }
                    "assert" => {
                        if let Some(a) = args_js.first() {
                            let msg = args_js.get(1).map(|m| m.as_str()).unwrap_or("\"Assertion failed\"");
                            format!("if (!{}) throw new Error({})", a, msg)
                        } else {
                            String::new()
                        }
                    }
                    _ => format!("{}({})", name, args_js.join(", ")),
                }
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let obj = self.gen_expr(object);
                let args_js: Vec<String> = args.iter().map(|a| self.gen_expr(a)).collect();
                format!("{}.{}({})", obj, method, args_js.join(", "))
            }
            Expr::Index { object, index } => {
                let obj = self.gen_expr(object);
                let idx = self.gen_expr(index);
                format!("{}[{}]", obj, idx)
            }
            Expr::Member { object, field } => {
                let obj = self.gen_expr(object);
                format!("{}.{}", obj, field)
            }
        }
    }

    /// Convert a RustScript string with {expr} interpolation to a JS expression.
    fn gen_interpolated_string(&self, s: &str) -> String {
        // If no interpolation, just return a JS string literal
        if !s.contains('{') {
            // Restore escaped braces: \u{E000} → {  \u{E001} → }
            let restored = s.replace('\u{E000}', "{").replace('\u{E001}', "}");
            return format!("\"{}\"", escape_js_string(&restored));
        }

        let mut parts: Vec<String> = Vec::new();
        let mut buf = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '\u{E000}' {
                // Escaped brace → literal {
                buf.push('{');
                i += 1;
            } else if chars[i] == '\u{E001}' {
                // Escaped brace → literal }
                buf.push('}');
                i += 1;
            } else if chars[i] == '{' {
                // flush literal
                if !buf.is_empty() {
                    parts.push(format!("\"{}\"", escape_js_string(&buf)));
                    buf.clear();
                }
                // find matching }
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
                i += 1; // skip closing }
                // The expression inside {} is treated as a JS expression directly
                // (variable names and function calls map 1:1)
                parts.push(format!("String({})", expr_str.trim()));
            } else {
                buf.push(chars[i]);
                i += 1;
            }
        }
        if !buf.is_empty() {
            parts.push(format!("\"{}\"", escape_js_string(&buf)));
        }

        if parts.is_empty() {
            "\"\"".to_string()
        } else if parts.len() == 1 {
            parts.into_iter().next().unwrap()
        } else {
            format!("({})", parts.join(" + "))
        }
    }

    // ── page element codegen ─────────────────────────────────

    fn gen_page_elements(&mut self, elements: &[Element]) {
        for el in elements {
            self.gen_element(el);
        }
    }

    fn gen_element(&mut self, element: &Element) {
        match element {
            Element::Tag {
                tag,
                text,
                attrs,
                style,
                events,
                children,
            } => {
                if tag == "__page_style__" {
                    // Page-level CSS → body styles (with shorthand mapping)
                    let mut css = String::from("body {\n");
                    for prop in style {
                        for (css_name, css_val) in map_style_prop(&prop.name, &prop.value) {
                            css.push_str(&format!("  {}: {};\n", css_name, css_val));
                        }
                    }
                    css.push_str("}\n");
                    self.css.push_str(&css);
                    return;
                }

                // Build inline style string
                let style_str = if style.is_empty() {
                    String::new()
                } else {
                    let s: Vec<String> = style
                        .iter()
                        .flat_map(|p| map_style_prop(&p.name, &p.value))
                        .map(|(n, v)| format!("{}:{}", n, v))
                        .collect();
                    format!(" style=\\\"{}\\\"", s.join(";"))
                };

                // Build attribute string
                let mut attr_str = String::new();
                for attr in attrs {
                    let val = self.gen_expr(&attr.value);
                    attr_str.push_str(&format!(" {}=\\\"\" + __esc({}) + \"\\\"", attr.name, val));
                }

                // Event attributes
                let mut evt_str = String::new();
                for event in events {
                    let id = self.evt_counter;
                    self.evt_counter += 1;
                    let handler_js = self.gen_body(&event.body);
                    self.events.push((
                        id.to_string(),
                        format!(
                            "function(e) {{ var event = {{ value: e.target.value, target: e.target }}; {} __render(); }}",
                            handler_js
                        ),
                    ));
                    evt_str.push_str(&format!(" data-evt-{}=\\\"{}\\\"", event.name, id));
                }

                // Special self-closing tags
                let self_closing = matches!(tag.as_str(), "br" | "hr" | "img" | "input");

                if tag == "text" {
                    // text element → just the text content, no wrapping tag
                    if let Some(txt) = text {
                        let t = self.gen_expr(txt);
                        self.js_render.push_str(&format!("__h += __esc({});\n", t));
                    }
                    return;
                }

                let open = format!("__h += \"<{}{}{}{}", tag, style_str, attr_str, evt_str);

                if self_closing {
                    self.js_render.push_str(&format!("{} />\";\n", open));
                    return;
                }

                self.js_render.push_str(&format!("{}>\";\n", open));

                // Text content
                if let Some(txt) = text {
                    let t = self.gen_expr(txt);
                    self.js_render.push_str(&format!("__h += __esc({});\n", t));
                }

                // Children
                for child in children {
                    self.gen_element(child);
                }

                self.js_render
                    .push_str(&format!("__h += \"</{}>\";\n", tag));
            }
            Element::If {
                cond,
                then_els,
                else_els,
            } => {
                let cond_js = self.gen_expr(cond);
                self.js_render.push_str(&format!("if ({}) {{\n", cond_js));
                for el in then_els {
                    self.gen_element(el);
                }
                self.js_render.push_str("}\n");
                if let Some(els) = else_els {
                    self.js_render.push_str(" else {\n");
                    for el in els {
                        self.gen_element(el);
                    }
                    self.js_render.push_str("}\n");
                }
            }
            Element::For { var, iter, body } => {
                let iter_js = self.gen_expr(iter);
                self.js_render.push_str(&format!(
                    "for (var __fi = 0; __fi < {iter}.length; __fi++) {{\nvar {v} = {iter}[__fi];\n",
                    iter = iter_js,
                    v = var,
                ));
                for el in body {
                    self.gen_element(el);
                }
                self.js_render.push_str("}\n");
            }
        }
    }

    // ── HTML assembly ────────────────────────────────────────

    fn build_html(&self) -> String {
        let default_css = r#"* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  background: #0d1117;
  color: #e6edf3;
  min-height: 100vh;
}
a { color: #58a6ff; }
"#;

        let events_js = if self.events.is_empty() {
            String::new()
        } else {
            let entries: Vec<String> = self
                .events
                .iter()
                .map(|(id, handler)| format!("  {}: {}", id, handler))
                .collect();
            format!("var __handlers = {{\n{}\n}};\n", entries.join(",\n"))
        };

        let event_delegation = if self.events.is_empty() {
            String::new()
        } else {
            r#"['click','input','change','submit','keydown','keyup'].forEach(function(evtType) {
  document.getElementById('__app').addEventListener(evtType, function(e) {
    var el = e.target.closest('[data-evt-' + evtType + ']');
    if (el) {
      var id = el.getAttribute('data-evt-' + evtType);
      if (__handlers[id]) __handlers[id](e);
    }
  });
});
"#
            .to_string()
        };

        let render_fn = if self.has_page {
            format!(
                r#"function __render() {{
  var __h = '';
{}  document.getElementById('__app').innerHTML = __h;
}}
__render();
"#,
                self.js_render
            )
        } else {
            String::new()
        };

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>RustScript App</title>
  <style>
{default_css}{css}  </style>
</head>
<body>
  <div id="__console" style="display:none;position:fixed;bottom:0;left:0;right:0;max-height:200px;overflow-y:auto;background:#161b22;border-top:1px solid #30363d;padding:8px 12px;font-family:monospace;font-size:13px;color:#8b949e;z-index:9999;"></div>
  <div id="__app"></div>
  <script>
// ─── RustScript Runtime ────────────────────────────────────
(function() {{
  var __con = document.getElementById('__console');
  function __print() {{
    var args = Array.prototype.slice.call(arguments);
    __con.style.display = 'block';
    var d = document.createElement('div');
    d.textContent = args.map(String).join(' ');
    __con.appendChild(d);
    __con.scrollTop = __con.scrollHeight;
    console.log.apply(console, args);
  }}
  function __esc(s) {{
    return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
  }}

  // ─── Variables ───────────────────────────────────────────
{vars}
  // ─── Functions ───────────────────────────────────────────
{fns}
  // ─── Top-level code ──────────────────────────────────────
{top}
  // ─── Event handlers ──────────────────────────────────────
{events}
{delegation}
  // ─── Render ──────────────────────────────────────────────
{render}
}})();
  </script>
</body>
</html>"#,
            default_css = default_css,
            css = self.css,
            vars = indent(&self.js_vars, "  "),
            fns = indent(&self.js_fns, "  "),
            top = indent(&self.js_top, "  "),
            events = indent(&events_js, "  "),
            delegation = indent(&event_delegation, "  "),
            render = indent(&render_fn, "  "),
        )
    }
}

// ── helpers ──────────────────────────────────────────────────

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace("</", "<\\/") // prevent </script> from closing the tag
}

fn indent(s: &str, prefix: &str) -> String {
    s.lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{}{}", prefix, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
