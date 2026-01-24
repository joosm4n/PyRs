use crate::{
    pyrs_obj::{Obj, PyObj, ToObj},
    pyrs_std::{FnPtr, Funcs, Import},
    pyrs_utils as Utils,
    pyrs_error::{PyException, PyError},
};

use std::{
    collections::HashMap,
    sync::Arc,
};

#[cfg(not(_YES_))]
macro_rules! dbg {
    ($($tt:tt)*) => {};
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Token<'a> {
    Ident(&'a str),
    Atom(&'a str),
    Op(Op),
    Sep(char),
    Def,
    Eof,
    Keyword(Keyword),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Op {
    Plus,
    Minus,
    Asterisk,
    ForwardSlash,
    Equals,
    Deref,

    Colon,
    SemiColon,
    Comma,
    DoubleQuotes,
    SingleQuote,

    RoundBracketsOpen,
    RoundBracketsClose,
    CurlyBracketsOpen,
    CurlyBracketsClose,
    SquareBracketsOpen,
    SquareBracketsClose,
    
    Pos,
    Neg,
    
    Not,
    Eq,
    Neq,
    LessThan,
    GreaterThan,
    LessEq,
    GreaterEq,
    
    List,
    Dot,
}

impl Op {
    pub fn try_get_prefix_binding(&self) -> Option<Op> {
        match self {
            Op::Plus => Some(Op::Pos),
            Op::Minus => Some(Op::Neg),
            Op::Asterisk => Some(Op::Deref),
            _ => None,
        }
    }

    pub fn prefix_binding_power(op: &Op) -> ((), f32) {
        match op {
            Op::Pos | Op::Neg => ((), 3.0),
            _ => panic!("Unknown operator {:?}", op),
        }
    }

    pub fn try_get_infix_binding(&self) -> Option<Op> {
        match self {
            Op::RoundBracketsOpen => Some(Op::RoundBracketsOpen),
            Op::RoundBracketsClose => Some(Op::RoundBracketsClose),
            Op::SquareBracketsOpen => Some(Op::SquareBracketsOpen),
            Op::SquareBracketsClose => Some(Op::SquareBracketsClose),
            Op::Equals => Some(Op::Equals),
            Op::Eq => Some(Op::Eq),
            Op::Neq => Some(Op::Neq),
            Op::Plus => Some(Op::Plus),
            Op::Minus => Some(Op::Minus),
            Op::Asterisk => Some(Op::Asterisk),
            Op::ForwardSlash => Some(Op::ForwardSlash),
            Op::Dot => Some(Op::Dot),
            Op::List => Some(Op::List),
            _ => None,
        }
    }

    pub fn infix_binding_power(op: &Op) -> (f32, f32) {
        match op {
            Op::RoundBracketsOpen | Op::RoundBracketsClose => (0.0, 0.1),
            Op::Equals => (0.2, 0.3),
            Op::Eq | Op::Neq | Op::LessEq | Op::LessThan | Op::GreaterEq | Op::GreaterThan => {
                (0.5, 0.6)
            }
            Op::Plus | Op::Minus => (1.0, 1.1),
            Op::Asterisk | Op::ForwardSlash => (2.0, 2.1),
            Op::Dot => (4.1, 4.0),
            _ => panic!("Unknown operator {:?}", op),
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ident: &str = match self {
            Op::Plus | Op::Pos => "+",
            Op::Minus | Op::Neg => "-",
            Op::Asterisk | Op::Deref => "*",
            Op::ForwardSlash => "/",
            Op::Equals => "=",
            Op::Eq => "==",
            Op::Neq => "!=",
            Op::LessThan => "<",
            Op::LessEq => "<=",
            Op::GreaterThan => ">",
            Op::GreaterEq => ">=",
            Op::Not => "!",
            Op::Colon => ":",
            Op::SemiColon => ";",
            Op::Comma => ",",
            Op::DoubleQuotes => "\"",
            Op::SingleQuote => "\'",
            Op::RoundBracketsOpen => "(",
            Op::RoundBracketsClose => ")",
            Op::CurlyBracketsOpen => "{",
            Op::CurlyBracketsClose => "}",
            Op::SquareBracketsOpen => "[",
            Op::SquareBracketsClose => "]",
            Op::Dot => ".",
            Op::List => "list",
        };
        write!(f, "{}", ident)
    }
}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Atom(atom) => write!(f, "Atom{{'{}'}}", atom),
            Token::Eof => write!(f, "EOF"),
            Token::Ident(ident) => write!(f, "Ident{{'{}'}}", ident),
            Token::Keyword(keyword) => write!(f, "Keyword{{'{}'}}", keyword),
            Token::Op(op) => write!(f, "Op{{'{}'}}", op),
            Token::Sep(sep) => write!(f, "Sep{{'{}'}}", sep),
            Token::Def => write!(f, "def"),
        }
    }
}

impl<'a> Token<'a> {
    pub fn try_get_keyword(word: &str) -> Option<Token<'a>> {
        let keyword = match word {
            "if" => Keyword::If,
            "elif" => Keyword::Elif,
            "else" => Keyword::Else,
            "for" => Keyword::For,
            "while" => Keyword::While,
            "def" => Keyword::Def,
            "True" => Keyword::True,
            "False" => Keyword::False,
            _ => return None,
        };
        return Some(Token::Keyword(keyword));
    }
}

