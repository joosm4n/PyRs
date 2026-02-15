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

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum PyBytecode {
    // Empty
    NOP,

    // Import
    ImportName(usize),
    ImportFrom(usize),

    // Fundamentals
    PopIter,
    PopTop,
    EndFor,
    Copy(usize),
    Swap(usize),

    // Unary
    UnaryNegative,
    UnaryNot,
    UnaryInvert,
    ToBool,

    // Binary
    BinaryOp(Op),
    BinaryAdd,
    BinaryMultiply,
    BinarySubtract,
    BinaryDivide,
    BinaryXOR,

    LoadConst(usize),
    LoadFast(usize),
    StoreFast(usize),
    LoadName(usize),
    StoreName(usize),
    LoadGlobal,
    StoreGlobal,
    PushNull,

    Cache,

    CallFunction(usize /* argc */),
    CallInstrinsic1(IntrinsicFunc),
    CallInstrinsic2(IntrinsicFunc),
    ReturnValue,
    MakeFunction,

    LoadBuildClass,

    PopJumpIfFalse(usize),
    PopJumpIfTrue(usize),
    JumpForward(usize),
    JumpBackward(usize),

    CompareOp(Op),

    UnpackSequence,
    UnpackEx,
    LoadDeref(usize),

    BuildList(usize),
    BuildTuple(usize),
    BuildSet(usize),
    BuildMap,
    BuildString(usize),
    ListAppend,

    ForIter(usize),
    GetIter,

    // not proper
    Error,
}

impl PyBytecode {

