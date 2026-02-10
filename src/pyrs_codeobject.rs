
use crate::{
    pyrs_bytecode::PyBytecode,
    pyrs_obj::{Obj, ToObj},
};

use std::{
    sync::{Arc, Mutex},
    rc::{Rc},
    cell::{RefCell},
    collections::HashMap,
};

pub type Cell = Rc<RefCell<Obj>>;

#[derive(Debug, Clone, PartialEq)]
enum CodeFlags {
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeObj 
{
    pub name: String,
    pub bytecode: Vec<PyBytecode>,
    pub consts: Vec<Obj>,
    pub names: Vec<String>,
    pub varnames: Vec<String>,
}

impl CodeObj {
    pub fn new(name: &str, code: Vec<PyBytecode>) -> Self {
        CodeObj {
            name: name.to_string(),
            bytecode: code,
            consts: vec![],
            names: vec![],
            varnames: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PyFrame
{
    pub code: Arc<CodeObj>,
    pub ip: usize,
    pub stack: Vec<Arc<Obj>>,
    pub locals: Vec<Arc<Obj>>,
}

#[derive(Debug, Clone)]
pub struct FuncObj
{
    pub code: Arc<CodeObj>,
    pub globals: Arc<HashMap<String, Arc<Obj>>>,
    pub closure: Vec<Arc<Mutex<Arc<Obj>>>>, // captured cells
}

impl ToObj for FuncObj {
    fn to_arc(self) -> Arc<Obj> {
        self.to_obj().into()
    }
    fn to_obj(self) -> Obj {
        Obj::Func(self)
    }
}