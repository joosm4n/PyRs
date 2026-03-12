use std::{
    collections::HashMap, hash::Hash, io::{self, Write}, path::PathBuf, sync::{Arc, Mutex}, usize
};

use crate::{
    pyrs_bytecode::PyBytecode, pyrs_codeobject::{FuncObj, PyCodeObj, PyTypeObj}, pyrs_error::{PyError, PyException}, pyrs_interpreter::Interpreter, pyrs_obj::{Obj, ToObj}, pyrs_parsing::Op, pyrs_pyobject::{AttrDict, PyObjPtr, PyObject}, pyrs_std::{FnPtr, RangeObj}
};

#[derive(Debug, Clone)]
pub struct PyFrame {
    pub code: Arc<PyCodeObj>,
    pub ip: usize,
    pub stack: Vec<PyObjPtr>,
    pub locals: Vec<PyObjPtr>,
    pub globals: Arc<Mutex<HashMap<String, PyObjPtr>>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PyVM {
    builtins: HashMap<String, PyObjPtr>,
    curr_namespace: String,
    frames: Vec<PyFrame>,

    error_state: bool,
    debug_mode: bool,
    null: PyObjPtr,
    working_dir: PathBuf,
}

#[allow(dead_code)]
impl PyVM {
    pub fn new() -> Self {
        PyVM {
            builtins: HashMap::new(), // placeholder for now 
            curr_namespace: String::from(""),
            frames: vec![],
            error_state: false,
            debug_mode: false,
            null: PyObjPtr::none(),
            working_dir: std::env::current_dir().unwrap_or(PathBuf::new()),
        }
    }

    pub fn set_debug_mode(&mut self, debug: bool) {
        self.debug_mode = debug;
    }

    pub fn execute(&mut self, code_obj: PyCodeObj) {

        if self.debug_mode {
            println!("Working dir: {:?}", self.working_dir);
        }

        let num_vars = code_obj.num_varnames;
        let frame_globals = Arc::new(Mutex::new(code_obj.globals.clone()));
        self.frames.push(PyFrame {
            code: Arc::new(code_obj),
            ip: 0,
            stack: vec![],
            locals: vec![self.null.clone(); num_vars],
            globals: frame_globals,
        });

        if self.debug_mode {
            self.print_frame();
        }

        loop {
            let frame = self.frame_mut();
            if frame.ip >= frame.code.bytecode.len() {
                break;
            }

            let instr = frame.code.bytecode[frame.ip].clone();
            self.execute_instruction(instr);
            
            self.frame_mut().ip += 1;
            if self.frames.is_empty() {
                break;
            }
        }
    }

    fn execute_instruction(&mut self, inst: PyBytecode) {
        if inst == PyBytecode::NOP {
            return;
        }

        if self.debug_mode {
            println!(
                "Executing: ({})   {:?}\nStack:",
                self.frame().ip,
                &inst,
            );
            self.print_stack();
            self.print_locals();
            println!();
        }

        match inst {
            PyBytecode::Copy(i) => self.copy(i),
            PyBytecode::Swap(i) => self.swap(i),
            
            PyBytecode::PopTop => self.pop_top(),
            PyBytecode::EndFor => self.end_for(),

            PyBytecode::LoadConst(namei) => self.load_const(namei),
            PyBytecode::LoadFast(i) => self.load_fast(i),
            PyBytecode::StoreFast(i) => self.store_fast(i),
            PyBytecode::LoadName(namei) => self.load_name(namei),
            PyBytecode::StoreName(namei) => self.store_name(namei),
            PyBytecode::LoadGlobal(namei) => self.load_global(namei),
            PyBytecode::StoreGlobal(namei) => self.store_global(namei),
            PyBytecode::LoadAttr(namei) => self.load_attr(namei),
            PyBytecode::StoreAttr(namei) => self.store_attr(namei),

            PyBytecode::PushNull => self.push_null(),

            PyBytecode::BuildList(len) => self.build_list(len),
            PyBytecode::ListAppend(i) => self.list_append(i),
            PyBytecode::BuildTuple(count) => self.build_tuple(count),

            PyBytecode::GetIter => self.get_iter(),
            PyBytecode::ForIter(delta) => self.for_iter(delta),
            PyBytecode::UnpackSequence => self.unpack_sequence(),

            PyBytecode::BinaryAdd => self.binary_add(),
            PyBytecode::BinarySubtract => self.binary_subtract(),
            PyBytecode::BinaryMultiply => self.binary_multiply(),
            PyBytecode::BinaryDivide => self.binary_divide(),

            PyBytecode::UnaryNegative => self.unary_negative(),

            PyBytecode::CallFunction(argc) => self.call_function(argc),
            PyBytecode::CallInstrinsic1(ptr) => self.call_intrinsic_1(ptr),
            PyBytecode::ReturnValue => self.return_value(),

            PyBytecode::PopJumpIfFalse(delta) => self.pop_jump_if_false(delta),
            PyBytecode::PopJumpIfTrue(delta) => self.pop_jump_if_true(delta),
            PyBytecode::JumpForward(delta) => self.jump_forward(delta),
            PyBytecode::JumpBackward(delta) => self.jump_backward(delta),

            PyBytecode::CompareOp(op) => self.compare_op(op),

            PyBytecode::MakeFunction => self.make_function(),

            PyBytecode::LoadBuildClass => self.load_build_class(),
            PyBytecode::ImportName(namei) => self.import_name(namei),
            PyBytecode::LoadSmallInt(int_) => self.push(int_.to_pyobj().to_ptr()),

            PyBytecode::Resume => {},
            PyBytecode::NOP => {},
            _ => panic!("\nUnimplementedError: Instruction {:?} not implemented! \n", inst),
        }
    }

