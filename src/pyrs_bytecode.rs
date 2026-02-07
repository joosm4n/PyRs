use crate::{
    pyrs_obj::{Obj, ToObj},
    pyrs_parsing::{Expression, Keyword, Op},
    pyrs_userclass::UserClassDef,
    pyrs_vm::IntrinsicFunc,
};

use std::{ 
    collections::HashMap,
    sync::Arc,
};

// Format: offset INSTRUCTION argument (value)
// 0 LOAD_CONST 0 (0)      # Load constant at index 0, which is the integer 0
// 2 STORE_NAME 0 (i)      # Store the top stack value into variable name at index 0 (variable "i")

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum PyBytecode {
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
    LoadFast(usize) = 101,
    StoreFast(usize) = 102,
    LoadName(String) = 103,
    StoreName(String) = 104,
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
    JumpIfFalse = 144,
    JumpAbsolute = 145,

    CompareOp(Op) = 160,

    UnpackSequence = 170,
    UnpackEx = 171,

    BuildList(usize) = 181,
    BuildTuple(usize) = 182,
    BuildSet(usize) = 183,
    BuildMap = 184,
    BuildString(usize) = 185,
    ListAppend = 186,

    ForIter(usize) = 191,
    GetIter = 192,

    NewStack = 201,
    DestroyStack = 202,

    // not proper
    Error(String) = 254,
}

impl PyBytecode {
    pub fn from_expr(expr: Expression, queue: &mut Vec<PyBytecode>) {
        // println!("Compiling: {}", expr.to_string());
        match expr {
            Expression::Ident(x) => {
                queue.push(PyBytecode::LoadName(x));
            }
            Expression::Atom(a) => queue.push(PyBytecode::LoadConst(a.to_obj())),
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
                                PyBytecode::from_expr(a, queue);
                            }
                        }
                        if name.is_empty() {
                            panic!();
                        }

