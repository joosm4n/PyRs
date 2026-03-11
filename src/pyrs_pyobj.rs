use crate::pyrs_obj::Obj;

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

#[derive(Debug, Clone)]
pub struct PyAttrDict(Mutex<HashMap<String, PyObj>>);

#[derive(Debug, Clone)]
pub struct PyObj {
    pub obj: Arc<Obj>,
    pub attrs: Arc<PyAttrDict>,
}

// Class objects
// Instance objects
// Method objects

// Arc<Obj> ptr to enum of all objs
// :
// maybe refactor into Arc to struct??
//

static PYOBJ_NONE: LazyLock<PyObj> = LazyLock::new(|| PyObj {
    obj: Arc::new(Obj::None),
    attrs: Arc::new(PyAttrDict {
        0: Mutex::new(HashMap::new()),
    }),
});

impl PyObj {
    pub fn none() -> &Self {
        LazyLock::force(&PYOBJ_NONE)
    }
}

impl PyAttrDict {
    pub fn new() -> Self {
        PyAttrDict {
            0: Mutex::new(HashMap::new()),
        }
    }

    pub fn object_base_attrs() -> Self {
        let mut attrs: HashMap<String, PyObj> = HashMap::new();
        attrs.insert("__class__".into, *PyObj::none());
        attrs.insert("__bases__".into(), *PyObj::none());
        attrs.insert("__dict__".into(), *PyObj::none());
        attrs.insert("__doc__".into(), *PyObj::none());
        attrs.insert("__module__".into(), *PyObj::none());
        attrs.insert("__init__".into(), *PyObj::none());
        attrs.insert("__str__".into(), *PyObj::none());
        attrs.insert("__repr__".into(), *PyObj::none());

        PyAttrDict {
            0: Mutex::new(attrs),
        }
    }
}