    pub fn dbg<T: std::fmt::Debug>(&self, p: &T) {
        if self.debug_mode {
            dbg!(p);
        }
    }

    pub fn set_working_dir(&mut self, path: &str) {
        self.working_dir = PathBuf::from(path);
    }

    pub fn append_working_dir(&mut self, path: &str) {
        let parts: Vec<&str> = path.split(&['/', '\\']).collect();
        for p in parts {
            self.working_dir.push(p);
        }
    }

    fn push_err(&mut self, e: PyException) {
        self.push(e.to_pyptr());
        self.error_state = true;
    }

    fn throw_err(&self, e: PyException) {
        let ip = self.frame().ip;
        println!();
        println!("---- PyVM Error ---- \n");

        self.print_debug_info();
        println!();

        print!("Error: at bytecode ");
        if let Some(inst) = &self.frame().code.bytecode.get(ip) {
            println!("({}) {}", ip, inst);
        }

        println!("\n{e}");

        println!();
        panic!("\n ^^^ PyVM Error Thrown ^^^ \n");
    }

    fn print_debug_info(&self) {

        println!("\n---- PyVM Debug Info ----\n");

        println!("\t-- Current Frame --");
        self.print_frame();
        let frame = self.frames.last().unwrap();
        println!(
            "\t-- Curr Instruction -- \n({}) \t{}",
            frame.ip, frame.code.bytecode.get(frame.ip).unwrap_or(&PyBytecode::NOP)
        );

        println!("\n\t-- Stack Trace --");
        self.print_stack();

        println!("\n\t-- Local Vars --\n{:?}", self.frame().locals);
    }

    fn throw(&mut self) {
        let ip = self.frame().ip;
        let e = self.pop();
        println!();
        println!("---- PyVM Error ---- \n");

        println!("Error: at bytecode instruction {}", ip,);

        self.print_instruction(ip);
        println!("\n{}", *e.get_ref());

        self.print_debug_info();

        println!();
        panic!("\n ^^^ PyVM Error Thrown ^^^ \n");
    }

    fn push(&mut self, obj: PyObjPtr) {
        //self.local_stacks.last_mut().unwrap().push(obj);
        self.frame_mut().stack.push(obj);
    }

    fn pop(&mut self) -> PyObjPtr {
        match self.frame_mut().stack.pop() {
            Some(obj) => obj,
            None => {
                let e = PyException {
                    error: PyError::StackError,
                    msg: "Tried to pop empty stack".to_string(),
                };
                self.throw_err(e);
                unreachable!();
            }
        }
    }

    fn frame(&self) -> &PyFrame {
        return self.frames.last().unwrap();
    }

    fn frame_mut(&mut self) -> &mut PyFrame {
        return self.frames.last_mut().unwrap();
    }

