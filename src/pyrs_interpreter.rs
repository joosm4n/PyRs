use std::{
    collections::HashMap,
    io::{self, Write},
    sync::Arc,
};

use crate::{
    pyrs_bytecode::PyBytecode,
    pyrs_error::PyException,
    pyrs_obj::{Obj, PyObj},
    pyrs_parsing::{Expression, Keyword},
    pyrs_std::{FnPtr, Funcs},
    pyrs_utils::get_indent,
    pyrs_vm::PyVM,
};

pub struct Interpreter {
    variables: HashMap<String, Arc<Obj>>,
    funcs: HashMap<String, FnPtr>,
    running: bool,
    curr_line: isize,

    curr_indent: usize,

    // Stack-based approach for nested blocks
    block_stack: Vec<BlockContext>,
    //cache: Expression,
    last_line: String,
    debug_mode: bool,
    repr: bool,

    vm: PyVM,
}

#[derive(Debug)]
struct BlockContext {
    indent_level: usize,
    keyword_expr: Expression, // The if/elif/else/for/while expression
    body: Vec<Expression>,    // Expressions in this block
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]

pub enum InterpreterFlags {
    Debug,
    AnyFile,
    Compile,
}

pub enum InterpreterCommand {
    Error(&'static str),
    Live,
    File(String, Vec<InterpreterFlags>),
    FromString(String),
    PrintHelp,
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
            debug_mode: false,
            repr: false,
            vm: PyVM::new(),
        }
    }

    pub fn set_debug_mode(&mut self, debug: bool) {
        self.debug_mode = debug;
        self.vm.set_debug_mode(debug);
    }

    pub fn get_version() -> &'static str {
        "pyrs-0-1"
    }

    pub fn print_help() {
        let help = r#"
        Usage:  PyRs <flags> <filename>
        
        or:     cargo run -- <flags> <filename>

        <flags>:
            -h, --help 
                Print help (this message)
            -a, --all
                Allow any file type to be interpreted, default only .py files
            -c, --compile
                Compiles the file
            -d, --debug
                Runs in debug mode, this means it will print various things inc stack traces or parsed exprs

        "#;
        println!("{help}");
    }

    fn eval_expr(&mut self, expr: &Expression) -> Result<Arc<Obj>, PyException> {
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

    fn interpret_blocks_until(&mut self, target_indent: usize) {
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

    pub fn parse_args(argv: &Vec<String>) -> Vec<InterpreterCommand> {
        let arg_err = "Invalid args. \nEg: cargo run -- test.py \n or: cargo run -- -a test.x";

        let mut commands = vec![];
        let mut flags = vec![];

        if argv.len() == 1 {
            return vec![InterpreterCommand::Live];
        } else {
            for (i, arg) in argv.iter().enumerate() {
                if i == 0 {
                    continue;
                }
                match arg.as_str() {
                    "-a" | "--all" => flags.push(InterpreterFlags::AnyFile),
                    "-d" | "--debug" => flags.push(InterpreterFlags::Debug),
                    "-c" | "--compile" => flags.push(InterpreterFlags::Compile),
                    "-h" | "--help" => commands.push(InterpreterCommand::PrintHelp),
                    a if a.contains('.') => {
                        let mut file_flags = vec![];
                        file_flags.append(&mut flags);
                        commands.push(InterpreterCommand::File(arg.to_string(), file_flags));
                        flags = vec![];
                    }
                    _ => return vec![InterpreterCommand::Error(arg_err)],
                };
            }
        }
        commands
    }

    pub fn interpret_line(&mut self, line_in: &str) {
        let mut line = line_in;
        self.curr_line += 1;
        let line_indent = get_indent(line);

        if let Some(top) = self.block_stack.last() {
            if line_indent < top.indent_level {
                self.interpret_blocks_until(line_indent);
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
        if let Some((line_before, _comment)) = line.split_once('#') {
            line = line_before;
        }

        let expr = Expression::from_line(&line);
        if line.trim().ends_with(":") {
            if let Expression::Keyword(_, _, _) = expr {
                self.start_block(line_indent + 4, expr);
            } else {
                panic!("Only keywords can start blocks");
            }
        } else {
            if self.block_stack.is_empty() {
                self.process_expr(&expr); // keyword args are in
            } else {
                self.push_to_current_block(expr);
            }
        }
    }

    fn process_expr(&mut self, expr: &Expression) {
        match expr {
            Expression::Keyword(keyword, _conds, args) => match keyword {
                Keyword::If => match self.eval_expr(&expr) {
                    Ok(cond) => {
                        if cond.__bool__() {
                            for a in args {
                                self.process_expr(&a);
                            }
                        }
                    }
                    Err(e) => e.print(),
                },
                Keyword::While => loop {
                    match self.eval_expr(&expr) {
                        Ok(cond) => {
                            if !cond.__bool__() {
                                break;
                            }
                            for a in args {
                                self.process_expr(&a);
                            }
                        }
                        Err(e) => {
                            e.print();
                            break;
                        }
                    }
                },
                _ => unimplemented!(),
            },
            _ => {}
        }

        if let Some((var_name, lhs)) = expr.is_assign() {
            let value = lhs.eval(&mut self.variables, &mut self.funcs);
            match value {
                Ok(val) => {
                    self.variables.insert(var_name.to_string(), val);
                }
                Err(e) => {
                    e.print();
                }
            }
            return;
        }

        let res = self.eval_expr(&expr);
        match res {
            Ok(obj) => {
                if self.repr && obj.as_ref() != &Obj::None {
                    println!("{}", obj.__repr__())
                }
            }
            Err(e) => {
                e.print();
            }
        }
    }

    pub fn live_interpret(&mut self) {
        self.repr = true;
        loop {
            if self.curr_indent > 0 {
                print!("... ");
            } else {
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
        let bytecode = Interpreter::compile_file(filepath);
        self.vm.execute(bytecode);
    }

    // vvvv using byte code vvvv
    pub fn compile_file(filepath: &str) -> Vec<PyBytecode> {
        let mut bytecode: Vec<PyBytecode> = vec![];
        let contents = match std::fs::read_to_string(filepath) {
            Ok(f) => f,
            Err(e) => panic!("Fileread error: {e}"),
        };
        let parsed = Expression::from_multiline(contents.as_str());
        //dbg!(&parsed);
        for expr in parsed {
            PyBytecode::from_expr(expr, &mut bytecode);
        }

        bytecode
    }

    #[allow(dead_code)]
    fn execute_expr(&mut self, expr: Expression) {
        let mut bytecode = vec![];
        PyBytecode::from_expr(expr, &mut bytecode);
        self.vm.execute(bytecode);
    }

    pub fn seralize_bytecode(filename: &str, bytecode: &Vec<PyBytecode>) -> std::io::Result<()> {
        use std::fs;
        let exists = fs::exists("__pycache__")?;
        if !exists {
            std::fs::create_dir("__pycache__")?;
        }

        println!("Compiling \'{}\'... ", filename);
        let name = filename.strip_suffix(".py").unwrap();
        let pyc_name = format!("__pycache__/{}.{}.pyc", name, Interpreter::get_version());
        let mut file = fs::File::create(&pyc_name)?;

        let contents = PyBytecode::to_string(bytecode);
        file.write_all(contents.as_bytes())?;

        println!("Compiled: {filename} into {pyc_name}");
        Ok(())
    }
}
