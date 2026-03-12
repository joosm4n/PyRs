use crate::{
    pyrs_codeobject::{PyCodeObj, PyCompileCtx, PyTypeObj},
    pyrs_obj::ToObj,
    pyrs_parsing::{Expression, Keyword, Op},
    pyrs_pyobject::{AttrDict, PyObjPtr, PyObject},
    pyrs_vm::IntrinsicFunc,
};

use std::{collections::HashMap, sync::Arc};
// Format: offset INSTRUCTION argument (value)
// 0 LOAD_CONST 0 (0)      # Load constant at index 0, which is the integer 0
// 2 STORE_NAME 0 (i)      # Store the top stack value into variable name at index 0 (variable "i")

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
#[repr(u8)]
pub enum PyBytecode {
    // Empty
    NOP,

    // Import
    ImportName(u8),
    ImportFrom(u8),

    // Fundamentals
    PopIter,
    PopTop,
    EndFor,
    Copy(u8),
    Swap(u8),

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

    LoadConst(u8),
    LoadFast(u8),
    StoreFast(u8),
    LoadName(u8),
    StoreName(u8),
    LoadGlobal(u8),
    StoreGlobal(u8),
    PushNull,

    Cache,

    CallFunction(u8 /* argc */),
    CallInstrinsic1(IntrinsicFunc),
    CallInstrinsic2(IntrinsicFunc),
    ReturnValue,
    MakeFunction,

    LoadBuildClass,

    PopJumpIfFalse(u8),
    PopJumpIfTrue(u8),
    JumpForward(u8),
    JumpBackward(u8),

    CompareOp(Op),

    UnpackSequence,
    UnpackEx,
    LoadDeref(u8),

    BuildList(u8),
    BuildTuple(u8),
    BuildSet(u8),
    BuildMap,
    BuildString(u8),
    ListAppend(u8),

    ForIter(u8),
    GetIter,
    Resume,
    LoadNameEx(u8),

    LoadAttr(u8),
    StoreAttr(u8),
    LoadSmallInt(u8),

    // not proper
    Error,
}