                        queue.push(PyBytecode::StoreName(name));
                        return;
                    }
                    Op::AddEquals | Op::SubEquals | Op::MulEquals | Op::DivEquals => {
                        for (idx, a) in args.into_iter().enumerate() {
                            if idx == 0 {
                                match a {
                                    Expression::Ident(ident) => {
                                        name = ident;
                                        queue.push(PyBytecode::LoadName(name.clone()));
                                    }
                                    _ => panic!(),
                                };
                            } else if idx == 1 {
                                PyBytecode::from_expr(a, queue);
                            } else {
                                panic!("Only 2 args possible for add/sub/mul/div assign op");
                            }
                        }
                        if name.is_empty() {
                            panic!();
                        }

                        queue.push(match op {
                            Op::AddEquals => PyBytecode::BinaryAdd,
                            Op::SubEquals => PyBytecode::BinarySubtract,
                            Op::MulEquals => PyBytecode::BinaryMultiply,
                            Op::DivEquals => PyBytecode::BinaryDivide,
                            _ => unreachable!(),
                        });

                        queue.push(PyBytecode::StoreName(name));
                        return;
                    }
                    Op::List => {
                        let obj_count = args.len();
                        for a in args {
                            PyBytecode::from_expr(a, queue);
                        }
                        queue.push(PyBytecode::BuildList(obj_count));
                        return;
                    }
                    Op::Set => {
                        let obj_cound = args.len();
                        for a in args {
                            PyBytecode::from_expr(a, queue);
                        }
                        queue.push(PyBytecode::BuildSet(obj_cound));
                        return;
                    }
                    Op::Tuple => {
                        let obj_cound = args.len();
                        for a in args {
                            PyBytecode::from_expr(a, queue);
                        }
                        queue.push(PyBytecode::BuildTuple(obj_cound));
                        return;
                    }
                    Op::Dot => {
                        dbg!(&args);
                        let mut sides = args;
                        let lhs = sides.first().unwrap().get_value_string();
                        match sides.pop().unwrap() {
                            Expression::Call(name, args) => {
                                PyBytecode::from_expr(Expression::Call(format!("{}.{}", lhs, name), args), queue);
                            }
                            _ => panic!(),
                        };
                    }
                    _ => {
                        for a in args {
                            PyBytecode::from_expr(a, queue);
                        }
                    }
                }

                queue.push( match op {
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
                    queue.push(PyBytecode::PushNull);
                }

                for a in args {
                    //dbg!(&a);
                    PyBytecode::from_expr(a, queue);
                }

                if let Some(intrinsic) = intrinsic_option {
                    queue.push(PyBytecode::CallInstrinsic1(intrinsic));
                } else {
                    queue.push(PyBytecode::LoadConst(name.as_str().to_obj()));
                    queue.push(PyBytecode::CallFunction(argc));
                    // todo: create tuple that is argc sized??
                }
            }
            Expression::Keyword(keyword, mut args, body) => {
                match keyword {
                    Keyword::True => queue.push(PyBytecode::LoadConst(Obj::Bool(true))),
                    Keyword::False => queue.push(PyBytecode::LoadConst(Obj::Bool(false))),
                    Keyword::Elif | Keyword::Else => {
                        panic!("Shouldn't have a stand alone elif/else expression")
                    }
                    Keyword::If => {
                        // Evaluate the if condition first
                        for c in args {
                            PyBytecode::from_expr(c, queue);
                        }

                        let parts = Expression::split_if_elif_else(body);

                        // Generate the main if body
                        let mut if_body = vec![];
                        let mut elif_else_parts = vec![];

                        for part in parts {
                            match part {
                                Expression::Keyword(Keyword::Elif, conds, body) => {
                                    elif_else_parts.push((conds, body));
                                }
                                Expression::Keyword(Keyword::Else, _, body) => {
                                    elif_else_parts.push((vec![], body)); // Empty condition for else
                                }
                                other => {
                                    PyBytecode::from_expr(other, &mut if_body);
                                }
                            }
                        }

                        if elif_else_parts.is_empty() {
                            // Simple if statement
                            queue.push(PyBytecode::PopJumpIfFalse(if_body.len()));
                            queue.append(&mut if_body);
                        } else {
                            // Complex if-elif-else
                            // For now, let's implement a simpler approach that works correctly
                            // even if not optimally efficient

                            // Generate all the elif/else bytecode first to know sizes
                            let mut all_elif_else_code = vec![];

                            for (conds, body_exprs) in elif_else_parts {
                                let mut block_code = vec![];

                                if !conds.is_empty() {
                                    // elif block
                                    for cond in conds {
                                        PyBytecode::from_expr(cond, &mut block_code);
                                    }

                                    let mut body_code = vec![];
                                    for expr in body_exprs {
                                        PyBytecode::from_expr(expr, &mut body_code);
                                    }

                                    block_code
                                        .push(PyBytecode::PopJumpIfFalse(body_code.len() + 1));
                                    block_code.append(&mut body_code);
                                    block_code.push(PyBytecode::JumpForward(0));
                                // Placeholder, will fix later
                                } else {
                                    // else block - no condition
                                    for expr in body_exprs {
                                        PyBytecode::from_expr(expr, &mut block_code);
                                    }
                                }

                                all_elif_else_code.append(&mut block_code);
                            }

                            // Fix the JumpForward placeholders
                            let mut jump_fixups = vec![];
                            for (i, instr) in all_elif_else_code.iter().enumerate() {
                                if matches!(instr, PyBytecode::JumpForward(0)) {
                                    let remaining = all_elif_else_code.len() - i - 1;
                                    jump_fixups.push((i, remaining));
                                }
                            }

                            for (idx, distance) in jump_fixups {
                                all_elif_else_code[idx] = PyBytecode::JumpForward(distance);
                            }

                            // Now emit the main if
                            //let skip_distance = if_body.len() + 1 + all_elif_else_code.len();
                            queue.push(PyBytecode::PopJumpIfFalse(if_body.len() + 1));
                            queue.append(&mut if_body);
                            queue.push(PyBytecode::JumpForward(all_elif_else_code.len()));
                            queue.append(&mut all_elif_else_code);
                        }
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
                    Keyword::For => {
                        let for_err =
                            "only for loops of form \'for Ident() in Ident()\' currently supported";
                        assert_eq!(args.len(), 2);

                        match args.pop().unwrap() {
                            Expression::Ident(ident) => {
                                queue.push(PyBytecode::LoadName(ident.clone()))
                            }
                            c if matches!(c, Expression::Call(_, _)) => {
                                PyBytecode::from_expr(c, queue)
                            }
                            e => panic!("{} found {}", for_err, e),
                        };

                        let x = match args.first().unwrap() {
                            Expression::Ident(ident) => ident,
                            e => panic!("{} found {}", for_err, e),
                        };

                        queue.push(PyBytecode::GetIter);

                        let mut for_code = vec![];
                        for b in body {
                            PyBytecode::from_expr(b, &mut for_code);
                        }
                        let contents_len = for_code.len(); // length of for loops contents

                        queue.push(PyBytecode::ForIter(contents_len + 2));
                        queue.push(PyBytecode::StoreName(x.into()));

                        queue.append(&mut for_code);
                        queue.push(PyBytecode::JumpBackward(contents_len + 3));
                    }
                    Keyword::Def => {
                        let func_args = args.split_off(1);
                        // dbg!(&func_args);

                        let name = match args.pop() {
                            Some(Expression::Ident(ident)) => ident,
                            Some(e) => {
                                panic!("Syntax Error: function name must be an identifier, not {e}")
                            }
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
                                Expression::Ident(ident) => {
                                    body_code.push(PyBytecode::StoreName(ident))
                                }
                                Expression::Operation(Op::Equals, vals) => {
                                    let name = vals.first().unwrap().clone();
                                    PyBytecode::from_expr(
                                        Expression::Operation(Op::Equals, vals),
                                        &mut body_code,
                                    );
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
                    }
                    Keyword::Class => {
                        //println!("\nClass");

                        //dbg!(&args);
                        let name = match args.first().unwrap() {
                            Expression::Ident(ident) => ident.clone(),
                            e => panic!("class name must be an identifier not: {:?}", e),
                        };

                        //dbg!(&body);
                        let mut fields: HashMap<String, (usize, Obj)> = HashMap::new();
                        let mut methods = UserClassDef::default_methods();
                        for (idx, f) in body.into_iter().enumerate() {
                            match f {
                                Expression::Operation(Op::Equals, mut v) => {
                                    let default_val = v.pop().unwrap();
                                    fields.insert(
                                        v[0].get_value_string(),
                                        (idx, default_val.to_obj()),
                                    );
                                }
                                Expression::Keyword(Keyword::Def, conds, body) => {
                                    let mut func = vec![];
                                    let fn_name = conds.first().unwrap().get_value_string();
                                    PyBytecode::from_expr(
                                        Expression::Keyword(Keyword::Def, conds, body),
                                        &mut func,
                                    );
                                    methods.insert(fn_name, func);
                                }
                                _ => panic!("invalid expr for default"),
                            }
                        }

                        let class = UserClassDef {
                            name: name,
                            fields: fields,
                            methods: methods,
                        };

                        queue.push(PyBytecode::LoadConst(Obj::ClassDef(Arc::new(class))));
                        queue.push(PyBytecode::LoadBuildClass);

                        //panic!("testing class");
                    }
                    Keyword::Return => {
                        for a in args {
                            PyBytecode::from_expr(a, queue);
                        }
                        queue.push(PyBytecode::ReturnValue);
                    }
                    Keyword::None => {
                        queue.push(PyBytecode::LoadConst(Obj::None));
                    }
                    Keyword::Pass => {
                        queue.push(PyBytecode::NOP);
                    }
                    k => panic!("Unknown keyword: {k}"),
                }
            }
            Expression::None => {} //e => panic!("(Expr) {:?} to bytecode not implemented", e),
        }
    }

    pub fn from_str(s: &str) -> Vec<PyBytecode> {
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

        let code = Interpreter::compile_file(&temp_file);

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
