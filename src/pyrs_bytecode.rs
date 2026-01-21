
use crate::{
    pyrs_error::{PyError, PyException}, pyrs_obj::{Obj, PyObj, ToObj}, pyrs_parsing::{Expression, Keyword, Op}
};

use std::{
    collections::{HashMap}, sync::Arc
};

    // Format: offset INSTRUCTION argument (value)
    // 0 LOAD_CONST 0 (0)      # Load constant at index 0, which is the integer 0
    // 2 STORE_NAME 0 (i)      # Store the top stack value into variable name at index 0 (variable "i")

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

    BuildList = 181,
    GetIter = 182, 
    ForIter = 183,
    ListAppend = 184,
    BuildMap = 185,
    

    // not proper
    Error(String) = 254,
}

// todo!("Make expressions fold in on themselves in multiline things");

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
                    Op::Equals => PyBytecode::NOP,

                    Op::Eq | Op::Neq |
                    Op::LessEq | Op::LessThan | 
                    Op::GreaterEq | Op::GreaterThan => PyBytecode::CompareOp(op),

                    e => PyBytecode::Error(format!("{e}")), 
                });
            }
            Expression::Func(ptr, args) => {
                match ptr.name.as_str() {
                    _ => { // normal function
                        let argc = args.len();
                        queue.push(PyBytecode::LoadConst(Obj::Function(ptr)));
                        for a in args {
                            PyBytecode::from_expr(a, queue);
                        }
                        queue.push(PyBytecode::CallFunction(argc));
                        queue.push(PyBytecode::PopTop);
                        return;
                    }
                };
            }
            Expression::Keyword(keyword, cond, args) => {
                match keyword {
                    Keyword::If => {
                        for c in cond {
                            PyBytecode::from_expr(c, queue);
                        }
                        let mut temp_queue: Vec<PyBytecode> = vec![];
                        for a in args {
                            PyBytecode::from_expr(a, &mut temp_queue);
                        }
                        
                        let delta = temp_queue.len();
                        queue.append(&mut temp_queue);
                        queue.push(PyBytecode::PopJumpIfFalse(delta))
                    }
                    Keyword::While => {
                        let condition_start = queue.len();
                        let mut condition_code = vec![];
                        for c in cond {
                            PyBytecode::from_expr(c, &mut condition_code);
                        }
                        for inst in condition_code.iter() {
                            queue.push(inst.clone());
                        }
                        
                        let mut contents_code: Vec<PyBytecode> = vec![];
                        for a in args {
                            PyBytecode::from_expr(a, &mut contents_code);
                        }

                        let delta = contents_code.len() + 1;
                        queue.push(Self::PopJumpIfFalse(delta)); // skip entire while loop
                        
                        queue.append(&mut contents_code);

                        let return_delta = queue.len() - condition_start + 1;
                        queue.push(PyBytecode::JumpBackward(return_delta));

                    }
                    _ => panic!(),
                }
            }
            Expression::Definition(name, args, ret_val) => {

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

pub struct PyVM
{
    vars: HashMap<String, Arc<Obj>>,
    stack: Vec<Arc<Obj>>,

    instruction_queue: Vec<PyBytecode>,
    instruction_counter: usize,
    error_state: bool,

    inst_array: [fn(); 255],
}

#[allow(dead_code)]
impl PyVM
{
    pub fn new() -> Self {
        PyVM {
            vars: HashMap::new(),
            stack: vec![],
            instruction_queue: vec![],
            instruction_counter: 0,
            error_state: false,
            inst_array: PyVM::get_fn_array(), 
        }
    }

    pub fn execute(&mut self, queue: Vec<PyBytecode>)
    {
        self.instruction_queue = queue;
        while let Some(instruction) = self.instruction_queue.get(self.instruction_counter)
        {
            self.execute_instruction(instruction.clone());
        }
    }

    fn execute_instruction(&mut self, inst: PyBytecode)
    {
        //println!("Executing: {:?}", inst);
        match inst {
            PyBytecode::PopTop => self.pop_top(),
            PyBytecode::EndFor => self.end_for(),

            PyBytecode::LoadConst(obj) => self.push(Arc::from(obj)),
            PyBytecode::LoadFast(name) => self.load_fast(name),
            PyBytecode::LoadName(name) => self.load_fast(name),
            PyBytecode::StoreFast(name) => self.store_fast(name),
            PyBytecode::StoreName(name) => self.store_name(name),

            PyBytecode::BinaryAdd => self.binary_add(),
            PyBytecode::BinarySubtract => self.binary_subtract(),
            PyBytecode::BinaryMultiply => self.binary_multiply(),
            PyBytecode::BinaryDivide => self.binary_divide(),

            PyBytecode::CallFunction(argc) => self.call_function(argc),
            PyBytecode::CallInstrinsic1(ptr) => self.call_intrisic_1(ptr),

            PyBytecode::PopJumpIfFalse(delta) => self.pop_jump_if_false(delta),
            PyBytecode::JumpBackward(delta) => self.jump_backward(delta),

            PyBytecode::CompareOp(op) => self.compare_op(op),

            PyBytecode::NOP => {},
            _ => panic!("{:?} not implemented ", inst),
        }
        if self.error_state {
            self.throw();
        }
        self.instruction_counter += 1;
    }

    fn push_err(&mut self, e: PyException)
    {
        self.stack.push(e.to_arc());
        self.error_state = true;
    }

    fn throw(&mut self)
    {
        let e = self.pop();
        println!();
        println!("---- PyVM Error ---- ");
        
        println!("Error at bytecode number: {}", self.instruction_counter);
        println!("{e}");

        println!();
        panic!();
    }

    fn push(&mut self, obj: Arc<Obj>)
    {
        self.stack.push(obj);
    }
    
    fn pop(&mut self) -> Arc<Obj> 
    {
        match self.stack.pop() {
            Some(obj) => obj,
            None => {
                let e = PyException { error: PyError::StackError, msg: "Tried to pop empty stack".to_string() };
                self.push_err(e);
                self.throw();
                unreachable!();
            }
        }
    }

    fn get(&self, stack_index: usize) -> Arc<Obj>
    {
        let actual_index= self.stack.len() - 1 - stack_index; 
        // len = 5, actual_index = 3, input_index = 1
        match self.stack.get(actual_index) {
            Some(obj) => obj.clone(),
            None => {
                let stack_size = self.stack.len();
                PyException{ 
                    error: PyError::StackError, 
                    msg: format!("Tried to get object off stack with invalid index: {actual_index}, when stack size was: {stack_size}", )
                }.to_arc()
            }
        }
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

    fn top(&self) -> Arc<Obj>
    {
        self.stack.last().unwrap().clone()
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
        self.vars.insert(name, obj);
    }

    fn store_name(&mut self, name: String)
    {
        let obj = self.pop();
        self.vars.insert(name, obj);
    }

    fn load_fast(&mut self, name: String)
    {
        let obj = match self.vars.get(&name) {
            Some(val) => val.clone(),
            None => PyException { error: PyError::UndefinedVariableError, msg: format!("No variable with name: \"{}\" in current scope", name)}.to_arc(),
        };
        self.push(obj);
    }

    fn call_function(&mut self, argc: usize)
    {
        let args = self.pop_n(argc);
        
        let func = self.pop();
        match func.as_ref() {
            Obj::Function(fn_ptr) => {
                let ret = (fn_ptr.ptr)(&args);
                self.push(ret);
            }
            _ => self.push_err(PyException { error: PyError::TypeError, msg: format!("{argc} element of stack: {func} was not a Obj::Function ")}),
        }
    }

    fn pop_jump_if_false(&mut self, delta: usize)
    {
        let cond = self.pop();
        if cond.__bool__() {
            self.instruction_counter += delta;
        }
    }

    fn pop_jump_if_true(&mut self, delta: usize)
    {
        let cond = self.pop();
        if !cond.__bool__() {
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
        let lhs = self.pop();
        let rhs = self.pop();
        let cond = PyObj::compare_op(&lhs, &rhs, &op);
        //dbg!(&rhs, &lhs, &op, &cond);
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

    fn call_intrisic_1(&mut self, ptr: IntrinsicFunc) 
    {
        let obj = self.pop();
        let ret = match ptr {
            IntrinsicFunc::Print => IntrinsicFunc::print(&obj),
        };
        match ret {
            Some(val) => self.push(Arc::from(val)),
            None => {},
        }
    }

    fn get_fn_array() -> [fn(); 255]
    {
        let mut a: [fn(); 255] = [no_instruction as fn(); 255];
        
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

fn other_fn()
{

}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum IntrinsicFunc
{
    Print,
}

impl IntrinsicFunc 
{
    fn print(obj: &Obj) -> Option<Obj>
    {
        println!("{}", obj);
        None
    }


}