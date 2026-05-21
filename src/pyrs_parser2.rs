// input: file
// ouput: item tree

use crate::pyrs_tokentypes::*;
use std::sync::Arc;

pub type DynError = Box<dyn std::error::Error>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FileData {
    file_path: String,
    file_name: String,
    contents: String,
}

impl FileData {
    pub fn new(file_path: String, file_name: String, contents: String) -> Self {
        FileData {
            file_path,
            file_name,
            contents,
        }
    }

    pub fn get_contents(&self) -> &str {
        &self.contents
    }

    pub fn empty() -> Self {
        Self {
            file_path: "".into(),
            file_name: "".into(),
            contents: "".into(),
        }
    }

    pub fn get_contents_fmt(&self) -> String {
        let mut s = format!("{}:{}", self.file_path, self.file_name);
        s.reserve(self.contents.len() * 1.5 as usize);
        for (line_no, line) in self.contents.lines().enumerate() {
            s.push_str(&format!("\n{}\t|  {}", line_no, line));
        }
        s
    }

    pub fn get_line(&self, num: usize) -> Option<&str> {
        let lines = self.contents.lines();
        if lines.count() <= num {
            None
        } else {
            self.contents.lines().nth(num)
        }
    }

    pub fn get_line_fmt(&self, num: usize, extra_data: bool) -> Option<String> {
        let ln = self.get_line(num)?;
        if extra_data {
            Some(format!(
                "  --> {}:{}:{}\n {}\t|  {}",
                self.file_path, self.file_name, num, num, ln
            ))
        } else {
            Some(format!(" {}\t|  {}", num, ln))
        }
    }
}

#[derive(Clone)]
pub struct ParserError {
    pub msg: String,
    pub token: TokenOwned,
    pub token_tree: Vec<TokenOwned>,
}

