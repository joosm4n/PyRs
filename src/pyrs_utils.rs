

    pub fn str_starts_with(input: &str, op: fn(char) -> bool) -> bool
    {
        input.chars().next().map_or(false, |c| op(c))
    }

    pub fn trim_first_and_last(value: &str) -> &str 
    {
        let mut chars = value.chars();
        chars.next();     
        chars.next_back();
        chars.as_str()    
    }

    pub fn get_indent(line: &str) -> usize 
    {
        let mut indent: usize = 0;
        for c in line.chars() {
            if c != ' ' {
                break;
            }
            indent += 1;
        }
        indent
    }

    pub fn split_to_lines(file: &str) -> Vec<&str>
    {
        dbg!(file);
        if file.is_empty() {
            return vec![];
        }

        let mut lines: Vec<&str> = vec![];
        let mut start_of_line_idx = 0usize;
        let mut final_idx = 0;
        for (curr_idx, c) in file.char_indices()
        {
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

    pub fn split_to_words(sentence: &str) -> Vec<&str>
    {
        if sentence.is_empty() {
            return vec![];
        }
        let mut words: Vec<&str> = vec![]; 

        let mut start_of_word_idx: usize = 0;
        let mut in_word = false;
        let mut in_str_lit = false;
        let mut last_was_dot = false;
        let mut last_was_eq = false;

        for (curr_idx, c) in sentence.char_indices()
        {
            let is_whitespace = c.is_whitespace();
            let is_symbol = !c.is_alphanumeric() && !is_whitespace;
            let is_num = c.is_numeric();
            let is_alpha = c.is_alphabetic() || c == '_';
            let is_dot = c == '.';
            let is_quotes = c == '"';
            let is_eq = c == '!' || c == '=' || c == '<' || c == '>';

            if is_quotes 
            {
                if !in_str_lit && !in_word {
                    in_str_lit = true;
                    start_of_word_idx = curr_idx;
                }
                else if in_str_lit {
                    words.push(&sentence[start_of_word_idx..curr_idx + 1]);
                    in_str_lit = false;
                    start_of_word_idx = curr_idx + 1;
                }
                else {
                    panic!("bad token: {}", c);
                }
                last_was_eq = false;
                last_was_dot = false;
            }
            else if is_eq
            {
                if !in_word && !in_str_lit {
                    in_word = true;
                    start_of_word_idx = curr_idx;
                }
                last_was_eq = true;
                last_was_dot = false;
            }
            else if is_alpha 
            {
                if !in_word && !in_str_lit {
                    in_word = true;
                    start_of_word_idx = curr_idx;
                }
                last_was_eq = false;
                last_was_dot = false;
            }
            else if is_num
            {
                if !in_word { 
                    in_word = true;
                    start_of_word_idx = curr_idx;
                }
                last_was_eq = false;
                last_was_dot = false;
            }
            else if is_dot
            {
                if !in_word { panic!("bad token: {c}")};
                last_was_eq = false;
                last_was_dot = true;
            }
            else if is_symbol && !in_str_lit
            {
                if in_word && !last_was_eq {
                    words.push(&sentence[start_of_word_idx..curr_idx]);
                }
                words.push(&sentence[curr_idx..(curr_idx+1)]);
                in_word = false;
                last_was_eq = false;
                last_was_dot = false;
            }
            else if is_whitespace && in_word
            {
                if last_was_dot { 
                    let curr_word = &sentence[start_of_word_idx..curr_idx];
                    panic!(
                        "[ParseError] Word cannot finish with a '.' \n\
                        [ParseError] Invalid word: '{}'", curr_word
                    ); 
                } 
                words.push(&sentence[start_of_word_idx..curr_idx]);
                in_word = false;
                last_was_eq = false;
                last_was_dot = false;
            }
            
        }

        if in_word
        {
            words.push(&sentence[start_of_word_idx..sentence.len()])
        }
        return words;
    }
