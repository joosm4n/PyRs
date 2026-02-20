use crate::{
    pyrs_bytecode::PyBytecode,
    pyrs_obj::{Obj, PyObj, ToObj},
};

use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CodeObj {
    pub name: String,
    pub bytecode: Vec<PyBytecode>,
    pub consts: Vec<Obj>,
    pub names: Vec<String>,
    pub varnames: Vec<String>,
    pub num_consts: usize,
    pub num_varnames: usize,
    pub num_names: usize,
}

impl CodeObj {
    pub fn new(name: &str, code: Vec<PyBytecode>) -> Self {
        CodeObj {
            name: name.to_string(),
            bytecode: code,
            consts: vec![],
            names: vec![],
            varnames: vec![],
            num_consts: 0,
            num_names: 0,
            num_varnames: 0,
        }
    }

    pub fn print_nice(&self) {
        println!("Name: \t{}", self.name);
        // println!("Filename: {}");
        // println!("Arg count: {}");
        // println!("Pos only args {}");
        // println!("Kw-only args {}");
        println!("Number of locals: \t{}", self.num_varnames);
        // println!("Stack size: {}");
        // println!("Flags: {}");

        println!("Constants:");
        for (i, c) in self.consts.iter().enumerate() {
            println!("\t{i}: {}", c.__str__());
        }

        println!("Names:");
        for (i, s) in self.names.iter().enumerate() {
            println!("\t{i}: {}", s);
        }

        println!("Variable names:");
        for (i, v) in self.varnames.iter().enumerate() {
            println!("\t{i}: {}", v);
        }

    }

    pub fn serialize(&self, indent: usize) -> String {
        let mut tabs = String::new();
        for _ in 0..indent {
            tabs.push('\t');
        }

        let mut contents = String::from(&tabs);

        contents.push_str(&format!("{tabs}<codeobj {}>\n", &self.name));
        contents.push_str(&format!("{tabs}consts:\n"));
        for (i, c) in self.consts.iter().enumerate() {
            match c {
                Obj::Code(code) => {
                    contents.push_str(&format!("{tabs}\t[{i}] {}\n", code.serialize(indent + 1)))
                }
                _ => contents.push_str(&format!("{tabs}\t{}\n", c)),
            }
        }

        contents.push_str(&format!("{tabs}names:\n{tabs}\t"));
        for n in &self.names {
            contents.push_str(&format!("{}, ", n));
        }

        contents.push_str(&format!("\n{tabs}bytecode:\n"));
        contents.push_str(&self.get_inst_string());

        contents.push_str(&format!("{tabs}<end {}>\n", &self.name));
        return contents;
    }

    fn get_inst_string(&self) -> String {
        let mut bytecode_string = String::new();
        for (idx, line) in self.bytecode.iter().enumerate() {
            bytecode_string.push_str(&format!("({idx}) \t\t{:?}", line).as_str());

            let arg: Option<String> = match line {
                PyBytecode::CallFunction(v) | PyBytecode::LoadConst(v) => Some(v.to_string()),

                PyBytecode::JumpBackward(v) => Some(format!("to {}", idx - *v as usize)),

                PyBytecode::LoadFast(v)
                | PyBytecode::StoreFast(v)
                | PyBytecode::LoadName(v)
                | PyBytecode::StoreName(v)
                | PyBytecode::LoadGlobal(v)
                | PyBytecode::StoreGlobal(v) => {
                    if let Some(name) = self.names.get(*v as usize) {
                        Some(name.clone())
                    } else if let Some(name) = self.varnames.get(*v as usize) {
                        Some(name.clone())
                    } else {
                        panic!()
                    }
                }
                _ => None,
            };
            if let Some(val) = arg {
                bytecode_string.push_str(&format!("\t\t({})\n", val));
            } else {
                bytecode_string.push_str(&format!("\n",));
            }
        }
        bytecode_string
    }
}

#[derive(Debug, Clone)]
pub struct CompileCtx {
    name: String,
    bytecode: Vec<PyBytecode>,
    consts: Vec<Obj>,
    names: Vec<String>,
    varnames: Vec<String>,
}

