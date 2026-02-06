pub fn str_starts_with(input: &str, op: fn(char) -> bool) -> bool {
    input.chars().next().map_or(false, |c| op(c))
}

pub fn trim_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
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
    return lines;
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

            c if !c.is_alphanumeric() && c != '.' => {
                words.push(&sentence[start_idx..start_idx + c.len_utf8()]);
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
