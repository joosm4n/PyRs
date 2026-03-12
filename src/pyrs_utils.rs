use core::mem::size_of;

use core::hash::Hash;
use std::collections::HashMap;
use std::ops::{AddAssign, ShlAssign};

pub struct PyUtils {}

impl PyUtils {
    pub fn str_starts_with(input: &str, op: fn(char) -> bool) -> bool {
        input.chars().next().map(op).unwrap_or(false)
    }

    pub fn trim_first_and_last(value: &str) -> &str {
        let mut chars = value.chars();
        chars.next();
        chars.next_back();
        chars.as_str()
    }

    pub fn usize_from_bytes(bytes: Vec<u8>) -> Option<usize> {
        if size_of::<usize>() != bytes.len() {
            None
        } else {
            let mut num: usize = 0;
            for b in bytes {
                num += b as usize;
                num <<= 8;
            }
            Some(num)
        }
    }

    pub fn get_indent(line: &str) -> usize {
        let mut indent: usize = 0;
        for c in line.chars() {
            match c {
                ' ' => indent += 1,
                '\t' => indent += 4,
                _ => break,
            }
        }
        indent
    }

    pub fn split_to_lines(file: &str) -> Vec<&str> {
        dbg!(file);
        if file.is_empty() {
            return vec![];
        }

        let mut lines: Vec<&str> = vec![];
        let mut start_of_line_idx = 0usize;
        let mut final_idx = 0;
        for (curr_idx, c) in file.char_indices() {
            if c == '\n' {
                lines.push(&file[start_of_line_idx..curr_idx]);
                start_of_line_idx = curr_idx;
            }
            final_idx = curr_idx;
        }
        lines.push(&file[start_of_line_idx..final_idx]);
        dbg!(&lines);
        lines
    }

    pub fn split_to_words(sentence: &str) -> Vec<&str> {
        if sentence.is_empty() {
            return vec![];
        }

        let mut words = Vec::new();
        let mut chars = sentence.char_indices().peekable();

        while let Some((start_idx, ch)) = chars.next() {
            match ch {
                // Handle whitespace - skip it
                c if c.is_whitespace() => continue,

                // Handle string literals
                '"' | '\'' => {
                    let quote_char = ch;
                    let mut end_idx = start_idx + ch.len_utf8();

                    // Find the closing quote
                    while let Some((idx, c)) = chars.next() {
                        end_idx = idx + c.len_utf8();
                        if c == quote_char {
                            break;
                        }
                    }
                    words.push(&sentence[start_idx..end_idx]);
                }

                '!' | '=' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' => {
                    if let Some(&(_, next_ch)) = chars.peek() {
                        if next_ch == '=' {
                            chars.next();
                            let end_idx = start_idx + ch.len_utf8() + next_ch.len_utf8();
                            words.push(&sentence[start_idx..end_idx]);
                        } else {
                            let end_idx = start_idx + ch.len_utf8();
                            words.push(&sentence[start_idx..end_idx]);
                        }
                    } else {
                        words.push(&sentence[start_idx..start_idx + ch.len_utf8()]);
                    }
                }

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

                    words.push(&sentence[start_idx..end_idx]);
                }

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

                    words.push(&sentence[start_idx..end_idx]);
                }

                c if !c.is_alphanumeric() && c != '.' => {
                    words.push(&sentence[start_idx..start_idx + c.len_utf8()]);
                }

                // Handle standalone dot
                '.' => {
                    words.push(&sentence[start_idx..start_idx + 1]);
                }

                // Handle any other characters
                _ => {
                    words.push(&sentence[start_idx..start_idx + ch.len_utf8()]);
                }
            }
        }
        //dbg!(&words);
        words
    }

    pub fn curr_time() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    pub fn curr_dir() -> String {
        std::env::current_dir().unwrap().to_str().unwrap().into()
    }

    pub fn hash_hashmap<T: core::hash::Hash, V: core::hash::Hash, H: std::hash::Hasher>(
        map: &HashMap<T, V>,
        state: &mut H,
    ) {
        let hashable: Vec<(&T, &V)> = map.iter().collect();
        hashable.hash(state);
    }
}

pub trait FromBytes: AddAssign + ShlAssign + From<u8> + Default + std::fmt::Debug {
    fn from_bytes_le(le_bytes: &[u8]) -> Option<Self> {
        if size_of::<Self>() != le_bytes.len() {
            None
        } else {
            let mut num = Self::default();
            let mut be_bytes = le_bytes.to_vec().clone();
            be_bytes.reverse();
            for b in be_bytes {
                num <<= Self::from(8);
                num += Self::from(b);
            }
            Some(num)
        }
    }

    fn from_bytes_be(be_bytes: &[u8]) -> Option<Self> {
        if size_of::<Self>() != be_bytes.len() {
            None
        } else {
            let mut num = Self::default();
            for b in be_bytes {
                num <<= Self::from(8);
                num += Self::from(*b);
            }
            Some(num)
        }
    }
}

impl FromBytes for usize {}
impl FromBytes for u64 {}
impl FromBytes for u32 {}
impl FromBytes for u16 {}
impl FromBytes for u8 {}
