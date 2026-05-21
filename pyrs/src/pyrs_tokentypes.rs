use crate::pyrs_parser2::FileData;
use std::{rc::Rc, sync::Arc};

#[repr(C)]
#[derive(Clone)]
pub struct Token<'a> {
    pub data: &'a str,
    pub file: Arc<FileData>,
    pub line: usize,
    pub col: usize,
    pub kind: TokenKind,
}

#[repr(C)]
#[derive(Clone)]
pub struct TokenOwned {
    pub data: String,
    pub file: Arc<FileData>,
    pub line: usize,
    pub col: usize,
    pub kind: TokenKind,
}

pub trait TokenData: std::fmt::Debug {
    fn get_line_fmt(&self) -> String;
    fn dbg_str(&self) -> String;
    fn to_owned_token(&self) -> TokenOwned;
}

impl<'a> TokenData for Token<'a> {
    fn get_line_fmt(&self) -> String {
        self.file.get_line_fmt(self.line, true).unwrap()
    }

    fn dbg_str(&self) -> String {
        let data = fmt_whitespace(self.data.into());
        format!("Token[\'{}\', {:?}]", data, self.kind)
    }
    fn to_owned_token(&self) -> TokenOwned {
        TokenOwned {
            data: self.data.to_owned(),
            file: self.file.clone(),
            line: self.line,
            col: self.col,
            kind: self.kind,
        }
    }
}

impl<'a> Token<'a> {
    pub fn get_line(&self) -> &str {
        self.file
            .get_line(self.line)
            .expect("Should never get here, as should have valid ref to file_data")
    }
    pub fn basic(data: &'a str, file: &Arc<FileData>, kind: TokenKind) -> Self {
        Token {
            data,
            file: file.clone(),
            line: 0,
            col: 0,
            kind,
        }
    }
}

impl<'a> std::fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dbg_str())
    }
}
impl<'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.data == other.data
    }
}

impl TokenData for TokenOwned {
    fn get_line_fmt(&self) -> String {
        self.file.get_line_fmt(self.line, true).unwrap()
    }

    fn dbg_str(&self) -> String {
        let fmtted = fmt_whitespace(self.data.clone());
        format!("Token[\'{}\', {:?}]", fmtted, self.kind)
    }
    fn to_owned_token(&self) -> TokenOwned {
        self.clone()
    }
}
impl std::fmt::Debug for TokenOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dbg_str())
    }
}
impl PartialEq for TokenOwned {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.data == other.data
    }
}

