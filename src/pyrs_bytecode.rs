use crate::{
    pyrs_obj::{Obj, ToObj},
    pyrs_parsing::{Expression, Keyword, Op},
    pyrs_userclass::{CustomClass},
    pyrs_codeobject::{CodeObj, CompileCtx},
    pyrs_vm::IntrinsicFunc,
};

use std::{collections::HashMap, sync::Arc,};

// Format: offset INSTRUCTION argument (value)
// 0 LOAD_CONST 0 (0)      # Load constant at index 0, which is the integer 0
// 2 STORE_NAME 0 (i)      # Store the top stack value into variable name at index 0 (variable "i")

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum PyBytecode {
    // Empty
    NOP = 0,

    // Import
    ImportName(usize) = 10,
    ImportFrom(usize) = 11,

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

    LoadConst(usize) = 100,
    LoadFast(usize) = 101,
    StoreFast(usize) = 102,
    LoadName(usize) = 103,
    StoreName(usize) = 104,
    LoadGlobal = 105,
    StoreGlobal = 106,
    PushNull = 107,

    Cache = 110,

    CallFunction(usize /* argc */) = 120,
    CallInstrinsic1(IntrinsicFunc) = 121,
    CallInstrinsic2(IntrinsicFunc) = 122,
    ReturnValue = 123,
    MakeFunction = 124,

    LoadBuildClass = 130,

    PopJumpIfFalse(usize) = 140,
    PopJumpIfTrue(usize) = 141,
    JumpForward(usize) = 142,
    JumpBackward(usize) = 143,

    CompareOp(Op) = 160,

    UnpackSequence = 170,
    UnpackEx = 171,
    LoadDeref(usize) = 172,

    BuildList(usize) = 181,
    BuildTuple(usize) = 182,
    BuildSet(usize) = 183,
    BuildMap = 184,
    BuildString(usize) = 185,
    ListAppend = 186,

    ForIter(usize) = 191,
    GetIter = 192,

    // not proper
    Error(String) = 254,
}

impl PyBytecode {

    fn compile_fn(body: Expression) -> Arc<CodeObj> {

        match body {
            Expression::Keyword(Keyword::Def, mut args, body) => {
                let _func_args = args.split_off(1);

                let name = match args.pop() {
                    Some(Expression::Ident(ident)) => ident,
                    _ => panic!("function name must be identifier"),
                };

                // Compile function body into its OWN bytecode
                let mut fn_ctx = CompileCtx::new(name.clone());

                for b in body {
                    PyBytecode::from_expr(b, &mut fn_ctx);
                }

                let const_num = fn_ctx.add_const(Obj::None);
                fn_ctx.push(PyBytecode::LoadConst(const_num));
                fn_ctx.push(PyBytecode::ReturnValue);

                Arc::new(fn_ctx.finish())
            }
            _ => unreachable!(),
        }

    }