pub struct Lexer<'a> {
    pub tokens: Vec<Token<'a>>,
}
impl<'a> std::fmt::Display for Lexer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lexer[")?;
        if self.tokens.len() == 0 {
            return write!(f, "]");
        }
        for token in &self.tokens[0..(self.tokens.len() - 1)] {
            write!(f, " {},", token)?;
        }
        if let Some(last) = self.tokens.last() {
            write!(f, " {}", last)?;
        }
        write!(f, "]")
    }
}

impl<'a> Lexer<'a> {

    pub fn from(words: &Vec<&'a str>) -> Self {
        let mut token_list: Vec<Token<'a>> = vec![];

        for &word in words {
            dbg!(format!("Parsing word: {}", word));

            let token: Token = match word {
                "+" => Token::Op(Op::Plus),
                "-" => Token::Op(Op::Minus),
                "/" => Token::Op(Op::ForwardSlash),
                "*" => Token::Op(Op::Asterisk),
                "=" => Token::Op(Op::Equals),
                "(" => Token::Op(Op::RoundBracketsOpen),
                ")" => Token::Op(Op::RoundBracketsClose),
                "[" => Token::Op(Op::SquareBracketsOpen),
                "]" => Token::Op(Op::SquareBracketsClose),
                ":" => Token::Op(Op::Colon),
                "!" => Token::Op(Op::Not),
                "==" => Token::Op(Op::Eq),
                "!=" => Token::Op(Op::Neq),
                "<" => Token::Op(Op::LessThan),
                ">" => Token::Op(Op::GreaterThan),
                "<=" => Token::Op(Op::LessEq),
                ">=" => Token::Op(Op::GreaterEq),
                "," => Token::Sep(','),
                "def" => Token::Def,
                word if Token::try_get_keyword(word).is_some() => {
                    Token::try_get_keyword(word).unwrap()
                }
                word if Utils::str_starts_with(word, char::is_numeric) => Token::Atom(word),
                word if Utils::str_starts_with(word, char::is_alphabetic) => Token::Ident(word),
                word if word.starts_with('\"') => Token::Atom(Utils::trim_first_and_last(word)),
                "" => continue,
                t => panic!("[Parse Error] Bad token: {:?}", t),
            };
            dbg!(format!("Parsed word is: {}", token));
            token_list.push(token);
        }
        token_list.reverse();
        return Lexer { tokens: token_list };
    }

