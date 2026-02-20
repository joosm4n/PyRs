use crate::{
    pyrs_bytecode::PyBytecode,
    pyrs_obj::{Obj, ToObj}, pyrs_utils::PyUtils,
};

use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
    collections::{HashMap},
};

#[derive(Debug, Clone, PartialEq)]
pub struct PyCodeObj {
    pub name: String,
    pub bytecode: Vec<PyBytecode>,
    pub consts: Vec<Obj>,
    pub names: Vec<String>,
    pub varnames: Vec<String>,
    pub num_consts: usize,
    pub num_varnames: usize,
    pub num_names: usize,
    pub globals: HashMap<String, Arc<Obj>>,
}

impl PyCodeObj {
    pub fn new(name: &str, code: Vec<PyBytecode>) -> Self {
        PyCodeObj {
            name: name.to_string(),
            bytecode: code,
            consts: vec![],
            names: vec![],
            varnames: vec![],
            num_consts: 0,
            num_names: 0,
            num_varnames: 0,
            globals: HashMap::new(),
        }
    }

    pub fn pretty_format(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("Name: \t{}\n", self.name));
        // s.push_str(&format!("Filename: {}");
        // s.push_str(&format!("Arg count: {}");
        // s.push_str(&format!("Pos only args {}");
        // s.push_str(&format!("Kw-only args {}");
        s.push_str(&format!("Number of locals: \t{}\n", self.num_varnames));
        // s.push_str(&format!("Stack size: {}");
        // s.push_str(&format!("Flags: {}");

        s.push_str(&format!("Constants:\n"));
        for (i, c) in self.consts.iter().enumerate() {
            s.push_str(&format!("\t{i}: {}\n", c.__str__()));
        }

        s.push_str(&format!("Names:\n"));
        for (i, n) in self.names.iter().enumerate() {
            s.push_str(&format!("\t{i}: {}\n", n));
        }

        s.push_str(&format!("Variable names:\n"));
        for (i, v) in self.varnames.iter().enumerate() {
            s.push_str(&format!("\t{i}: {}\n", v));
        }
        s
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
pub struct PyCompileCtx {
    name: String,
    bytecode: Vec<PyBytecode>,
    consts: Vec<Obj>,
    names: Vec<String>,
    varnames: Vec<String>,
    globals: HashMap<String, Arc<Obj>>,
}

impl PyCompileCtx {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            bytecode: vec![PyBytecode::Resume],
            consts: vec![],
            names: vec![],
            varnames: vec![],
            globals: HashMap::new(),
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

    pub fn add_varname<T: Into<String>>(&mut self, name: T) -> u8 {
        let name_s: String = name.into();
        if let Some(i) = self.varnames.iter().position(|n| n == &name_s) {
            i as u8
        } else {
            let i = self.varnames.len();
            self.varnames.push(name_s);
            i as u8
        }
    }

    pub fn add_name<T: Into<String>>(&mut self, name: T) -> u8 {
        let name_s: String = name.into();
        if let Some(i) = self.names.iter().position(|n| n == &name_s) {
            i as u8
        } else {
            let i = self.names.len();
            self.names.push(name_s);
            i as u8
        }
    }

    pub fn add_varname_load<T: Into<String>>(&mut self, name: T) -> PyBytecode {
        let name_s: String = name.into();
        if let Some(_) = self.globals.get(&name_s) {
            let i = self.names.iter().position(|n| n == &name_s).unwrap();
            PyBytecode::LoadGlobal(i as u8)
        } else if let Some(i) = self.varnames.iter().position(|n| n == &name_s) {
            PyBytecode::LoadName(i as u8)
        } else {
            let i = self.varnames.len();
            self.varnames.push(name_s);
            PyBytecode::LoadFast(i as u8)
        }
    }

