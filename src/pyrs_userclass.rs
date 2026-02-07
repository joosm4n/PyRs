use crate::{ 
    pyrs_bytecode::PyBytecode, pyrs_error::{PyError, PyException}, pyrs_obj::{Obj}
};
use std::{
    collections::HashMap,
    sync::{Arc},
};

#[derive(Debug, Clone, PartialEq)]
pub struct UserClassDef {
    pub name: String,
    pub fields: HashMap<String, (usize, Obj)>, // offset
    pub methods: HashMap<String, Vec<PyBytecode>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserClassInstance {
    pub class: Arc<UserClassDef>,
    pub fields: Vec<Arc<Obj>>,
}

impl UserClassDef
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

    pub fn default_methods() -> HashMap<String, Vec<PyBytecode>>
    {
        let mut map = HashMap::new();
        map.insert("__init__".into(), vec![PyBytecode::ReturnValue]);
        map.insert("__str__".into(), vec![PyBytecode::ReturnValue]);
        map
    }

}

impl UserClassInstance
{
    pub fn get_field(&self, field: &String) -> Result<&Arc<Obj>, PyException>
    {
        if let Some((idx, _)) = self.class.fields.get(field) {
            Ok(&self.fields[*idx])
        }
        else {
            Err(PyException{
                error: PyError::UndefinedVariableError,
                msg: format!("no field \'{field}\' for object {}", &self.class.name),
            })
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