    pub fn next(&mut self) -> Token<'a> {
        self.tokens.pop().unwrap_or(Token::Eof)
    }

    pub fn peek(&mut self) -> Token<'a> {
        self.tokens.last().copied().unwrap_or(Token::Eof)
    }

    pub fn parse_expression(&mut self, min_bp: f32) -> Expression {
        dbg!(format!("Parsing expr: {}", self));
        let mut lhs = match self.next() {
            Token::Eof => return Expression::None,
            Token::Atom(it) => Expression::Atom(it.to_string()),
            Token::Ident(ident) => match Funcs::try_get(ident) {
                Some(func) => {
                    let open = self.next();
                    assert_eq!(
                        open,
                        Token::Op(Op::RoundBracketsOpen),
                        "[Expression Error] Bad token: {}, must be '('",
                        open
                    );
                    let mut args: Vec<Expression> = vec![];
                    while self.peek() != Token::Op(Op::RoundBracketsClose) {
                        if self.peek() == Token::Sep(',') {
                            self.next();
                        }
                        args.push(self.parse_expression(0.0));
                    }
                    let close = self.next();
                    assert_eq!(
                        close,
                        Token::Op(Op::RoundBracketsClose),
                        "[Expression Error] Bad token: {}",
                        close
                    );
                    Expression::Func(func, args)
                }
                None => Expression::Ident(ident.to_string()),
            },
            Token::Keyword(keyword) => {
                match keyword {
                    Keyword::True => Expression::Keyword(Keyword::True, vec![], vec![]),
                    Keyword::False => Expression::Keyword(Keyword::False, vec![], vec![]),
                    Keyword::If | Keyword::Elif | Keyword::For | Keyword::While => {
                        let mut conditions: Vec<Expression> = vec![];
                        while self.peek() != Token::Op(Op::Colon) && self.peek() != Token::Eof {
                            dbg!(self.peek());
                            conditions.push(self.parse_expression(0.0));
                            dbg!(&conditions);
                        }
                        return Expression::Keyword(keyword, conditions, vec![])
                    }
                    Keyword::Def => {
                        let  mut args = vec![];
                        let info = vec![];

                        while self.peek() != Token::Op(Op::Colon) && self.peek() != Token::Eof {
                            args.push(self.parse_expression(0.0));
                            dbg!(&conditions);
                        }

                        Expression::Keyword(Keyword::Def, args, info)
                    } 
                    _ => unimplemented!(),
                }
            }
            Token::Op(op) => {
                if let Some(prefix) = op.try_get_prefix_binding() {
                    let ((), r_bp) = Op::prefix_binding_power(&prefix);
                    let rhs = self.parse_expression(r_bp);
                    return Expression::Operation(prefix, vec![rhs]);
                }

                match op {
                    Op::Colon => {
                        return Expression::Operation(Op::Colon, vec![]);
                    }
                    Op::RoundBracketsOpen => {
                        let lhs = self.parse_expression(0.0);
                        let open = self.next();
                        assert_eq!(
                            open,
                            Token::Op(Op::RoundBracketsClose),
                            "Expression Error: Bad token: \'{}\'. Expected \')\'.",
                            open
                        );
                        lhs
                    }
                    Op::SquareBracketsOpen => {
                        let mut args = vec![];
                        loop {
                            let next = self.peek();
                            dbg!(&next);
                            match next {
                                Token::Eof => panic!("Expected \']\' at end of file"),
                                Token::Op(Op::SquareBracketsClose) => { self.next(); break; },
                                Token::Sep(_) => { self.next(); continue; },
                                _ => args.push(self.parse_expression(0.0)),
                            }
                        }
                        dbg!(&args);
                        return Expression::Operation(Op::List, args);
                    }
                    t => panic!("[Expression Error] Unimplemented Op: {}", t),
                }
            }
            Token::Sep(_) => return Expression::None,
            t => panic!("[Expression Error] Bad token: {}", t),
        };
        loop {
            let op = match self.peek() {
                Token::Eof => break,
                Token::Op(Op::RoundBracketsClose) => break,
                Token::Op(Op::SquareBracketsClose) => break,
                Token::Op(Op::Colon) => break,
                Token::Op(op) => op,
                Token::Sep(_) => break,
                t => panic!(
                    "[Expression Error] Bad token: {:?}\nOnly 1 final atom after evaluation per line allowed!",
                    t
                ),
            };
            self.next();
            let (l_bp, r_bp) = Op::infix_binding_power(&op);
            if l_bp < min_bp {
                break;
            }
            let rhs = self.parse_expression(r_bp);
            lhs = Expression::Operation(op, vec![lhs, rhs])
        }
        lhs
    }
}

impl<'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::Ident(ident_a), Token::Ident(ident_b)) => *ident_a == *ident_b,
            (Token::Atom(word_a), Token::Atom(word_b)) => *word_a == *word_b,
            (Token::Op(op_a), Token::Op(op_b)) => *op_a == *op_b,
            (Token::Sep(sep_a), Token::Sep(sep_b)) => sep_a == sep_b,
            (Token::Eof, Token::Eof) => true,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        return !self.eq(other);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Keyword {
    If,
    Elif,
    Else,
    For,
    While,
    Def,
    True,
    False,
}