    pub fn add_name_load<T: Into<String>>(&mut self, name: T) -> PyBytecode {
        let name_s: String = name.into();
        if let Some(_) = self.globals.get(&name_s) {
            let i = self.names.iter().position(|n| n == &name_s).unwrap();
            PyBytecode::LoadGlobal(i as u8)
        } else if let Some(i) = self.names.iter().position(|n| n == &name_s) {
            PyBytecode::LoadName(i as u8)
        } else {
            let i = self.names.len();
            self.varnames.push(name_s);
            PyBytecode::LoadName(i as u8)
        }
    }

    pub fn add_varname_store<T: Into<String>>(&mut self, name: T) -> PyBytecode {
        let name_s: String = name.into();
        if let Some(i) = self.names.iter().position(|n| n == &name_s) {
            PyBytecode::StoreName(i as u8)
        } else if let Some(i) = self.varnames.iter().position(|n| n == &name_s) {
            PyBytecode::StoreFast(i as u8)
        } else {
            let i = self.varnames.len();
            self.varnames.push(name_s);
            PyBytecode::StoreFast(i as u8)
        }
    }

    pub fn add_name_store<T: Into<String>>(&mut self, name: T) -> PyBytecode {
        let name_s: String = name.into();
        if let Some(i) = self.names.iter().position(|n| n == &name_s) {
            PyBytecode::StoreName(i as u8)
        } else {
            let i = self.names.len();
            self.names.push(name_s);
            PyBytecode::StoreName(i as u8)
        }
    }

    pub fn get_last_name(&self) -> Option<&String> {
        self.names.last()
    }

    pub fn add_global<T: Into<String>>(&mut self, name: T, obj: Obj) {
        let name_s: String = name.into();
        self.names.push(name_s.clone());
        self.globals.insert(name_s, obj.to_arc());
    }

    pub fn extract_code(self) -> Vec<PyBytecode> {
        self.bytecode
    }

    pub fn finish(self) -> PyCodeObj {
        let n_c = self.consts.len();
        let n_v = self.varnames.len();
        let n_n = self.names.len();
        PyCodeObj {
            name: self.name,
            bytecode: self.bytecode,
            consts: self.consts,
            names: self.names,
            varnames: self.varnames,
            num_consts: n_c,
            num_names: n_n,
            num_varnames: n_v,
            globals: self.globals,
        }
    }

    pub fn serialize(&self, indent: usize) -> String {
        self.clone().finish().serialize(indent)
    }
}

impl Deref for PyCompileCtx {
    type Target = Vec<PyBytecode>;
    fn deref(&self) -> &Self::Target {
        &self.bytecode
    }
}

impl DerefMut for PyCompileCtx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytecode
    }
}

impl core::hash::Hash for PyCodeObj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.bytecode.hash(state);
        self.names.hash(state);
        self.consts.hash(state);
        self.varnames.hash(state);
        PyUtils::hash_hashmap(&self.globals, state);
    }
}

impl core::hash::Hash for FuncObj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code.as_ref().hash(state);
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
    pub code: Arc<PyCodeObj>,
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

        contents.push_str(&format!("{tabs}closure:\n{tabs}\t{:?}\n", self.closure));
        return contents;
    }
}

#[derive(Debug, Clone)]
pub struct PyTypeObj {
    pub name: String,
    pub fields: HashMap<String, Arc<Obj>>,
}

pub trait PyClassBase {
    fn new_instance(&self) -> PyClassInst;
}

use uuid::Uuid;

impl PyClassBase for Arc<PyTypeObj> {

    fn new_instance(&self) -> PyClassInst {
        PyClassInst {
            fields: self.fields.clone(),
            class_base: self.clone(),
            id: Uuid::new_v4(),
        }
    }
}

impl core::hash::Hash for PyTypeObj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        PyUtils::hash_hashmap(&self.fields, state);
    }
}

#[derive(Debug, Clone)]
pub struct PyClassInst {
    pub fields: HashMap<String, Arc<Obj>>,
    pub class_base: Arc<PyTypeObj>,
    pub id: Uuid,
}
impl core::hash::Hash for PyClassInst {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        PyUtils::hash_hashmap(&self.fields, state);
        self.class_base.hash(state);
        self.id.hash(state);
    }
}
