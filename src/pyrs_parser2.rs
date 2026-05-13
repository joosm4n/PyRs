// input: file
// ouput: item tree

use std::sync::Arc;

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

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub enum ItemKind {
    #[default]
    Unknown,
    NumLiteral,
    StrLiteral,
    Ident,
    Operator,
}

#[derive(Clone)]
pub struct Item<'a> {
    pub data: &'a str,
    pub file: Arc<FileData>,
    pub line: usize,
    pub col: usize,
    pub kind: ItemKind,
}

impl<'a> Item<'a> {
    pub fn get_line(&self) -> &str {
        self.file
            .get_line(self.line)
            .expect("Should never get here, as should have valid ref to file_data")
    }
    pub fn get_line_nice(&self) -> String {
        let ln = self.get_line();
        format!(" {}\t|{}", self.line, ln)
    }

    pub fn basic(data: &'a str, file: Arc<FileData>, kind: ItemKind) -> Self {
        Item {
            data,
            file,
            line: 0,
            col: 0,
            kind,
        }
    }
}

impl<'a> std::fmt::Debug for Item<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item[data: {}, kind:{:?}]", self.data, self.kind)
    }
}
impl<'a> PartialEq for Item<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.data == other.data
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn parse<'a>(src: &'a str, file_data: Arc<FileData>) -> Vec<Item<'a>> {
        let trimmed_lines: Vec<_> = src.lines().collect();
        let mut itms = vec![];
        for (line_no, line) in trimmed_lines.as_slice().iter().enumerate() {
            let mut split = Parser::split_line_to_items(line, file_data.clone(), line_no);
            itms.append(&mut split);
        }
        itms
    }

    fn split_line_to_items<'a>(
        line: &'a str,
        file_data: Arc<FileData>,
        line_no: usize,
    ) -> Vec<Item<'a>> {
        if line.is_empty() {
            return vec![];
        }

        let mut words: Vec<(usize, usize, ItemKind)> = vec![];
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
                    words.push((start_idx, end_idx, ItemKind::StrLiteral));
                }

                // Operator
                '!' | '=' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '{'
                | '}' | '(' | ')' | '[' | ']' => {
                    if let Some(&(_, next_ch)) = chars.peek() {
                        if next_ch == '=' {
                            chars.next();
                            let end_idx = start_idx + ch.len_utf8() + next_ch.len_utf8();
                            words.push((start_idx, end_idx, ItemKind::Operator));
                        } else {
                            let end_idx = start_idx + ch.len_utf8();
                            words.push((start_idx, end_idx, ItemKind::Operator));
                        }
                    } else {
                        words.push((start_idx, start_idx + ch.len_utf8(), ItemKind::Operator));
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

                    words.push((start_idx, end_idx, ItemKind::NumLiteral));
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

                    words.push((start_idx, end_idx, ItemKind::Ident));
                }

                // ??
                c if !c.is_alphanumeric() && c != '.' => {
                    words.push((start_idx, start_idx + c.len_utf8(), ItemKind::Unknown));
                }

                // Standalone Dot
                '.' => {
                    words.push((start_idx, start_idx + 1, ItemKind::Unknown));
                }

                // Handle any other characters
                _ => {
                    words.push((start_idx, start_idx + ch.len_utf8(), ItemKind::Unknown));
                }
            }
        }

        words
            .into_iter()
            .map(|(s, e, tk_kind)| Item {
                data: &line[s..e],
                file: file_data.clone(),
                line: line_no,
                col: s,
                kind: tk_kind,
            })
            .collect()
    }

    pub fn parse_no_file<'a>(raw_str: &'a str) -> (Arc<FileData>, Vec<Item<'a>>) {
        let fd = Arc::new(FileData::new(
            "NOFILE".into(),
            "NOFILE".into(),
            raw_str.into(),
        ));
        (fd.clone(), Parser::parse(raw_str, fd.clone()))
    }

    pub fn _parse_test<'a>(raw_str: &'a str, file_data: Arc<FileData>) -> Vec<Item<'a>> {
        Parser::parse(raw_str, file_data.clone())
    }
}
