use pyrs::*;

#[allow(unused_imports)]
use crate::{
    pyrs_bytecode::PyBytecode,
    pyrs_error::{PyException, PyPanicHandle},
    pyrs_interpreter::{Interpreter, InterpreterCommand, InterpreterFlags},
    pyrs_std::{FnPtr, Funcs},
    pyrs_vm::{IntrinsicFunc, PyVM},
};

fn main() -> std::io::Result<()> {
    let mut interp = Interpreter::new();
    let commands = Interpreter::parse_args();
    for (i, cmd) in commands.into_iter().enumerate() {
        match cmd {
            InterpreterCommand::Live => interp.live_interpret(),
            InterpreterCommand::File(filepath, flags) => {
                if flags.contains(&InterpreterFlags::Debug) {
                    interp.set_debug_mode(true);
                }
                if flags.contains(&InterpreterFlags::StepMode) {
                    interp.set_step_mode(true);
                }

                let is_py_file = filepath.ends_with(".py");
                if !flags.contains(&InterpreterFlags::AnyFile) && !is_py_file {
                    println!("To use and file type use the \'-a\' flag before the file");
                    return Ok(());
                }

                if flags.contains(&InterpreterFlags::Compile) {
                    let code_obj = Interpreter::compile_file(&filepath).handle_panic();
                    Interpreter::seralize_codeobj(&filepath, &code_obj)?;
                } else {
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

mod _test {

    use pyrs_macros::Builder;

    #[derive(Builder)]
    struct PyrsStruct {
        f1: String,
        f2: u32,
    }

    #[test]
    fn pyrs_macros_test() {
        let my_struct = PyrsStruct::builder().f1("Hello".to_string()).f2(42).build();
    }
}