impl Keyword {}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Keyword::If => "if",
            Keyword::Elif => "elif",
            Keyword::Else => "else",
            Keyword::For => "for",
            Keyword::Def => "def",
            Keyword::While => "while",
            Keyword::True => "True",
            Keyword::False => "False",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expression {
    None,
    Ident(String),
    Atom(String),
    Operation(Op, Vec<Expression>),
    Func(FnPtr, Vec<Expression>),
    Keyword(Keyword, Vec<Expression>, Vec<Expression>),
    Definition(String, Vec<Expression>, String),
}

impl Default for Expression {
    fn default() -> Self {
        Expression::None
    }
}

impl Expression {
    pub fn get_value_string(&self) -> String {
        match self {
            Expression::Ident(ident) => ident.clone(),
            Expression::Atom(atom) => atom.clone(),
            _ => unimplemented!(),
        }
    }

    pub fn from_multiline(input: &str) -> Vec<Expression> {
        let lines: Vec<&str> = input.lines().collect();
        let mut exprs: Vec<Expression> = vec![];
        let mut block_stack: Vec<(usize, Expression, Vec<Expression>)> = vec![];
        
        for line in lines {
            let mut trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some((code, _comment)) = trimmed.split_once('#') {
                trimmed = code;
            }
            
            let indent = crate::pyrs_utils::get_indent(line);
            //println!("Indent: {indent} for line: {line}");
            
            // Close blocks if indentation decreased
            while !block_stack.is_empty() {
                let (block_indent, _, _) = block_stack.last().unwrap();
                
                if indent > *block_indent {
                    break;
                }

                let (_, mut keyword_expr, body) = block_stack.pop().unwrap();
                if let Expression::Keyword(kw, cond, _) = keyword_expr {
                    keyword_expr = Expression::Keyword(kw, cond, body);
                }
                
                if let Some((_, _, parent_body)) = block_stack.last_mut() {
                    parent_body.push(keyword_expr);
                } else {
                    exprs.push(keyword_expr);
                }
                
            }
            
            let expr = Expression::from_line(trimmed);
            
            // If line ends with ':', start a new block
            if trimmed.ends_with(":") {
                block_stack.push((indent, expr, vec![]));
            } else if let Some((_, _, body)) = block_stack.last_mut() {
                // Add to current block
                body.push(expr);
            } else {
                // Top-level expression
                exprs.push(expr);
            }

        }
        
        // Finalize remaining blocks
        while let Some((_, mut keyword_expr, body)) = block_stack.pop() {
            if let Expression::Keyword(kw, cond, _) = keyword_expr {
                keyword_expr = Expression::Keyword(kw, cond, body);
            }
            exprs.push(keyword_expr);
        }
        
        exprs
    }

    pub fn from_line(input: &str) -> Expression {
        let word_list = Utils::split_to_words(&input);
        let mut token_list = Lexer::from(&word_list);

        let expr = token_list.parse_expression(0f32);
        dbg!("Parsed expr: {:?}", &expr);
        expr
    }

    pub fn is_assign(&self) -> Option<(String, &Expression)> {
        match self {
            Expression::None => return None,
            Expression::Func(_, _) => return None,
            Expression::Atom(_) => return None,
            Expression::Ident(_) => return None,
            Expression::Keyword(_, _, _) => return None,
            Expression::Definition(_name, _args, _ret_type) => return None,
            Expression::Operation(c, operands) => {
                if *c == Op::Equals {
                    let var_name = match operands.first().unwrap() {
                        Expression::Atom(c) => c.to_string(),
                        Expression::Ident(ident) => ident.to_string(),
                        Expression::Keyword(kw, _cond, _args) => {
                            println!("Syntax Error: cannot assign to {}", kw);
                            return None;
                        }
                        _ => unreachable!(),
                    };
                    return Some((var_name, operands.last().unwrap()));
                }
                return None;
            }
        }
    }