impl ParserError {
    pub fn new_dyn<'a>(msg: String, token: Token<'a>, token_list: Vec<Token<'a>>) -> DynError {
        let mut owned = token.to_owned_token();
        owned.kind = TokenKind::ErrorToken;
        Box::new(Self {
            msg,
            token: owned,
            token_tree: token_list.iter().map(|t| t.to_owned_token()).collect(),
        })
    }
    pub fn empty() -> ParserError {
        Self {
            msg: "".into(),
            token: TokenOwned {
                data: "".into(),
                file: Arc::new(FileData::empty()),
                line: 0,
                col: 0,
                kind: TokenKind::ErrorToken,
            },
            token_tree: vec![],
        }
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "ParseError: {} for: {:?}",
            self.token.dbg_str(),
            self.msg
        )
    }
}
impl std::fmt::Debug for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n{}", self.token.get_line_fmt())?;
        writeln!(f, "  = {}", self)
        // writeln!(f, "  = ParseError: {} for: {:?}", self.msg, self.dbg_str())
    }
}
impl PartialEq for ParserError {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl std::error::Error for ParserError {}

#[derive(Debug, Clone, Copy, Default)]
pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn parse<'a>(src: &'a str, file_data: Arc<FileData>) -> Result<Vec<Token<'a>>, DynError> {
        // let trimmed_lines: Vec<_> = src.lines().collect();
        let trimmed_lines: Vec<_> = src.split_inclusive('\n').collect();
        let mut tokens = vec![];
        for (line_no, line) in trimmed_lines.as_slice().iter().enumerate() {
            let mut split = Parser::split_line_to_items(line, file_data.clone(), line_no)?;
            tokens.append(&mut split);
        }
        tokens.push(Token {
            data: &src[src.len() - 1..],
            file: file_data.clone(),
            line: trimmed_lines.len() - 1,
            col: 0,
            kind: TokenKind::EndMarker,
        });
        Ok(tokens)
    }

    fn split_line_to_items<'a>(
        line: &'a str,
        file_data: Arc<FileData>,
        line_no: usize,
    ) -> Result<Vec<Token<'a>>, DynError> {
        if line.is_empty() {
            return Ok(vec![]);
        }

        let mut words: Vec<Token> = vec![];
        let mut chars = line.char_indices().peekable();
        let mut hit_tokens = false;

        'parse_loop: while let Some((start_idx, ch)) = chars.next() {
            match ch {
                // Handle indent
                c if (c == ' ' || c == '\t' || c == '\u{000c}') && !hit_tokens => {
                    while let Some(&(end_idx, next_char)) = chars.peek() {
                        if !next_char.is_whitespace() {
                            words.push(Token {
                                data: &line[0..end_idx],
                                file: file_data.clone(),
                                line: line_no,
                                col: start_idx,
                                kind: TokenKind::Indent,
                            });
                            hit_tokens = true;
                            break;
                        } else {
                            chars.next();
                        }
                    }
                }

                // Whitespace: ' ', tab, formfeed
                ' ' | '\t' | '\u{000c}' => continue,

                // String literals
                '\"' | '\'' => {
                    hit_tokens = true;
                    let quote_char = ch;
                    let mut end_idx = start_idx + ch.len_utf8();

                    // Find the closing quote
                    for (idx, c) in chars.by_ref() {
                        end_idx = idx + c.len_utf8();
                        if c == quote_char {
                            break;
                        }
                    }
                    words.push(Token {
                        data: &line[start_idx..end_idx],
                        file: file_data.clone(),
                        line: line_no,
                        col: start_idx,
                        kind: TokenKind::String,
                    });
                }

                // Operator
                '!' | '=' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '{'
                | '}' | '(' | ')' | '[' | ']' => {
                    hit_tokens = true;
                    if let Some(&(_, next_ch)) = chars.peek() {
                        if next_ch == '=' {
                            let mut op_str = ch.to_string();
                            op_str.push(next_ch);
                            let op = Op::new(&op_str).expect("Already checked for it");
                            chars.next();
                            let end_idx = start_idx + ch.len_utf8() + next_ch.len_utf8();
                            words.push(Token {
                                data: &line[start_idx..end_idx],
                                file: file_data.clone(),
                                line: line_no,
                                col: start_idx,
                                kind: TokenKind::Op(op),
                            });
                        } else {
                            let op = Op::new(&ch.to_string()).expect("Already checked for ok");
                            let end_idx = start_idx + ch.len_utf8();
                            words.push(Token {
                                data: &line[start_idx..end_idx],
                                file: file_data.clone(),
                                line: line_no,
                                col: start_idx,
                                kind: TokenKind::Op(op),
                            });
                        }
                    } else {
                        let op = Op::new(&ch.to_string()).expect("Already checked for ok");
                        words.push(Token {
                            data: &line[start_idx..start_idx + 1],
                            file: file_data.clone(),
                            line: line_no,
                            col: start_idx,
                            kind: TokenKind::Op(op),
                        });
                    }
                }

                // Number Literal
                c if c.is_numeric() => {
                    hit_tokens = true;
                    let mut end_idx = start_idx + c.len_utf8();
                    let mut has_dot = false;
                    let mut last_was_underscore = false;
                    let mut lit_kind = NumLit::Dec;

                    // TODO: Finish the alternate num literal parsing

                    // dec | bin | oct | hex | zero
                    if c == '0' {
                        if let Some((_idx, next_ch)) = chars.peek() {
                            lit_kind = match *next_ch {
                                'b' | 'B' => {
                                    chars.next();
                                    end_idx += 1;
                                    NumLit::Bin
                                }
                                'o' | 'O' => {
                                    chars.next();
                                    end_idx += 1;
                                    NumLit::Oct
                                }
                                'x' | 'X' => {
                                    chars.next();
                                    end_idx += 1;
                                    NumLit::Hex
                                }
                                c if c.is_whitespace() => NumLit::Dec,
                                '_' => {
                                    chars.next();
                                    end_idx += 1;
                                    let mut zero_ok = false;

                                    if chars
                                        .peek()
                                        .is_some_and(|(_idx, next_ch2)| *next_ch2 == '0')
                                    {
                                        chars.next();
                                        end_idx += 1;
                                        if let Some((_idx, next_ch3)) = chars.peek().copied() {
                                            if !NumLit::is_valid_kind(next_ch3, NumLit::Hex)
                                                || next_ch3.is_whitespace()
                                            {
                                                if next_ch3.is_whitespace() {
                                                    end_idx -= 1;
                                                }
                                                zero_ok = true;
                                            }
                                        } else {
                                            zero_ok = true;
                                        }
                                    }

                                    if !zero_ok {
                                        return Err(ParserError::new_dyn(
                                            "Only can have a \'_\' after a \'0\' when followed by a 0. Eg: 0_0 not 0_x5.".into(),
                                            Token{
                                                data: &line[start_idx..end_idx],
                                                file: file_data.clone(),
                                                line: line_no,
                                                col: start_idx,
                                                kind: TokenKind::Number(lit_kind)
                                            },
                                            words,
                                        ));
                                    } else {
                                        NumLit::Zero
                                    }
                                }
                                c if NumLit::is_valid_kind(c, NumLit::Dec) => {
                                    chars.next();
                                    end_idx += 1;
                                    NumLit::Dec
                                }
                                _ => {
                                    return Err(ParserError::new_dyn(
                                        "Invalid Number Literal".into(),
                                        Token {
                                            data: &line[start_idx..end_idx + 1],
                                            file: file_data.clone(),
                                            line: line_no,
                                            col: start_idx,
                                            kind: TokenKind::Number(lit_kind),
                                        },
                                        words,
                                    ));
                                }
                            }
                        }
                    }

                    // normal number literal
                    while let Some(&(idx, next_ch)) = chars.peek() {
                        match next_ch {
                            nc if NumLit::is_valid_kind(nc, lit_kind) => {
                                chars.next();
                                end_idx = idx + next_ch.len_utf8();
                                last_was_underscore = false;
                            }

                            nc if nc == '.' && !has_dot => {
                                // Look ahead to see if there's a digit after the dot
                                let mut temp_chars = chars.clone();
                                temp_chars.next(); // consume the dot
                                if temp_chars
                                    .peek()
                                    .is_some_and(|&(_, char_after_dot)| char_after_dot.is_numeric())
                                {
                                    // It's a float like 3.14
                                    chars.next();
                                    end_idx = idx + next_ch.len_utf8();
                                    has_dot = true;
                                } else {
                                    // If dot is not followed by a number, stop here
                                    break;
                                }
                                last_was_underscore = false;
                            }

                            '_' => {
                                if !last_was_underscore {
                                    chars.next();
                                    end_idx = idx + next_ch.len_utf8();
                                    last_was_underscore = true;
                                } else {
                                    let tok = Token {
                                        data: &line[start_idx..end_idx + 1],
                                        file: file_data.clone(),
                                        line: line_no,
                                        col: end_idx + 1,
                                        kind: TokenKind::Number(lit_kind),
                                    };
                                    return Err(ParserError::new_dyn(
                                        "Underscores can only occur inbetween digits".into(),
                                        tok,
                                        words,
                                    ));
                                }
                            }
                            _ => {
                                break;
                            }
                        }
                    }

                    if last_was_underscore {
                        let tok = Token {
                            data: &line[start_idx..end_idx + 1],
                            file: file_data.clone(),
                            line: line_no,
                            col: end_idx + 1,
                            kind: TokenKind::Number(lit_kind),
                        };
                        return Err(ParserError::new_dyn(
                            "Underscores can only occur inbetween digits".into(),
                            tok,
                            words,
                        ));
                    }

                    words.push(Token {
                        data: &line[start_idx..end_idx],
                        file: file_data.clone(),
                        line: line_no,
                        col: start_idx,
                        kind: TokenKind::Number(lit_kind),
                    });
                }

                // Identifiers
                c if c.is_alphabetic() || c == '_' => {
                    hit_tokens = true;
                    let mut end_idx = start_idx + c.len_utf8();

                    while let Some(&(idx, next_ch)) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            chars.next();
                            end_idx = idx + next_ch.len_utf8();
                        } else {
                            break;
                        }
                    }

                    words.push(Token {
                        data: &line[start_idx..end_idx],
                        file: file_data.clone(),
                        line: line_no,
                        col: start_idx,
                        kind: TokenKind::Name,
                    });
                }

                // Comment
                '#' => {
                    let mut end_idx = start_idx + 1;
                    for (idx, _nc) in chars.by_ref() {
                        end_idx = idx;
                    }
                    words.push(Token {
                        data: &line[start_idx..end_idx],
                        file: file_data.clone(),
                        line: line_no,
                        col: start_idx,
                        kind: TokenKind::Commment,
                    });
                    words.push(Token {
                        data: &line[start_idx..end_idx],
                        file: file_data.clone(),
                        line: line_no,
                        col: start_idx,
                        kind: TokenKind::NewLine,
                    });
                    break 'parse_loop;
                }

                '\n' | '\r' => {
                    words.push(Token {
                        data: &line[start_idx..start_idx + 1],
                        file: file_data.clone(),
                        line: line_no,
                        col: start_idx,
                        kind: TokenKind::NewLine,
                    });
                    break 'parse_loop;
                }

                '\\' => {
                    // for success it should be either nothing or whitespace until end
                    for (_idx, next_ch) in chars.by_ref() {
                        if !next_ch.is_whitespace() {
                            return Err(ParserError::new_dyn(
                                "Must have '\\' as last character".into(),
                                Token {
                                    data: &line[start_idx..start_idx + 1],
                                    file: file_data.clone(),
                                    line: line_no,
                                    col: start_idx,
                                    kind: TokenKind::NL,
                                },
                                words,
                            ));
                            // break;
                        }
                    }
                    hit_tokens = true;
                    words.push(Token {
                        data: &line[start_idx..start_idx + 1],
                        file: file_data.clone(),
                        line: line_no,
                        col: start_idx,
                        kind: TokenKind::NL,
                    });
                }

                // Honestly can't remember what this does...
                c if !c.is_alphanumeric() && c != '.' => {
                    let tk = match Op::new(&c.to_string()) {
                        Some(o) => TokenKind::Op(o),
                        None => {
                            println!("Unknown Token: {}", c);
                            TokenKind::Unknown
                        }
                    };
                    hit_tokens = true;
                    words.push(Token {
                        data: &line[start_idx..start_idx + 1],
                        file: file_data.clone(),
                        line: line_no,
                        col: start_idx,
                        kind: tk,
                    });
                }

                // Standalone Dot
                '.' => {
                    hit_tokens = true;
                    words.push(Token {
                        data: &line[start_idx..start_idx + 1],
                        file: file_data.clone(),
                        line: line_no,
                        col: start_idx,
                        kind: TokenKind::Op(Op::DOT_),
                    });
                }

                // Handle any other characters
                _ => {
                    println!("Unknown Token: {}", ch);
                    hit_tokens = true;

                    // words.push((start_idx, start_idx + ch.len_utf8(), TokenKind::Unknown));
                    words.push(Token {
                        data: &line[start_idx..start_idx + ch.len_utf8()],
                        file: file_data.clone(),
                        line: line_no,
                        col: start_idx,
                        kind: TokenKind::Unknown,
                    });
                }
            }
        }

        Ok(words)
    }

    pub fn parse_no_file<'a>(
        raw_str: &'a str,
    ) -> (Arc<FileData>, Result<Vec<Token<'a>>, DynError>) {
        let fd = Arc::new(FileData::new(
            "NOFILE".into(),
            "NOFILE".into(),
            raw_str.into(),
        ));
        (fd.clone(), Parser::parse(raw_str, fd.clone()))
    }

    pub fn _parse_test<'a>(raw_str: &'a str) -> Result<Vec<Token<'a>>, DynError> {
        let fd = Arc::new(FileData::new(
            "NOFILE".into(),
            "NOFILE".into(),
            raw_str.into(),
        ));
        let mut tks = Parser::parse(raw_str, fd)?;
        tks.pop();
        Ok(tks.to_owned())
    }
}
