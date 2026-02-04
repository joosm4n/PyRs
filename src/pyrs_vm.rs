
use std::{
    collections::HashMap, io::{self, Write}, sync::Arc, usize, ops::{AddAssign},
};

use rug::Integer;

use crate::{
    pyrs_error::{PyError, PyException}, pyrs_obj::{Obj, PyObj, ToObj}, pyrs_parsing::{Op}, pyrs_bytecode::{PyBytecode}
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PyVM
{
    var_maps: Vec<HashMap<String, Arc<Obj>>>,

    funcs: HashMap<String, usize>,
    local_stacks: Vec<Vec<Arc<Obj>>>,

    instruction_queue: Vec<PyBytecode>,
    instruction_counter: usize,
    error_state: bool,

    debug_mode: bool,
}

#[allow(dead_code)]
impl PyVM
{
    pub fn new() -> Self {
        PyVM {
            var_maps: vec![HashMap::new()],
            funcs: HashMap::new(),
            local_stacks: vec![Vec::new()],
            instruction_queue: vec![],
            instruction_counter: 0,
            error_state: false,
            debug_mode: false,
        }
    }

    pub fn set_debug_mode(&mut self, debug: bool)
    {
        self.debug_mode = debug;
    }

    pub fn execute(&mut self, queue: Vec<PyBytecode>)
    {
        self.instruction_queue = queue;
        if self.debug_mode {
            self.print_instruction_queue();
        }
        while let Some(instruction) = self.instruction_queue.get(self.instruction_counter)
        {
            self.execute_instruction(instruction.clone());
        }
    }

    fn execute_instruction(&mut self, inst: PyBytecode)
    {
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

            PyBytecode::LoadConst(obj) => self.push(Arc::from(obj)),
            PyBytecode::LoadFast(name) => self.load_fast(name),
            PyBytecode::LoadName(name) => self.load_fast(name),
            PyBytecode::StoreFast(name) => self.store_fast(name),
            PyBytecode::StoreName(name) => self.store_name(name),

            PyBytecode::BuildList(len) => self.build_list(len),
            PyBytecode::BuildTuple(count) => self.build_tuple(count),

            PyBytecode::GetIter => self.get_iter(),
            PyBytecode::ForIter(delta) => self.for_iter(delta),

            PyBytecode::BinaryAdd => self.binary_add(),
            PyBytecode::BinarySubtract => self.binary_subtract(),
            PyBytecode::BinaryMultiply => self.binary_multiply(),
            PyBytecode::BinaryDivide => self.binary_divide(),
            
            PyBytecode::UnaryNegative => self.unary_negative(),

            PyBytecode::CallFunction(argc) => self.call_function(argc),
            PyBytecode::CallInstrinsic1(ptr) => self.call_intrinsic_1(ptr),

            PyBytecode::PopJumpIfFalse(delta) => self.pop_jump_if_false(delta),
            PyBytecode::PopJumpIfTrue(delta) => self.pop_jump_if_true(delta),
            PyBytecode::JumpForward(delta) => self.jump_forward(delta),
            PyBytecode::JumpBackward(delta) => self.jump_backward(delta),

            PyBytecode::CompareOp(op) => self.compare_op(op),

            PyBytecode::MakeFunction => self.make_function(),
            PyBytecode::NewStack => self.push_stack(),
            PyBytecode::DestroyStack => self.pop_stack(),

            PyBytecode::ReturnValue => self.return_value(),
            
            PyBytecode::NOP => {},
            _ => panic!("Instruction {:?} not implemented ", inst),
        }
        if self.error_state {
            self.throw();
        }
        self.instruction_counter += 1;
    }

    pub fn get_vars(&self) -> &Vec<HashMap<String, Arc<Obj>>>
    {
        &self.var_maps
    } 

    fn push_err(&mut self, e: PyException)
    {
        self.push(e.to_arc());
        self.error_state = true;
    }

    fn print_debug_info(&self) 
    {
        self.print_instruction_queue();
        println!("Curr Instruction: \n{}", self.instruction_queue[self.instruction_counter]);

        println!("\nStack Trace: ");
        self.print_stack();
    }

    fn throw(&mut self)
    {
        let e = self.pop();
        println!();
        println!("---- PyVM Error ---- ");
        
        println!("Error: at bytecode instruction {}", self.instruction_counter);
        self.print_instruction(self.instruction_counter);
        println!("{e}");

        self.print_debug_info();

        println!();
        panic!("PyVM Error Thrown");
    }

    fn push(&mut self, obj: Arc<Obj>)
    {
        self.local_stacks.last_mut().unwrap().push(obj);
    }
    
    fn pop(&mut self) -> Arc<Obj> 
    {
        match self.local_stacks.last_mut().unwrap().pop() {
            Some(obj) => obj,
            None => {
                let e = PyException { error: PyError::StackError, msg: "Tried to pop empty stack".to_string() };
                self.push_err(e);
                self.throw();
                unreachable!();
            }
        }
    }

    fn get_local_vars(&self) -> &HashMap<String, Arc<Obj>>
    {
        return self.var_maps.last().unwrap();
    }

    fn get_local_vars_mut(&mut self) -> &mut HashMap<String, Arc<Obj>>
    {
        return self.var_maps.last_mut().unwrap();
    }

    fn get_local_stack(&self) -> &Vec<Arc<Obj>>
    {
        return self.local_stacks.last().unwrap();
    }

    fn get_local_stack_mut(&mut self) -> &mut Vec<Arc<Obj>>
    {
        return self.local_stacks.last_mut().unwrap();
    }
    
    fn pop_n(&mut self, count: usize) -> Vec<Arc<Obj>>
    {
        let mut objs = vec![];
        for _ in 0..count {
            objs.push(self.pop());
        }
        objs.reverse();
        objs
    }

    fn pop_n_or(&mut self, count: usize, or: Obj) -> Vec<Arc<Obj>>
    {
        let mut objs = vec![];
        for _ in 0..count {
            if let Some(obj) = self.get_local_stack_mut().pop() {
                objs.push(obj);
            }
            else {
                objs.push(or.clone().into());
            }
        }
        objs.reverse();
        objs
    }

    fn pop_until(&mut self, stop_obj: &Obj) -> Vec<Arc<Obj>>
    {
        let mut objs = vec![];
        while self.top().as_ref() != stop_obj {
            objs.push(self.pop());
        }

        objs.reverse();
        objs
    }

    fn top(&self) -> Arc<Obj>
    {
        self.local_stacks.last().unwrap().last().unwrap().clone()
    }

    pub fn print_stack(&self)
    {
        println!("VM Stack:");
        for (idx, a) in self.local_stacks.iter().enumerate() {
            println!(" ({:?}) \t{:?}", idx, a);
        }
        println!();
    }

    pub fn view_stack(&self) -> &Vec<Vec<Arc<Obj>>>
    {
        return &self.local_stacks;
    }

    fn print_instruction(&self, index: usize)
    {
        if index < self.instruction_queue.len() {
            println!("\t ({}) \t{}", index, self.instruction_queue[index]);
        }
    }

    fn print_instruction_queue(&self) 
    {
        println!("Instructions: ");
        println!("{}", PyBytecode::to_string(&self.instruction_queue));
    }

    // Instructions
    fn pop_top(&mut self)
    {
        self.pop();
    }

    fn end_for(&mut self)
    {
        self.pop();
    }

    fn store_fast(&mut self, name: String)
    {
        let obj = self.pop();
        self.get_local_vars_mut().insert(name, obj);
    }

    fn store_name(&mut self, name: String)
    {
        let obj = self.pop();
        self.get_local_vars_mut().insert(name, obj);
    }

    fn load_fast(&mut self, name: String)
    {
        match self.get_local_vars().get(&name) {
            Some(val) => self.push(val.clone()),
            None => { 
                self.push_err(PyException { error: PyError::UndefinedVariableError, msg: format!("No variable with name: \"{}\" in current scope", name)})
            },
        };
        
    }

    fn build_list(&mut self, len: usize)
    {
        let objs = self.pop_n(len);
        let list = Arc::from(Obj::List(objs));
        self.push(list);
    }

    fn build_tuple(&mut self, count: usize)
    {
        let objs = self.pop_n(count);
        let tuple = Arc::from(Obj::Tuple(objs));
        self.push(tuple);
    }

    fn build_set(&mut self, count: usize)
    {
        let objs = self.pop_n(count);
        let set = Arc::from(Obj::Set(objs));
        self.push(set);
    }

    fn get_iter(&mut self)
    {
        let obj = self.pop();
        let iter = match obj.iter() {
            Some(i) => Obj::Iter(i),
            None => {
                Obj::Except(
                    PyException { 
                        error: PyError::TypeError, 
                        msg: format!("Obj {} not iterable", obj),
                    }
                )
            }
        };
        self.push(iter.into())
    }

    fn for_iter(&mut self, delta: usize)
    {
        let top = self.pop();
        match top.as_ref() {
            Obj::Iter(iter) => {
                let mut iter_clone = iter.clone();
                match iter_clone.next() {
                    Some(item) => {
                        self.push(Arc::from(Obj::Iter(iter_clone)));
                        self.push(item);
                    }
                    None => {
                        self.instruction_counter += delta;
                    }
                }
            },
            _ => {
                self.push_err(PyException {
                    error: PyError::TypeError, 
                    msg: format!("FOR_ITER expected iterator, found {}", top), 
                });
            }
        };

    }

    fn pop_jump_if_false(&mut self, delta: usize)
    {
        let cond = self.pop();
        if !cond.__bool__() {
            self.instruction_counter += delta;
        }
    }

    fn pop_jump_if_true(&mut self, delta: usize)
    {
        let cond = self.pop();
        if cond.__bool__() {
            self.instruction_counter += delta;
        }
    }

    fn jump_forward(&mut self, delta: usize)
    {
        self.instruction_counter += delta;
    }

    fn jump_backward(&mut self, delta: usize)
    {
        self.instruction_counter -= delta;
    }

    fn compare_op(&mut self, op: Op)
    {
        let rhs = self.pop();
        let lhs = self.pop();
        let cond = Obj::compare_op(&lhs, &rhs, &op);
        // dbg!(&rhs, &lhs, &op, &cond);
        self.push(cond.to_arc());
    }

    fn binary_add(&mut self)
    {
        let rhs = self.pop();
        let lhs = self.pop();
        match Obj::__add__(&lhs, &rhs) {
            Ok(val) => self.push(Arc::from(val)),
            Err(e) => self.push_err(e),
        }
    }

    fn binary_subtract(&mut self)
    {
        let rhs = self.pop();
        let lhs = self.pop();
        match Obj::__sub__(&lhs, &rhs) {
            Ok(val) => self.push(val),
            Err(e) => self.push_err(e),
        };
    }

    fn binary_multiply(&mut self)
    {
        let rhs = self.pop();
        let lhs = self.pop();
        let ret = match Obj::__mul__(&lhs, &rhs) {
            Ok(val) => val,
            Err(e) => {
                println!("{e}");
                e.to_arc()
            }
        };
        self.push(Arc::from(ret));
    }

    fn binary_divide(&mut self)
    {
        let rhs = self.pop();
        let lhs = self.pop();
        let ret = match Obj::__div__(&lhs, &rhs) {
            Ok(val) => val,
            Err(e) => {
                println!("{e}");
                e.to_arc()
            }
        };
        self.push(Arc::from(ret));
    }

    fn unary_negative(&mut self)
    {
        let obj = self.pop();
        match Obj::__neg__(&obj) {
            Ok(o) => self.push(o),
            Err(e) => self.push_err(e),
        }
    }

    fn call_function(&mut self, argc: usize)
    {
        let func = self.pop();
        let args = self.pop_n_or(argc, Obj::None);

        self.push_stack();
        let return_addr = self.instruction_counter + 1;
        self.push(Obj::Int(return_addr.into()).into()); // return pos pointer
        
        for a in args {
            self.push(a);
        }

        match self.funcs.get(&func.__str__()) {
            Some(addr) => {
                self.instruction_counter = *addr;
                //println!("Calling: {}", func.__str__());
            }
            None => { 
                self.push_err( PyException {
                    error: PyError::SyntaxError,
                    msg: format!("not a name of a func: {}", func.__str__())
                });
            },
        }
    }

    fn return_value(&mut self)
    {
        let mut last= Obj::None.into();
        let fn_stack = self.get_local_stack_mut();
        while let Some(obj) = fn_stack.pop() {
            last = obj;
        }
        if last.as_ref() == &Obj::None { 
            self.push_err(PyException { 
                error: PyError::StackError,
                msg: "Must have already popped the return pointer ".to_string()
            });
            self.throw(); 
        }
        self.instruction_counter = last.__int__() as usize;
        self.pop_stack();
    }

    fn call_intrinsic_1(&mut self, ptr: IntrinsicFunc) 
    {
        let args = self.pop_until(&Obj::None);
        self.pop();

        let ret = match ptr {
            IntrinsicFunc::Print => IntrinsicFunc::print(&args),
            IntrinsicFunc::Input => IntrinsicFunc::input(&args),
            IntrinsicFunc::Range => IntrinsicFunc::range(&args),
        };
        match ret {
            Some(val) => {
                self.push(Arc::from(val));
            },
            None => {},
        }
    }

    fn make_function(&mut self)
    {
        let addr = self.pop();
        let name = self.pop();
        
        self.funcs.insert(name.__str__(), addr.__int__() as usize);
    }

    fn push_stack(&mut self)
    {
        self.var_maps.push(HashMap::new());
        self.local_stacks.push(vec![]);
    }

    fn pop_stack(&mut self)
    {
        self.var_maps.pop();
        self.local_stacks.pop();
    }

    #[allow(dead_code)]
    fn get_fn_array() -> [fn(); 255]
    {
        let a: [fn(); 255] = [no_instruction as fn(); 255];
        
        /* 

        // Empty
        a[u8::from(PyBytecode::NOP) as usize] = no_instruction as fn(); 

        a[u8::from(PyBytecode::PopTop) as usize] = other_fn as fn();
        a[u8::from(PyBytecode::Copy) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::Swap) as usize] = no_instruction as fn();

        a[u8::from(PyBytecode::UnaryNegative) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::UnaryNot) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::UnaryInvert) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::ToBool) as usize] = no_instruction as fn();

        a[u8::from(PyBytecode::BinaryOp) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::BinaryAdd) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::BinaryMultiply) as usize] = no_instruction as fn(); 
        a[u8::from(PyBytecode::BinarySubtract) as usize] = no_instruction as fn(); 
        a[u8::from(PyBytecode::BinaryDivide) as usize] = no_instruction as fn();

        a[u8::from(PyBytecode::LoadConst) as usize] = no_instruction as fn(); 
        a[u8::from(PyBytecode::LoadFast) as usize] = no_instruction as fn();  
        a[u8::from(PyBytecode::StoreFast) as usize] = no_instruction as fn(); 
        a[u8::from(PyBytecode::LoadName) as usize] = no_instruction as fn();  
        a[u8::from(PyBytecode::StoreName) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::LoadGlobal) as usize] = no_instruction as fn(); 
        a[u8::from(PyBytecode::StoreGlobal) as usize] = no_instruction as fn();

        a[u8::from(PyBytecode::CallFunction) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::CallInstrinsic1) as usize] = no_instruction as fn(); 
        a[u8::from(PyBytecode::CallInstrinsic2) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::ReturnValue) as usize] = no_instruction as fn();

        a[u8::from(PyBytecode::PopJumpIfFalse) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::PopJumpIfTrue) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::JumpForward) as usize] = no_instruction as fn(); 
        a[u8::from(PyBytecode::JumpBackward) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::JumpIfFalse) as usize] = no_instruction as fn(); 
        a[u8::from(PyBytecode::JumpAbsolute) as usize] = no_instruction as fn();

        a[u8::from(PyBytecode::CompareOp) as usize] = no_instruction as fn();

        a[u8::from(PyBytecode::MakeFunction) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::BuildList) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::GetIter) as usize] = no_instruction as fn(); 
        a[u8::from(PyBytecode::ForIter) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::ListAppend) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::BuildMap) as usize] = no_instruction as fn();
        a[u8::from(PyBytecode::BinaryXOR) as usize] = no_instruction as fn();

        a[u8::from(PyBytecode::Error) as usize] = no_instruction as fn();

        */
        return a;
    }

}