    fn pop_n(&mut self, count: u8) -> Vec<PyObjPtr> {
        let mut objs: Vec<PyObjPtr> = vec![];
        for _ in 0..count {
            objs.push(self.pop());
        }
        objs.reverse();
        objs
    }

    fn pop_n_or(&mut self, count: u8, or: PyObjPtr) -> Vec<PyObjPtr> {
        let mut objs = vec![];
        for _ in 0..count {
            if let Some(obj) = self.frame_mut().stack.pop() {
                objs.push(obj);
            } else {
                objs.push(or.clone().into());
            }
        }
        objs.reverse();
        objs
    }

    fn pop_until(&mut self, stop_obj: &PyObjPtr) -> Vec<PyObjPtr> {
        let mut objs = vec![];
        while self.top() != stop_obj {
            objs.push(self.pop());
        }

        objs.reverse();
        objs
    }

    fn pop_until_null(&mut self) -> Vec<PyObjPtr> {
        let mut objs = vec![];
        loop {
            let top = self.pop();
            if top == self.null {
                break;
            }
            objs.push(top);
        }
        objs.reverse();
        objs
    }

    fn top(&self) -> &PyObjPtr {
        match self.frames.last() {
            Some(v) => {
                match v.stack.last() {
                    Some(v) => v,
                    None => {
                        self.throw_err(PyException {
                            error: PyError::StackError,
                            msg: "Tried to pop empty stack".to_string(),
                        });
                        unreachable!();

                    }
                }
            }
            None => {
                self.throw_err( PyException {
                    error: PyError::FrameError,
                    msg: "Tried frame in vector empty frames".to_string(),
                });
                unreachable!();
            }
        }
    }

    fn get_name(&self, namei: u8) -> Option<&String> {
        self.frame().code.names.get(namei as usize)
    }

    fn get_varname(&self, namei: u8) -> Option<&String> {
        self.frame().code.varnames.get(namei as usize)
    }

    pub fn print_stack(&self) {
        for (idx, a) in self.frame().stack.iter().enumerate() {
            println!(" [{}] \t{}", idx, a.get_ref().__str__());
        }
        println!();
    }

    pub fn print_locals(&self) {
        let mut s = String::from("Locals: [");
        for l in &self.frame().locals {
            s.push_str(&l.get_ref().__str__());
            s.push(',');
            s.push(' ');
        }
        if self.frame().locals.len() > 0 {
            s.pop(); s.pop();
        }
        s.push(']');
        println!("{s}");
    }

    fn print_frame(&self) {
        println!("\n{}", self.frame().code.pretty_format());
    }

    fn print_instruction(&self, index: usize) {
        let inst_queue = &self.frame().code.bytecode;
        if index < inst_queue.len() {
            println!("({}) \t\t{}", index, inst_queue[index]);
        }
    }

    fn print_instruction_queue(&mut self) {
        let inst_queue = &self.frame_mut().code.bytecode;
        println!("\nInstructions: ");
        println!("{}", PyBytecode::to_string(inst_queue));
    }

    // -------------- Instructions ----------------
    fn pop_top(&mut self) {
        self.pop();
    }

    fn end_for(&mut self) {
        while !matches!(self.top().get_ref().obj, Obj::Iter(_)) {
            self.pop();
        }
    }

    fn copy(&mut self, i: u8) {
        let frame = self.frame_mut();
        let val = frame.stack[frame.stack.len() - 1 - i as usize].clone();
        frame.stack.push(val);
    }

    fn swap(&mut self, i: u8) {
        let frame = self.frame_mut();
        let len = frame.stack.len();
        frame.stack.swap(len - 1, len - 1 - i as usize);
    }

    fn load_const(&mut self, i: u8) {
        let obj = self.frame_mut().code.consts[i as usize].clone() ;
        self.frame_mut().stack.push(obj);
    }

    fn store_fast(&mut self, i: u8) {
        let obj = self.frame_mut().stack.pop().unwrap();
        //dbg!(&obj);

        let frame = self.frame_mut();
        if let Some(_) = frame.locals.get_mut(i as usize) {
            frame.locals[i as usize] = obj.__new__();
        }
        else {
            self.throw_err(PyException {
                error: PyError::IndexError,
                msg: format!("Tried to access local stack at index {} when it only has {} elements!", self.frame().locals.len(), i),
            });
        }
    }

