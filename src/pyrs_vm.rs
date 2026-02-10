
use std::{
    collections::HashMap,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
    usize,
};

use crate::{
    pyrs_bytecode::PyBytecode,
    pyrs_error::{PyError, PyException},
    pyrs_interpreter::Interpreter,
    pyrs_obj::{Obj, PyObj, ToObj},
    pyrs_parsing::Op,
    pyrs_std::RangeObj,
    pyrs_userclass::{CustomClass},
    pyrs_codeobject::{PyFrame, FuncObj},
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PyVM {
    builtins: HashMap<String, Arc<Obj>>,
    globals: HashMap<String, Arc<Obj>>,
    var_maps: Vec<HashMap<String, Arc<Obj>>>,
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
            var_maps: vec![HashMap::new()],
            curr_namespace: String::from(""),
            error_state: false,
            debug_mode: false,
            null: Obj::Null.into(),
            working_dir: std::env::current_dir().unwrap_or(PathBuf::new()),
        }
    }

    pub fn set_debug_mode(&mut self, debug: bool) {
        self.debug_mode = debug;
    }

    pub fn execute(&mut self, bytecode: Vec<PyBytecode>) {
        println!("Working in: {:?}", self.working_dir);
        self.instruction_queue = bytecode;
        if self.debug_mode {
            self.print_instruction_queue();
        }
        while let Some(instruction) = self.instruction_queue.get(self.instruction_counter) {
            self.execute_instruction(instruction.clone());
        }
    }

    fn execute_instruction(&mut self, inst: PyBytecode) {
        if inst == PyBytecode::NOP {
            self.instruction_counter += 1;
            return;
        }

        if self.debug_mode {
            println!("Executing: ({})   {:?}", self.instruction_counter, inst);
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

            PyBytecode::LoadDeref(name) => self.load_deref(&name),

            PyBytecode::PopJumpIfFalse(delta) => self.pop_jump_if_false(delta),
            PyBytecode::PopJumpIfTrue(delta) => self.pop_jump_if_true(delta),
            PyBytecode::JumpForward(delta) => self.jump_forward(delta),
            PyBytecode::JumpBackward(delta) => self.jump_backward(delta),

            PyBytecode::CompareOp(op) => self.compare_op(op),

            PyBytecode::MakeFunction => self.make_function(),

            PyBytecode::LoadBuildClass => self.load_build_class(),
            PyBytecode::ImportFrom(name) => self.import_from(&name),
            PyBytecode::ImportName(name) => self.import_name(&name),

            PyBytecode::NOP => {}
            _ => panic!("Instruction {:?} not implemented ", inst),
        }
        if self.error_state {
            self.throw();
        }
        self.instruction_counter += 1;
    }

    pub fn get_vars(&self) -> &Vec<HashMap<String, Arc<Obj>>> {
        &self.var_maps
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

    fn print_debug_info(&self) {
        self.print_instruction_queue();
        println!(
            "Curr Instruction: \n({}) \t{}",
            self.instruction_counter, self.instruction_queue[self.instruction_counter]
        );

        println!("\nStack Trace: ");
        self.print_stack();

        println!("\nVariableMaps: ");
        self.get_vars();
    }

    fn throw(&mut self) {
        let e = self.pop();
        println!();
        println!("---- PyVM Error ---- \n");

        println!(
            "Error: at bytecode instruction {}",
            self.instruction_counter
        );
        self.print_instruction(self.instruction_counter);
        println!("\n{e}");

        self.print_debug_info();

        println!();
        panic!("\n ^^^ PyVM Error Thrown ^^^ \n");
    }

    fn push(&mut self, obj: Arc<Obj>) {
        self.local_stacks.last_mut().unwrap().push(obj);
    }

    fn pop(&mut self) -> Arc<Obj> {
        match self.local_stacks.last_mut().unwrap().pop() {
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

    fn get_local_vars(&self) -> &HashMap<String, Arc<Obj>> {
        return self.var_maps.last().unwrap();
    }

    fn get_local_vars_mut(&mut self) -> &mut HashMap<String, Arc<Obj>> {
        return self.var_maps.last_mut().unwrap();
    }

    fn frame(&mut self) -> &mut PyFrame {
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
            if let Some(obj) = self.get_local_stack_mut().pop() {
                objs.push(obj);
            } else {
                objs.push(or.clone().into());
            }
        }
        objs.reverse();
        objs
    }

    fn pop_until(&mut self, stop_obj: &Obj) -> Vec<Arc<Obj>> {
        let mut objs = vec![];
        while self.top().as_ref() != stop_obj {
            objs.push(self.pop());
        }

        objs.reverse();
        objs
    }

    fn pop_until_null(&mut self) -> Vec<Arc<Obj>> {
        let mut objs = vec![];
        while self.top().as_ref() != self.null_obj.as_ref() {
            objs.push(self.pop());
        }
        objs.reverse();
        objs
    }

    fn top(&self) -> Arc<Obj> {
        self.local_stacks.last().unwrap().last().unwrap().clone()
    }

    pub fn print_stack(&self) {
        for (idx, a) in self.local_stacks.iter().enumerate() {
            println!(" ({:?}) \t{:?}", idx, a);
        }
        println!();
    }

    pub fn print_var_maps(&self) {
        for (idx, a) in self.var_maps.iter().enumerate() {
            println!(" ({:?}) \t{:?}", idx, a);
        }
        println!();
    }

    pub fn view_stack(&self) -> &Vec<Vec<Arc<Obj>>> {
        return &self.local_stacks;
    }

    fn print_instruction(&self, index: usize) {
        if index < self.instruction_queue.len() {
            println!("({}) \t\t{}", index, self.instruction_queue[index]);
        }
    }

    fn print_instruction_queue(&self) {
        println!("\nInstructions: ");
        println!("{}", PyBytecode::to_string(&self.instruction_queue));
    }

    // -------------- Instructions ----------------
    fn pop_top(&mut self) {
        self.frame().stack.pop().unwrap();
    }

    fn end_for(&mut self) {
        self.pop();
    }

    fn copy(&mut self, n: usize) {
        let frame = self.frame();
        let val = frame.stack[frame.stack.len() - 1 - n].clone();
        frame.stack.push(val);
    }

    fn swap(&mut self, n: usize) {
        let frame = self.frame();
        let len = frame.stack.len();
        frame.stack.swap(len - 1, len - 1 - n);
    }

    fn load_const(&mut self, i: usize) {
        let c = self.frame().code.consts[i].clone();
        self.frame().stack.push(Arc::new(c));
    }

    fn store_fast(&mut self, i: usize) {
        let val = self.frame().stack.pop().unwrap();
        self.frame().locals[i] = val;
    }

    fn load_fast(&mut self, namei: usize) {
        let val = self.frame().locals[namei].clone();
        self.frame().stack.push(val);
    }

    fn store_name(&mut self, namei: usize) {
        let name = self.frame().code.names[namei].clone();
        let val = self.frame().stack.pop().unwrap();
        self.globals.insert(name, val);
    }

    fn load_name(&mut self, i: usize) {
        let name = &self.frame().code.names[i];

        if let Some(v) = self.globals.get(name) {
            self.frame().stack.push(v.clone());
        } else if let Some(v) = self.builtins.get(name) {
            self.frame().stack.push(v.clone());
        } else {
            panic!("NameError: {name}");
        }
    }

    fn push_null(&mut self) {
        self.push(self.null_obj.clone());
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
        let obj = self.frame().stack.pop().unwrap();
        match obj.iter_py() {
            Some(i) => self.frame().stack.push(Obj::Iter(i).into()),
            None => panic!("TypeError: not iterable"),
        }
    }

    fn for_iter(&mut self, delta: usize) {
        let iter = self.frame().stack.pop().unwrap();

        if let Obj::Iter(mut it) = iter.as_ref().clone() {
            if let Some(item) = it.next() {
                self.frame().stack.push(Obj::Iter(it).into());
                self.frame().stack.push(item);
            } else {
                self.frame().ip += delta;
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
        let frame = self.frame();
        let cond = frame.stack.pop().unwrap();
        if !cond.__bool__() {
            frame.ip += delta;
        }
    }

    fn pop_jump_if_true(&mut self, delta: usize) {
        let frame = self.frame();
        let cond = frame.stack.pop().unwrap();
        if cond.__bool__() {
            frame.ip += delta;
        }
    }

    fn jump_forward(&mut self, delta: usize) {
        self.frame().ip += delta;
    }

    fn jump_backward(&mut self, delta: usize) {
        self.frame().ip -= delta;
    }

    fn compare_op(&mut self, op: Op) {
        let rhs = self.frame().stack.pop().unwrap();
        let lhs = self.frame().stack.pop().unwrap();
        let res = Obj::compare_op(&lhs, &rhs, &op);
        self.frame().stack.push(res.to_arc());
    }

    fn binary_add(&mut self) {
        let rhs = self.frame().stack.pop().unwrap();
        let lhs = self.frame().stack.pop().unwrap();
        match Obj::__add__(&lhs, &rhs) {
            Ok(v) => self.frame().stack.push(v),
            Err(e) => panic!("{e}"),
        }
    }

    fn binary_subtract(&mut self) {
        let rhs = self.frame().stack.pop().unwrap();
        let lhs = self.frame().stack.pop().unwrap();
        match Obj::__sub__(&lhs, &rhs) {
            Ok(v) => self.frame().stack.push(v),
            Err(e) => panic!("{e}"),
        };
    }

    fn binary_multiply(&mut self) {
        let rhs = self.frame().stack.pop().unwrap();
        let lhs = self.frame().stack.pop().unwrap();
        match Obj::__mul__(&lhs, &rhs) {
            Ok(v) => self.frame().stack.push(v),
            Err(e) => panic!("{e}"),
        };
    }

    fn binary_divide(&mut self) {
        let rhs = self.frame().stack.pop().unwrap();
        let lhs = self.frame().stack.pop().unwrap();
        match Obj::__div__(&lhs, &rhs) {
            Ok(v) => self.frame().stack.push(v),
            Err(e) => panic!("{e}"),
        };
    }

    fn unary_negative(&mut self) {
        let v = self.frame().stack.pop().unwrap();
        match Obj::__neg__(&v) {
            Ok(o) => self.frame().stack.push(o),
            Err(e) => panic!("{e}"),
        }
    }

    fn call_function(&mut self, argc: usize) {
       let args = self.frame().stack.split_off(
            self.frame().stack.len() - argc
        );

        let func = self.frame().stack.pop().unwrap();

        let func = match func.as_ref() {
            Obj::Func(f) => f.clone(),
            _ => panic!("Not callable"),
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
        let ret = self.frame()
            .stack
            .pop()
            .unwrap_or(self.null.clone());

        self.frames.pop();

        if let Some(f) = self.frames.last_mut() {
            f.stack.push(ret);
        }
    }

    fn call_intrinsic_1(&mut self, f: IntrinsicFunc) {
        let arg = self.frame().stack.pop().unwrap();
        if let Some(v) = f.call(vec![arg]) {
            self.frame().stack.push(v);
        }
    }

    fn make_function(&mut self) {
        let code = match self.frame().stack.pop().unwrap().as_ref() {
            Obj::Code(c) => c.clone(),
            _ => panic!("MAKE_FUNCTION expects CodeObj"),
        };

        let func = Obj::Func(FuncObj {
            code: code.into(),
            globals: self.globals.clone().into(),
            closure: vec![],
        });

        self.frame().stack.push(func.into());
    }

    fn load_deref(&mut self, field: &String) {
        let obj = self.pop();
        let ret = match obj.__dict__(field) {
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

    fn import_from(&mut self, name: &str) {
        let filepath: String = self.working_dir.to_str().unwrap().to_owned() + name + ".py";
        let module = match Interpreter::compile_module(&filepath) {
            Ok(m) => m,
            Err(e) => {
                self.push_err(e);
                self.throw();
                unreachable!();
            }
        };

        let name = module.name.clone();
        self.push(Arc::new(Obj::Module(module)));
        self.store_name(name);
    }

    fn import_name(&mut self, name: &str) {
        let filepath: String = self.working_dir.to_str().unwrap().to_owned() + "/" + name + ".py";
        let module = match Interpreter::compile_module(&filepath) {
            Ok(m) => m,
            Err(e) => {
                self.push_err(e);
                self.throw();
                unreachable!();
            }
        };

        let name = module.name.clone();
        self.push(Arc::new(Obj::Module(module)));
        self.store_name(name);
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
