
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
    pyrs_interpreter::{Interpreter, InterpreterCommand, InterpreterFlags},
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
    let commands = Interpreter::parse_args(&argv);
    for (i, cmd) in commands.into_iter().enumerate() {
        match cmd {
            InterpreterCommand::Live => interp.live_interpret(),
            InterpreterCommand::File(filepath, flags) => { 
                if flags.contains(&InterpreterFlags::Debug) {
                    interp.set_debug_mode(true);
                }

                let is_py_file = filepath.ends_with(".py");
                if !flags.contains(&InterpreterFlags::AnyFile) && !is_py_file {
                    println!("To use and file type use the \'-a\' flag before the file");
                    return Ok(());
                }

                if flags.contains(&InterpreterFlags::Compile) {
                    let bytecode = Interpreter::compile_file(&filepath);
                    Interpreter::seralize_bytecode(&filepath, &bytecode)?;
                } 
                else {
                    interp.interpret_file(&filepath);
                }
            }
            InterpreterCommand::FromString(words) => interp.interpret_line(&words),
            InterpreterCommand::Error(msg) => println!("Error on command {i}: {msg}"),
            InterpreterCommand::PrintHelp => Interpreter::print_help(),
        }
    }
    Ok(())
}
