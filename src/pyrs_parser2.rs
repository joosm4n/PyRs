// input: file
// ouput: token tree

#[derive(Debug, Clone)]
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
        self.contents.as_str()
    }

    pub fn get_contents_fmt(&self) -> String {
        let mut s = format!("{}:{}", self.file_path, self.file_name);
        s.reserve(self.contents.len() * 1.5 as usize);
        for (line_no, line) in self.contents.lines().enumerate() {
            s.push_str(&format!("\n{}\t|{}", line_no, line));
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
                "  --> {}:{}:{}\n {}\t|{}",
                self.file_path, self.file_name, num, num, ln
            ))
        } else {
            Some(format!(" {}\t|{}", num, ln))
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    Unknown,
    Literal,
    Ident,
    Operator,
}

#[derive(Clone)]
pub struct Token<'a> {
    pub data: &'a str,
    pub file: &'a FileData,
    pub line: usize,
    pub col: usize,
    pub kind: TokenKind,
}

impl<'a> Token<'a> {
    pub fn get_line(&self) -> &str {
        self.file
            .get_line(self.line)
            .expect("Should never get here, as should have valid ref to file_data")
    }
    pub fn get_line_nice(&self) -> String {
        let ln = self.get_line();
        format!(" {}\t|{}", self.line, ln)
    }
}

impl<'a> std::fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token[data: {}, kind:{:?}]", self.data, self.kind)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn parse<'a>(file_data: &'a FileData) -> Vec<Token<'a>> {
        //let trimmed_lines: Vec<_> = file_data.contents.lines().map(|s| s.trim()).collect();
        let trimmed_lines: Vec<_> = file_data.contents.lines().collect();
        let mut tks = vec![];
        for (line_no, line) in trimmed_lines.as_slice().iter().enumerate() {
            let mut split = Parser::split_line_to_tokens(line, file_data, line_no);
            tks.append(&mut split);
        }
        tks
    }

    fn split_line_to_tokens<'a>(
        line: &'a str,
        file_data: &'a FileData,
        line_no: usize,
    ) -> Vec<Token<'a>> {
        if line.is_empty() {
            return vec![];
        }

        let mut words: Vec<(usize, usize, TokenKind)> = vec![];
        let mut chars = line.char_indices().peekable();

        while let Some((start_idx, ch)) = chars.next() {
            match ch {
                // Handle whitespace - skip it
                c if c.is_whitespace() => continue,

                // String literals
                '\"' | '\'' => {
                    let quote_char = ch;
                    let mut end_idx = start_idx + ch.len_utf8();

                    // Find the closing quote
                    for (idx, c) in chars.by_ref() {
                        end_idx = idx + c.len_utf8();
                        if c == quote_char {
                            break;
                        }
                    }
                    words.push((start_idx, end_idx, TokenKind::Literal));
                }

                // Operator
                '!' | '=' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '{'
                | '}' | '(' | ')' | '[' | ']' => {
                    if let Some(&(_, next_ch)) = chars.peek() {
                        if next_ch == '=' {
                            chars.next();
                            let end_idx = start_idx + ch.len_utf8() + next_ch.len_utf8();
                            words.push((start_idx, end_idx, TokenKind::Operator));
                        } else {
                            let end_idx = start_idx + ch.len_utf8();
                            words.push((start_idx, end_idx, TokenKind::Operator));
                        }
                    } else {
                        words.push((start_idx, start_idx + ch.len_utf8(), TokenKind::Unknown));
                    }
                }

                // Number Literal
                c if c.is_numeric() => {
                    let mut end_idx = start_idx + c.len_utf8();
                    let mut has_dot = false;

                    while let Some(&(idx, next_ch)) = chars.peek() {
                        if next_ch.is_numeric() {
                            chars.next();
                            end_idx = idx + next_ch.len_utf8();
                        } else if next_ch == '.' && !has_dot {
                            // Look ahead to see if there's a digit after the dot
                            let mut temp_chars = chars.clone();
                            temp_chars.next(); // consume the dot
                            if let Some(&(_, char_after_dot)) = temp_chars.peek() {
                                if char_after_dot.is_numeric() {
                                    // It's a float like 3.14
                                    chars.next();
                                    end_idx = idx + next_ch.len_utf8();
                                    has_dot = true;
                                } else {
                                    // Dot is not followed by a number, stop here
                                    break;
                                }
                            } else {
                                // Dot at end of input, stop here
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    words.push((start_idx, end_idx, TokenKind::Literal));
                }

                // Identifiers
                c if c.is_alphabetic() || c == '_' => {
                    let mut end_idx = start_idx + c.len_utf8();

                    while let Some(&(idx, next_ch)) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            chars.next();
                            end_idx = idx + next_ch.len_utf8();
                        } else {
                            break;
                        }
                    }

                    words.push((start_idx, end_idx, TokenKind::Ident));
                }

                // ??
                c if !c.is_alphanumeric() && c != '.' => {
                    words.push((start_idx, start_idx + c.len_utf8(), TokenKind::Unknown));
                }

                // Standalone Dot
                '.' => {
                    words.push((start_idx, start_idx + 1, TokenKind::Unknown));
                }

                // Handle any other characters
                _ => {
                    words.push((start_idx, start_idx + ch.len_utf8(), TokenKind::Unknown));
                }
            }
        }

        words
            .into_iter()
            .map(|(s, e, tk_kind)| Token {
                data: &line[s..e],
                file: file_data,
                line: line_no,
                col: s,
                kind: tk_kind,
            })
            .collect()
    }
}
