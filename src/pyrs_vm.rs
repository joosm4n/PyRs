use std::{
    collections::HashMap,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
    usize,
};

use crate::{
    pyrs_bytecode::PyBytecode,
    pyrs_codeobject::{CodeObj, FuncObj, PyFrame},
    pyrs_error::{PyError, PyException},
    pyrs_interpreter::Interpreter,
    pyrs_obj::{Obj, PyObj, ToObj},
    pyrs_parsing::Op,
    pyrs_std::RangeObj,
    pyrs_userclass::CustomClass,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PyVM {
    builtins: HashMap<String, Arc<Obj>>,
    globals: HashMap<String, Arc<Obj>>,
    curr_namespace: String,

    frames: Vec<PyFrame>,

    error_state: bool,

    debug_mode: bool,

    null: Arc<Obj>,

    working_dir: PathBuf,
}

#[allow(dead_code)]
impl PyVM {
    pub fn new() -> Self {
        PyVM {
            builtins: HashMap::new(),
            globals: HashMap::new(),
            curr_namespace: String::from(""),
            frames: vec![],
            error_state: false,
            debug_mode: false,
            null: Obj::Null.into(),
            working_dir: std::env::current_dir().unwrap_or(PathBuf::new()),
        }
    }

    pub fn set_debug_mode(&mut self, debug: bool) {
        self.debug_mode = debug;
    }

    pub fn execute(&mut self, code: CodeObj) {
        println!("Working in: {:?}", self.working_dir);

        self.frames.push(PyFrame {
            code: Arc::new(code),
            ip: 0,
            stack: vec![],
            locals: vec![],
        });

        if self.debug_mode {
            self.print_instruction_queue();
        }

        loop {
            let frame = self.frame_mut();
            if frame.ip >= frame.code.bytecode.len() {
                break;
            }

            let instr = frame.code.bytecode[frame.ip].clone();
            frame.ip += 1;
            self.execute_instruction(instr);

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
                "Executing: ({})   {:?}\nStack for ({}):\n",
                self.frame().ip,
                &inst,
                self.frame().ip
            );

            self.print_stack();
        }

        match inst {
            PyBytecode::PopTop => self.pop_top(),
            PyBytecode::EndFor => self.end_for(),

            PyBytecode::LoadConst(namei) => self.load_const(namei),
            PyBytecode::LoadFast(i) => self.load_fast(i),
            PyBytecode::StoreFast(i) => self.store_fast(i),
            PyBytecode::LoadName(namei) => self.load_name(namei),
            PyBytecode::StoreName(namei) => self.store_name(namei),

            PyBytecode::PushNull => self.push_null(),

            PyBytecode::BuildList(len) => self.build_list(len),
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

            PyBytecode::LoadDeref(namei) => self.load_deref(namei),

            PyBytecode::PopJumpIfFalse(delta) => self.pop_jump_if_false(delta),
            PyBytecode::PopJumpIfTrue(delta) => self.pop_jump_if_true(delta),
            PyBytecode::JumpForward(delta) => self.jump_forward(delta),
            PyBytecode::JumpBackward(delta) => self.jump_backward(delta),

            PyBytecode::CompareOp(op) => self.compare_op(op),

            PyBytecode::MakeFunction => self.make_function(),

            PyBytecode::LoadBuildClass => self.load_build_class(),
            PyBytecode::ImportName(namei) => self.import_name(namei),

            PyBytecode::NOP => {}
            _ => panic!("Instruction {:?} not implemented ", inst),
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
        self.push(e.to_arc());
        self.error_state = true;
    }

    fn print_debug_info(&mut self) {
        self.print_instruction_queue();
        let frame = self.frames.last().unwrap();
        println!(
            "Curr Instruction: \n({}) \t{}",
            frame.ip, frame.code.bytecode[frame.ip]
        );

        println!("\nStack Trace: ");
        self.print_stack();

        println!("\nVariableMaps: \n{:#?}", self.frame().locals);
    }

    fn throw(&mut self) {
        let ip = self.frame_mut().ip;
        let e = self.pop();
        println!();
        println!("---- PyVM Error ---- \n");

        println!("Error: at bytecode instruction {}", ip,);

        self.print_instruction(ip);
        println!("\n{e}");

        self.print_debug_info();

        println!();
        panic!("\n ^^^ PyVM Error Thrown ^^^ \n");
    }

    fn push(&mut self, obj: Arc<Obj>) {
        //self.local_stacks.last_mut().unwrap().push(obj);
        self.frame_mut().stack.push(obj);
    }

    fn pop(&mut self) -> Arc<Obj> {
        match self.frame_mut().stack.pop() {
            Some(obj) => obj,
            None => {
                let e = PyException {
                    error: PyError::StackError,
                    msg: "Tried to pop empty stack".to_string(),
                };
                self.push_err(e);
                self.throw();
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

    fn pop_n(&mut self, count: usize) -> Vec<Arc<Obj>> {
        let mut objs = vec![];
        for _ in 0..count {
            objs.push(self.pop());
        }
        objs.reverse();
        objs
    }

    fn pop_n_or(&mut self, count: usize, or: Arc<Obj>) -> Vec<Arc<Obj>> {
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

    fn pop_until(&mut self, stop_obj: &Arc<Obj>) -> Vec<Arc<Obj>> {
        let mut objs = vec![];
        while self.top() != stop_obj {
            objs.push(self.pop());
        }

        objs.reverse();
        objs
    }

    fn pop_until_null(&mut self) -> Vec<Arc<Obj>> {
        let mut objs = vec![];
        loop {
            if self.top().as_ref() == self.null.as_ref() {
                break;
            }
            objs.push(self.pop());
        }
        objs.reverse();
        objs
    }

    fn top(&self) -> &Arc<Obj> {
        self.frames.last().unwrap().stack.last().unwrap()
    }

    pub fn print_stack(&mut self) {
        for (idx, a) in self.frame_mut().stack.iter().enumerate() {
            println!(" [{:?}] \t{}", idx, a.__str__());
        }
        println!();
    }

    fn print_instruction(&mut self, index: usize) {
        let inst_queue = &self.frame_mut().code.bytecode;
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
        self.frame_mut().stack.pop().unwrap();
    }

    fn end_for(&mut self) {
        self.pop();
    }

    fn copy(&mut self, n: usize) {
        let frame = self.frame_mut();
        let val = frame.stack[frame.stack.len() - 1 - n].clone();
        frame.stack.push(val);
    }

    fn swap(&mut self, n: usize) {
        let frame = self.frame_mut();
        let len = frame.stack.len();
        frame.stack.swap(len - 1, len - 1 - n);
    }

    fn load_const(&mut self, i: usize) {
        let obj = self.frame_mut().code.consts[i].clone();
        self.frame_mut().stack.push(Arc::new(obj));
    }

    fn store_fast(&mut self, i: usize) {
        let val = self.frame_mut().stack.pop().unwrap();
        self.frame_mut().locals[i] = val;
    }

    fn load_fast(&mut self, namei: usize) {
        let val = self.frame_mut().locals[namei].clone();
        self.frame_mut().stack.push(val);
    }

    fn store_name(&mut self, namei: usize) {
        let name = self.frame_mut().code.names[namei].clone();
        let val = self.frame_mut().stack.pop().unwrap();
        self.globals.insert(name, val);
    }

    fn load_name(&mut self, i: usize) {
        let name = self.frame().code.names[i].clone();

        if let Some(v) = self.globals.get(&name).cloned() {
            self.frame_mut().stack.push(v);
        } else if let Some(v) = self.builtins.get(&name).cloned() {
            self.frame_mut().stack.push(v);
        } else {
            panic!("NameError: {name}");
        }
    }

    fn push_null(&mut self) {
        self.push(self.null.clone());
    }

    fn build_list(&mut self, len: usize) {
        let objs = self.pop_n(len);
        let list = objs.to_arc();
        self.push(list);
    }

    fn build_tuple(&mut self, count: usize) {
        let objs = self.pop_n(count);
        let tuple = Arc::from(Obj::Tuple(objs));
        self.push(tuple);
    }

    fn build_set(&mut self, count: usize) {
        let objs = self.pop_n(count);
        let set = Arc::from(Obj::Set(objs));
        self.push(set);
    }

    fn get_iter(&mut self) {
        let obj = self.frame_mut().stack.pop().unwrap();
        match obj.iter_py() {
            Some(i) => self.frame_mut().stack.push(Obj::Iter(i).into()),
            None => panic!("TypeError: not iterable"),
        }
    }

    fn for_iter(&mut self, delta: usize) {
        let iter = self.frame_mut().stack.pop().unwrap();
        if let Obj::Iter(mut it) = iter.as_ref().clone() {
            if let Some(item) = it.next() {
                self.frame_mut().stack.push(Obj::Iter(it).into());
                self.frame_mut().stack.push(item);
            } else {
                self.frame_mut().ip += delta;
            }
        } else {
            panic!("FOR_ITER expected iterator");
        }
    }

    fn unpack_sequence(&mut self) {
        let seq = self.pop();
        if let Some(iter) = seq.iter_py() {
            for o in iter.get_items() {
                self.push(o);
            }
        } else {
            panic!("Must be iterable sequence on top of stack");
        }
    }

    fn pop_jump_if_false(&mut self, delta: usize) {
        let frame = self.frame_mut();
        let cond = frame.stack.pop().unwrap();
        if !cond.__bool__() {
            frame.ip += delta;
        }
    }

    fn pop_jump_if_true(&mut self, delta: usize) {
        let frame = self.frame_mut();
        let cond = frame.stack.pop().unwrap();
        if cond.__bool__() {
            frame.ip += delta;
        }
    }

    fn jump_forward(&mut self, delta: usize) {
        self.frame_mut().ip += delta;
    }

    fn jump_backward(&mut self, delta: usize) {
        self.frame_mut().ip -= delta;
    }

    fn compare_op(&mut self, op: Op) {
        let rhs = self.frame_mut().stack.pop().unwrap();
        let lhs = self.frame_mut().stack.pop().unwrap();
        let res = Obj::compare_op(&lhs, &rhs, &op);
        self.frame_mut().stack.push(res.to_arc());
    }

    fn binary_add(&mut self) {
        let rhs = self.frame_mut().stack.pop().unwrap();
        let lhs = self.frame_mut().stack.pop().unwrap();
        match Obj::__add__(&lhs, &rhs) {
            Ok(v) => self.frame_mut().stack.push(v),
            Err(e) => panic!("{e}"),
        }
    }

    fn binary_subtract(&mut self) {
        let rhs = self.frame_mut().stack.pop().unwrap();
        let lhs = self.frame_mut().stack.pop().unwrap();
        match Obj::__sub__(&lhs, &rhs) {
            Ok(v) => self.frame_mut().stack.push(v),
            Err(e) => panic!("{e}"),
        };
    }

    fn binary_multiply(&mut self) {
        let rhs = self.frame_mut().stack.pop().unwrap();
        let lhs = self.frame_mut().stack.pop().unwrap();
        match Obj::__mul__(&lhs, &rhs) {
            Ok(v) => self.frame_mut().stack.push(v),
            Err(e) => panic!("{e}"),
        };
    }

    fn binary_divide(&mut self) {
        let rhs = self.frame_mut().stack.pop().unwrap();
        let lhs = self.frame_mut().stack.pop().unwrap();
        match Obj::__div__(&lhs, &rhs) {
            Ok(v) => self.frame_mut().stack.push(v),
            Err(e) => panic!("{e}"),
        };
    }

    fn unary_negative(&mut self) {
        let v = self.frame_mut().stack.pop().unwrap();
        match Obj::__neg__(&v) {
            Ok(o) => self.frame_mut().stack.push(o),
            Err(e) => panic!("{e}"),
        }
    }

    fn call_function(&mut self, argc: usize) {
        let call_pos = self.frame().stack.len() - argc;
        let args = self.frame_mut().stack.split_off(call_pos);

        let func = self.frame_mut().stack.pop().unwrap();

        let func = match func.as_ref() {
            Obj::Func(f) => f.clone(),
            o => panic!("Obj {:?} is not callable", o),
        };

        let mut new_frame = PyFrame {
            code: func.code.clone(),
            ip: 0,
            stack: Vec::new(),
            locals: vec![self.null.clone(); func.code.varnames.len()],
        };

        for (i, arg) in args.into_iter().enumerate() {
            new_frame.locals[i] = arg;
        }

        self.frames.push(new_frame);
    }

    fn return_value(&mut self) {
        let ret = self.frame_mut().stack.pop().unwrap_or(self.null.clone());

        self.frames.pop();

        if let Some(f) = self.frames.last_mut() {
            f.stack.push(ret);
        }
    }

    fn call_intrinsic_1(&mut self, f: IntrinsicFunc) {
        let arg = self.frame_mut().stack.pop().unwrap();
        if let Some(v) = f.call(&vec![arg]) {
            self.frame_mut().stack.push(v);
        }
    }

    fn make_function(&mut self) {
        let code = match self.frame_mut().stack.pop().unwrap().as_ref() {
            Obj::Code(c) => c.clone(),
            _ => panic!("MAKE_FUNCTION expects CodeObj"),
        };

        let func = Obj::Func(FuncObj {
            code: code.into(),
            globals: self.globals.clone().into(),
            closure: vec![],
        });

        self.frame_mut().stack.push(func.into());
    }

    fn load_deref(&mut self, field: usize) {
        let obj = self.pop();
        let ret = match obj.__dict__(&field.to_string()) {
            Some(o) => o.clone(),
            None => PyException {
                error: PyError::UndefinedVariableError,
                msg: format!("No variable with name \'{field}\' in obj {obj}"),
            }
            .to_arc(),
        };
        self.push(ret);
    }

    fn load_build_class(&mut self) {
        panic!();
    }

    fn import_name(&mut self, namei: usize) {
        self.load_name(namei);
        let name = self.pop().__str__();

        let filepath: String = self.working_dir.to_str().unwrap().to_owned() + "/" + &name + ".py";
        let module = match Interpreter::compile_file(&filepath) {
            Ok(m) => m,
            Err(e) => panic!("can't load module \'{}\': {}", &name, e),
        };
    }
}

#[allow(dead_code)]
fn no_instruction() {}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum IntrinsicFunc {
    Print,
    Input,
    Range,
}

impl IntrinsicFunc {
    pub fn call(&self, args: &Vec<Arc<Obj>>) -> Option<Arc<Obj>> {
        match self {
            IntrinsicFunc::Print => IntrinsicFunc::input(args),
            IntrinsicFunc::Input => IntrinsicFunc::print(args),
            IntrinsicFunc::Range => IntrinsicFunc::range(args),
        }
    }

    pub fn try_get(name: &str) -> Option<IntrinsicFunc> {
        let func = match name {
            "print" => IntrinsicFunc::Print,
            "input" => IntrinsicFunc::Input,
            "range" => IntrinsicFunc::Range,
            _ => return None,
        };
        Some(func)
    }

    fn print(objs: &Vec<Arc<Obj>>) -> Option<Arc<Obj>> {
        for o in objs {
            print!("{} ", o);
        }
        println!();
        None
    }

    fn input(words: &Vec<Arc<Obj>>) -> Option<Arc<Obj>> {
        if words.len() != 1 {
            panic!();
        }
        print!("{}", words.first().unwrap().__str__());
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        Some(Obj::Str(input.trim().to_string()).into())
    }

    fn range(limits: &Vec<Arc<Obj>>) -> Option<Arc<Obj>> {
        let (start, end, inc) = {
            let s = match limits[0].as_ref() {
                Obj::Int(i) => Some(i.clone()),
                _ => None,
            };
            let e = match limits[1].as_ref() {
                Obj::Int(i) => Some(i.clone()),
                _ => None,
            };
            let i = match limits[2].as_ref() {
                Obj::Int(i) => Some(i.clone()),
                _ => None,
            };
            (s, e, i)
        };

        let r = RangeObj::from(start, end, inc);
        let objs = r.to_vec();
        Some(objs.to_arc())
    }
}
