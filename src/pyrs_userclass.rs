use crate::pyrs_obj::Obj;
use crate::pyrs_parsing::{Token, Expression};

#[derive(Debug, Clone, PartialEq)]
pub struct UserClass {
    pub name: String,
    pub fields: Vec<Obj>,
    pub funcs: Vec<Expression>,
}

// class <name>:
// \t def __init__(self):
// \t\t self.x = 0
// \t\t self.y = 1

impl UserClass {
    pub fn from_token(_token: &Token) {
        unimplemented!();
    }
}

impl std::fmt::Display for UserClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {:?}", self.name, self.fields)
    }
}
