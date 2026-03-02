//! Recursive-descent parser for RustScript.

use crate::ast::*;
use crate::token::{Spanned, Token, is_html_tag};

pub struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // ── helpers ──────────────────────────────────────────────

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|s| &s.token)
            .unwrap_or(&Token::Eof)
    }

    fn peek_second(&self) -> &Token {
        self.tokens
            .get(self.pos + 1)
            .map(|s| &s.token)
            .unwrap_or(&Token::Eof)
    }

    fn loc(&self) -> (usize, usize) {
        self.tokens
            .get(self.pos)
            .map(|s| (s.line, s.col))
            .unwrap_or((0, 0))
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos].token;
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let (line, col) = self.loc();
        let tok = self.peek().clone();
        if &tok == expected {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "[{}:{}] Expected {:?}, got {:?}",
                line, col, expected, tok
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        let (line, col) = self.loc();
        match self.peek().clone() {
            Token::Ident(name) => {
                self.advance();
                Ok(name)
            }
            other => Err(format!(
                "[{}:{}] Expected identifier, got {:?}",
                line, col, other
            )),
        }
    }

    fn at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    // ── program ──────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut stmts = Vec::new();
        while !self.at_end() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program { stmts })
    }

    // ── statements ───────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek().clone() {
            Token::Import => self.parse_import(),
            Token::Let => self.parse_let(),
            Token::Fn => self.parse_fn_decl(),
            Token::Return => self.parse_return(),
            Token::If => self.parse_if_stmt(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Page => self.parse_page(),
            _ => self.parse_assign_or_expr(),
        }
    }

    fn parse_import(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'import'
        let (line, col) = self.loc();
        match self.peek().clone() {
            Token::Str(path) => {
                self.advance();
                Ok(Stmt::Import { path })
            }
            other => Err(format!(
                "[{}:{}] Expected string path after 'import', got {:?}",
                line, col, other
            )),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'let'
        let name = self.expect_ident()?;
        self.expect(&Token::Assign)?;
        let value = self.parse_expr()?;
        Ok(Stmt::Let { name, value })
    }

    fn parse_fn_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'fn'
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::FnDecl { name, params, body })
    }

    fn parse_param_list(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        if *self.peek() == Token::RParen {
            return Ok(params);
        }
        params.push(self.expect_ident()?);
        while *self.peek() == Token::Comma {
            self.advance();
            params.push(self.expect_ident()?);
        }
        Ok(params)
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'return'
        // If next token starts a new statement or is }, no return value
        if matches!(
            self.peek(),
            Token::RBrace
                | Token::Eof
                | Token::Let
                | Token::Fn
                | Token::Return
                | Token::If
                | Token::While
                | Token::For
                | Token::Page
        ) {
            return Ok(Stmt::Return(None));
        }
        let expr = self.parse_expr()?;
        Ok(Stmt::Return(Some(expr)))
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'if'
        let cond = self.parse_expr()?;
        let then_body = self.parse_block()?;
        let else_body = if *self.peek() == Token::Else {
            self.advance();
            if *self.peek() == Token::If {
                // else if  →  else { if ... }
                let elif = self.parse_if_stmt()?;
                Some(vec![elif])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If {
            cond,
            then_body,
            else_body,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'while'
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'for'
        let var = self.expect_ident()?;
        self.expect(&Token::In)?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For { var, iter, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while *self.peek() != Token::RBrace && !self.at_end() {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }

    fn parse_assign_or_expr(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expr()?;

        // Check for assignment:  ident = expr
        match self.peek().clone() {
            Token::Assign => {
                self.advance();
                let value = self.parse_expr()?;
                match expr {
                    Expr::Ident(name) => Ok(Stmt::Assign { name, value }),
                    Expr::Index { object, index } => {
                        if let Expr::Ident(name) = *object {
                            Ok(Stmt::IndexAssign {
                                list: name,
                                index: *index,
                                value,
                            })
                        } else {
                            Err("Can only assign to identifier index".into())
                        }
                    }
                    _ => Err("Invalid assignment target".into()),
                }
            }
            Token::PlusAssign => {
                self.advance();
                let rhs = self.parse_expr()?;
                if let Expr::Ident(name) = expr {
                    Ok(Stmt::Assign {
                        value: Expr::BinOp {
                            left: Box::new(Expr::Ident(name.clone())),
                            op: BinOp::Add,
                            right: Box::new(rhs),
                        },
                        name,
                    })
                } else {
                    Err("Invalid += target".into())
                }
            }
            Token::MinusAssign => {
                self.advance();
                let rhs = self.parse_expr()?;
                if let Expr::Ident(name) = expr {
                    Ok(Stmt::Assign {
                        value: Expr::BinOp {
                            left: Box::new(Expr::Ident(name.clone())),
                            op: BinOp::Sub,
                            right: Box::new(rhs),
                        },
                        name,
                    })
                } else {
                    Err("Invalid -= target".into())
                }
            }
            _ => Ok(Stmt::Expr(expr)),
        }
    }

    // ── expressions (precedence climbing) ────────────────────

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while *self.peek() == Token::Or {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while *self.peek() == Token::And {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                Token::Eq => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_addition()?;
        loop {
            let op = match self.peek() {
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::LtEq,
                Token::GtEq => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek().clone() {
                Token::LParen => {
                    // Function call
                    if let Expr::Ident(name) = expr {
                        self.advance(); // (
                        let args = self.parse_arg_list()?;
                        self.expect(&Token::RParen)?;
                        expr = Expr::Call { name, args };
                    } else {
                        break;
                    }
                }
                Token::LBracket => {
                    // Index access
                    self.advance(); // [
                    let index = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::Dot => {
                    self.advance(); // .
                    let field = self.expect_ident()?;
                    // Check for method call
                    if *self.peek() == Token::LParen {
                        self.advance(); // (
                        let args = self.parse_arg_list()?;
                        self.expect(&Token::RParen)?;
                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: field,
                            args,
                        };
                    } else {
                        expr = Expr::Member {
                            object: Box::new(expr),
                            field,
                        };
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let (line, col) = self.loc();
        match self.peek().clone() {
            Token::Int(n) => {
                self.advance();
                Ok(Expr::Int(n))
            }
            Token::Float(n) => {
                self.advance();
                Ok(Expr::Float(n))
            }
            Token::Str(s) => {
                self.advance();
                Ok(Expr::Str(s))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            Token::Ident(name) => {
                self.advance();
                Ok(Expr::Ident(name))
            }
            Token::LParen => {
                self.advance(); // (
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.advance(); // [
                let items = self.parse_arg_list()?;
                self.expect(&Token::RBracket)?;
                Ok(Expr::List(items))
            }
            other => Err(format!(
                "[{}:{}] Unexpected token in expression: {:?}",
                line, col, other
            )),
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if matches!(self.peek(), Token::RParen | Token::RBracket) {
            return Ok(args);
        }
        args.push(self.parse_expr()?);
        while *self.peek() == Token::Comma {
            self.advance();
            // allow trailing comma
            if matches!(self.peek(), Token::RParen | Token::RBracket) {
                break;
            }
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    // ── page parsing ─────────────────────────────────────────

    fn parse_page(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'page'
        self.expect(&Token::LBrace)?;
        let elements = self.parse_elements()?;
        self.expect(&Token::RBrace)?;
        Ok(Stmt::Page { elements })
    }

    fn parse_elements(&mut self) -> Result<Vec<Element>, String> {
        let mut elements = Vec::new();
        while *self.peek() != Token::RBrace && !self.at_end() {
            // skip 'style' at page level → becomes a special root style element
            if *self.peek() == Token::Style {
                elements.push(self.parse_page_level_style()?);
                continue;
            }
            if *self.peek() == Token::If {
                elements.push(self.parse_if_element()?);
                continue;
            }
            if *self.peek() == Token::For {
                elements.push(self.parse_for_element()?);
                continue;
            }
            // Must be an HTML tag
            elements.push(self.parse_element()?);
        }
        Ok(elements)
    }

    fn parse_page_level_style(&mut self) -> Result<Element, String> {
        // page-level style { } → becomes a special <style> tag
        self.advance(); // consume 'style'
        self.expect(&Token::LBrace)?;
        let props = self.parse_style_props()?;
        self.expect(&Token::RBrace)?;
        Ok(Element::Tag {
            tag: "__page_style__".to_string(),
            text: None,
            attrs: Vec::new(),
            style: props,
            events: Vec::new(),
            children: Vec::new(),
        })
    }

    fn parse_element(&mut self) -> Result<Element, String> {
        let (line, col) = self.loc();
        let tag = self.expect_ident()?;
        if !is_html_tag(&tag) {
            return Err(format!(
                "[{}:{}] Unknown HTML tag: '{}'. Use a known element like div, p, h1, button, etc.",
                line, col, tag
            ));
        }

        // optional text content after tag name
        let text = if let Token::Str(_) = self.peek() {
            Some(self.parse_expr()?)
        } else {
            None
        };

        // optional body { ... }
        if *self.peek() == Token::LBrace {
            self.advance(); // {
            let mut style = Vec::new();
            let mut events = Vec::new();
            let mut children = Vec::new();
            let mut attrs = Vec::new();

            while *self.peek() != Token::RBrace && !self.at_end() {
                if *self.peek() == Token::Style {
                    self.advance();
                    self.expect(&Token::LBrace)?;
                    style = self.parse_style_props()?;
                    self.expect(&Token::RBrace)?;
                } else if *self.peek() == Token::On {
                    events.push(self.parse_event()?);
                } else if *self.peek() == Token::If {
                    children.push(self.parse_if_element()?);
                } else if *self.peek() == Token::For {
                    children.push(self.parse_for_element()?);
                } else if let Token::Ident(ref name) = self.peek().clone() {
                    if is_html_tag(name) {
                        children.push(self.parse_element()?);
                    } else {
                        // Could be an attribute:  name: expr
                        // Peek ahead for colon
                        if *self.peek_second() == Token::Colon {
                            let attr_name = self.expect_ident()?;
                            self.advance(); // :
                            let attr_value = self.parse_expr()?;
                            attrs.push(Attribute {
                                name: attr_name,
                                value: attr_value,
                            });
                        } else {
                            // expression statement (like function call?)
                            let expr = self.parse_expr()?;
                            // Wrap in a text element
                            children.push(Element::Tag {
                                tag: "span".to_string(),
                                text: Some(expr),
                                attrs: Vec::new(),
                                style: Vec::new(),
                                events: Vec::new(),
                                children: Vec::new(),
                            });
                        }
                    }
                } else {
                    let (l, c) = self.loc();
                    return Err(format!(
                        "[{}:{}] Unexpected token in element body: {:?}",
                        l,
                        c,
                        self.peek()
                    ));
                }
            }
            self.expect(&Token::RBrace)?;
            Ok(Element::Tag {
                tag,
                text,
                attrs,
                style,
                events,
                children,
            })
        } else {
            // Self-closing element with optional text
            Ok(Element::Tag {
                tag,
                text,
                attrs: Vec::new(),
                style: Vec::new(),
                events: Vec::new(),
                children: Vec::new(),
            })
        }
    }

    /// Try to consume the current token as a bare word (identifier or keyword used as a name).
    /// This is needed for style property names like `font-style` or `self-align`
    /// where a keyword (e.g. `style`, `for`, `in`) appears after a hyphen.
    fn expect_ident_or_keyword(&mut self) -> Result<String, String> {
        let (line, col) = self.loc();
        let name = match self.peek().clone() {
            Token::Ident(s) => s,
            Token::Style => "style".to_string(),
            Token::On => "on".to_string(),
            Token::If => "if".to_string(),
            Token::Else => "else".to_string(),
            Token::While => "while".to_string(),
            Token::For => "for".to_string(),
            Token::In => "in".to_string(),
            Token::Let => "let".to_string(),
            Token::Fn => "fn".to_string(),
            Token::Return => "return".to_string(),
            Token::Import => "import".to_string(),
            Token::Page => "page".to_string(),
            Token::True => "true".to_string(),
            Token::False => "false".to_string(),
            Token::And => "and".to_string(),
            Token::Or => "or".to_string(),
            Token::Not => "not".to_string(),
            other => {
                return Err(format!(
                    "[{}:{}] Expected identifier, got {:?}",
                    line, col, other
                ));
            }
        };
        self.advance();
        Ok(name)
    }

    fn parse_style_props(&mut self) -> Result<Vec<StyleProp>, String> {
        let mut props = Vec::new();
        while *self.peek() != Token::RBrace && !self.at_end() {
            // property name (may contain hyphens: font-family, font-style, etc.)
            let mut name = self.expect_ident_or_keyword()?;
            while *self.peek() == Token::Minus {
                self.advance();
                let part = self.expect_ident_or_keyword()?;
                name = format!("{}-{}", name, part);
            }

            // If colon follows → name: "value"
            // Otherwise → flag-style property (e.g. bold, center, row)
            if *self.peek() == Token::Colon {
                self.advance(); // consume ':'
                let value = match self.peek().clone() {
                    Token::Str(s) => {
                        self.advance();
                        s
                    }
                    _ => {
                        let (l, c) = self.loc();
                        return Err(format!(
                            "[{}:{}] Style value must be a quoted string, got {:?}",
                            l,
                            c,
                            self.peek()
                        ));
                    }
                };
                props.push(StyleProp { name, value });
            } else {
                // Flag property — no value (codegen decides the CSS output)
                props.push(StyleProp {
                    name,
                    value: String::new(),
                });
            }
        }
        Ok(props)
    }

    fn parse_event(&mut self) -> Result<Event, String> {
        self.advance(); // consume 'on'
        let name = self.expect_ident()?;
        let body = self.parse_block()?;
        Ok(Event { name, body })
    }

    fn parse_if_element(&mut self) -> Result<Element, String> {
        self.advance(); // consume 'if'
        let cond = self.parse_expr()?;
        self.expect(&Token::LBrace)?;
        let then_els = self.parse_elements()?;
        self.expect(&Token::RBrace)?;
        let else_els = if *self.peek() == Token::Else {
            self.advance();
            if *self.peek() == Token::If {
                Some(vec![self.parse_if_element()?])
            } else {
                self.expect(&Token::LBrace)?;
                let els = self.parse_elements()?;
                self.expect(&Token::RBrace)?;
                Some(els)
            }
        } else {
            None
        };
        Ok(Element::If {
            cond,
            then_els,
            else_els,
        })
    }

    fn parse_for_element(&mut self) -> Result<Element, String> {
        self.advance(); // consume 'for'
        let var = self.expect_ident()?;
        self.expect(&Token::In)?;
        let iter = self.parse_expr()?;
        self.expect(&Token::LBrace)?;
        let body = self.parse_elements()?;
        self.expect(&Token::RBrace)?;
        Ok(Element::For { var, iter, body })
    }
}
