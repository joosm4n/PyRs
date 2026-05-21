use crate::pyrs_obj::Obj;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct PyModule {
    pub name: String,
    pub globals: Arc<RwLock<HashMap<String, Arc<Obj>>>>,
}

impl core::hash::Hash for PyModule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl std::fmt::Display for PyModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output.push_str(&format!("Module Name: {}", self.name));
        write!(f, "{}", output)
    }
}
