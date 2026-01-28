
use crate::{
    pyrs_obj::{Obj, ToObj}, pyrs_parsing::{Expression, Keyword, Op}, pyrs_vm::{IntrinsicFunc}
};

    // Format: offset INSTRUCTION argument (value)
    // 0 LOAD_CONST 0 (0)      # Load constant at index 0, which is the integer 0
    // 2 STORE_NAME 0 (i)      # Store the top stack value into variable name at index 0 (variable "i")

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum PyBytecode
{
    // Empty
    NOP = 0,
    
    // Fundamentals
    PopTop = 20,
    EndFor = 21,
    Copy(usize) = 22,
    Swap(usize) = 23,

    // Unary
    UnaryNegative = 40,
    UnaryNot = 41,
    UnaryInvert = 42,
    ToBool = 43,

    // Binary
    BinaryOp(Op) = 80,
    BinaryAdd = 81, 
    BinaryMultiply = 82, 
    BinarySubtract = 83, 
    BinaryDivide = 84,
    BinaryXOR = 85,

    LoadConst(Obj) = 100,
    LoadFast(String) = 101, 
    StoreFast(String) = 102,
    LoadName(String) = 103,
    StoreName(String) = 104,
    LoadGlobal = 105, 
    StoreGlobal = 106,
    
    CallFunction(usize /* argc */) = 120, 
    CallInstrinsic1(IntrinsicFunc) = 121, 
    CallInstrinsic2(IntrinsicFunc) = 122,
    ReturnValue = 123,
    MakeFunction = 124,
    
    PopJumpIfFalse(usize) = 140,
    PopJumpIfTrue(usize) = 141,
    JumpForward(usize) = 142, 
    JumpBackward(usize) = 143,
    JumpIfFalse = 144, 
    JumpAbsolute = 145,

    CompareOp(Op) = 160,

    BuildList(usize) = 181,
    BuildTuple(usize) = 182, 
    BuildMap = 183,
    ListAppend = 184,
    ForIter = 185,
    GetIter = 186,

    NewStack = 201,
    DestroyStack = 202,

    // not proper
    Error(String) = 254,
}