// Helper
fn fmt_whitespace(s: String) -> String {
    s.replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TokenKind {
    #[default]
    Unknown, // Phase out at some point
    Name,
    Number(NumLit),
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

#[allow(unused)]
impl TokenKind {
    const NUM_TOKENS: usize = 22;
}

use paste::paste;
use rug::Integer;

macro_rules! make_ops {
    ( $( $name:ident => $sym:expr ),* $(,)? ) => {
        paste! {
            $(
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                pub struct [<$name Op>];
                impl [<$name Op>] {
                    pub const fn as_str(&self) -> &'static str { $sym }
                }
            )*

            #[derive(Debug, Clone,Copy, PartialEq, Eq)]
            pub enum Op {
                $(
                    [<$name>]([<$name Op>]),
                )*
            }

            impl Op {
                $(
                    pub const [<$name _>]: Op = Op::[<$name>]([<$name Op>]);
                )*

                pub fn symbol(&self) -> &'static str {
                    match self {
                        $(
                            Op::[<$name>](x) => x.as_str(),
                        )*
                    }
                }
                pub fn new<S: Into<String>>(symbol: S) -> Option<Self>{
                    let s: String = symbol.into();
                    match s.as_str() {
                        $(
                            $sym => Some(Op::[<$name>]([<$name Op>])),
                        )*
                        _ => None,
                    }
                }
            }
        }
    };
}

make_ops! {
    LPAR => "(",
    RPAR => ")",
    LSQB => "[",
    RSQB => "]",
    COLON => ":",
    COMMA => ",",
    SEMI => ";",
    PLUS => "+",
    MINUS => "-",
    STAR => "*",
    SLASH => "/",
    VBAR => "|",
    AMPER => "&",
    LESS => "<",
    GREATER => ">",
    EQUAL => "=",
    DOT => ".",
    PERCENT => "%",
    LBRACE => "{",
    RBRACE => "}",
    EQEQUAL => "==",
    NOTEQUAL => "!=",
    LESSEQUAL => "<=",
    GREATEREQUAL => ">=",
    TILDE => "~",
    CIRCUMFLEX => "^",
    LEFTSHIFT => "<<",
    RIGHTSHIFT => ">>",
    DOUBLESTAR => "**",
    PLUSEQUAL => "+=",
    MINEQUAL => "-=",
    STAREQUAL => "*=",
    SLASHEQUAL => "/=",
    PERCENTEQUAL => "%=",
    AMPEREQUAL => "&=",
    VBAREQUAL => "|=",
    CIRCUMFLEXEQUAL => "^=",
    LEFTSHIFTEQUAL => "<<=",
    RIGHTSHIFTEQUAL => ">>=",
    DOUBLESTAREQUAL => "**=",
    DOUBLESLASH => "//",
    DOUBLESLASHEQUAL => "//=",
    AT => "@",
    ATEQUAL => "@=",
    RARROW => "->",
    ELLIPSIS => "...",
    COLONEQUAL => ":=",
    EXCLAMATION => "!",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameKind {
    Identifier,
    Keyword,
    SoftKeyword,
}

macro_rules! make_keywords {
    ( $( $name:ident => $sym:expr ),* $(,)? ) => {
        paste! {
            $(
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                pub struct [<$name KW>];
                impl [<$name KW>] {
                    pub const fn as_str(&self) -> &'static str { $sym }
                }
            )*

            #[derive(Debug, Clone,Copy, PartialEq, Eq)]
            pub enum Keyword {
                $(
                    [<$name>]([<$name KW>]),
                )*
            }

            impl Keyword {

                pub fn symbol(&self) -> &'static str {
                    match self {
                        $(
                            Keyword::[<$name>](x) => x.as_str(),
                        )*
                    }
                }
                pub fn new<S: Into<String>>(symbol: S) -> Option<Self>{
                    let s: String = symbol.into();
                    match s.as_str() {
                        $(
                            $sym => Some(Keyword::[<$name>]([<$name KW>])),
                        )*
                        _ => None,
                    }
                }
            }
        }
    };
}

make_keywords!(
   False => "False",
   Await => "await",
   Else => "else",
   Import => "import",
   Pass => "pass",
   None => "None",
   Break => "break",
   Except => "except",
   In => "in",
   Raise => "raise",
   True => "True",
   Class => "class",
   Finally => "finally",
   Is => "is",
   Return => "return",
   And => "and",
   Continue => "continue",
   For => "for",
   Lambda => "lambda",
   Try => "try",
   As => "as",
   Def => "def",
   From => "from",
   Nonlocal => "nonlocal",
   While => "while",
   Assert => "assert",
   Del => "del",
   Global => "global",
   Not => "not",
   With => "with",
   Async => "async",
   Elif => "elif",
   If => "if",
   Or => "or",
   Yield => "yield",
);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum NumLit {
    #[default]
    Dec,
    Bin,
    Oct,
    Hex,
    Zero,
    Float,
}

impl NumLit {
    pub fn is_valid_kind(num: char, kind: NumLit) -> bool {
        match kind {
            NumLit::Float | NumLit::Dec => num.is_ascii_digit(),
            NumLit::Bin => num == '0' || num == '1',
            NumLit::Oct => ('0'..'8').contains(&num),
            NumLit::Hex => num.is_ascii_hexdigit(),
            NumLit::Zero => num == '0',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Number {
    Integer(Rc<str>, NumLit),
    FloatNumber(Rc<str>),
    ImagNumber(Rc<str>),
}

impl Number {
    pub fn new<'a>(token: &Token<'a>) -> Option<Self> {
        Some(match token.kind {
            TokenKind::Number(NumLit::Float) => Number::FloatNumber(token.data.into()),
            TokenKind::Number(nl) => Number::Integer(token.data.into(), nl),
            _ => return None,
        })
    }
}