    // turns expressions into objects
    pub fn eval(
        &self,
        variables: &mut HashMap<String, Arc<Obj>>,
        funcs: &mut HashMap<String, FnPtr>,
    ) -> Result<Arc<Obj>, PyException> {

        // println!("Eval: {self}");
        let ret: Arc<Obj> = match self {
            Expression::None => Obj::None.into(),
            Expression::Atom(c) => Obj::from_atom(c).into(),
            Expression::Ident(ident) => {
                let obj = match variables.get(ident) {
                    Some(var) => var.clone(),
                    None => {
                        return Err(PyException{
                            error: PyError::UndefinedVariableError,
                            msg: format!(
                                ": could not find the variable \"{ident}\" in the current scope"
                            ),
                        });
                    }
                };
                obj
            }
            Expression::Operation(operator, operands) => {
                
                // assign
                let first = operands.first().unwrap();
                if *operator == Op::Equals {
                    let value = operands.get(1).unwrap().eval(&mut *variables, &mut *funcs)?; 
                    let var_name = first.get_value_string();
                    variables.insert(var_name, value.clone());
                    return Ok(value);
                }
                else if *operator == Op::List {
                    let mut objs: Vec<Arc<Obj>> = vec![];
                    for o in operands {
                        let obj = o.eval(variables, funcs)?;
                        objs.push(Arc::from(obj));
                    }
                    return Ok(Obj::List(objs).into());
                }

                // unary
                let rhs = operands.get(1).unwrap().eval(&mut *variables, &mut *funcs)?;
                let lhs = first.eval(&mut *variables, &mut *funcs)?;
                match operator {
                    Op::Pos => return Obj::__pos__(&lhs),
                    Op::Neg => return Obj::__neg__(&lhs),
                    _ => {}
                };

                // binary
                let val: Arc<Obj> = match operator {
                    Op::Plus => Obj::__add__(&lhs, &rhs)?,
                    Op::Minus => Obj::__sub__(&lhs, &rhs)?,
                    Op::Asterisk => Obj::__mul__(&lhs, &rhs)?,
                    Op::ForwardSlash => Obj::__div__(&lhs, &rhs)?,
                    Op::Deref => Obj::__deref__(&lhs)?,
                    Op::Eq => Obj::__eq__(&lhs, &rhs).to_arc(),
                    Op::Neq => Obj::__ne__(&lhs, &rhs).to_arc(),
                    Op::LessThan => Obj::__lt__(&lhs, &rhs).to_arc(),
                    Op::GreaterThan => Obj::__gt__(&lhs, &rhs).to_arc(),
                    Op::LessEq => Obj::__le__(&lhs, &rhs).to_arc(),
                    Op::GreaterEq => Obj::__ge__(&lhs, &rhs).to_arc(),
                    Op::Equals => Obj::__default__().into(),
                    op => panic!("Bad operator: {}", op),
                };
                val
            }
            Expression::Keyword(keyword, conds, _args) => match keyword {
                Keyword::True => true.to_arc(),
                Keyword::False => false.to_arc(),
                Keyword::If | Keyword::While => {
                    let condition = conds
                        .iter()
                        .map(|x| 
                            x.eval(&mut *variables, &mut *funcs)
                            .unwrap()
                            .__bool__())
                        .all(|x| x);
                    condition.to_arc()
                }
                _ => panic!("Unimplemented Keyword: {:?}", keyword),
            }
            Expression::Func(func, vals) => {
                let mut args: Vec<Arc<Obj>> = vec![];
                for val in vals { 
                    args.push(val.eval(&mut *variables, &mut *funcs)?);
                }
                (func.ptr)(&args)
            }
            Expression::Definition(_name, _args, _ret_type) => {
                Obj::None.into()
            }
        };
        Ok(ret)
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::None => write!(f, "None"),
            Expression::Atom(i) => write!(f, "Atom({})", i),
            Expression::Ident(ident) => write!(f, "Ident({})", ident),
            Expression::Operation(head, rest) => {
                write!(f, "Op[{}", head)?;
                for s in rest {
                    write!(f, " {}", s)?
                }
                write!(f, "]")
            }
            Expression::Keyword(keyword, conds, args) => {
                write!(f, "Keyword[{} conds[", keyword)?;
                for c in conds {
                    write!(f, " {}", c)?;
                }
                write!(f, "] args[")?;
                for a in args {
                    write!(f, " {}", a)?;
                }
                write!(f, "]]")
            }
            Expression::Func(func, args) => {
                write!(f, "Func[{} args[", func)?;
                for a in args {
                    write!(f, " {}", a)?;
                }
                write!(f, "]]")
            }
            Expression::Definition(name, args, ret_type) => {
                write!(f, "def[ {} args[", name)?;
                for a in args {
                    write!(f, " {}", a)?;
                }
                write!(f, "] ret[{}]]", ret_type)
            }
        }
    }
}