    fn compile_fn(body: Expression) -> Arc<CodeObj> {

        match body {
            Expression::Keyword(Keyword::Def, mut args, body) => {
                let func_args = args.split_off(1);

                let name = match args.pop() {
                    Some(Expression::Ident(ident)) => ident,
                    _ => panic!("function name must be identifier"),
                };

                // Compile function body into its OWN bytecode
                let mut fn_ctx = CompileCtx::new(&name);

                for a in func_args {
                    match a {
                        Expression::Ident(name) => {
                            fn_ctx.add_name(name);
                        }
                        _ => panic!(),
                    }
                    // PyBytecode::from_expr(a, &mut fn_ctx);
                }
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
                context.push(PyBytecode::LoadFast(namei));
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
                        context.push(PyBytecode::StoreFast(namei));
                        return;
                    }
                    Op::AddEquals | Op::SubEquals | Op::MulEquals | Op::DivEquals => {
                        for (idx, a) in args.into_iter().enumerate() {
                            if idx == 0 {
                                match a {
                                    Expression::Ident(ident) => {
                                        name = ident;
                                        let namei = context.add_name(name.clone());
                                        context.push(PyBytecode::LoadFast(namei));
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
                        context.push(PyBytecode::StoreFast(namei));
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
                        panic!();
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
                        /*
                        for c in args {
                            PyBytecode::from_expr(c, context);
                        }
                        */

                        let parts = Expression::split_if_elif_else(args, body);

                        let mut elif_else_parts = vec![];
                        let mut done_if = false;
                        for part in parts {
                            match part {
                                Expression::Keyword(Keyword::If, conds, body_) => {
                                    elif_else_parts.push((conds, body_));
                                    done_if = true;
                                }
                                Expression::Keyword(Keyword::Elif, conds, body_) => {
                                    assert!(done_if);
                                    elif_else_parts.push((conds, body_));
                                }
                                Expression::Keyword(Keyword::Else, _, body_) => {
                                    assert!(done_if);
                                    elif_else_parts.push((vec![], body_)); // Empty condition for else
                                    break;
                                }
                                _ => panic!(),
                            }
                        }

                        let start_elif_else_spot = context.len();
                        let mut place_holders: Vec<(usize, usize)> = vec![]; // (part_len, pos)
                        //dbg!(&elif_else_parts);

                        let mut has_else = false;
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
                                context[jump_spot] = PyBytecode::PopJumpIfFalse(body_code_len);

                                place_holders.push((context.len() - start_cond, context.len()));
                                context.push(PyBytecode::JumpForward(0)); // placeholder to jump to end
                            }
                            else {
                                has_else = true;
                                for expr in body_exprs {
                                    PyBytecode::from_expr(expr, context);
                                }
                                break;
                            }
                        }

                        let end_spot = context.len();
                        let mut dist_to_end = (end_spot - start_elif_else_spot - 2) as i64;

                        for (part_len, jump_to_end_spot) in place_holders {
                            dist_to_end -= part_len as i64;
                            if dist_to_end < 0 {
                                dist_to_end = 0
                            }
                            //println!("jump_spot: {}, jump_dist: {}", jump_to_end_spot, dist_to_end);
                            context[jump_to_end_spot] = PyBytecode::JumpForward(dist_to_end as usize);
                        }

                        if !has_else {
                            if let Some(last) = context.last_mut() {
                                *last = PyBytecode::JumpForward(0);
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
                        let iter_spot = context.len();
                        context.push(PyBytecode::ForIter(0));  // placeholder
                        
                        let x_namei = context.add_name(x.into());
                        context.push(PyBytecode::StoreFast(x_namei));

                        let start_for_code_spot = context.len();
                        for b in body {
                            PyBytecode::from_expr(b, context);
                        }
                        let contents_len = context.len() - start_for_code_spot; // length of for loops contents
                        context[iter_spot] = PyBytecode::ForIter(contents_len + 2);     // insert right val          
                        context.push(PyBytecode::JumpBackward(contents_len + 3));
                    }
                    Keyword::Def => {
                        let fn_code = PyBytecode::compile_fn(Expression::Keyword(Keyword::Def, args, body));
                        let name = fn_code.name.clone();
                        let idx = context.add_const(Obj::Code(fn_code));

                        // Emit instructions for *creating* the function
                        context.push(PyBytecode::LoadConst(idx));
                        context.push(PyBytecode::MakeFunction);

                        //dbg!(&name);
                        let namei = context.add_name(name);
                        //dbg!(&namei);
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

    pub const TYPE_STR_LEN: usize = 16;

    pub const fn get_byte_map() -> [[u8; PyBytecode::TYPE_STR_LEN]; 255]
    {
        let mut bytecode_map = [[b'_';  PyBytecode::TYPE_STR_LEN]; 255];
        let mut i = 0;
        while i < 255 {
            let index = i as usize;
            bytecode_map[index] = *PyBytecode::from_bytes(&[i, 0]).get_type_str_slice();
            i += 1;
        }
        bytecode_map
    }

    pub const fn get_type_str_slice(&self) -> &[u8; PyBytecode::TYPE_STR_LEN] {
        match self {
            PyBytecode::NOP =>                  &[b'N',b'O',b'P',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,
            PyBytecode::ImportName(_) =>        &[b'I',b'm',b'p',b'o',b'r',b't',b'N',b'a',b'm',b'e',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::ImportFrom(_) =>        &[b'I',b'm',b'p',b'o',b'r',b't',b'F',b'r',b'o',b'm',b'_',b'_',b'_',b'_',b'_',b'_'] , 
            PyBytecode::PopIter =>              &[b'P',b'o',b'p',b'I',b't',b'e',b'r',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,    
            PyBytecode::PopTop =>               &[b'P',b'o',b'p',b'T',b'o',b'p',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,    
            PyBytecode::EndFor =>               &[b'E',b'n',b'd',b'F',b'o',b'r',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,    
            PyBytecode::Copy(_) =>              &[b'C',b'o',b'p',b'y',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::Swap(_) =>              &[b'S',b'w',b'a',b'p',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::UnaryNegative =>        &[b'U',b'n',b'a',b'r',b'y',b'N',b'e',b'g',b'a',b't',b'i',b'v',b'e',b'_',b'_',b'_'] , 
            PyBytecode::UnaryNot =>             &[b'U',b'n',b'a',b'r',b'y',b'N',b'o',b't',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::UnaryInvert =>          &[b'U',b'n',b'a',b'r',b'y',b'I',b'n',b'v',b'e',b'r',b't',b'_',b'_',b'_',b'_',b'_'] , 
            PyBytecode::ToBool =>               &[b'T',b'o',b'B',b'o',b'o',b'l',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,    
            PyBytecode::BinaryOp(_) =>          &[b'B',b'i',b'n',b'a',b'r',b'y',b'O',b'p',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::BinaryAdd =>            &[b'B',b'i',b'n',b'a',b'r',b'y',b'A',b'd',b'd',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::BinaryMultiply =>       &[b'B',b'i',b'n',b'a',b'r',b'y',b'M',b'u',b'l',b't',b'i',b'p',b'l',b'y',b'_',b'_'] , 
            PyBytecode::BinarySubtract =>       &[b'B',b'i',b'n',b'a',b'r',b'y',b'S',b'u',b'b',b't',b'r',b'a',b'c',b't',b'_',b'_'] ,  
            PyBytecode::BinaryDivide =>         &[b'B',b'i',b'n',b'a',b'r',b'y',b'D',b'i',b'v',b'i',b'd',b'e',b'_',b'_',b'_',b'_'] , 
            PyBytecode::BinaryXOR =>            &[b'B',b'i',b'n',b'a',b'r',b'y',b'X',b'O',b'R',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,    
            PyBytecode::LoadConst(_) =>         &[b'L',b'o',b'a',b'd',b'C',b'o',b'n',b's',b't',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::LoadFast(_) =>          &[b'L',b'o',b'a',b'd',b'F',b'a',b's',b't',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::StoreFast(_) =>         &[b'S',b't',b'o',b'r',b'e',b'F',b'a',b's',b't',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::LoadName(_) =>          &[b'L',b'o',b'a',b'd',b'N',b'a',b'm',b'e',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::StoreName(_) =>         &[b'S',b't',b'o',b'r',b'e',b'N',b'a',b'm',b'e',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::LoadGlobal =>           &[b'L',b'o',b'a',b'd',b'G',b'l',b'o',b'b',b'a',b'l',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::StoreGlobal =>          &[b'S',b't',b'o',b'r',b'e',b'G',b'l',b'o',b'b',b'a',b'l',b'_',b'_',b'_',b'_',b'_'] , 
            PyBytecode::PushNull =>             &[b'P',b'u',b's',b'h',b'N',b'u',b'l',b'l',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::Cache =>                &[b'C',b'a',b'c',b'h',b'e',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::CallFunction(_) =>      &[b'C',b'a',b'l',b'l',b'F',b'u',b'n',b'c',b't',b'i',b'o',b'n',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::CallInstrinsic1(_) =>   &[b'C',b'a',b'l',b'l',b'I',b'n',b's',b't',b'r',b'i',b'n',b's',b'i',b'c',b'1',b'_'] , 
            PyBytecode::CallInstrinsic2(_) =>   &[b'C',b'a',b'l',b'l',b'I',b'n',b's',b't',b'r',b'i',b'n',b's',b'i',b'c',b'2',b'_'] ,  
            PyBytecode::ReturnValue =>          &[b'R',b'e',b't',b'u',b'r',b'n',b'V',b'a',b'l',b'u',b'e',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::MakeFunction =>         &[b'M',b'a',b'k',b'e',b'F',b'u',b'n',b'c',b't',b'i',b'o',b'n',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::LoadBuildClass =>       &[b'L',b'o',b'a',b'd',b'B',b'u',b'i',b'l',b'd',b'C',b'l',b'a',b's',b's',b'_',b'_'] , 
            PyBytecode::PopJumpIfFalse(_) =>    &[b'P',b'o',b'p',b'J',b'u',b'm',b'p',b'I',b'f',b'F',b'a',b'l',b's',b'e',b'_',b'_'] ,  
            PyBytecode::PopJumpIfTrue(_) =>     &[b'P',b'o',b'p',b'J',b'u',b'm',b'p',b'I',b'f',b'T',b'r',b'u',b'e',b'_',b'_',b'_'] , 
            PyBytecode::JumpForward(_) =>       &[b'J',b'u',b'm',b'p',b'F',b'o',b'r',b'w',b'a',b'r',b'd',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::JumpBackward(_) =>      &[b'J',b'u',b'm',b'p',b'B',b'a',b'c',b'k',b'w',b'a',b'r',b'd',b'_',b'_',b'_',b'_'] , 
            PyBytecode::CompareOp(_) =>         &[b'C',b'o',b'm',b'p',b'a',b'r',b'e',b'O',b'p',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::UnpackSequence =>       &[b'U',b'n',b'p',b'a',b'c',b'k',b'S',b'e',b'q',b'u',b'e',b'n',b'c',b'e',b'_',b'_'] , 
            PyBytecode::UnpackEx =>             &[b'U',b'n',b'p',b'a',b'c',b'k',b'E',b'x',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::LoadDeref(_) =>         &[b'L',b'o',b'a',b'd',b'D',b'e',b'r',b'e',b'f',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::BuildList(_) =>         &[b'B',b'u',b'i',b'l',b'd',b'L',b'i',b's',b't',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::BuildTuple(_) =>        &[b'B',b'u',b'i',b'l',b'd',b'T',b'u',b'p',b'l',b'e',b'_',b'_',b'_',b'_',b'_',b'_'] , 
            PyBytecode::BuildSet(_) =>          &[b'B',b'u',b'i',b'l',b'd',b'S',b'e',b't',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::BuildMap =>             &[b'B',b'u',b'i',b'l',b'd',b'M',b'a',b'p',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::BuildString(_) =>       &[b'B',b'u',b'i',b'l',b'd',b'S',b't',b'r',b'i',b'n',b'g',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::ListAppend =>           &[b'L',b'i',b's',b't',b'A',b'p',b'p',b'e',b'n',b'd',b'_',b'_',b'_',b'_',b'_',b'_'] , 
            PyBytecode::ForIter(_) =>           &[b'F',b'o',b'r',b'I',b't',b'e',b'r',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::GetIter =>              &[b'G',b'e',b't',b'I',b't',b'e',b'r',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] ,  
            PyBytecode::Error =>                &[b'E',b'r',b'r',b'o',b'r',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_',b'_'] , 
        }
    }

    pub const fn get_type_str(&self) -> &str {
        let s = self.get_type_str_slice();
        unsafe { str::from_utf8_unchecked(s) }
    }

    pub const fn from_bytes(bytes: &[u8; 2]) -> Self {
        let enum_: u8 = bytes[0];
        let data: usize = bytes[1] as usize;
        match enum_ {
            0   => PyBytecode::NOP,
            1   => PyBytecode::ImportName(data),
            2   => PyBytecode::ImportFrom(data),
            3   => PyBytecode::PopIter,
            4   => PyBytecode::PopTop,
            5   => PyBytecode::EndFor,
            6   => PyBytecode::Copy(data),
            7   => PyBytecode::Swap(data),
            8   => PyBytecode::UnaryNegative,
            9   => PyBytecode::UnaryNot,
            10  => PyBytecode::UnaryInvert,
            11  => PyBytecode::ToBool,
            12  => PyBytecode::BinaryOp(Op::from_usize(data)),
            13  => PyBytecode::BinaryAdd,
            14  => PyBytecode::BinaryMultiply,
            15  => PyBytecode::BinarySubtract,
            16  => PyBytecode::BinaryDivide,
            17  => PyBytecode::BinaryXOR,
            18  => PyBytecode::LoadConst(data),
            19  => PyBytecode::LoadFast(data),
            20  => PyBytecode::StoreFast(data),
            21  => PyBytecode::LoadName(data),
            22  => PyBytecode::StoreName(data),
            23  => PyBytecode::LoadGlobal,
            24  => PyBytecode::StoreGlobal,
            25  => PyBytecode::PushNull,
            26  => PyBytecode::Cache,
            27  => PyBytecode::CallFunction(data),
            28  => PyBytecode::CallInstrinsic1(IntrinsicFunc::from_usize(data)),
            29  => PyBytecode::CallInstrinsic2(IntrinsicFunc::from_usize(data)),
            30  => PyBytecode::ReturnValue,
            31  => PyBytecode::MakeFunction,
            32  => PyBytecode::LoadBuildClass,
            33  => PyBytecode::PopJumpIfFalse(data),
            34  => PyBytecode::PopJumpIfTrue(data),
            35  => PyBytecode::JumpForward(data),
            36  => PyBytecode::JumpBackward(data),
            37  => PyBytecode::CompareOp(Op::from_usize(data)),
            38  => PyBytecode::UnpackSequence,
            39  => PyBytecode::UnpackEx,
            40  => PyBytecode::LoadDeref(data),
            41  => PyBytecode::BuildList(data),
            42  => PyBytecode::BuildTuple(data),
            43  => PyBytecode::BuildSet(data),
            44  => PyBytecode::BuildMap,
            45  => PyBytecode::BuildString(data),
            46  => PyBytecode::ListAppend,
            47  => PyBytecode::ForIter(data),
            48  => PyBytecode::GetIter,
            255 => PyBytecode::Error,

            _ => PyBytecode::Error,
        }
    }

    pub const fn to_bytes(&self) -> [u8; 2]
    {
        match self {
            PyBytecode::NOP =>                                      [0, 0],     
            PyBytecode::ImportName(v) =>                    [1, *v as u8],     
            PyBytecode::ImportFrom(v) =>                    [2, *v as u8],    
            PyBytecode::PopIter =>                                  [3, 0],     
            PyBytecode::PopTop =>                                   [4, 0],     
            PyBytecode::EndFor =>                                   [5, 0],     
            PyBytecode::Copy(v) =>                          [6, *v as u8],     
            PyBytecode::Swap(v) =>                          [7, *v as u8],     
            PyBytecode::UnaryNegative =>                            [8, 0],     
            PyBytecode::UnaryNot =>                                 [9, 0],     
            PyBytecode::UnaryInvert =>                              [10, 0],     
            PyBytecode::ToBool =>                                   [11, 0],    
            PyBytecode::BinaryOp(v) =>                         [12, *v as u8],     
            PyBytecode::BinaryAdd =>                                [13, 0],     
            PyBytecode::BinaryMultiply =>                           [14, 0],     
            PyBytecode::BinarySubtract =>                           [15, 0],     
            PyBytecode::BinaryDivide =>                             [16, 0],    
            PyBytecode::BinaryXOR =>                                [17, 0],     
            PyBytecode::LoadConst(v) =>                     [18, *v as u8],     
            PyBytecode::LoadFast(v) =>                      [19, *v as u8],     
            PyBytecode::StoreFast(v) =>                     [20, *v as u8],     
            PyBytecode::LoadName(v) =>                      [21, *v as u8],     
            PyBytecode::StoreName(v) =>                     [22, *v as u8],     
            PyBytecode::LoadGlobal =>                               [23, 0],     
            PyBytecode::StoreGlobal =>                              [24, 0],     
            PyBytecode::PushNull =>                                 [25, 0],
            PyBytecode::Cache =>                                    [26, 0],
            PyBytecode::CallFunction(v) =>                  [27, *v as u8],
            PyBytecode::CallInstrinsic1(v) =>       [28, *v as u8],
            PyBytecode::CallInstrinsic2(v) =>       [29, *v as u8],
            PyBytecode::ReturnValue =>                              [30, 0],
            PyBytecode::MakeFunction =>                             [31, 0],
            PyBytecode::LoadBuildClass =>                           [32, 0],
            PyBytecode::PopJumpIfFalse(v) =>                [33, *v as u8],
            PyBytecode::PopJumpIfTrue(v) =>                 [34, *v as u8],
            PyBytecode::JumpForward(v) =>                   [35, *v as u8],
            PyBytecode::JumpBackward(v) =>                  [36, *v as u8],
            PyBytecode::CompareOp(v) =>                        [37, *v as u8],
            PyBytecode::UnpackSequence =>                           [38, 0],
            PyBytecode::UnpackEx =>                                 [39, 0],
            PyBytecode::LoadDeref(v) =>                     [40, *v as u8],
            PyBytecode::BuildList(v) =>                     [41, *v as u8],
            PyBytecode::BuildTuple(v) =>                    [42, *v as u8],
            PyBytecode::BuildSet(v) =>                      [43, *v as u8],
            PyBytecode::BuildMap =>                                 [44, 0],
            PyBytecode::BuildString(v) =>                   [45, *v as u8],
            PyBytecode::ListAppend =>                               [46, 0],
            PyBytecode::ForIter(v) =>                       [47, *v as u8],
            PyBytecode::GetIter =>                                  [48, 0],
            PyBytecode::Error =>                                    [254, 0],
        }
    }

}

impl std::convert::From<PyBytecode> for u8 {
    fn from(value: PyBytecode) -> u8 {
        match value {
            PyBytecode::NOP =>                  0,     
            PyBytecode::ImportName(_) =>        1,     
            PyBytecode::ImportFrom(_) =>        2,    
            PyBytecode::PopIter =>              3,     
            PyBytecode::PopTop =>               4,     
            PyBytecode::EndFor =>               5,     
            PyBytecode::Copy(_) =>              6,     
            PyBytecode::Swap(_) =>              7,     
            PyBytecode::UnaryNegative =>        8,     
            PyBytecode::UnaryNot =>             9,     
            PyBytecode::UnaryInvert =>          10,     
            PyBytecode::ToBool =>               11,    
            PyBytecode::BinaryOp(_) =>          12,     
            PyBytecode::BinaryAdd =>            13,     
            PyBytecode::BinaryMultiply =>       14,     
            PyBytecode::BinarySubtract =>       15,     
            PyBytecode::BinaryDivide =>         16,    
            PyBytecode::BinaryXOR =>            17,     
            PyBytecode::LoadConst(_) =>         18,     
            PyBytecode::LoadFast(_) =>          19,     
            PyBytecode::StoreFast(_) =>         20,     
            PyBytecode::LoadName(_) =>          21,     
            PyBytecode::StoreName(_) =>         22,     
            PyBytecode::LoadGlobal =>           23,     
            PyBytecode::StoreGlobal =>          24,     
            PyBytecode::PushNull =>             25,
            PyBytecode::Cache =>                26,
            PyBytecode::CallFunction(_) =>      27,
            PyBytecode::CallInstrinsic1(_) =>   28,
            PyBytecode::CallInstrinsic2(_) =>   29,
            PyBytecode::ReturnValue =>          30,
            PyBytecode::MakeFunction =>         31,
            PyBytecode::LoadBuildClass =>       32,
            PyBytecode::PopJumpIfFalse(_) =>    33,
            PyBytecode::PopJumpIfTrue(_) =>     34,
            PyBytecode::JumpForward(_) =>       35,
            PyBytecode::JumpBackward(_) =>      36,
            PyBytecode::CompareOp(_) =>         37,
            PyBytecode::UnpackSequence =>       38,
            PyBytecode::UnpackEx =>             39,
            PyBytecode::LoadDeref(_) =>         40,
            PyBytecode::BuildList(_) =>         41,
            PyBytecode::BuildTuple(_) =>        42,
            PyBytecode::BuildSet(_) =>          43,
            PyBytecode::BuildMap =>             44,
            PyBytecode::BuildString(_) =>       45,
            PyBytecode::ListAppend =>           46,
            PyBytecode::ForIter(_) =>           47,
            PyBytecode::GetIter =>              48,
            PyBytecode::Error =>                254,
        }
    }
}

impl std::fmt::Display for PyBytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}