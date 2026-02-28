/// Lexer for RustScript – converts source text into a stream of tokens.

use crate::token::{Spanned, Token};

pub struct Lexer {
    src: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            src: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    // ── helpers ──────────────────────────────────────────────

    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.src.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.src.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // skip whitespace
            while let Some(c) = self.peek() {
                if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
                    self.advance();
                } else {
                    break;
                }
            }
            // skip line comment  # ...
            if self.peek() == Some('#') {
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue; // re-check for more whitespace / comments
            }
            break;
        }
    }

    // ── main entry ───────────────────────────────────────────

    /// Tokenize the entire source, returning a Vec of Spanned tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Spanned>, String> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            let line = self.line;
            let col = self.col;

            let ch = match self.peek() {
                Some(c) => c,
                None => {
                    tokens.push(Spanned {
                        token: Token::Eof,
                        line,
                        col,
                    });
                    break;
                }
            };

            let tok = match ch {
                // ── string literal ──
                '"' => self.read_string()?,

                // ── number literal ──
                '0'..='9' => self.read_number()?,

                // ── identifier / keyword ──
                'a'..='z' | 'A'..='Z' | '_' => self.read_ident(),

                // ── two-char operators ──
                '=' if self.peek_next() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::Eq
                }
                '!' if self.peek_next() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::NotEq
                }
                '<' if self.peek_next() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::LtEq
                }
                '>' if self.peek_next() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::GtEq
                }
                '+' if self.peek_next() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::PlusAssign
                }
                '-' if self.peek_next() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::MinusAssign
                }

                // ── single-char tokens ──
                '+' => { self.advance(); Token::Plus }
                '-' => { self.advance(); Token::Minus }
                '*' => { self.advance(); Token::Star }
                '/' => { self.advance(); Token::Slash }
                '%' => { self.advance(); Token::Percent }
                '=' => { self.advance(); Token::Assign }
                '<' => { self.advance(); Token::Lt }
                '>' => { self.advance(); Token::Gt }
                '(' => { self.advance(); Token::LParen }
                ')' => { self.advance(); Token::RParen }
                '{' => { self.advance(); Token::LBrace }
                '}' => { self.advance(); Token::RBrace }
                '[' => { self.advance(); Token::LBracket }
                ']' => { self.advance(); Token::RBracket }
                ',' => { self.advance(); Token::Comma }
                ':' => { self.advance(); Token::Colon }
                '.' => { self.advance(); Token::Dot }

                other => {
                    return Err(format!(
                        "[{}:{}] Unexpected character '{}'",
                        line, col, other
                    ));
                }
            };

            tokens.push(Spanned {
                token: tok,
                line,
                col,
            });
        }
        Ok(tokens)
    }

    // ── token readers ────────────────────────────────────────

    fn read_string(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_col = self.col;
        self.advance(); // consume opening "
        let mut buf = String::new();
        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    return Ok(Token::Str(buf));
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => { self.advance(); buf.push('\n'); }
                        Some('t') => { self.advance(); buf.push('\t'); }
                        Some('\\') => { self.advance(); buf.push('\\'); }
                        Some('"') => { self.advance(); buf.push('"'); }
                        Some('{') => { self.advance(); buf.push('\u{E000}'); }
                        Some('}') => { self.advance(); buf.push('\u{E001}'); }
                        Some(c) => { self.advance(); buf.push('\\'); buf.push(c); }
                        None => {
                            return Err(format!(
                                "[{}:{}] Unterminated string",
                                start_line, start_col
                            ));
                        }
                    }
                }
                Some(c) => {
                    self.advance();
                    buf.push(c);
                }
                None => {
                    return Err(format!(
                        "[{}:{}] Unterminated string",
                        start_line, start_col
                    ));
                }
            }
        }
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let mut buf = String::new();
        let mut is_float = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                buf.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                // check next char is digit to distinguish from method call dot
                if let Some(next) = self.peek_next() {
                    if next.is_ascii_digit() {
                        is_float = true;
                        buf.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if is_float {
            let val: f64 = buf
                .parse()
                .map_err(|_| format!("Invalid float literal: {}", buf))?;
            Ok(Token::Float(val))
        } else {
            let val: i64 = buf
                .parse()
                .map_err(|_| format!("Invalid integer literal: {}", buf))?;
            Ok(Token::Int(val))
        }
    }

    fn read_ident(&mut self) -> Token {
        let mut buf = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                buf.push(c);
                self.advance();
            } else {
                break;
            }
        }
        // keyword check
        match buf.as_str() {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "import" => Token::Import,
            "page" => Token::Page,
            "style" => Token::Style,
            "on" => Token::On,
            "true" => Token::True,
            "false" => Token::False,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            _ => Token::Ident(buf),
        }
    }
}