    fn load_fast(&mut self, namei: u8) {
        let val = self.frame_mut().locals[namei as usize].__new__();
        self.frame_mut().stack.push(val);
    }

    fn store_name(&mut self, namei: u8) {
        let val = self.pop();
        self.frame_mut().locals[namei as usize] = val;
    }

    fn load_name(&mut self, i: u8) {
        let name = self.frame().code.names[i as usize].clone();
        dbg!(&name);
        {    
            let frame = self.frame_mut();
            if let Some(v) = frame.globals.lock().expect("unable to lock globals").get(&name).cloned() {
                frame.stack.push(v);
                return;
            }
        }

        if let Some(v) = self.builtins.get(&name).cloned() {
            self.frame_mut().stack.push(v);
            return;
        }
        
        self.throw_err(PyException {
            error: PyError::UndefinedVariableError,
            msg: format!("unknown variable \'{name}\'. Failed at {} {}", line!(), file!()),
        });
    }

    fn store_global(&mut self, namei: u8) {
        let val = self.pop();
        let name = self.frame().code.names[namei as usize].clone();
        self.frame_mut().globals.lock().expect("unable to lock globals").insert(name, val);
    }

    fn load_global(&mut self, namei: u8) {
        let namei = namei as usize;
        {
            let frame = self.frame_mut();
            let name = frame.code.names[namei].clone();
            let locked = frame.globals.lock().expect("unable to lock globals");

            if let Some(v) = locked.get(&name).cloned() {
                let val = match &v.get_ref().obj {
                    Obj::Type(ty) => ty.new_instance().to_pyptr(),
                    _ => v.clone(),
                };
                frame.stack.push(val);
                return;
            }
            
            if let Some(intrinsic) = IntrinsicFunc::try_get(name) {
                frame.stack.push(intrinsic.get_funcptr());
                return;
            }
        }

        self.throw_err( PyException {
            error: PyError::UndefinedVariableError, 
            msg: format!("unknown global variable \'{}\'", self.frame().code.names[namei as usize].clone()),
        });
    }

    fn store_attr(&mut self, namei: u8) {
        let obj = self.pop();
        let value = self.pop();
        let attr_name = match self.get_name(namei) {
            Some(s) => s,
            None => {
                self.throw_err(PyException { 
                    error: PyError::UndefinedVariableError, 
                    msg: format!("no name at name[{}] of current codeobj", namei),
                });
                unreachable!();
            }
        };

        match obj.get_ref().__set_attr__(attr_name, value) {
            None => {},
            Some(e) => self.throw_err(e),
        };
    }

    fn load_attr(&mut self, namei: u8) {
        let obj = self.pop();
        let name = match self.get_name(namei) {
            Some(s) => s,
            None => {
                self.throw_err(PyException { 
                    error: PyError::UndefinedVariableError, 
                    msg: format!("could not find variable at names[{}] in current code_obj. Failed at {} {}", namei, line!(), file!()), 
                });
                unreachable!();
            }
        };

        match obj.get_ref().__getattr__(&name) {
            Ok(val) => self.push(val),
            Err(e) => { 
                self.throw_err(e);
                unreachable!();
            }
        };
    }

    fn push_null(&mut self) {
        self.push(self.null.clone());
    }

    fn build_list(&mut self, len: u8) {
        let objs = self.pop_n(len);
        let list = objs.to_pyptr();
        self.push(list);
    }

    fn list_append(&mut self, i: u8) {

        let top = self.pop();
        let mut objs = vec![];
        match top.clone().get_ref().iter_py() {
            Some(iter) => {
                for o in iter {
                    objs.push(o);
                }
            }
            None => {
                objs.push(top);
           }
        }

        let stack_idx = self.frame().stack.len() -1 - i as usize;
        if let Some(stack_i) = self.frame().stack.get(stack_idx).cloned() {

            match &mut stack_i.get_ref().obj {
                Obj::List( list) => {
                    for o in objs {
                        list.push(o);
                    }
                }
                o => self.throw_err(PyException { 
                    error: PyError::TypeError,
                    msg: format!(" {:?} at stack[{}] cannot be appended to", o, stack_idx), 
                }),
            }
        }
        else {
            self.throw_err(PyException { 
                error: PyError::StackError, 
                msg: format!(" no value at stack[{stack_idx}]"), 
            });
        }
    }

