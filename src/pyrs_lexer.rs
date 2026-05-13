#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Op {
    LPAR,
    RPAR,
    LSQB,
    RSQB,
    COLON,
    COMMA,
    SEMI,
    PLUS,
    MINUS,
    STAR,
    SLASH,
    VBAR,
    AMPER,
    LESS,
    GREATER,
    EQUAL,
    DOT,
    PERCENT,
    LBRACE,
    RBRACE,
    EQEQUAL,
    NOTEQUAL,
    LESSEQUAL,
    GREATEREQUAL,
    TILDE,
    CIRCUMFLEX,
    LEFTSHIFT,
    RIGHTSHIFT,
    DOUBLESTAR,
    PLUSEQUAL,
    MINEQUAL,
    STAREQUAL,
    SLASHEQUAL,
    PERCENTEQUAL,
    AMPEREQUAL,
    VBAREQUAL,
    CIRCUMFLEXEQUAL,
    LEFTSHIFTEQUAL,
    RIGHTSHIFTEQUAL,
    DOUBLESTAREQUAL,
    DOUBLESLASH,
    DOUBLESLASHEQUAL,
    AT,
    ATEQUAL,
    RARROW,
    ELLIPSIS,
    COLONEQUAL,
    EXCLAMATION,
}
impl Op {
    fn value(self) -> &'static str {
        match self {
            Op::LPAR => "(",
            Op::RPAR => ")",
            Op::LSQB => "[",
            Op::RSQB => "]",
            Op::COLON => ":",
            Op::COMMA => ",",
            Op::SEMI => ";",
            Op::PLUS => "+",
            Op::MINUS => "-",
            Op::STAR => "*",
            Op::SLASH => "/",
            Op::VBAR => "|",
            Op::AMPER => "&",
            Op::LESS => "<",
            Op::GREATER => ">",
            Op::EQUAL => "=",
            Op::DOT => ".",
            Op::PERCENT => "%",
            Op::LBRACE => "{",
            Op::RBRACE => "}",
            Op::EQEQUAL => "==",
            Op::NOTEQUAL => "!=",
            Op::LESSEQUAL => "<=",
            Op::GREATEREQUAL => ">=",
            Op::TILDE => "~",
            Op::CIRCUMFLEX => "^",
            Op::LEFTSHIFT => "<<",
            Op::RIGHTSHIFT => ">>",
            Op::DOUBLESTAR => "**",
            Op::PLUSEQUAL => "+=",
            Op::MINEQUAL => "-=",
            Op::STAREQUAL => "*=",
            Op::SLASHEQUAL => "/=",
            Op::PERCENTEQUAL => "%=",
            Op::AMPEREQUAL => "&=",
            Op::VBAREQUAL => "|=",
            Op::CIRCUMFLEXEQUAL => "^=",
            Op::LEFTSHIFTEQUAL => "<<=",
            Op::RIGHTSHIFTEQUAL => ">>=",
            Op::DOUBLESTAREQUAL => "**=",
            Op::DOUBLESLASH => "//",
            Op::DOUBLESLASHEQUAL => "//=",
            Op::AT => "@",
            Op::ATEQUAL => "@=",
            Op::RARROW => "->",
            Op::ELLIPSIS => "...",
            Op::COLONEQUAL => ":=",
            Op::EXCLAMATION => "!",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub enum TokenKind {
    #[default]
    Unknown,
    Name,
    Number,
    String,
    Op(Op),
    Commment,
    NewLine,
    NL,
    Indent,
    Dedent,
    FStringStart,
    FStringMiddle,
    FStringEnd,
    TStringStart,
    TStringMiddle,
    TStringEnd,
    EndMarker,
    Encoding,
    TypeIgnore,
    TypeComment,
    SoftKeyword,
    ErrorToken,
}

impl TokenKind {
    const NUM_TOKENS: usize = 22;
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Token {
    kind: TokenKind,
}

pub struct Lexer {}

impl Lexer {
    pub fn lex() -> Vec<Token> {
        let mut tokens = vec![];

        tokens
    }
}
