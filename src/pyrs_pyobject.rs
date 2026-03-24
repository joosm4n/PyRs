use crate::{
    pyrs_codeobject::{FuncObj, PyClassInst, PyCodeObj, PyTypeObj},
    pyrs_error::PyException,
    pyrs_obj::{Obj, PyObjIter},
    pyrs_std::{FnPtr, RangeObj},
};
use rug::Integer;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, LazyLock, Mutex, MutexGuard},
};

#[derive(Debug, Clone)]
pub struct PyObject {
    pub obj: Obj,
    pub attrs: AttrDict,
    pub local_attrs: AttrDict,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AttrDict(pub HashMap<Arc<str>, PyObjPtr>);

#[derive(Clone)]
pub enum PyObjPtr {
    Const(Arc<PyObject>),
    Mut(Arc<Mutex<PyObject>>),
}

#[derive(Debug)]
pub enum PyObjRef<'a> {
    Const(&'a PyObject),
    Mut(MutexGuard<'a, PyObject>),
}

static PYOBJECT_NONE: LazyLock<PyObjPtr> = LazyLock::new(|| {
    PyObjPtr::Const(Arc::new(PyObject {
        obj: Obj::None,
        attrs: AttrDict::new(),
        local_attrs: AttrDict::new(),
    }))
});

impl PyObject {
    pub fn none() -> PyObjPtr {
        PYOBJECT_NONE.clone()
    }

    pub fn new_int(value: Integer) -> Self {
        PyObject {
            obj: Obj::Int(value),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_float(value: f64) -> Self {
        PyObject {
            obj: Obj::Float(value),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_str<T: Into<String>>(s: T) -> Self {
        PyObject {
            obj: Obj::Str(s.into()),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }

    pub fn new_bool(b: bool) -> Self {
        PyObject {
            obj: Obj::Bool(b),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }

    pub fn new_exception(ex: PyException) -> Self {
        PyObject {
            obj: Obj::Except(ex),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_list(vec: Vec<PyObjPtr>) -> Self {
        PyObject {
            obj: Obj::List(vec),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_tuple(vec: Vec<PyObjPtr>) -> Self {
        PyObject {
            obj: Obj::Tuple(vec),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_set(vec: Vec<PyObjPtr>) -> Self {
        PyObject {
            obj: Obj::Set(vec),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_type(ty: PyTypeObj) -> Self {
        PyObject {
            obj: Obj::Type(ty),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_fnptr(f: FnPtr) -> Self {
        PyObject {
            obj: Obj::FuncPtr(f),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_iter(it: PyObjIter) -> Self {
        PyObject {
            obj: Obj::Iter(it),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_codeobj(c: PyCodeObj) -> Self {
        PyObject {
            obj: Obj::Code(Arc::new(c)),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_codeobj_arc(c: Arc<PyCodeObj>) -> Self {
        PyObject {
            obj: Obj::Code(c),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_classinst(c: PyClassInst) -> Self {
        PyObject {
            obj: Obj::ClassInst(c),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_function(f: FuncObj) -> Self {
        PyObject {
            obj: Obj::FunctionObj(f),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_buildclass() -> Self {
        PyObject {
            obj: Obj::BuildClass,
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }
    pub fn new_range(r: RangeObj) -> Self {
        PyObject {
            obj: Obj::Range(r),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new(),
        }
    }

    pub fn to_ptr(self) -> PyObjPtr {
        match &self.obj {
            Obj::None => PyObjPtr::none(),
            Obj::Bool(_) => PyObjPtr::Const(Arc::new(self)),
            Obj::Float(_) => PyObjPtr::Const(Arc::new(self)),
            Obj::Str(_) => PyObjPtr::Mut(Arc::new(Mutex::new(self))),
            Obj::Int(_) => PyObjPtr::Const(Arc::new(self)),
            Obj::FuncPtr(_) => PyObjPtr::Const(Arc::new(self)),
            Obj::Except(_) => PyObjPtr::Const(Arc::new(self)),
            Obj::List(_) => PyObjPtr::Mut(Arc::new(Mutex::new(self))),
            Obj::Set(_) => PyObjPtr::Mut(Arc::new(Mutex::new(self))),
            Obj::Tuple(_) => PyObjPtr::Mut(Arc::new(Mutex::new(self))),
            Obj::Range(_) => PyObjPtr::Const(Arc::new(self)),
            Obj::Dict(_) => PyObjPtr::Mut(Arc::new(Mutex::new(self))),
            Obj::Iter(_) => PyObjPtr::Mut(Arc::new(Mutex::new(self))),
            Obj::Type(_) => PyObjPtr::Mut(Arc::new(Mutex::new(self))),
            Obj::ClassInst(_) => PyObjPtr::Mut(Arc::new(Mutex::new(self))),
            Obj::Code(_) => PyObjPtr::Const(Arc::new(self)),
            Obj::FunctionObj(_) => PyObjPtr::Mut(Arc::new(Mutex::new(self))),
            Obj::BuildClass => PyObjPtr::Const(Arc::new(self)),
        }
    }
}

impl std::hash::Hash for PyObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}

impl Deref for AttrDict {
    type Target = HashMap<Arc<str>, PyObjPtr>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for AttrDict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PyObjPtr {
    pub fn none() -> PyObjPtr {
        PYOBJECT_NONE.clone()
    }

    pub fn __new__(&self) -> PyObjPtr {
        self.clone()
    }

    pub fn get_ref(&self) -> PyObjRef<'_> {
        match self {
            PyObjPtr::Const(v) => PyObjRef::Const(v.as_ref()),
            PyObjPtr::Mut(v) => PyObjRef::Mut(v.as_ref().lock().expect("unable to lock mutex")),
        }
    }
    pub fn ptr_eq(lhs: &Self, rhs: &Self) -> bool {
        match (lhs, rhs) {
            (PyObjPtr::Const(l), PyObjPtr::Const(r)) => Arc::ptr_eq(l, r),
            (PyObjPtr::Mut(l), PyObjPtr::Mut(r)) => Arc::ptr_eq(l, r),
            _ => false,
        }
    }
}

impl AttrDict {
    pub fn new() -> Self {
        Self::default()
    }
}

impl std::fmt::Display for PyObjPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self.get_ref())
    }
}

impl std::fmt::Debug for PyObjPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ptr({:?})", *self.get_ref())
    }
}

impl PartialEq for PyObjPtr {
    fn eq(&self, other: &Self) -> bool {
        PyObjPtr::ptr_eq(self, other)
    }
}

impl std::hash::Hash for PyObjPtr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_ref().hash(state);
    }
}

impl<'a> Deref for PyObjRef<'a> {
    type Target = PyObject;
    fn deref(&self) -> &Self::Target {
        match self {
            PyObjRef::Const(v) => v,
            PyObjRef::Mut(v) => v,
        }
    }
}

impl<'a> DerefMut for PyObjRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            PyObjRef::Mut(v) => v,
            PyObjRef::Const(_) => panic!("Cannot mutate this object"),
        }
    }
}