    fn build_tuple(&mut self, count: u8) {
        let objs = self.pop_n(count);
        let tuple = PyObject::new_tuple(objs).to_ptr();
        self.push(tuple);
    }

    fn build_set(&mut self, count: u8) {
        let objs = self.pop_n(count);
        let set = PyObject::new_set(objs).to_ptr();
        self.push(set);
    }

    fn get_iter(&mut self) {
        let obj = self.pop();
        match obj.clone().get_ref().iter_py() {
            Some(iter) => self.push(iter.to_pyptr()),
            None => { 
                self.throw_err( PyException { 
                    error: PyError::TypeError,
                    msg: format!("TypeError: {:?} not iterable", obj),
                });
            }
        }
    }

    fn for_iter(&mut self, delta: u8) {
        let iter = self.top().clone();
        match &mut iter.clone().get_ref().obj {
            Obj::Iter(it) => {
                if let Some(item) = it.next() {
                    self.push(item);
                }
                else {
                    self.pop();
                    self.frame_mut().ip += delta as usize;
                }
            }
            e => {
                self.throw_err(PyException{
                    error: PyError::TypeError,
                    msg: format!("Instrustion: FOR_ITER expected iterator at top of stack not {:?}", e)
                });
            }
        }
    }

    fn unpack_sequence(&mut self) {
        let seq = self.pop();
        if let Some(iter) = seq.clone().get_ref().iter_py() {
            for o in iter.get_items() {
                self.push(o);
            }
        } else {
            panic!("Must be iterable sequence on top of stack");
        }
    }

    fn pop_jump_if_false(&mut self, delta: u8) {
        let frame = self.frame_mut();
        let cond = frame.stack.pop().unwrap();
        if !cond.get_ref().__bool__() {
            frame.ip += delta as usize;
        }
    }

    fn pop_jump_if_true(&mut self, delta: u8) {
        let frame = self.frame_mut();
        let cond = frame.stack.pop().unwrap();
        if cond.get_ref().__bool__() {
            frame.ip += delta as usize;
        }
    }

    fn jump_forward(&mut self, delta: u8) {
        self.frame_mut().ip += delta as usize;
    }

    fn jump_backward(&mut self, delta: u8) {
        self.frame_mut().ip -= delta as usize;
    }

    fn compare_op(&mut self, op: Op) {
        let rhs = self.frame_mut().stack.pop().unwrap();
        let lhs = self.frame_mut().stack.pop().unwrap();
        let res = PyObject::compare_op(&lhs.get_ref(), &rhs.get_ref(), &op);
        self.frame_mut().stack.push(res.to_pyptr());
    }

    fn binary_add(&mut self) {
        let rhs = self.frame_mut().stack.pop().unwrap();
        let lhs = self.frame_mut().stack.pop().unwrap();
        match PyObject::__add__(&lhs, &rhs) {
            Ok(v) => self.frame_mut().stack.push(v),
            Err(e) => panic!("{e}"),
        };
    }

    fn binary_subtract(&mut self) {
        let rhs = self.frame_mut().stack.pop().unwrap();
        let lhs = self.frame_mut().stack.pop().unwrap();
        match PyObject::__sub__(&lhs, &rhs) {
            Ok(v) => self.frame_mut().stack.push(v),
            Err(e) => panic!("{e}"),
        };
    }

    fn binary_multiply(&mut self) {
        let rhs = self.frame_mut().stack.pop().unwrap();
        let lhs = self.frame_mut().stack.pop().unwrap();
        match PyObject::__mul__(&lhs, &rhs) {
            Ok(v) => self.frame_mut().stack.push(v),
            Err(e) => panic!("{e}"),
        };
    }

    fn binary_divide(&mut self) {
        let rhs = self.frame_mut().stack.pop().unwrap();
        let lhs = self.frame_mut().stack.pop().unwrap();
        match PyObject::__div__(&lhs, &rhs) {
            Ok(v) => self.frame_mut().stack.push(v),
            Err(e) => panic!("{e}"),
        };
    }

    fn unary_negative(&mut self) {
        let v = self.frame_mut().stack.pop().unwrap();
        match PyObject::__neg__(&v) {
            Ok(o) => self.frame_mut().stack.push(o),
            Err(e) => panic!("{e}"),
        }
    }