    pub fn from_expr(expr: Expression, context: &mut CompileCtx) {
        // println!("Compiling: {}", expr.to_string());
        match expr {
            Expression::Ident(x) => {
                let namei = context.add_name(x);
                context.push(PyBytecode::LoadName(namei));
            }
            Expression::Atom(a) => { 
                let i = context.add_const(a.to_obj());
                context.push(PyBytecode::LoadConst(i));
            }
            Expression::Operation(op, args) => {
                let mut name = String::new();
                match op {
                    Op::Equals => {
                        for (idx, a) in args.into_iter().enumerate() {
                            if idx == 0 {
                                match a {
                                    Expression::Ident(ident) => name = ident,
                                    _ => panic!(),
                                };
                            } else {
                                PyBytecode::from_expr(a, context);
                            }
                        }
                        if name.is_empty() {
                            panic!();
                        }

                        let namei = context.add_name(name);
                        context.push(PyBytecode::StoreName(namei));
                        return;
                    }
                    Op::AddEquals | Op::SubEquals | Op::MulEquals | Op::DivEquals => {
                        for (idx, a) in args.into_iter().enumerate() {
                            if idx == 0 {
                                match a {
                                    Expression::Ident(ident) => {
                                        name = ident;
                                        let namei = context.add_name(name.clone());
                                        context.push(PyBytecode::LoadName(namei));
                                    }
                                    _ => panic!(),
                                };
                            } else if idx == 1 {
                                PyBytecode::from_expr(a, context);
                            } else {
                                panic!("Only 2 args possible for add/sub/mul/div assign op");
                            }
                        }

                        if name.is_empty() {
                            panic!("SyntaxError: name is empty\n{} ", context.serialize(0));
                        }

                        context.push(match op {
                            Op::AddEquals => PyBytecode::BinaryAdd,
                            Op::SubEquals => PyBytecode::BinarySubtract,
                            Op::MulEquals => PyBytecode::BinaryMultiply,
                            Op::DivEquals => PyBytecode::BinaryDivide,
                            _ => unreachable!(),
                        });

                        let namei = context.add_name(name);
                        context.push(PyBytecode::StoreName(namei));
                        return;
                    }
                    Op::List => {
                        let obj_count = args.len();
                        for a in args {
                            PyBytecode::from_expr(a, context);
                        }
                        context.push(PyBytecode::BuildList(obj_count));
                        return;
                    }
                    Op::Set => {
                        let obj_cound = args.len();
                        for a in args {
                            PyBytecode::from_expr(a, context);
                        }
                        context.push(PyBytecode::BuildSet(obj_cound));
                        return;
                    }
                    Op::Tuple => {
                        let obj_cound = args.len();
                        for a in args {
                            PyBytecode::from_expr(a, context);
                        }
                        context.push(PyBytecode::BuildTuple(obj_cound));
                        return;
                    }
                    Op::Dot => {
                        let mut lhs = String::new();
                        let mut rhs = String::new();
                        let mut body = Expression::None;
                        for (idx, a) in args.into_iter().enumerate() {
                            match idx {
                                0 => lhs = a.get_value_string(),
                                1 => {
                                    rhs = match &a {
                                        Expression::Call(name, _args) => name.clone(),
                                        _ => panic!(),
                                    };
                                    body = a;
                                }
                                _ => panic!(),
                            }
                        }

                        let namei = context.add_name(lhs.into());
                        context.push(PyBytecode::LoadName(namei));

                        let namei = context.add_name(rhs.into());
                        context.push(PyBytecode::LoadDeref(namei));

                        PyBytecode::from_expr(body, context);
                        return;
                    }
                    _ => {
                        for a in args {
                            PyBytecode::from_expr(a, context);
                        }
                    }
                }

                context.push( match op {
                    Op::Plus => PyBytecode::BinaryAdd,
                    Op::Minus => PyBytecode::BinarySubtract,
                    Op::Asterisk => PyBytecode::BinaryMultiply,
                    Op::ForwardSlash => PyBytecode::BinaryDivide,

                    Op::Eq
                    | Op::Neq
                    | Op::LessEq
                    | Op::LessThan
                    | Op::GreaterEq
                    | Op::GreaterThan => PyBytecode::CompareOp(op),

                    Op::Neg => PyBytecode::UnaryNegative,
                    Op::Unpack => PyBytecode::UnpackSequence,

                    e => {
                        println!("Op {e} to PyBytecode not implemented! Pushed Error to instructions instead");
                        PyBytecode::Error(format!("{e}"))
                    },
                });
            }
            Expression::Call(name, args) => {
                let argc = args.len();
                // dbg!(&args);

                let intrinsic_option = IntrinsicFunc::try_get(&name);
                if intrinsic_option.is_some() {
                    context.push(PyBytecode::PushNull);
                }

                for a in args {
                    //dbg!(&a);
                    PyBytecode::from_expr(a, context);
                }

                if let Some(intrinsic) = intrinsic_option {
                    context.push(PyBytecode::CallInstrinsic1(intrinsic));
                } else {
                    let namei = context.add_name(name);
                    context.push(PyBytecode::LoadName(namei));
                    context.push(PyBytecode::CallFunction(argc));
                }
            }
            Expression::Keyword(keyword, mut args, body) => {
                match keyword {
                    Keyword::True => { 
                        let i = context.add_const(Obj::Bool(true));
                        context.push(PyBytecode::LoadConst(i));
                    }
                    Keyword::False => {
                        let i = context.add_const(Obj::Bool(false));
                        context.push(PyBytecode::LoadConst(i));
                    }
                    Keyword::Elif | Keyword::Else => {
                        panic!("Shouldn't have a stand alone elif/else expression")
                    }
                    Keyword::If => {


                        // Evaluate the if condition first
                        for c in args {
                            PyBytecode::from_expr(c, context);
                        }

                        let parts = Expression::split_if_elif_else(body);

                        let mut elif_else_parts = vec![];
                        let jump_spot = context.len();
                        context.push(PyBytecode::PopJumpIfFalse(0)); // placeholder

                        for part in parts {
                            match part {
                                Expression::Keyword(Keyword::Elif, conds, body) => {
                                    elif_else_parts.push((conds, body));
                                }
                                Expression::Keyword(Keyword::Else, _, body) => {
                                    elif_else_parts.push((vec![], body)); // Empty condition for else
                                }
                                other => {
                                    PyBytecode::from_expr(other, context);
                                }
                            }
                        }

                        if elif_else_parts.is_empty() {
                            let if_body_len = context.len() - jump_spot;
                            context[jump_spot] = PyBytecode::PopJumpIfFalse(if_body_len);
                        } 


                        else {
                            let start_elif_else_spot = context.len();
                            let mut place_holders: Vec<(usize, usize)> = vec![]; // (part_len, jump_to_end_pos)

                            for (conds, body_exprs) in elif_else_parts {

                                if !conds.is_empty() {

                                    let start_cond = context.len();
                                    for cond in conds {
                                        PyBytecode::from_expr(cond, context);
                                    }
                                    let jump_spot = context.len();
                                    context.push(PyBytecode::PopJumpIfFalse(0)); // placeholder to skip body

                                    for expr in body_exprs {
                                        PyBytecode::from_expr(expr, context);
                                    }

                                    let body_code_len = context.len() - jump_spot;
                                    context[jump_spot] = PyBytecode::PopJumpIfFalse(body_code_len + 1);

                                    place_holders.push((start_cond, context.len()));
                                    context.push(PyBytecode::JumpForward(0)); // placeholder to jump to end
                                }
                                else {
                                    for expr in body_exprs {
                                        PyBytecode::from_expr(expr, context);
                                    }
                                }
                            }
                            let end_spot = context.len();
                            let mut dist_to_end = end_spot - start_elif_else_spot;

                            for (part_len, jump_to_end_spot) in place_holders {
                                dist_to_end -= part_len;
                                context[jump_to_end_spot] = PyBytecode::JumpForward(dist_to_end);
                            }
                        }
                    }
                    Keyword::While => {
                        let condition_start = context.len();
                        for c in args {
                            PyBytecode::from_expr(c, context);
                        }
                        let jump_spot = context.len();
                        context.push(PyBytecode::PopJumpIfFalse(0)); // place holder

                        for a in body {
                            PyBytecode::from_expr(a, context);
                        }
                        let delta = (context.len() - jump_spot) + 1;
                        context[jump_spot] = PyBytecode::PopJumpIfFalse(delta); // skip entire while loop

                        let return_delta = context.len() - condition_start + 1;
                        context.push(PyBytecode::JumpBackward(return_delta));

                        let i = context.add_const(Obj::None);
                        context.push(PyBytecode::LoadConst(i));
                    }
                    Keyword::For => {
                        let for_err =
                            "only for loops of form \'for Ident() in Ident()\' currently supported";
                        assert_eq!(args.len(), 2);

                        match args.pop().unwrap() {
                            Expression::Ident(ident) => {
                                let namei = context.add_name(ident.clone());
                                context.push(PyBytecode::LoadName(namei))
                            }
                            c if matches!(c, Expression::Call(_, _)) => {
                                PyBytecode::from_expr(c, context)
                            }
                            e => panic!("{} found {}", for_err, e),
                        };

                        let x = match args.first().unwrap() {
                            Expression::Ident(ident) => ident,
                            e => panic!("{} found {}", for_err, e),
                        };

                        context.push(PyBytecode::GetIter);

                        let start_for_code_spot = context.len();
                        for b in body {
                            PyBytecode::from_expr(b, context);
                        }
                        let contents_len = context.len() - start_for_code_spot; // length of for loops contents

                        context.push(PyBytecode::ForIter(contents_len + 2));

                        let namei = context.add_name(x.into());
                        context.push(PyBytecode::StoreName(namei));
                        context.push(PyBytecode::JumpBackward(contents_len + 3));
                    }
                    Keyword::Def => {
                        let fn_code = PyBytecode::compile_fn(Expression::Keyword(Keyword::Def, args, body));
                        let name = fn_code.name.clone();
                        let idx = context.add_const(Obj::Code(fn_code));

                        // Emit instructions for *creating* the function
                        context.push(PyBytecode::LoadConst(idx));
                        context.push(PyBytecode::MakeFunction);
                        let namei = context.add_name(name);
                        context.push(PyBytecode::StoreName(namei));
                    }
                    Keyword::Class => {
                        //println!("\nClass");

                        //dbg!(&args);
                        let name = match args.first().unwrap() {
                            Expression::Ident(ident) => ident.clone(),
                            e => panic!("class name must be an identifier not: {:?}", e),
                        };

                        //dbg!(&body);
                        let mut fields: HashMap<String, Arc<Obj>> = HashMap::new();
                        for field in body.into_iter() {
                            match field {
                                Expression::Operation(Op::Equals, mut v) => {
                                    let default_val = v.pop().unwrap();
                                    fields.insert(v[0].get_value_string(), default_val.to_arc());
                                }
                                Expression::Keyword(Keyword::Def, conds, body) => {
                                    let fn_code = PyBytecode::compile_fn(Expression::Keyword(Keyword::Def, conds, body));
                                    let name = fn_code.name.clone();
                                    let idx = context.add_const(Obj::Code(fn_code));

                                    context.push(PyBytecode::LoadConst(idx));
                                    context.push(PyBytecode::MakeFunction);
                                    let namei = context.add_name(name);
                                    context.push(PyBytecode::StoreName(namei));
                                }
                                _ => panic!("invalid expr for default"),
                            }
                        }

                        let class = CustomClass {
                            name: name.clone(),
                            fields: fields,
                        };

                        let i = context.add_const(Obj::CustomClass(class));
                        context.push(PyBytecode::LoadConst(i));
                        let namei = context.add_name(name);
                        context.push(PyBytecode::StoreName(namei));

                        //panic!("testing class");
                    }
                    Keyword::Import => {
                        let name = args.first().unwrap().get_value_string();
                        let namei = context.add_name(name);
                        context.push(PyBytecode::ImportName(namei));
                    }
                    Keyword::Return => {
                        for a in args {
                            PyBytecode::from_expr(a, context);
                        }
                        context.push(PyBytecode::ReturnValue);
                    }
                    Keyword::None => {
                        let i = context.add_const(Obj::None);
                        context.push(PyBytecode::LoadConst(i));
                    }
                    Keyword::Pass => {
                        context.push(PyBytecode::NOP);
                    }
                    k => panic!("Unknown keyword: {k}"),
                }
            }
            Expression::None => {} //e => panic!("(Expr) {:?} to bytecode not implemented", e),
        }
    }

    pub fn from_str(s: &str) -> CodeObj {

        use crate::pyrs_interpreter::Interpreter;
        use std::fs;
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};

        let thread_id = std::thread::current().id();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temp_file = format!("__temp_bytecode_{:?}_{}__.py", thread_id, timestamp);
        //println!("temp_file: {temp_file}");

        let mut file = fs::File::create(&temp_file).expect("Failed to create temp file");
        file.write_all(s.as_bytes())
            .expect("Failed to write to temp file");

        let code = match Interpreter::compile_file(&temp_file) {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };

        // Clean up
        fs::remove_file(temp_file).expect("Failed to delete temp file");

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

impl std::convert::From<PyBytecode> for u8 {
    fn from(bytecode: PyBytecode) -> u8 {
        unsafe { *(&bytecode as *const PyBytecode as *const u8) }
    }
}

impl std::fmt::Display for PyBytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}