impl PyBytecode
{
    pub fn from_expr(expr: Expression, queue: &mut Vec<PyBytecode>)
    {
        match expr {
            Expression::Ident(x) => {
                queue.push(PyBytecode::LoadName(x));
            }
            Expression::Atom(a) => {
                queue.push(PyBytecode::LoadConst(a.to_obj()))
            }
            Expression::Operation(op, args) => {
                
                let mut name = String::new();
                if op == Op::Equals {
                    for (idx, a) in args.into_iter().enumerate() {
                        if idx == 0 {
                            match a {
                                Expression::Ident(ident ) => name = ident,
                                _ => panic!(),
                            };
                        }
                        else {
                            PyBytecode::from_expr(a, queue);
                        }
                    }
                    if name.is_empty() {
                        panic!();
                    }
                    queue.push(PyBytecode::StoreName(name));
                    return;
                }
                else if op == Op::List {
                    let obj_count = args.len();
                    for a in args {
                        PyBytecode::from_expr(a, queue);
                    }
                    queue.push(PyBytecode::BuildList(obj_count));
                    return;
                }
                else {
                    for a in args {
                        PyBytecode::from_expr(a, queue);
                    }
                }
                queue.push(match op {
                    Op::Plus => PyBytecode::BinaryAdd,
                    Op::Minus => PyBytecode::BinarySubtract,
                    Op::Asterisk => PyBytecode::BinaryMultiply,
                    Op::ForwardSlash => PyBytecode::BinaryDivide,
                    
                    Op::Eq | Op::Neq |
                    Op::LessEq | Op::LessThan | 
                    Op::GreaterEq | Op::GreaterThan => PyBytecode::CompareOp(op),
                    
                    e => PyBytecode::Error(format!("{e}")),
                });
            }
            Expression::Call(name, args) => {

                let argc = args.len();
                // dbg!(&args);
                queue.push(PyBytecode::LoadConst(Obj::None));
                for a in args {
                    //dbg!(&a);
                    PyBytecode::from_expr(a, queue);
                }
                
                if let Some(intrinsic) = IntrinsicFunc::try_get(&name) {
                    queue.push(PyBytecode::CallInstrinsic1(intrinsic));
                }
                else {
                    queue.push(PyBytecode::LoadConst(name.as_str().to_obj()));
                    // create tuple that is argc sized
                    // push default args into it (Obj::None?)
                    // then replace these with used args
                    queue.push(PyBytecode::CallFunction(argc));
                }
            }
            Expression::Func(ptr, args) => {

                let fn_name = ptr.name.as_str().to_string();
                queue.push(PyBytecode::LoadConst(Obj::None));
                
                //dbg!(&fn_name);
                if let Some(intrinsic) = IntrinsicFunc::try_get(fn_name.as_str()) {
                    for a in args {
                        PyBytecode::from_expr(a, queue);
                    }
                    queue.push(PyBytecode::CallInstrinsic1(intrinsic))
                }
                else {
                    panic!();
                }
            }
            Expression::Keyword(keyword, mut args, body) => {
                match keyword {
                    Keyword::If => {
                        for c in args {
                            PyBytecode::from_expr(c, queue);
                        }
                        let mut temp_queue: Vec<PyBytecode> = vec![];
                        for a in body {
                            PyBytecode::from_expr(a, &mut temp_queue);
                        }
                        let delta = temp_queue.len();
                        queue.append(&mut temp_queue);
                        queue.push(PyBytecode::PopJumpIfFalse(delta))
                    }
                    Keyword::While => {
                        let condition_start = queue.len();
                        let mut condition_code = vec![];
                        for c in args {
                            PyBytecode::from_expr(c, &mut condition_code);
                        }
                        for inst in condition_code.iter() {
                            queue.push(inst.clone());
                        }
                        
                        let mut contents_code: Vec<PyBytecode> = vec![];
                        for a in body {
                            PyBytecode::from_expr(a, &mut contents_code);
                        }

                        let delta = contents_code.len() + 1;
                        queue.push(Self::PopJumpIfFalse(delta)); // skip entire while loop
                        
                        queue.append(&mut contents_code);

                        let return_delta = queue.len() - condition_start + 1;
                        queue.push(PyBytecode::JumpBackward(return_delta));
                        
                        queue.push(PyBytecode::LoadConst(Obj::None));
                    }
                    Keyword::Def => {

                        let func_args = args.split_off(1);
                        // dbg!(&func_args);

                        let name = match args.pop() {
                            Some(Expression::Ident(ident)) => ident,
                            Some(e) => panic!("Syntax Error: function name must be an identifier, not {e}"),
                            None => panic!(),
                        };

                        let func_addr = queue.len() + 3;
                        let mut body_code = vec![];

                        // define function and location
                        queue.push(PyBytecode::LoadConst(name.to_obj()));
                        queue.push(PyBytecode::LoadConst(func_addr.to_obj()));
                        queue.push(PyBytecode::MakeFunction);

                        for a in func_args {
                            match a {
                                Expression::Ident(ident) => body_code.push(PyBytecode::StoreName(ident)),
                                Expression::Operation(Op::Equals, vals) => {
                                    let name = vals.first().unwrap().clone();
                                    PyBytecode::from_expr(Expression::Operation(Op::Equals, vals), &mut body_code);
                                    body_code.push(PyBytecode::LoadName(name.get_value_string()));
                                }
                                _ => panic!(),
                            }
                        }

                        for b in body {
                            PyBytecode::from_expr(b, &mut body_code);
                        }
                        
                        body_code.push(PyBytecode::ReturnValue);

                        //dbg!(&body_code);
                        queue.push(PyBytecode::JumpForward(body_code.len()));
                        queue.append(&mut body_code);

                        //panic!();
                        /* def

                        - the function is just an address to the bytecode
                        - args are placed in the stack as (values??) 
                        - a temp stack for local variables is made
                        - args are stored to the variables
                        - do bytecode

                        fn generate_fn()
                        {
                            push inst define function + name
                            args[ return_ptr, ]
                            store_fast x argc
                            *
                                function body 
                            *
                            place return on stack
                            return to previous code
                        }

                        INSTS:
                        skip func
                        init local stack
                        pop args into stack
                        do bytecode
                        return value/tuple

                        */
                    }
                    _ => panic!(),
                }
            }
            Expression::None => {},
            //e => panic!("(Expr) {:?} to bytecode not implemented", e),
        }
    }

    pub fn from_str(s: &str) -> Vec<PyBytecode>
    {
        use crate::pyrs_interpreter::Interpreter;
        
        use std::fs;
        use std::io::Write;
        
        let temp_file = "__temp_bytecode__.py";
        let mut file = fs::File::create(temp_file)
            .expect("Failed to create temp file");
        file.write_all(s.as_bytes())
            .expect("Failed to write to temp file");
        
        let code = Interpreter::compile_file(temp_file);
        
        // Clean up
        fs::remove_file(temp_file)
            .expect("Failed to delete temp file");
        code

    }

    pub fn to_string(vec: &Vec<Self>) -> String {
        let mut string = String::new();
        for (idx, line) in vec.iter().enumerate() {
            string.push_str(format!("({idx}) \t\t{:?}\n", line).as_str());
        }
        string
    }

}

impl std::convert::From<PyBytecode> for u8
{
    fn from(bytecode: PyBytecode) -> u8 {
        unsafe { *(&bytecode as *const PyBytecode as *const u8)}
    }
}

impl std::fmt::Display for PyBytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