    fn call_function(&mut self, argc: u8) {
        let args = self.pop_n(argc);
        let _self_or_null = self.pop();
        let func = self.pop();

        let fn_ref = func.get_ref();
        let fn_obj: Result<&FuncObj, &FnPtr> = match &fn_ref.obj {
            Obj::FunctionObj(f) => Ok(f),
            Obj::FuncPtr(ptr) => Err(ptr), // not error just using to see which one it is
            Obj::BuildClass => {
                let class = self.build_class(args);
                self.push(class.to_pyptr());
                return;
            }
            o => {
                self.throw_err(PyException { error: PyError::TypeError, 
                    msg: format!("Obj {:?} is not callable", o),
                });
                unreachable!();
            },
        };

        match fn_obj {
            Ok(func) => {
                let mut new_frame = PyFrame {
                    code: func.code.clone(),
                    ip: 0,
                    stack: Vec::new(),
                    locals: vec![self.null.clone(); func.code.varnames.len()],
                    globals: self.frame().globals.clone(),
                };

                for (i, arg) in args.into_iter().enumerate() {
                    new_frame.locals[i] = arg;
                }

                self.frames.push(new_frame);
            }
            Err(ptr) => {
                self.frame_mut().stack.push((ptr.ptr)(&args));
            }
        }
        //self.print_debug_info();
    }

    fn return_value(&mut self) {
        let ret = self.frame_mut().stack.pop().unwrap_or(self.null.clone());
        self.frames.pop();
        if let Some(f) = self.frames.last_mut() {
            f.stack.push(ret);
        }
    }

    fn call_intrinsic_1(&mut self, f: IntrinsicFunc) {
        let args = self.pop_until_null();
        self.frame_mut().stack.push(f.call(&args));
    }

    fn make_function(&mut self) {
        let code = match &self.frame_mut().stack.pop().unwrap().get_ref().obj {
            Obj::Code(c) => c.clone(),
            Obj::Type(t) => t.code.clone(),
            o => {
                self.throw_err(PyException {
                    error: PyError::TypeError,
                    msg: format!("MAKE_FUNCTION expects CodeObj not {:?}. Failed at {} {}", o, line!(), file!()),
                });
                unreachable!();
            }
        };

        let func = PyObject::new_function(FuncObj {
            code: code.into(),
        });

        self.frame_mut().stack.push(func.to_ptr());
    }

    fn load_build_class(&mut self) {
        self.push(PyObject::new_buildclass().to_ptr());
    }

    fn import_name(&mut self, namei: u8) {
        let name = match self.get_name(namei) {
            Some(n) => n,
            None => panic!(),
        };
        dbg!(&name);

        let filepath: String = self.working_dir.to_str().unwrap().to_owned() + "/" + &name + ".py";
        let module = match Interpreter::compile_file(&filepath) {
            Ok(m) => m,
            Err(e) => panic!("can't load module \'{}\': {}", &name, e),
        };
        let mod_obj = module.to_pyptr();
        let mod_name = name.clone();
        self.frame_mut().globals.lock().expect("unable to lock globals").insert(mod_name, mod_obj);
    }

    fn build_class(&mut self, args: Vec<PyObjPtr>) -> PyTypeObj {
        let code = match &(&args[0]).get_ref().obj {
            Obj::FunctionObj(c) => c.code.clone(),
            _ => panic!(),
        };
        let name = match &(&args[1]).get_ref().obj {
            Obj::Str(s) => s.clone(),
            _ => panic!(),
        };
        let mut fields: AttrDict = AttrDict::new();
        fields.insert("__name__".to_string(), name.clone().to_pyptr());
        let mut stack: Vec<PyObjPtr> = vec![];
        for bc in code.bytecode.iter().copied() {
            match bc {
                PyBytecode::Resume | PyBytecode::NOP => {},
                PyBytecode::LoadSmallInt(i) => stack.push(i.to_pyptr()),
                PyBytecode::LoadName(i) => stack.push(code.names[i as usize].clone().to_pyptr()),
                PyBytecode::LoadConst(i) => stack.push(code.consts[i as usize].clone()),
                PyBytecode::StoreName(i) => { fields.insert(code.names[i as usize].clone(), stack.pop().unwrap()); },
                // _ => panic!("Instruction not good for "),
                _ => {},
            }
        }

        PyTypeObj {
            name: name,
            static_attribs: fields,
            code: code,
        }
    }

}

#[allow(dead_code)]
fn no_instruction() {}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
#[repr(u8)]
pub enum IntrinsicFunc {
    Print,
    Input,
    Range,
    Exit,
}

impl IntrinsicFunc {

