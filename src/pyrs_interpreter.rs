
use std::collections::HashMap;
use std::fs::{File};
use std::io::{self, BufRead, BufReader, Write};

use crate::pyrs_obj::{Obj, PyException, PyObj};
use crate::pyrs_parsing::{Expression, Keyword};
use crate::pyrs_std::{FnPtr, Funcs};
use crate::pyrs_utils::{get_indent};

/*
*/
macro_rules! dbg {
    ($($tt:tt)*) => {};
}

pub struct Interpreter
{
    variables: HashMap<String, Obj>,
    funcs: HashMap<String, FnPtr>,
    running: bool,
    curr_line: isize,

    curr_indent: usize,

    // Stack-based approach for nested blocks
    block_stack: Vec<BlockContext>,
    //cache: Expression,

    last_line: String,
    show_output: bool,
}

#[derive(Debug)]
struct BlockContext {
    indent_level: usize,
    keyword_expr: Expression,  // The if/elif/else/for/while expression
    body: Vec<Expression>,      // Expressions in this block
}

pub enum InterpreterCommand<'a> {
    Error(&'static str),
    Live,
    PyFile(&'a str),
    AnyFile(&'a str),
    FromString(&'a str),
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
            running: true,
            funcs: Funcs::get_std_map(),
            curr_line: -1,
            curr_indent: 0,
            //cache: Expression::None,
            block_stack: Vec::new(),
            last_line: String::new(),
            show_output: false,
        }
    }

    fn eval_expr(&mut self, expr: &Expression) -> Result<Obj, PyException> {
        expr.eval(&mut self.variables, &mut self.funcs)
    }

    fn push_to_current_block(&mut self, expr: Expression) {
        if let Some(context) = self.block_stack.last_mut() {
            context.body.push(expr);
        } else {
            // No block context, execute immediately
            self.process_expr(&expr);
        }
    }

    fn start_block(&mut self, indent_level: usize, keyword_expr: Expression) {
        self.block_stack.push(BlockContext {
            indent_level,
            keyword_expr,
            body: Vec::new(),
        });
    }

    fn finalize_blocks_until(&mut self, target_indent: usize) {
        while let Some(context) = self.block_stack.last() {
            if context.indent_level <= target_indent {
                break;
            }
            
            let context = self.block_stack.pop().unwrap();
            let complete_expr = match context.keyword_expr {
                Expression::Keyword(kw, conds, _empty) => {
                    Expression::Keyword(kw, conds, context.body)
                }
                _ => panic!("Expected keyword expression"),
            };
            
            // Either add to parent block or execute
            if let Some(parent) = self.block_stack.last_mut() {
                parent.body.push(complete_expr);
            } else {
                self.process_expr(&complete_expr);
            }
        }
    }

    pub fn parse_args<'a>(argv: &'a Vec<String>) -> InterpreterCommand<'a> {
        let arg_err = "Invalid args. \nEg: cargo run -- test.py \n or: cargo run -- -a test.x";

        if argv.len() == 1 {
            return InterpreterCommand::Live;
        } else if argv.len() == 2 {
            let arg1 = argv.get(1).unwrap();
            if arg1.ends_with(".py") {
                return InterpreterCommand::PyFile(&arg1);
            } else {
                return InterpreterCommand::Error(arg_err);
            }
        } else if argv.len() == 3 {
            let arg1 = argv.get(1).unwrap();
            let arg2 = argv.get(2).unwrap();
            if arg1 == "-a" {
                return InterpreterCommand::AnyFile(&arg2);
            } else if arg1 == "-s" {
                return InterpreterCommand::FromString(&arg2);
            } else {
                return InterpreterCommand::Error(arg_err);
            }
        } else {
            return InterpreterCommand::Error(arg_err);
        }
    }

    pub fn interpret_line(&mut self, line: &str) {

        self.curr_line += 1;
        dbg!(self.curr_line);

        let line_indent = get_indent(line);
        dbg!(line_indent);

        dbg!(&self.block_stack);

        if let Some(top) = self.block_stack.last() {
            if line_indent < top.indent_level {
                self.finalize_blocks_until(line_indent);
            }
        }

        match line.trim() {
            "exit" => {
                println!("Use exit() or Ctrl-Z plus Return to exit");
                return;
            }
            "exit()" | "^Z" => {
                self.running = false;
                return;
            }
            "" => return,
            _ => (),
        }

        let expr = Expression::from_line(&line);
        if line.trim().ends_with(":") {
            if let Expression::Keyword(_, _, _) = expr {
                self.start_block(line_indent + 4, expr);
            } 
            else {
                panic!("Only keywords can start blocks");
            }
        } 
        else {
            if self.block_stack.is_empty() {
                self.process_expr(&expr);
            } 
            else {
                self.push_to_current_block(expr);
            }
        }

    }

    fn process_expr(&mut self, expr: &Expression) 
    {
        match expr {
            Expression::Keyword(keyword, _conds , args) => {
                match keyword {
                    Keyword::If => {
                        match self.eval_expr(expr) {
                            Ok(cond) => { 
                                if cond.__bool__() {
                                    for a in args {
                                        self.process_expr(a);
                                    }
                                }
                            }
                            Err(e) => e.print(),
                        }
                    }
                    Keyword::While => {
                        loop {
                            match self.eval_expr(expr) {
                                Ok(cond) => { 
                                    if !cond.__bool__() { break; }
                                    for a in args {
                                        self.process_expr(a);
                                    }
                                }
                                Err(e) => {
                                    e.print();
                                    break;
                                }
                            }
                        }
                    }
                    _ => unimplemented!()
                }
            }
            _ => {}
        }

        if let Some((var_name, lhs)) = expr.is_assign() {
            let value = lhs.eval(&mut self.variables, &mut self.funcs);
            match value {
                Ok(val) => { self.variables.insert(var_name.to_string(), val); }
                Err(e) => { e.print(); }
            }
            return;
        }

        let res = self.eval_expr(expr);
        match res {
            Ok(obj) => {
                if self.show_output && obj != Obj::None {
                    println!("{}", obj)
                }
            }
            Err(e) => { e.print(); }
        }

    }

    pub fn live_interpret(&mut self) {
        self.show_output = true;
        loop {
            if self.curr_indent > 0 {
                print!("... ");
            }
            else {
                print!(">>> ");
            }
            io::stdout().flush().unwrap();
            let input = {
                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf).unwrap();
                buf
            };

            self.interpret_line(&input);
            self.last_line = input.to_string();
            if !self.running {
                break;
            }
        }
    }

    pub fn interpret_file(&mut self, filepath: &str) {
        let file = match File::open(filepath) {
            Ok(f) => f,
            Err(e) => panic!("[Fileread Error] {}", e),
        };
        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => break,
            };

            self.interpret_line(&line);
            self.last_line = line.to_string();

            if !self.running {
                return;
            }
        }

        self.finalize_blocks_until(0);
    }
}
