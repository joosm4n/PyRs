
use crate::{
    pyrs_bytecode::PyBytecode, pyrs_obj::Obj
};
use std::{
    collections::HashMap,
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct PyModule
{
    pub name: String,
    pub vars: HashMap<String, Arc<Obj>>,
    pub code: Vec<PyBytecode>,
}

impl std::fmt::Display for PyModule
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output.push_str(&format!("Module Name: {}", self.name));
        output.push_str("\nVars:");
        for (name, val) in &self.vars {
            output.push_str(&format!("\n{name}: {val}"));
        }
        output.push_str(&format!("Bytecode: \n{}", PyBytecode::to_string(&self.code)));
        write!(f, "{}", output)
    }
}
