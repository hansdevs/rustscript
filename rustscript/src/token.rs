/// Token types for the RustScript language.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Literals ──────────────────────────────────────────────
    Int(i64),
    Float(f64),
    Str(String),

    // ── Identifier ───────────────────────────────────────────
    Ident(String),

    // ── Keywords ─────────────────────────────────────────────
    Let,
    Fn,
    Return,
    If,
    Elif,
    Else,
    While,
    For,
    In,
    Import,
    Page,
    Style,
    On,
    True,
    False,
    None,
    And,
    Or,
    Not,
    Break,
    Continue,

    // ── Operators ────────────────────────────────────────────
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    StarStar,    // **  (power)
    SlashSlash,  // //  (floor division)
    Assign,      // =
    Eq,          // ==
    NotEq,       // !=
    Lt,          // <
    Gt,          // >
    LtEq,        // <=
    GtEq,        // >=
    PlusAssign,  // +=
    MinusAssign, // -=
    StarAssign,  // *=
    SlashAssign, // /=
    Pipe,        // |>  (pipe operator)

    // ── Delimiters ───────────────────────────────────────────
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Comma,    // ,
    Colon,    // :
    Dot,      // .
    Arrow,    // ->

    // ── Special ──────────────────────────────────────────────
    Eof,
}

/// Check whether a name is a known HTML element tag.
pub fn is_html_tag(name: &str) -> bool {
    matches!(
        name,
        "div"
            | "span"
            | "p"
            | "a"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "button"
            | "input"
            | "textarea"
            | "select"
            | "option"
            | "label"
            | "form"
            | "img"
            | "br"
            | "hr"
            | "ul"
            | "ol"
            | "li"
            | "table"
            | "tr"
            | "td"
            | "th"
            | "header"
            | "footer"
            | "nav"
            | "section"
            | "main"
            | "article"
            | "aside"
            | "video"
            | "audio"
            | "canvas"
            | "pre"
            | "code"
            | "text"
    )
}

/// Spanned token – a token with its source location.
#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}
