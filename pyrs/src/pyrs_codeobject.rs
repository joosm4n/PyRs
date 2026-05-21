use crate::{
    pyrs_bytecode::PyBytecode,
    pyrs_obj::{Obj, ToObj},
    pyrs_pyobject::{AttrDict, PyObjPtr, PyObject},
    pyrs_utils::PyUtils,
};

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PyCodeObj {
    pub name: Arc<str>,
    pub bytecode: Vec<PyBytecode>,
    pub consts: Vec<PyObjPtr>,
    pub names: Vec<Arc<str>>,
    pub varnames: Vec<Arc<str>>,
    pub num_consts: usize,
    pub num_varnames: usize,
    pub num_names: usize,
    pub globals: AttrDict,
}

impl PyCodeObj {
    pub fn new(name: &str, code: Vec<PyBytecode>) -> Self {
        PyCodeObj {
            name: name.into(),
            bytecode: code,
            consts: vec![],
            names: vec![],
            varnames: vec![],
            num_consts: 0,
            num_names: 0,
            num_varnames: 0,
            globals: AttrDict::new(),
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

        s.push_str("Constants:\n");
        for (i, c) in self.consts.iter().enumerate() {
            s.push_str(&format!("\t{i}: {}\n", c.get_ref().__str__()));
        }

        s.push_str("Names:\n");
        for (i, n) in self.names.iter().enumerate() {
            s.push_str(&format!("\t{i}: {}\n", n));
        }

        s.push_str("Variable names:\n");
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
            let co = c.get_ref();
            match &co.obj {
                Obj::Code(code) => {
                    contents.push_str(&format!("{tabs}\t[{i}] {}\n", code.serialize(indent + 1)))
                }
                _ => contents.push_str(&format!("{tabs}\t{}\n", *co)),
                // crashing here??
            }
        }

        contents.push_str(&format!("{tabs}names:\n{tabs}\t"));
        for n in &self.names {
            contents.push_str(&format!("{}, ", n));
        }

        contents.push_str(&format!("\n{tabs}bytecode:\n"));
        contents.push_str(&self.get_inst_string());

        contents.push_str(&format!("{tabs}<end {}>\n", &self.name));
        contents
    }

    fn get_inst_string(&self) -> String {
        let mut bytecode_string = String::new();
        for (idx, line) in self.bytecode.iter().enumerate() {
            bytecode_string.push_str(format!("({idx}) \t\t{:?}", line).as_str());

            let arg: Option<String> = match line {
                PyBytecode::LoadConst(v) => {
                    let con = &self.consts[*v as usize].get_ref();
                    Some(con.to_string())
                }
                PyBytecode::JumpBackward(v) => Some(format!("to {}", idx - *v as usize)),

                PyBytecode::LoadFast(v)
                | PyBytecode::StoreFast(v)
                | PyBytecode::LoadName(v)
                | PyBytecode::StoreName(v)
                | PyBytecode::LoadGlobal(v)
                | PyBytecode::StoreGlobal(v) => {
                    if let Some(name) = self.names.get(*v as usize) {
                        Some(name.to_string())
                    } else if let Some(name) = self.varnames.get(*v as usize) {
                        Some(name.to_string())
                    } else {
                        panic!()
                    }
                }
                _ => None,
            };
            if let Some(val) = arg {
                bytecode_string.push_str(&format!("\t\t({})\n", val));
            } else {
                bytecode_string.push('\n');
            }
        }
        bytecode_string
    }
}

#[derive(Debug, Clone)]
pub struct PyCompileCtx {
    name: Arc<str>,
    bytecode: Vec<PyBytecode>,
    consts: Vec<PyObjPtr>,
    names: Vec<Arc<str>>,
    varnames: Vec<Arc<str>>,
    globals: AttrDict,
}

impl PyCompileCtx {
    pub fn new(name: Arc<str>) -> Self {
        Self {
            name,
            bytecode: vec![PyBytecode::Resume],
            consts: vec![],
            names: vec![],
            varnames: vec![],
            globals: AttrDict::new(),
        }
    }

    pub fn add_const(&mut self, obj: PyObjPtr) -> u8 {
        if let Some(i) = self.consts.iter().position(|o| o == &obj) {
            i as u8
        } else {
            let i = self.consts.len();
            self.consts.push(obj);
            i as u8
        }
    }