impl CompileCtx {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            bytecode: vec![PyBytecode::Resume],
            consts: vec![],
            names: vec![],
            varnames: vec![],
        }
    }

    pub fn add_const(&mut self, obj: Obj) -> u8 {
        if let Some(i) = self.consts.iter().position(|o| o == &obj) {
            i as u8
        } else {
            let i = self.consts.len();
            self.consts.push(obj);
            i as u8
        }
    }

    pub fn load_const(&mut self, obj: Obj) {
        if let Some(i) = self.consts.iter().position(|o| o == &obj) {
            self.push(PyBytecode::LoadConst(i as u8));
        } else {
            let i = self.consts.len();
            self.consts.push(obj);
            self.push(PyBytecode::LoadConst(i as u8));
        }
    }

    pub fn add_name(&mut self, name: String) -> u8 {
        if let Some(i) = self.names.iter().position(|n| n == &name) {
            i as u8
        } else if let Some(i) = self.varnames.iter().position(|n| n == &name) {
            i as u8
        } else {
            let i = self.varnames.len();
            self.varnames.push(name);
            i as u8
        }
    }

    pub fn add_name_load(&mut self, name: String) -> PyBytecode {
        if let Some(i) = self.names.iter().position(|n| n == &name) {
            PyBytecode::LoadName(i as u8)
        } else if let Some(i) = self.varnames.iter().position(|n| n == &name) {
            PyBytecode::LoadFast(i as u8)
        } else {
            let i = self.varnames.len();
            self.varnames.push(name);
            PyBytecode::LoadFast(i as u8)
        }
    }

    pub fn add_name_store(&mut self, name: String) -> PyBytecode {
        if let Some(i) = self.names.iter().position(|n| n == &name) {
            PyBytecode::StoreName(i as u8)
        } else if let Some(i) = self.varnames.iter().position(|n| n == &name) {
            PyBytecode::StoreFast(i as u8)
        } else {
            let i = self.varnames.len();
            self.varnames.push(name);
            PyBytecode::StoreFast(i as u8)
        }
    }

    pub fn add_global(&mut self, _name: String, _obj: Obj) -> u8 {
        0
    }

    pub fn extract_code(self) -> Vec<PyBytecode> {
        self.bytecode
    }

    pub fn finish(self) -> CodeObj {
        let n_c = self.consts.len();
        let n_v = self.varnames.len();
        let n_n = self.names.len();
        CodeObj {
            name: self.name,
            bytecode: self.bytecode,
            consts: self.consts,
            names: self.names,
            varnames: self.varnames,
            num_consts: n_c,
            num_names: n_n,
            num_varnames: n_v,
        }
    }

    pub fn serialize(&self, indent: usize) -> String {
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

impl core::hash::Hash for FuncObj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code.as_ref().hash(state);
        self.globals.hash(state);
        let clsre: Vec<Arc<Obj>> = self
            .closure
            .iter()
            .map(|x| x.lock().unwrap().clone())
            .collect();
        clsre.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct FuncObj {
    pub code: Arc<CodeObj>,
    pub globals: Vec<Arc<Obj>>,
    pub closure: Vec<Arc<Mutex<Arc<Obj>>>>, // captured cells
}

impl ToObj for FuncObj {
    fn to_arc(self) -> Arc<Obj> {
        self.to_obj().into()
    }
    fn to_obj(self) -> Obj {
        Obj::FunctionObj(self)
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
        for val in &self.globals {
            contents.push_str(&format!("{tabs}\t{}\n", val.__repr__()));
        }

        contents.push_str(&format!("{tabs}closure:\n{tabs}\t{:?}\n", self.closure));
        return contents;
    }
}

#[derive(Debug, Clone)]
pub struct ClassObj {
    pub name: String,
    pub code: Arc<CodeObj>,
    pub fields: Vec<Arc<Obj>>,
}

impl core::hash::Hash for ClassObj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.code.as_ref().hash(state);
        self.fields.hash(state);
    }
}