    pub const SHIFT_AMOUNT: u8 = 122;

    pub fn call(&self, args: &Vec<PyObjPtr>) -> PyObjPtr {
        match self {
            IntrinsicFunc::Print => IntrinsicFunc::print(args),
            IntrinsicFunc::Input => IntrinsicFunc::input(args),
            IntrinsicFunc::Range => IntrinsicFunc::range(args),
            IntrinsicFunc::Exit => IntrinsicFunc::exit(args),
        }
    }

    pub fn try_get<'a, T: AsRef<str>>(name: T) -> Option<IntrinsicFunc> {
        let func = match name.as_ref() {
            "print" => IntrinsicFunc::Print,
            "input" => IntrinsicFunc::Input,
            "range" => IntrinsicFunc::Range,
            "exit" => IntrinsicFunc::Exit,
            _ => return None,
        };
        Some(func)
    }

    pub fn get_funcptr(&self) -> PyObjPtr {
        match self {
            &IntrinsicFunc::Print => FnPtr{
                ptr: IntrinsicFunc::print, 
                name: "print".into(),
            }.to_pyptr(),
            &IntrinsicFunc::Input => FnPtr{
                    ptr: IntrinsicFunc::input, 
                    name: "input".into(),
                }.to_pyptr(),
            &IntrinsicFunc::Range => FnPtr{
                    ptr: IntrinsicFunc::range, 
                    name: "range".into(),
                }.to_pyptr(),
            &IntrinsicFunc::Exit => FnPtr{
                    ptr: IntrinsicFunc::exit, 
                    name: "exit".into(),
                }.to_pyptr(),
        }
    }
 
    fn print(objs: &Vec<PyObjPtr>) -> PyObjPtr {
        for o in objs {
            print!("{} ", o.get_ref().__str__());
        }
        println!();
        PyObject::none()
    }

    fn input(words: &Vec<PyObjPtr>) -> PyObjPtr {
        if words.len() != 1 {
            panic!();
        }
        print!("{}", words.first().unwrap().get_ref().__str__());
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        (input.trim().to_string()).to_pyptr()
    }

    fn range(limits: &Vec<PyObjPtr>) -> PyObjPtr {
        let (start, end, inc) = {
            let s = match limits.get(0) {
                Some(o) => o.get_ref().__integer__(),
                None => None,
            };
            let e = match limits.get(1) {
                Some(o) => o.get_ref().__integer__(),
                None => None,
            };
            let i = match limits.get(2) {
                Some(o) => o.get_ref().__integer__(),
                None => None,
            };
            (s, e, i)
        };

        let r = RangeObj::from(start, end, inc);
        PyObject::new_range(r).to_ptr()

        //let objs = r.to_vec();
        //Some(objs.to_arc())
    }

    fn exit(args: &Vec<PyObjPtr>) -> PyObjPtr {
        let mut exit_code = 0;
        if let Some(code) = args.first() {
            exit_code = code.get_ref().__int__() as i32;
        }
        std::process::exit(exit_code);
    }

    pub const fn from_u8(value: u8) -> IntrinsicFunc {
        unsafe { std::mem::transmute(value) }
    }

    pub fn new_builtins() -> HashMap<String, PyObjPtr> {
        let mut map = HashMap::new();
        map.insert(
            "print".to_string(),
            FnPtr{
                ptr: IntrinsicFunc::print, 
                name: "print".into(),
            }.to_pyptr()
        );
        map.insert( 
            "input".to_string(),
            FnPtr{
                ptr: IntrinsicFunc::input, 
                name: "input".into(),
            }.to_pyptr()
        );
        map.insert( 
            "range".to_string(),
            FnPtr{
                ptr: IntrinsicFunc::range, 
                name: "range".into(),
            }.to_pyptr()
        );
        map.insert(
            "exit".to_string(),
            FnPtr{
                ptr: IntrinsicFunc::exit, 
                name: "exit".into(),
            }.to_pyptr(),
        );
        map
    }

}

impl std::convert::From<u8> for IntrinsicFunc {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
