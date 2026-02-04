use crate::{ 
    pyrs_obj::Obj,
    pyrs_bytecode::{PyBytecode},
};


#[derive(Debug, Clone, PartialEq)]
pub struct UserClass {
    pub name: String,
    pub fields: Vec<Obj>,
    pub funcs: Vec<PyBytecode>,
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

impl UserClass {
    
}

impl std::fmt::Display for UserClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {:?}", self.name, self.fields)
    }
}
