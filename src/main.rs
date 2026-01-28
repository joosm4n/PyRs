
pub mod pyrs_obj;
pub mod pyrs_parsing;
pub mod pyrs_std;
pub mod pyrs_error;
pub mod pyrs_userclass;
pub mod pyrs_utils;
pub mod pyrs_interpreter;
pub mod pyrs_bytecode;
pub mod pyrs_vm;
mod pyrs_tests; 

#[allow(unused_imports)]
use crate::{
    pyrs_interpreter::{Interpreter, InterpreterCommand},
    pyrs_obj::{Obj},
    pyrs_error::{PyException}, 
    pyrs_parsing::{Expression, Token, Op},
    pyrs_std::{FnPtr, Funcs},
    pyrs_bytecode::{PyBytecode},
    pyrs_vm::{PyVM, IntrinsicFunc},
};

fn main() -> std::io::Result<()> {

    let args = std::env::args();
    let mut argv: Vec<String> = vec![];
    for a in args{
        argv.push(a);
    }
    
    let mut interp = Interpreter::new();
    let cmd = Interpreter::parse_args(&argv);
    match cmd {
        InterpreterCommand::Live => interp.live_interpret(),
        InterpreterCommand::AnyFile(file) => interp.interpret_file(file),
        InterpreterCommand::PyFile(py) => interp.interpret_file(py),
        InterpreterCommand::FromString(words) => interp.interpret_line(words),
        InterpreterCommand::Error(msg) => println!("{}", msg),
        InterpreterCommand::CompileFile(filepath) => { 
            let bytecode = Interpreter::compile_file(filepath);
            Interpreter::seralize_bytecode(filepath, &bytecode)?;
        }
    }
    Ok(())
}
