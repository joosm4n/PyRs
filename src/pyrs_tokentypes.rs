use crate::pyrs_parser2::FileData;
use std::sync::Arc;

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
//
// #[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
// pub enum Op {
//     LPAR,
//     RPAR,
//     LSQB,
//     RSQB,
//     COLON,
//     COMMA,
//     SEMI,
//     PLUS,
//     MINUS,
//     STAR,
//     SLASH,
//     VBAR,
//     AMPER,
//     LESS,
//     GREATER,
//     EQUAL,
//     DOT,
//     PERCENT,
//     LBRACE,
//     RBRACE,
//     EQEQUAL,
//     NOTEQUAL,
//     LESSEQUAL,
//     GREATEREQUAL,
//     TILDE,
//     CIRCUMFLEX,
//     LEFTSHIFT,
//     RIGHTSHIFT,
//     DOUBLESTAR,
//     PLUSEQUAL,
//     MINEQUAL,
//     STAREQUAL,
//     SLASHEQUAL,
//     PERCENTEQUAL,
//     AMPEREQUAL,
//     VBAREQUAL,
//     CIRCUMFLEXEQUAL,
//     LEFTSHIFTEQUAL,
//     RIGHTSHIFTEQUAL,
//     DOUBLESTAREQUAL,
//     DOUBLESLASH,
//     DOUBLESLASHEQUAL,
//     AT,
//     ATEQUAL,
//     RARROW,
//     ELLIPSIS,
//     COLONEQUAL,
//     EXCLAMATION,
// }
//
// impl Op {
//     pub fn new(op: &str) -> Option<Self> {
//         let o = match op {
//             "(" => Op::LPAR,
//             ")" => Op::RPAR,
//             "[" => Op::LSQB,
//             "]" => Op::RSQB,
//             ":" => Op::COLON,
//             "," => Op::COMMA,
//             ";" => Op::SEMI,
//             "+" => Op::PLUS,
//             "-" => Op::MINUS,
//             "*" => Op::STAR,
//             "/" => Op::SLASH,
//             "|" => Op::VBAR,
//             "&" => Op::AMPER,
//             "<" => Op::LESS,
//             ">" => Op::GREATER,
//             "=" => Op::EQUAL,
//             "." => Op::DOT,
//             "%" => Op::PERCENT,
//             "{" => Op::LBRACE,
//             "}" => Op::RBRACE,
//             "==" => Op::EQEQUAL,
//             "!=" => Op::NOTEQUAL,
//             "<=" => Op::LESSEQUAL,
//             ">=" => Op::GREATEREQUAL,
//             "~" => Op::TILDE,
//             "^" => Op::CIRCUMFLEX,
//             "<<" => Op::LEFTSHIFT,
//             ">>" => Op::RIGHTSHIFT,
//             "**" => Op::DOUBLESTAR,
//             "+=" => Op::PLUSEQUAL,
//             "-=" => Op::MINEQUAL,
//             "*=" => Op::STAREQUAL,
//             "/=" => Op::SLASHEQUAL,
//             "%=" => Op::PERCENTEQUAL,
//             "&=" => Op::AMPEREQUAL,
//             "|=" => Op::VBAREQUAL,
//             "^=" => Op::CIRCUMFLEXEQUAL,
//             "<<=" => Op::LEFTSHIFTEQUAL,
//             ">>=" => Op::RIGHTSHIFTEQUAL,
//             "**=" => Op::DOUBLESTAREQUAL,
//             "//" => Op::DOUBLESLASH,
//             "//=" => Op::DOUBLESLASHEQUAL,
//             "@" => Op::AT,
//             "@=" => Op::ATEQUAL,
//             "->" => Op::RARROW,
//             "..." => Op::ELLIPSIS,
//             ":=" => Op::COLONEQUAL,
//             "!" => Op::EXCLAMATION,
//             _ => return None,
//         };
//         Some(o)
//     }
//
//     pub fn value(self) -> &'static str {
//         match self {
//             Op::LPAR => "(",
//             Op::RPAR => ")",
//             Op::LSQB => "[",
//             Op::RSQB => "]",
//             Op::COLON => ":",
//             Op::COMMA => ",",
//             Op::SEMI => ";",
//             Op::PLUS => "+",
//             Op::MINUS => "-",
//             Op::STAR => "*",
//             Op::SLASH => "/",
//             Op::VBAR => "|",
//             Op::AMPER => "&",
//             Op::LESS => "<",
//             Op::GREATER => ">",
//             Op::EQUAL => "=",
//             Op::DOT => ".",
//             Op::PERCENT => "%",
//             Op::LBRACE => "{",
//             Op::RBRACE => "}",
//             Op::EQEQUAL => "==",
//             Op::NOTEQUAL => "!=",
//             Op::LESSEQUAL => "<=",
//             Op::GREATEREQUAL => ">=",
//             Op::TILDE => "~",
//             Op::CIRCUMFLEX => "^",
//             Op::LEFTSHIFT => "<<",
//             Op::RIGHTSHIFT => ">>",
//             Op::DOUBLESTAR => "**",
//             Op::PLUSEQUAL => "+=",
//             Op::MINEQUAL => "-=",
//             Op::STAREQUAL => "*=",
//             Op::SLASHEQUAL => "/=",
//             Op::PERCENTEQUAL => "%=",
//             Op::AMPEREQUAL => "&=",
//             Op::VBAREQUAL => "|=",
//             Op::CIRCUMFLEXEQUAL => "^=",
//             Op::LEFTSHIFTEQUAL => "<<=",
//             Op::RIGHTSHIFTEQUAL => ">>=",
//             Op::DOUBLESTAREQUAL => "**=",
//             Op::DOUBLESLASH => "//",
//             Op::DOUBLESLASHEQUAL => "//=",
//             Op::AT => "@",
//             Op::ATEQUAL => "@=",
//             Op::RARROW => "->",
//             Op::ELLIPSIS => "...",
//             Op::COLONEQUAL => ":=",
//             Op::EXCLAMATION => "!",
//         }
//     }
// }