    pub fn load_const(&mut self, obj: PyObjPtr) {
        if let Some(i) = self.consts.iter().position(|o| o == &obj) {
            self.push(PyBytecode::LoadConst(i as u8));
        } else {
            let i = self.consts.len();
            self.consts.push(obj);
            self.push(PyBytecode::LoadConst(i as u8));
        }
    }

    pub fn add_varname(&mut self, name: Arc<str>) -> u8 {
        if let Some(i) = self.varnames.iter().position(|n| n == &name) {
            i as u8
        } else {
            let i = self.varnames.len();
            self.varnames.push(name);
            i as u8
        }
    }

    pub fn add_name(&mut self, name: Arc<str>) -> u8 {
        if let Some(i) = self.names.iter().position(|n| n == &name) {
            i as u8
        } else {
            let i = self.names.len();
            self.names.push(name);
            i as u8
        }
    }

    pub fn add_varname_load(&mut self, name: Arc<str>) -> PyBytecode {
        if self.globals.contains_key(&name) {
            let i = self.names.iter().position(|n| n == &name).unwrap();
            PyBytecode::LoadGlobal(i as u8)
        } else if let Some(i) = self.varnames.iter().position(|n| n == &name) {
            PyBytecode::LoadFast(i as u8)
        } else {
            let i = self.varnames.len();
            self.varnames.push(name);
            PyBytecode::LoadFast(i as u8)
        }
    }

    pub fn add_name_load(&mut self, name: Arc<str>) -> PyBytecode {
        if let Some(i) = self.names.iter().position(|n| n == &name) {
            PyBytecode::LoadName(i as u8)
        } else if self.globals.contains_key(&name) {
            let i = self.names.iter().position(|n| n == &name).unwrap();
            PyBytecode::LoadGlobal(i as u8)
        } else {
            let i = self.names.len();
            self.names.push(name);
            PyBytecode::LoadName(i as u8)
        }
    }

    pub fn add_varname_store(&mut self, name: Arc<str>) -> PyBytecode {
        if let Some(i) = self.varnames.iter().position(|n| n == &name) {
            PyBytecode::StoreFast(i as u8)
        } else {
            let i = self.varnames.len();
            self.varnames.push(name);
            PyBytecode::StoreFast(i as u8)
        }
    }

    pub fn add_name_store(&mut self, name: Arc<str>) -> PyBytecode {
        if let Some(i) = self.names.iter().position(|n| n == &name) {
            PyBytecode::StoreName(i as u8)
        } else {
            let i = self.names.len();
            self.names.push(name);
            PyBytecode::StoreName(i as u8)
        }
    }

    pub fn get_last_name(&self) -> Option<Arc<str>> {
        self.names.last().cloned()
    }

    pub fn get_context_name(&self) -> Arc<str> {
        self.name.clone()
    }

    pub fn add_global(&mut self, name: Arc<str>, obj: PyObject) {
        self.names.push(name.clone());
        self.globals.insert(name, obj.to_ptr());
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
        self.code.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct FuncObj {
    pub code: Arc<PyCodeObj>,
}

impl ToObj for FuncObj {
    fn to_pyptr(self) -> PyObjPtr {
        self.to_pyobj().to_ptr()
    }
    fn to_pyobj(self) -> PyObject {
        PyObject::new_function(self)
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
        contents
    }
}

#[derive(Debug, Clone)]
pub struct PyTypeObj {
    pub name: Arc<str>,
    pub static_attribs: AttrDict,
    pub code: Arc<PyCodeObj>,
}

use uuid::Uuid;

impl PyTypeObj {
    pub fn new_instance(&self) -> PyClassInst {
        PyClassInst {
            fields: self.static_attribs.clone(),
            id: Uuid::new_v4(),
        }
    }
}

impl core::hash::Hash for PyTypeObj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        PyUtils::hash_hashmap(&self.static_attribs, state);
    }
}

#[derive(Debug, Clone)]
pub struct PyClassInst {
    pub fields: AttrDict,
    pub id: Uuid,
}
impl core::hash::Hash for PyClassInst {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        PyUtils::hash_hashmap(&self.fields, state);
        self.id.hash(state);
    }
}
