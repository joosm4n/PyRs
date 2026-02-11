
use crate::{
    pyrs_bytecode::PyBytecode,
    pyrs_obj::{Obj, ToObj, PyObj},
};

use std::{ 
    collections::HashMap, 
    rc::Rc, 
    sync::{Arc, Mutex},
    ops::{Deref, DerefMut}
};

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

impl FuncObj {

    pub fn serialize(&self, indent: usize) -> String {
        let mut tabs = String::new();
        for _ in 0..indent {
            tabs.push('\t');
        }

        let mut contents = String::new();
        contents.push_str(&format!("{tabs}<funcobj>\n"));
        contents.push_str(&format!("{tabs}\t{}\n", self.code.serialize(indent + 1)));

        contents.push_str(&format!("{tabs}globals:\n"));
        for (key, val) in self.globals.as_ref() {
            contents.push_str(&format!("{tabs}\t{}: {}\n", key, val.__repr__()));
        }
        
        contents.push_str(&format!("{tabs}closure:\n{tabs}\t{:?}\n", self.closure));
        return contents;
    }
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

    pub fn serialize(&self, indent: usize) -> String
    {
        let mut tabs = String::new();
        for _ in 0..indent {
            tabs.push('\t');
        }

        let mut contents = String::from(&tabs);

        contents.push_str(&format!("{tabs}<codeobj {}>\n", &self.name));
        contents.push_str(&format!("{tabs}consts:\n"));
        for c in &self.consts {
            match c {
                Obj::Code(code) => contents.push_str(&format!("{tabs}\t{}\n", code.serialize(indent + 1))),
                _ => contents.push_str(&format!("{tabs}\t{}\n", c)),
            }
        }

        contents.push_str(&format!("{tabs}names:\n{tabs}\t"));
        for n in &self.names {
            contents.push_str(&format!("{}, ", n));
        }

        contents.push_str(&format!("\n{tabs}bytecode:\n"));
        contents.push_str(&format!("{}", &PyBytecode::to_string(&self.bytecode)));
        contents.push_str(&format!("{tabs}<codeobj {}>\n", &self.name));
        return contents;
    }
}

#[derive(Debug, Clone)]
pub struct CompileCtx
{
    name: String,
    bytecode: Vec<PyBytecode>,
    consts: Vec<Obj>,
    names: Vec<String>,
    varnames: Vec<String>,
}

impl CompileCtx {

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bytecode: vec![],
            consts: vec![],
            names: vec![],
            varnames: vec![],
        }
    }

    pub fn add_const(&mut self, obj: Obj) -> usize {
        if let Some(i) = self.consts.iter().position(|o| o == &obj) {
            i
        } else {
            let i = self.consts.len();
            self.consts.push(obj);
            i
        }
    }

    pub fn add_name(&mut self, name: String) -> usize {
        if let Some(i) = self.names.iter().position(|n| n == &name) {
            i
        } else {
            let i = self.names.len();
            self.names.push(name);
            i
        }
    }

    pub fn extract_code(self) -> Vec<PyBytecode> {
        self.bytecode
    }

    pub fn finish(self) -> CodeObj {
        CodeObj {
            name: self.name,
            bytecode: self.bytecode,
            consts: self.consts,
            names: self.names,
            varnames: self.varnames,
        }
    }

    pub fn serialize(&self, indent: usize) -> String
    {
        self.clone().finish().serialize(indent)
    } 

}

impl Deref for CompileCtx {
    type Target = Vec<PyBytecode>;
    fn deref(&self) -> &Self::Target {
        &self.bytecode
    }
}

impl DerefMut for CompileCtx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytecode
    }
}