// Cargo.toml: paste = "1.0"
use paste::paste;

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
    FALSE => "False",
    AWAIT => "await",
    ELSE => "else",
    IMPORT => "import",
    PASS => "pass",
    NONE => "None",
    BREAK => "break",
    EXCEPT => "except",
    IN => "in",
    RAISE => "raise",
    TRUE => "True",
    CLASS => "class",
    FINALLY => "finally",
    IS => "is",
    RETURN => "return",
    AND => "and",
    CONTINUE => "continue",
    FOR => "for",
    LAMBDA => "lambda",
    TRY => "try",
    AS => "as",
    DEF => "def",
    FROM => "from",
    NONLOCAL => "nonlocal",
    WHILE => "while",
    ASSERT => "assert",
    DEL => "del",
    GLOBAL => "global",
    NOT => "not",
    WITH => "with",
    ASYNC => "async",
    ELIF => "elif",
    IF => "if",
    OR => "or",
    YIELD => "yield",
);

// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
// pub enum Keyword {
//     FALSE,
//     AWAIT,
//     ELSE,
//     IMPORT,
//     PASS,
//     NONE,
//     BREAK,
//     EXCEPT,
//     IN,
//     RAISE,
//     TRUE,
//     CLASS,
//     FINALLY,
//     IS,
//     RETURN,
//     AND,
//     CONTINUE,
//     FOR,
//     LAMBDA,
//     TRY,
//     AS,
//     DEF,
//     FROM,
//     NONLOCAL,
//     WHILE,
//     ASSERT,
//     DEL,
//     GLOBAL,
//     NOT,
//     WITH,
//     ASYNC,
//     ELIF,
//     IF,
//     OR,
//     YIELD,
// }
//
// impl Keyword {
//     pub fn new(keyword: &str) -> Option<Keyword> {
//         let kw = match keyword {
//             "False" => Keyword::FALSE,
//             "await" => Keyword::AWAIT,
//             "else" => Keyword::ELSE,
//             "import" => Keyword::IMPORT,
//             "pass" => Keyword::PASS,
//             "None" => Keyword::NONE,
//             "break" => Keyword::BREAK,
//             "except" => Keyword::EXCEPT,
//             "in" => Keyword::IN,
//             "raise" => Keyword::RAISE,
//             "True" => Keyword::TRUE,
//             "class" => Keyword::CLASS,
//             "finally" => Keyword::FINALLY,
//             "is" => Keyword::IS,
//             "return" => Keyword::RETURN,
//             "and" => Keyword::AND,
//             "continue" => Keyword::CONTINUE,
//             "for" => Keyword::FOR,
//             "lambda" => Keyword::LAMBDA,
//             "try" => Keyword::TRY,
//             "as" => Keyword::AS,
//             "def" => Keyword::DEF,
//             "from" => Keyword::FROM,
//             "nonlocal" => Keyword::NONLOCAL,
//             "while" => Keyword::WHILE,
//             "assert" => Keyword::ASSERT,
//             "del" => Keyword::DEL,
//             "global" => Keyword::GLOBAL,
//             "not" => Keyword::NOT,
//             "with" => Keyword::WITH,
//             "async" => Keyword::ASYNC,
//             "elif" => Keyword::ELIF,
//             "if" => Keyword::IF,
//             "or" => Keyword::OR,
//             "yield" => Keyword::YIELD,
//             _ => return None,
//         };
//         Some(kw)
//     }
//     pub fn value(&self) -> &'static str {
//         match self {
//             Keyword::FALSE => "False",
//             Keyword::AWAIT => "await",
//             Keyword::ELSE => "else",
//             Keyword::IMPORT => "import",
//             Keyword::PASS => "pass",
//             Keyword::NONE => "None",
//             Keyword::BREAK => "break",
//             Keyword::EXCEPT => "except",
//             Keyword::IN => "in",
//             Keyword::RAISE => "raise",
//             Keyword::TRUE => "True",
//             Keyword::CLASS => "class",
//             Keyword::FINALLY => "finally",
//             Keyword::IS => "is",
//             Keyword::RETURN => "return",
//             Keyword::AND => "and",
//             Keyword::CONTINUE => "continue",
//             Keyword::FOR => "for",
//             Keyword::LAMBDA => "lambda",
//             Keyword::TRY => "try",
//             Keyword::AS => "as",
//             Keyword::DEF => "def",
//             Keyword::FROM => "from",
//             Keyword::NONLOCAL => "nonlocal",
//             Keyword::WHILE => "while",
//             Keyword::ASSERT => "assert",
//             Keyword::DEL => "del",
//             Keyword::GLOBAL => "global",
//             Keyword::NOT => "not",
//             Keyword::WITH => "with",
//             Keyword::ASYNC => "async",
//             Keyword::ELIF => "elif",
//             Keyword::IF => "if",
//             Keyword::OR => "or",
//             Keyword::YIELD => "yield",
//         }
//     }
// }
//
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum NumLit {
    #[default]
    Dec,
    Bin,
    Oct,
    Hex,
    Zero,
}

impl NumLit {
    pub fn is_valid_kind(num: char, kind: NumLit) -> bool {
        match kind {
            NumLit::Dec => num.is_ascii_digit(),
            NumLit::Bin => num == '0' || num == '1',
            NumLit::Oct => ('0'..'8').contains(&num),
            NumLit::Hex => num.is_ascii_hexdigit(),
            NumLit::Zero => num == '0',
        }
    }
}
