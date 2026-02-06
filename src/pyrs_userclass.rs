use crate::{ 
    pyrs_bytecode::PyBytecode, pyrs_error::{PyError, PyException}, pyrs_obj::{Obj, ToObj}
};
use std::{
    collections::HashMap,
    sync::{Arc},
};

#[derive(Debug, Clone, PartialEq)]
pub struct UserClass {
    pub name: String,
    pub fields: HashMap<String, usize>, // offset
    pub methods: HashMap<String, Vec<PyBytecode>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserClassInstance {
    pub class: Arc<UserClass>,
    pub fields: Vec<Arc<Obj>>,
}

impl UserClass
{
    pub fn new_instance(class: &Arc<Self>) -> UserClassInstance {
        UserClassInstance {
            class: class.clone(),
            fields: class.default_fields() 
        }
    }

    fn default_fields(&self) -> Vec<Arc<Obj>>
    {
        let mut fields = vec![];
        for _ in 0..self.fields.len() { // placeholder, construct default of type later
            fields.push(Obj::None.into());
        }
        fields
    }

}

impl UserClassInstance
{
    pub fn get_field(&self, field: &String) -> Arc<Obj>
    {
        if let Some(idx) = self.class.fields.get(field) {
            self.fields[*idx].clone()
        }
        else {
            return PyException{
                error: PyError::UndefinedVariableError,
                msg: format!("no field \'{field}\' for object {}", &self.class.name),
            }.to_arc()
        }
    } 
}


// class <name>:
// \t def __init__(self):
// \t\t self.x = 0
// \t\t self.y = 1

/*

What to implement:
    - default func impls (in bytecode)

    basically i can make a class a instruction addr,
    fields an instance a hashmap
    access with . operator 


*/

impl std::fmt::Display for UserClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {:?}", self.name, self.fields)
    }
}