impl PyBytecode {
    pub fn from_expr(expr: Expression, context: &mut PyCompileCtx) {
        // println!("Compiling: {}", expr.to_string());
        match expr {
            Expression::Ident(x) => {
                let load_name = context.add_varname_load(x);
                context.push(load_name);
            }
            Expression::Atom(a) => match a.parse::<u8>() {
                Ok(small_int) => {
                    context.push(PyBytecode::LoadSmallInt(small_int));
                }
                Err(_) => {
                    let i = context.add_const(a.to_pyptr());
                    context.push(PyBytecode::LoadConst(i));
                }
            },
            Expression::Operation(op, args) => {
                let mut name = String::new();
                match op {
                    Op::Equals => {
                        let mut attr: Option<Expression> = None;
                        for (idx, a) in args.into_iter().enumerate() {
                            if idx == 0 {
                                match a {
                                    Expression::Ident(ident) => {
                                        name = ident;
                                        let _namei = context.add_varname(name.clone());
                                    }
                                    dot @ Expression::Operation(Op::Dot, _) => attr = Some(dot),
                                    e => panic!("SyntaxError: invalid expr {e}"),
                                };
                            } else {
                                match a {
                                    Expression::Call(fn_name, args) => {
                                        let argc = args.len();
                                        for a in args {
                                            //dbg!(&a);
                                            PyBytecode::from_expr(a, context);
                                        }
                                        let namei = context.add_varname(fn_name);
                                        context.push(PyBytecode::PushNull);
                                        context.push(PyBytecode::LoadName(namei));
                                        context.push(PyBytecode::CallFunction(argc as u8));
                                    }
                                    _ => PyBytecode::from_expr(a, context),
                                }
                            }
                        }

                        let store_attr: bool;
                        match attr {
                            Some(dot) => {
                                store_attr = true;
                                PyBytecode::from_expr(dot, context);
                            }
                            None => store_attr = false,
                        }

                        if name.is_empty() {
                            name = match context.get_last_name().cloned() {
                                Some(n) => n,
                                None => panic!(),
                            }
                        }

                        if store_attr {
                            let store_spot = context.len() - 1;
                            let attr_name = context.add_name(name);
                            context[store_spot] = PyBytecode::StoreAttr(attr_name);
                        } else {
                            let store_name = context.add_varname_store(name);
                            context.push(store_name);
                        }
                        return;
                    }
                    Op::AddEquals | Op::SubEquals | Op::MulEquals | Op::DivEquals => {
                        for (idx, a) in args.into_iter().enumerate() {
                            if idx == 0 {
                                match a {
                                    Expression::Ident(ident) => {
                                        name = ident;
                                        let load_name = context.add_varname_load(name.clone());
                                        context.push(load_name);
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

                        let store_name = context.add_varname_store(name);
                        context.push(store_name);
                        return;
                    }
                    Op::List => {
                        context.push(PyBytecode::BuildList(0));

                        let objs = args.into_iter().map(|x| x.to_pyptr()).collect();
                        let i = context.add_const(PyObject::new_tuple(objs).to_ptr());
                        context.push(PyBytecode::LoadConst(i));

                        context.push(PyBytecode::ListAppend(0));
                        return;
                    }
                    Op::Set => {
                        let obj_cound = args.len();
                        for a in args {
                            PyBytecode::from_expr(a, context);
                        }
                        context.push(PyBytecode::BuildSet(obj_cound as u8));
                        return;
                    }
                    Op::Tuple => {
                        let obj_cound = args.len();
                        for a in args {
                            PyBytecode::from_expr(a, context);
                        }
                        context.push(PyBytecode::BuildTuple(obj_cound as u8));
                        return;
                    }
                    Op::Dot => {
                        for (idx, a) in args.into_iter().enumerate() {
                            match idx {
                                0 => {
                                    let namei = context.add_varname_load(a.get_value_string());
                                    context.push(namei);
                                }
                                1 => {
                                    match a {
                                        c @ Expression::Call(_, _) => {
                                            PyBytecode::from_expr(c, context);
                                        }
                                        Expression::Ident(ident) => {
                                            let namei = context.add_name(ident);
                                            context.push(PyBytecode::LoadAttr(namei));
                                        }
                                        _ => panic!(),
                                    };
                                }
                                _ => panic!(),
                            }
                        }
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

                if IntrinsicFunc::try_get(&name).is_some() {
                    let namei = context.add_name(name);
                    context.push(PyBytecode::LoadGlobal(namei));
                    context.push(PyBytecode::PushNull);
                } else {
                    let namei = context.add_varname(name);
                    context.push(PyBytecode::LoadName(namei));
                    context.push(PyBytecode::PushNull);
                }

                for a in args {
                    //dbg!(&a);
                    PyBytecode::from_expr(a, context);
                }

                context.push(PyBytecode::CallFunction(argc as u8));
            }
            Expression::Keyword(keyword, mut args, body) => {
                match keyword {
                    Keyword::True => {
                        let i = context.add_const(true.to_pyptr());
                        context.push(PyBytecode::LoadConst(i));
                    }
                    Keyword::False => {
                        let i = context.add_const(false.to_pyptr());
                        context.push(PyBytecode::LoadConst(i));
                    }
                    Keyword::Elif | Keyword::Else => {
                        panic!("Shouldn't have a stand alone elif/else expression")
                    }
                    Keyword::If => {
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
                                context[jump_spot] =
                                    PyBytecode::PopJumpIfFalse(body_code_len as u8);

                                place_holders.push((context.len() - start_cond, context.len()));
                                context.push(PyBytecode::JumpForward(0)); // placeholder to jump to end
                            } else {
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
                            context[jump_to_end_spot] = PyBytecode::JumpForward(dist_to_end as u8);
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
                        context[jump_spot] = PyBytecode::PopJumpIfFalse(delta as u8); // skip entire while loop

                        let return_delta = context.len() - condition_start + 1;
                        context.push(PyBytecode::JumpBackward(return_delta as u8));

                        let i = context.add_const(PyObject::none());
                        context.push(PyBytecode::LoadConst(i));
                    }
                    Keyword::For => {
                        let for_err =
                            "only for loops of form \'for Ident() in Ident()\' currently supported";
                        assert_eq!(args.len(), 2);

                        match args.pop().unwrap() {
                            Expression::Ident(ident) => {
                                let namei = context.add_varname(ident.clone());
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
                        context.push(PyBytecode::ForIter(0)); // placeholder

                        let x_namei = context.add_varname(x);
                        context.push(PyBytecode::StoreFast(x_namei));

                        let start_for_code_spot = context.len();
                        for b in body {
                            PyBytecode::from_expr(b, context);
                        }
                        context.push(PyBytecode::EndFor);
                        let contents_len = context.len() - start_for_code_spot; // length of for loops contents
                        context[iter_spot] = PyBytecode::ForIter((contents_len + 2) as u8); // insert right val
                        context.push(PyBytecode::JumpBackward((contents_len + 3) as u8));
                    }
                    Keyword::Def => {
                        let fn_code =
                            PyBytecode::compile_fn(Expression::Keyword(Keyword::Def, args, body));
                        let name = fn_code.name.clone();
                        let idx = context.add_const(fn_code.to_pyptr());

                        context.push(PyBytecode::LoadConst(idx));
                        context.push(PyBytecode::MakeFunction);

                        //dbg!(&name);
                        let namei = context.add_varname(name);
                        //dbg!(&namei);
                        context.push(PyBytecode::StoreFast(namei));
                    }
                    Keyword::Class => {
                        let class = PyBytecode::compile_class(args, body, context);
                        let class_name = class.name.clone();
                        let code_namei = context.add_const(class.to_pyptr());
                        let namei = context.add_const(class_name.clone().to_pyptr());
                        context.push(PyBytecode::LoadBuildClass);
                        context.push(PyBytecode::PushNull);
                        context.push(PyBytecode::LoadConst(code_namei));
                        context.push(PyBytecode::MakeFunction);
                        context.push(PyBytecode::LoadConst(namei));
                        context.push(PyBytecode::CallFunction(2));
                        let i = context.add_varname(class_name);
                        context.push(PyBytecode::StoreFast(i));
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
                        let i = context.add_const(PyObject::none());
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

    pub fn from_string(s: &str) -> PyCodeObj {
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

    fn compile_fn(body: Expression) -> Arc<PyCodeObj> {
        match body {
            Expression::Keyword(Keyword::Def, mut args, body) => {
                let func_args = args.split_off(1);

                let name = match args.pop() {
                    Some(Expression::Ident(ident)) => ident,
                    _ => panic!("function name must be identifier"),
                };

                // Compile function body into its OWN bytecode
                let mut fn_ctx = PyCompileCtx::new(&name);

                for a in func_args {
                    match a {
                        Expression::Ident(name) => {
                            fn_ctx.add_varname(name);
                        }
                        _ => panic!(),
                    }
                    // PyBytecode::from_expr(a, &mut fn_ctx);
                }
                for b in body {
                    PyBytecode::from_expr(b, &mut fn_ctx);
                }

                let const_num = fn_ctx.add_const(PyObjPtr::none());
                fn_ctx.push(PyBytecode::LoadConst(const_num));
                fn_ctx.push(PyBytecode::ReturnValue);

                Arc::new(fn_ctx.finish())
            }
            _ => unreachable!(),
        }
    }

    fn compile_class(
        args: Vec<Expression>,
        body: Vec<Expression>,
        parent_context: &PyCompileCtx,
    ) -> PyTypeObj {
        //dbg!(&args);
        let name = match args.first().unwrap() {
            Expression::Ident(ident) => ident.clone(),
            e => panic!("class name must be an identifier not: {:?}", e),
        };

        let mut ctx = PyCompileCtx::new(&name);
        let name__ = ctx.add_name_load("__name__");
        ctx.push(name__);
        let module__ = ctx.add_name_store("__module__");
        ctx.push(module__);

        {
            let parent_name = parent_context.get_context_name();
            ctx.load_const(format!("{parent_name}.<locals>.{name}").to_pyptr());
        }

        let qualname__ = ctx.add_name_store("__qualname__");
        ctx.push(qualname__);

        // let firstlineno__ = ctx.add_name("__firstlineno__");

        let mut class_fields: HashMap<String, PyObjPtr> = HashMap::new();
        for field in body.into_iter() {
            match field {
                Expression::Operation(Op::Equals, mut v) => {
                    let member_name = v[0].get_value_string();
                    let default_val = v.pop().unwrap();
                    PyBytecode::from_expr(default_val, &mut ctx);
                    let namei = ctx.add_name(&member_name);
                    ctx.push(PyBytecode::StoreName(namei));
                    class_fields.insert(member_name, PyObjPtr::none());
                }
                Expression::Keyword(Keyword::Def, conds, body) => {
                    let fn_code =
                        PyBytecode::compile_fn(Expression::Keyword(Keyword::Def, conds, body));
                    let name = fn_code.name.clone();
                    let idx = ctx.add_const(fn_code.to_pyptr());

                    ctx.push(PyBytecode::LoadConst(idx));
                    ctx.push(PyBytecode::MakeFunction);

                    let namei = ctx.add_name(&name);
                    ctx.push(PyBytecode::StoreName(namei));
                    class_fields.insert(name, PyObjPtr::none());
                }
                _ => panic!("invalid expr for default"),
            }
        }

        PyTypeObj {
            name,
            static_attribs: AttrDict { 0: class_fields },
            code: Arc::new(ctx.finish()),
        }
    }

    pub fn to_string(vec: &Vec<Self>) -> String {
        let mut string = String::new();
        for (idx, line) in vec.iter().enumerate() {
            string.push_str(format!("({idx}) \t\t{:?}\n", line).as_str());
        }
        string
    }

    pub const TYPE_STR_LEN: usize = 16;

    pub const fn get_byte_map() -> [[u8; PyBytecode::TYPE_STR_LEN]; 255] {
        let mut bytecode_map = [[b'_'; PyBytecode::TYPE_STR_LEN]; 255];
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
            PyBytecode::NOP =>                  b"NOP_____________",
            PyBytecode::ImportName(_) =>        b"ImportName______",
            PyBytecode::ImportFrom(_) =>        b"ImportFrom______",
            PyBytecode::PopIter =>              b"PopIter_________",
            PyBytecode::PopTop =>               b"PopTop__________",
            PyBytecode::EndFor =>               b"EndFor__________",
            PyBytecode::Copy(_) =>              b"Copy____________",
            PyBytecode::Swap(_) =>              b"Swap____________",             
            PyBytecode::UnaryNegative =>        b"UnaryNegative___",        
            PyBytecode::UnaryInvert =>          b"UnaryInvert_____",
            PyBytecode::UnaryNot =>             b"UnaryNot________",
            PyBytecode::ToBool =>               b"ToBool__________",
            PyBytecode::BinaryOp(_) =>          b"BinaryOp________",
            PyBytecode::BinaryAdd =>            b"BinaryAdd_______",
            PyBytecode::BinaryMultiply =>       b"BinaryMultiply__",
            PyBytecode::BinarySubtract =>       b"BinarySubtract__",
            PyBytecode::BinaryDivide =>         b"BinaryDivide____",
            PyBytecode::BinaryXOR =>            b"BinaryXOR_______",
            PyBytecode::LoadConst(_) =>         b"LoadConst_______",
            PyBytecode::LoadFast(_) =>          b"LoadFast________",
            PyBytecode::StoreFast(_) =>         b"StoreFast_______",
            PyBytecode::LoadName(_) =>          b"LoadName________",
            PyBytecode::StoreName(_) =>         b"StoreName_______",
            PyBytecode::LoadGlobal(_) =>        b"LoadGlobal______",
            PyBytecode::StoreGlobal(_) =>       b"StoreGlobal_____",
            PyBytecode::PushNull =>             b"PushNull________",
            PyBytecode::Cache =>                b"Cache___________",
            PyBytecode::CallFunction(_) =>      b"CallFunction____",
            PyBytecode::CallInstrinsic1(_) =>   b"CallInstrinsic1_",
            PyBytecode::CallInstrinsic2(_) =>   b"CallInstrinsic2_",
            PyBytecode::ReturnValue =>          b"ReturnValue_____",
            PyBytecode::MakeFunction =>         b"MakeFunction____",
            PyBytecode::LoadBuildClass =>       b"LoadBuildClass__",
            PyBytecode::PopJumpIfFalse(_) =>    b"PopJumpIfFalse__",
            PyBytecode::PopJumpIfTrue(_) =>     b"PopJumpIfTrue___",
            PyBytecode::JumpForward(_) =>       b"JumpForward_____",
            PyBytecode::JumpBackward(_) =>      b"JumpBackward____",
            PyBytecode::CompareOp(_) =>         b"CompareOp_______",
            PyBytecode::UnpackSequence =>       b"UnpackSequence__",
            PyBytecode::UnpackEx =>             b"UnpackEx________",
            PyBytecode::LoadDeref(_) =>         b"LoadDeref_______",
            PyBytecode::BuildList(_) =>         b"BuildList_______",
            PyBytecode::BuildTuple(_) =>        b"BuildTuple______",
            PyBytecode::BuildSet(_) =>          b"BuildSet________",
            PyBytecode::BuildMap =>             b"BuildMap________",
            PyBytecode::BuildString(_) =>       b"BuildString_____",
            PyBytecode::ListAppend(_) =>        b"ListAppend______",
            PyBytecode::ForIter(_) =>           b"ForIter_________",
            PyBytecode::GetIter =>              b"GetIter_________",
            PyBytecode::Resume =>               b"Resume__________",
            PyBytecode::LoadNameEx(_) =>        b"LoadNameEx______",
            PyBytecode::LoadAttr(_) =>          b"LoadAttr________",
            PyBytecode::StoreAttr(_) =>         b"StoreAttr_______",
            PyBytecode::LoadSmallInt(_) =>      b"LoadSmallInt____",
            PyBytecode::Error =>                b"Error___________",
        }
    }

    pub const fn get_type_str(&self) -> &str {
        let s = self.get_type_str_slice();
        unsafe { str::from_utf8_unchecked(s) }
    }

    pub const fn from_bytes(bytes: &[u8; 2]) -> Self {
        let enum_: u8 = bytes[0];
        let data: u8 = bytes[1];
        match enum_ {
            0 => PyBytecode::NOP,
            1 => PyBytecode::ImportName(data),
            2 => PyBytecode::ImportFrom(data),
            3 => PyBytecode::PopIter,
            4 => PyBytecode::PopTop,
            5 => PyBytecode::EndFor,
            6 => PyBytecode::Copy(data),
            7 => PyBytecode::Swap(data),
            8 => PyBytecode::UnaryNegative,
            9 => PyBytecode::UnaryNot,
            10 => PyBytecode::UnaryInvert,
            11 => PyBytecode::ToBool,
            12 => PyBytecode::BinaryOp(Op::from_u8(data)),
            13 => PyBytecode::BinaryAdd,
            14 => PyBytecode::BinaryMultiply,
            15 => PyBytecode::BinarySubtract,
            16 => PyBytecode::BinaryDivide,
            17 => PyBytecode::BinaryXOR,
            18 => PyBytecode::LoadConst(data),
            19 => PyBytecode::LoadFast(data),
            20 => PyBytecode::StoreFast(data),
            21 => PyBytecode::LoadName(data),
            22 => PyBytecode::StoreName(data),
            23 => PyBytecode::LoadGlobal(data),
            24 => PyBytecode::StoreGlobal(data),
            25 => PyBytecode::PushNull,
            26 => PyBytecode::Cache,
            27 => PyBytecode::CallFunction(data),
            28 => PyBytecode::CallInstrinsic1(IntrinsicFunc::from_u8(data)),
            29 => PyBytecode::CallInstrinsic2(IntrinsicFunc::from_u8(data)),
            30 => PyBytecode::ReturnValue,
            31 => PyBytecode::MakeFunction,
            32 => PyBytecode::LoadBuildClass,
            33 => PyBytecode::PopJumpIfFalse(data),
            34 => PyBytecode::PopJumpIfTrue(data),
            35 => PyBytecode::JumpForward(data),
            36 => PyBytecode::JumpBackward(data),
            37 => PyBytecode::CompareOp(Op::from_u8(data)),
            38 => PyBytecode::UnpackSequence,
            39 => PyBytecode::UnpackEx,
            40 => PyBytecode::LoadDeref(data),
            41 => PyBytecode::BuildList(data),
            42 => PyBytecode::BuildTuple(data),
            43 => PyBytecode::BuildSet(data),
            44 => PyBytecode::BuildMap,
            45 => PyBytecode::BuildString(data),
            46 => PyBytecode::ListAppend(data),
            47 => PyBytecode::ForIter(data),
            48 => PyBytecode::GetIter,
            49 => PyBytecode::Resume,
            50 => PyBytecode::LoadNameEx(data),
            51 => PyBytecode::StoreAttr(data),
            52 => PyBytecode::LoadAttr(data),
            53 => PyBytecode::LoadSmallInt(data),
            255 => PyBytecode::Error,

            _ => PyBytecode::Error,
        }
    }

    pub const fn to_bytes(&self) -> [u8; 2] {
        match self {
            PyBytecode::NOP => [0, 0],
            PyBytecode::ImportName(v) => [1, *v as u8],
            PyBytecode::ImportFrom(v) => [2, *v as u8],
            PyBytecode::PopIter => [3, 0],
            PyBytecode::PopTop => [4, 0],
            PyBytecode::EndFor => [5, 0],
            PyBytecode::Copy(v) => [6, *v as u8],
            PyBytecode::Swap(v) => [7, *v as u8],
            PyBytecode::UnaryNegative => [8, 0],
            PyBytecode::UnaryNot => [9, 0],
            PyBytecode::UnaryInvert => [10, 0],
            PyBytecode::ToBool => [11, 0],
            PyBytecode::BinaryOp(v) => [12, *v as u8],
            PyBytecode::BinaryAdd => [13, 0],
            PyBytecode::BinaryMultiply => [14, 0],
            PyBytecode::BinarySubtract => [15, 0],
            PyBytecode::BinaryDivide => [16, 0],
            PyBytecode::BinaryXOR => [17, 0],
            PyBytecode::LoadConst(v) => [18, *v as u8],
            PyBytecode::LoadFast(v) => [19, *v as u8],
            PyBytecode::StoreFast(v) => [20, *v as u8],
            PyBytecode::LoadName(v) => [21, *v as u8],
            PyBytecode::StoreName(v) => [22, *v as u8],
            PyBytecode::LoadGlobal(v) => [23, *v as u8],
            PyBytecode::StoreGlobal(v) => [24, *v as u8],
            PyBytecode::PushNull => [25, 0],
            PyBytecode::Cache => [26, 0],
            PyBytecode::CallFunction(v) => [27, *v as u8],
            PyBytecode::CallInstrinsic1(v) => [28, *v as u8],
            PyBytecode::CallInstrinsic2(v) => [29, *v as u8],
            PyBytecode::ReturnValue => [30, 0],
            PyBytecode::MakeFunction => [31, 0],
            PyBytecode::LoadBuildClass => [32, 0],
            PyBytecode::PopJumpIfFalse(v) => [33, *v as u8],
            PyBytecode::PopJumpIfTrue(v) => [34, *v as u8],
            PyBytecode::JumpForward(v) => [35, *v as u8],
            PyBytecode::JumpBackward(v) => [36, *v as u8],
            PyBytecode::CompareOp(v) => [37, *v as u8],
            PyBytecode::UnpackSequence => [38, 0],
            PyBytecode::UnpackEx => [39, 0],
            PyBytecode::LoadDeref(v) => [40, *v as u8],
            PyBytecode::BuildList(v) => [41, *v as u8],
            PyBytecode::BuildTuple(v) => [42, *v as u8],
            PyBytecode::BuildSet(v) => [43, *v as u8],
            PyBytecode::BuildMap => [44, 0],
            PyBytecode::BuildString(v) => [45, *v as u8],
            PyBytecode::ListAppend(v) => [46, *v as u8],
            PyBytecode::ForIter(v) => [47, *v as u8],
            PyBytecode::GetIter => [48, 0],
            PyBytecode::Resume => [49, 0],
            PyBytecode::LoadNameEx(v) => [50, *v as u8],
            PyBytecode::StoreAttr(v) => [51, *v as u8],
            PyBytecode::LoadAttr(v) => [52, *v as u8],
            PyBytecode::LoadSmallInt(v) => [53, *v as u8],
            PyBytecode::Error => [254, 0],
        }
    }
}

impl std::convert::From<PyBytecode> for u8 {
    fn from(value: PyBytecode) -> u8 {
        match value {
            PyBytecode::NOP => 0,
            PyBytecode::ImportName(_) => 1,
            PyBytecode::ImportFrom(_) => 2,
            PyBytecode::PopIter => 3,
            PyBytecode::PopTop => 4,
            PyBytecode::EndFor => 5,
            PyBytecode::Copy(_) => 6,
            PyBytecode::Swap(_) => 7,
            PyBytecode::UnaryNegative => 8,
            PyBytecode::UnaryNot => 9,
            PyBytecode::UnaryInvert => 10,
            PyBytecode::ToBool => 11,
            PyBytecode::BinaryOp(_) => 12,
            PyBytecode::BinaryAdd => 13,
            PyBytecode::BinaryMultiply => 14,
            PyBytecode::BinarySubtract => 15,
            PyBytecode::BinaryDivide => 16,
            PyBytecode::BinaryXOR => 17,
            PyBytecode::LoadConst(_) => 18,
            PyBytecode::LoadFast(_) => 19,
            PyBytecode::StoreFast(_) => 20,
            PyBytecode::LoadName(_) => 21,
            PyBytecode::StoreName(_) => 22,
            PyBytecode::LoadGlobal(_) => 23,
            PyBytecode::StoreGlobal(_) => 24,
            PyBytecode::PushNull => 25,
            PyBytecode::Cache => 26,
            PyBytecode::CallFunction(_) => 27,
            PyBytecode::CallInstrinsic1(_) => 28,
            PyBytecode::CallInstrinsic2(_) => 29,
            PyBytecode::ReturnValue => 30,
            PyBytecode::MakeFunction => 31,
            PyBytecode::LoadBuildClass => 32,
            PyBytecode::PopJumpIfFalse(_) => 33,
            PyBytecode::PopJumpIfTrue(_) => 34,
            PyBytecode::JumpForward(_) => 35,
            PyBytecode::JumpBackward(_) => 36,
            PyBytecode::CompareOp(_) => 37,
            PyBytecode::UnpackSequence => 38,
            PyBytecode::UnpackEx => 39,
            PyBytecode::LoadDeref(_) => 40,
            PyBytecode::BuildList(_) => 41,
            PyBytecode::BuildTuple(_) => 42,
            PyBytecode::BuildSet(_) => 43,
            PyBytecode::BuildMap => 44,
            PyBytecode::BuildString(_) => 45,
            PyBytecode::ListAppend(_) => 46,
            PyBytecode::ForIter(_) => 47,
            PyBytecode::GetIter => 48,
            PyBytecode::Resume => 49,
            PyBytecode::LoadNameEx(_) => 50,
            PyBytecode::StoreAttr(_) => 51,
            PyBytecode::LoadAttr(_) => 52,
            PyBytecode::LoadSmallInt(_) => 53,
            PyBytecode::Error => 254,
        }
    }
}

impl std::fmt::Display for PyBytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
