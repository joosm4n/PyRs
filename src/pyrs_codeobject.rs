

use crate::pyrs_bytecode::{PyBytecode};

#[derive(Debug, Clone, PartialEq)]
pub struct CodeObject
{
    co_nlocals: usize,
    co_argcount: usize,
    co_varnames: Vec<String>,
    co_names: Vec<String>,
    co_freevars: Vec<String>,
    co_cellvars: Vec<String>,
    co_posonlyargcount: usize,
    co_kwonlyargcount: usize,
    co_firstlineno: usize,
    co_lnotab: usize,
    co_stacksize: usize,
    co_code: Vec<PyBytecode>,
    co_consts: Vec<String>,
    co_flags: usize,
}

impl CodeObject
{
    
}