fn no_instruction() 
{

}


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum IntrinsicFunc
{
    Print, Input, Range,
}

impl IntrinsicFunc
{
    pub fn try_get(name: &str) -> Option<IntrinsicFunc>
    {
        let func = match name {
            "print" => IntrinsicFunc::Print,
            "input" => IntrinsicFunc::Input,
            "range" => IntrinsicFunc::Range,
            _ => return None, 
        };
        Some(func)
    }

    fn print(objs: &Vec<Arc<Obj>>) -> Option<Arc<Obj>>
    {
        for o in objs {
            print!("{} ", o);
        }
        println!();
        None
    }

    fn input(words: &Vec<Arc<Obj>>) -> Option<Arc<Obj>>
    {
        if words.len() != 1 {
            panic!();
        }
        print!("{}", words.first().unwrap().__str__());
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("error: unable to read user input");
        Some(Obj::Str(input.trim().to_string()).into())
    }

    fn range(limits: &Vec<Arc<Obj>>) -> Option<Arc<Obj>> 
    {
        let (mut start, end, mut inc): (Integer, Integer, Integer) = match limits.len() {
            1 => {
                let e = match limits[0].as_ref() { Obj::Int(i) => i.clone(), _ => return None };
                (Integer::from(0), e, Integer::from(1))
            }
            2 => {
                let s = match limits[0].as_ref() { Obj::Int(i) => i.clone(), _ => return None };
                let e = match limits[1].as_ref() { Obj::Int(i) => i.clone(), _ => return None };
                (s, e, Integer::from(1))
            }
            3 => {
                let s = match limits[0].as_ref() { Obj::Int(i) => i.clone(), _ => return None };
                let e = match limits[1].as_ref() { Obj::Int(i) => i.clone(), _ => return None };
                let i = match limits[2].as_ref() { Obj::Int(i) => i.clone(), _ => return None };
                (s, e, i)
            }
            _ => return None,
        };

        if start < end {
            inc = Integer::from(-1);
        }

        let parts = (start.clone() - end.clone()).abs() / inc.clone();

        let mut objs = Vec::new();
        for _ in 0..parts.to_u64_wrapping() {
            objs.push(Obj::Int(start.clone()).into());
            start.add_assign(inc.clone());
        }

        Some(Obj::List(objs).into())
    }

}