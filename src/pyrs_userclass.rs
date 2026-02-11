use crate::{
    pyrs_obj::{Obj, ToObj},
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, PartialEq)]
pub struct CustomClass {
    pub name: String,
    pub fields: HashMap<String, Arc<Obj>>,
}

impl CustomClass {
    pub fn new(name: &str) -> Self {
        CustomClass {
            name: name.to_string(),
            fields: HashMap::new(),
        }
    }
}

impl ToObj for CustomClass {
    fn to_arc(self) -> Arc<Obj> {
        self.to_obj().into()
    }
    fn to_obj(self) -> Obj {
        Obj::CustomClass(self)
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
