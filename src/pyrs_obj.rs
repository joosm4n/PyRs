use crate::{
    pyrs_codeobject::{PyCodeObj, FuncObj, PyTypeObj, PyClassInst},
    pyrs_error::{PyError, PyException},
    pyrs_parsing::{Expression, Op},
    pyrs_std::{FnPtr, RangeObj},
    pyrs_utils::PyUtils,
    pyrs_pyobject::{PyObject, PyObjPtr, AttrDict},
};
use std::{
    collections::HashMap,
    ops::{Add, Mul, Neg, Sub},
    process::{ExitCode, Termination},
    str::FromStr,
    sync::{Arc, Mutex},
};

use rug::Integer;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Obj {

    None,

    Bool(bool),
    Float(f64),
    Str(String),
    Int(Integer),

    FuncPtr(FnPtr),

    Except(PyException),

    List(Vec<PyObjPtr>),             // [], mutable, ordered, duplicates, int indexing,
    Tuple(Vec<PyObjPtr>),            // (), immutable, ordered, duplicates, int indexing,
    Set(Vec<PyObjPtr>),              // {}, mutable, unordered, no dupes, no indexing,
    Range(RangeObj),

    Dict(HashMap<PyObjPtr, PyObjPtr>),

    Iter(PyObjIter),

    Type(PyTypeObj),
    ClassInst(PyClassInst),

    Code(Arc<PyCodeObj>),
    FunctionObj(FuncObj),

    BuildClass,

    // Binary
    // - bytes
    // - bytearray,
    // - memoryview,

    // Set
    // - frozenset

    // Mapping
    // - dict (HashMap)
}
 /*
 pub fn attrs_obj() -> AttrDict {
    let map: AttrDict =  AttrDict::new();
    return map;
}
*/
    // dir(object) = 
    // ['__class__', '__delattr__', '__dir__', '__doc__', '__eq__', '__format__', 
    // '__ge__', '__getattribute__', '__getstate__', '__gt__', '__hash__', '__init__', 
    // '__init_subclass__', '__le__', '__lt__', '__ne__', '__new__', '__reduce__', 
    // '__reduce_ex__', '__repr__', '__setattr__', '__sizeof__', '__str__', '__subclasshook__']

    // dir(int) = 
    // ['__abs__', '__add__', '__and__', '__bool__', '__ceil__', '__class__', '__delattr__', 
    // '__dir__', '__divmod__', '__doc__', '__eq__', '__float__', '__floor__', '__floordiv__',
    // '__format__', '__ge__', '__getattribute__', '__getnewargs__', '__getstate__', '__gt__', 
    // '__hash__', '__index__', '__init__', '__init_subclass__', '__int__', '__invert__',
    // '__le__', '__lshift__', '__lt__', '__mod__', '__mul__', '__ne__', '__neg__', '__new__',
    // '__or__', '__pos__', '__pow__', '__radd__', '__rand__', '__rdivmod__', '__reduce__',
    // '__reduce_ex__', '__repr__', '__rfloordiv__', '__rlshift__', '__rmod__', '__rmul__', 
    // '__ror__', '__round__', '__rpow__', '__rrshift__', '__rshift__', '__rsub__', '__rtruediv__',
    // '__rxor__', '__setattr__', '__sizeof__', '__str__', '__sub__', '__subclasshook__', 
    // '__truediv__', '__trunc__', '__xor__', 'as_integer_ratio', 'bit_count', 'bit_length',
    // 'conjugate', 'denominator', 'from_bytes', 'imag', 'is_integer', 'numerator', 'real', 'to_bytes']
    

impl PyObject {

    pub fn from<T: ToObj>(arg: T) -> PyObjPtr {
        arg.to_pyptr()
    }

    pub fn new_vec() -> Vec<Obj> {
        return vec![];
    }

    pub fn new_arc_vec() -> Vec<PyObjPtr> {
        return vec![];
    }

    pub fn new_map() -> HashMap<String, PyObjPtr> {
        return HashMap::new();
    }

    pub fn empty_dict() -> PyObject {
        PyObject {
            obj: Obj::Dict(HashMap::new()),
            attrs: AttrDict::new(),
            local_attrs: AttrDict::new()
        }
    }

    pub fn is_num(&self) -> bool {
        match self.obj {
            Obj::Float(_) | Obj::Int(_) => true,
            _ => false,
        }
    }

    pub fn from_atom(c: &str) -> Self {
        if let Ok(val) = Integer::from_str(c) {
            return PyObject::new_int(val);
        }
        if let Ok(val) = c.parse::<f64>() {
            return PyObject::new_float(val);
        } else {
            PyObject::new_str(c)
        }
    }

    pub fn is_iterable(&self) -> bool {
        match &self.obj {
            Obj::Set(_) | Obj::Str(_) | Obj::List(_) | Obj::Dict(_) | Obj::Tuple(_) => true,
            _ => false,
        }
    }

    pub fn iter_next(&mut self) -> Option<PyObjPtr> {
        match &mut self.obj {
            Obj::Iter(i) => i.next(),
            _ => None,
        }
    }

    pub fn add(lhs: &Self, rhs: &Self) -> Self {

        let err = PyObject::new_exception(PyException {
            error: PyError::TypeError,
            msg: format!("No valid way to add: {} and {}", lhs, rhs.clone(),),
        });

        let lhs = &lhs.obj;
        let rhs = &rhs.obj;

        let obj = match (lhs, rhs) {
            (Obj::Float(dbl), other) => {
                let val = match other {
                    Obj::Float(v) => *v,
                    Obj::Int(v) => v.to_f64(),
                    _ => return err,
                };
                PyObject::new_float(dbl + val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => PyObject::new_int(int.clone().add(v)),
                Obj::Float(v) => PyObject::new_float(int.to_f64() + v),
                _ => return err,
            },
            (Obj::Str(s), other) => match other {
                Obj::Str(v) => PyObject::new_str(format!("{s}{v}")),
                _ => return err,
            },
            (Obj::List(l1), other) => match other {
                Obj::List(l2) => {
                    let mut new_list = Vec::with_capacity(l1.len() + l2.len());
                    new_list.extend(l1.iter().cloned());
                    new_list.extend(l2.iter().cloned());
                    PyObject::new_list(new_list)
                }
                _ => {
                    return PyObject::new_exception(PyException {
                        error: PyError::TypeError,
                        msg: format!("can only concatenate list (not \"{:?}\") to list", other),
                    });
                }
            },
            _ => return err,
        };
        obj
    }

    pub fn sub(lhs: &Self, rhs: &Self) -> Self {
        let err = PyObject::new_exception(PyException {
            error: PyError::TypeError,
            msg: format!("No valid way to subtract: {} and {}", lhs, rhs.clone(),),
        });
        let lhs = &lhs.obj;
        let rhs = &rhs.obj;

        let obj = match (lhs, rhs) {
            (Obj::Float(dbl), other) => {
                let val = match other {
                    Obj::Float(v) => *v,
                    Obj::Int(v) => v.to_f64(),
                    _ => return err,
                };
                PyObject::new_float(dbl - val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => PyObject::new_int(int.clone().sub(v)),
                Obj::Float(v) => PyObject::new_float(int.to_f64() - v),
                _ => return err,
            },
            _ => return err,
        };
        obj
    }

    pub fn mul(lhs_: &Self, rhs_: &Self) -> Self {
        let err = PyObject::new_exception(PyException {
            error: PyError::TypeError,
            msg: format!("No valid way to subtract: {} and {}", lhs_, rhs_.clone(),),
        });

        let lhs = &lhs_.obj;
        let rhs = &rhs_.obj;

        let obj = match (lhs, rhs) {
            (Obj::Float(dbl), other) => {
                let val = match other {
                    Obj::Float(v) => *v,
                    Obj::Int(v) => v.to_f64(),
                    _ => return err,
                };
                PyObject::new_float(dbl * val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => PyObject::new_int(int.clone().mul(v)),
                Obj::Float(v) => PyObject::new_float(int.to_f64() * v),
                _ => return err,
            },
            (Obj::Str(s), other) => match other {
                Obj::Int(v) => {
                    if *v >= 0 {
                        let mut mult = String::new();
                        for _i in 0..v.to_u64().unwrap() {
                            mult = format!("{mult}{s}");
                        }
                        PyObject::new_str(mult)
                    } else {
                        return PyObject::new_exception(PyException {
                            error: PyError::TypeError,
                            msg: format!(" can't multiply sequence by non-int of type {}", lhs_),
                        });
                    }
                }
                _ => return err,
            },
            _ => return err,
        };
        obj
    }

    pub fn div(lhs: &Self, rhs: &Self) -> Self {
        let type_err = PyObject::new_exception(PyException {
            error: PyError::TypeError,
            msg: format!("No valid way to divide: {} and {}", lhs, rhs.clone(),),
        });
        let zero_div_err = PyObject::new_exception(PyException {
            error: PyError::ZeroDivisionError,
            msg: format!(" tried to divide {lhs} by {rhs}"),
        });

        let lhs = &lhs.obj;
        let rhs = &rhs.obj;

        let obj = match (lhs, rhs) {
            (Obj::Float(dbl), other) => {
                let val = match other {
                    Obj::Float(v) => *v,
                    Obj::Int(v) => v.to_f64(),
                    _ => return type_err,
                };
                if val == 0f64 {
                    return zero_div_err;
                }
                PyObject::new_float(dbl / val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => {
                    if *v == Integer::ZERO {
                        return zero_div_err;
                    }
                    PyObject::new_float(int.to_f64() / v.to_f64())
                }
                Obj::Float(v) => {
                    if *v == 0f64 {
                        return zero_div_err;
                    }
                    PyObject::new_float(int.to_f64() / v)
                }
                _ => return type_err,
            },
            _ => return type_err,
        };
        obj.into()
    }

    pub fn __typestr__(&self) -> &'static str {
        match self.obj {
            Obj::None => "None",
            Obj::Int(_) => "int",
            Obj::Float(_) => "float",
            Obj::Bool(_) => "bool",
            Obj::Str(_) => "str",
            Obj::List(_) => "list",
            Obj::Tuple(_) => "tuple",
            Obj::FunctionObj(_) => "function",
            _ => "Not Implemented", 
        }
    }

    pub fn __int__(&self) -> isize {
        match &self.obj {
            Obj::Bool(v) => *v as isize,
            Obj::Int(v) => v.to_isize_wrapping(),
            Obj::Float(v) => *v as isize,
            _ => panic!(),
        }
    }

    pub fn __integer__(&self) -> Option<Integer> {
        match &self.obj {
            Obj::Int(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn __bool__(&self) -> bool {
        let ret = match &self.obj {
            Obj::None => false,
            Obj::Bool(v) => *v,
            Obj::Float(v) => *v != 0f64,
            Obj::Int(v) => *v != Integer::ZERO,
            Obj::Str(v) => *v != "",
            Obj::List(vec) |
            Obj::Tuple(vec) | 
            Obj::Set(vec) => vec.len() != 0usize,
            _ => panic!("TypeError: __bool__() not implemented for: {:?}", self),
        };
        return ret;
    }

    pub fn __unpack__(self) -> Result<Vec<PyObjPtr>, PyException> {
        if self.is_iterable() {
            Ok(match &self.obj {
                Obj::List(vec) |
                Obj::Set(vec) | 
                Obj::Tuple(vec) => vec.clone(),
                Obj::Range(range) => range.to_vec(),
                Obj::Dict(dict) => dict.into_iter().map(|(key, _)| key.clone()).collect(),
                _ => unreachable!(),
            })
        } else {
            Err(PyException {
                error: PyError::TypeError,
                msg: format!("Cannot unpack a non iterable type: {:?}", self),
            })
        }
    }

    pub fn __str__(&self) -> String {
        match &self.obj {
            Obj::None => format!("None"),
            Obj::Bool(val) => match val {
                true => format!("True"),
                false => format!("False"),
            },
            Obj::Float(val) => format!("{}", val),
            Obj::Str(s) => format!("{}", s),
            Obj::Int(val) => format!("{}", val),
            Obj::FuncPtr(ptr) => format!("{}", ptr),
            Obj::Except(e) => format!("{}", e),
            Obj::List(vec) => {
                let mut list = String::from("[");
                for o in vec {
                    list.push_str(o.get_ref().__repr__().as_str());
                    list.push(',');
                    list.push(' ');
                }
                if list.len() > 2 {
                    list.pop();
                    list.pop();
                }
                list.push_str("]");
                format!("{}", list)
            }
            Obj::Tuple(objs) => {
                let mut tuple = String::from("(");
                for o in objs {
                    tuple.push_str(o.get_ref().__repr__().as_str());
                    tuple.push(',');
                    tuple.push(' ');
                }
                tuple.pop();
                tuple.pop();
                tuple.push_str(")");
                format!("{}", tuple)
            }
            Obj::Set(objs) => {
                let mut set = String::from("{");
                for o in objs {
                    set.push_str(o.get_ref().__repr__().as_str());
                    set.push(',');
                    set.push(' ');
                }
                set.pop();
                set.pop();
                set.push_str("}");
                format!("{}", set)
            }
            Obj::Dict(objs) => {
                let mut map = String::from("{");
                for (key, value) in objs {
                    map.push_str(key.get_ref().__repr__().as_str());
                    map.push(':');
                    map.push_str(value.get_ref().__repr__().as_str());
                    map.push(',');
                    map.push(' ');
                }
                map.pop();
                map.pop();
                map.push_str("}");
                format!("{}", map)
            }
            Obj::Range(range) => {
                let mut r = String::from("range(");
                if let Some(start) = &range.start {
                    r.push_str(&format!("{}", start.to_string()));
                };
                if let Some(end) = &range.end {
                    r.push_str(&format!(", {}", end.to_string()));
                };
                if let Some(inc) = &range.inc {
                    r.push_str(&format!(", {}", inc.to_string()));
                };
                r.push(')');
                r
            }
            Obj::Iter(iter) => {
                format!("Iter[ index({}) {:?} ]", iter.index, iter.items)
            }
            Obj::ClassInst(class) => {
                format!(
                    "<class {}>",
                    *class.fields.get("__name__").unwrap().get_ref(),
                )
            }
            Obj::Type(class) => {
                format!(
                    "<type {}>",
                    class.name
                )
            }
            Obj::FunctionObj(func) => {
                format!("<function {}>", func.code.name)
            }
            Obj::Code(codeobj) => {
                format!("<code object {}>", codeobj.name)
            }
            Obj::BuildClass => {
                format!("<buildclass>")
            }
        }
    }

    pub fn __repr__(&self) -> String {
        match &self.obj {
            Obj::Str(s) => format!("\'{}\'", s),
            _ => self.__str__(),
        }
    }

    pub fn __len__(&self) -> usize {
        match &self.obj {
            Obj::Set(vec ) |
            Obj::Tuple(vec) |
            Obj::List(vec) => vec.len(),
            _ => panic!("TypeError: __len__() not implemented for: {:?}", self),
        }
    }

    pub fn compare_op(lhs: &Self, rhs: &Self, op: &Op) -> bool {
        let ret = match op {
            Op::Eq => lhs.eq(rhs),
            Op::Neq => lhs.ne(rhs),
            Op::LessThan => lhs.lt(rhs),
            Op::GreaterThan => lhs.gt(rhs),
            Op::LessEq => lhs.le(rhs),
            Op::GreaterEq => lhs.ge(rhs),
            _ => return false,
        };
        ret
    }

    pub fn __lt__(lhs: &PyObjPtr, rhs: &PyObjPtr) -> bool {
        lhs.get_ref().lt(&rhs.get_ref())
    }

    pub fn __gt__(lhs: &PyObjPtr, rhs: &PyObjPtr) -> bool {
        lhs.get_ref().gt(&rhs.get_ref())
    }

    pub fn __le__(lhs: &PyObjPtr, rhs: &PyObjPtr) -> bool {
        lhs.get_ref().le(&rhs.get_ref()) || lhs.eq(rhs)
    }
    pub fn __ge__(lhs: &PyObjPtr, rhs: &PyObjPtr) -> bool {
        lhs.get_ref().gt(&rhs.get_ref()) || lhs.eq(rhs)
    }

    pub fn __add__(lhs: &PyObjPtr, rhs: &PyObjPtr) -> Result<PyObjPtr, PyException> {
        let res= PyObject::add(&lhs.get_ref(), &rhs.get_ref());
        match res.obj {
            Obj::Except(e) => Err(e),
            _ => Ok(res.to_ptr()),
        }
    }

    pub fn __sub__(lhs: &PyObjPtr, rhs: &PyObjPtr) -> Result<PyObjPtr, PyException> {
        let res= PyObject::sub(&lhs.get_ref(), &rhs.get_ref());
        match res.obj {
            Obj::Except(e) => Err(e),
            _ => Ok(res.to_ptr()),
        }
    }

    pub fn __mul__(lhs: &PyObjPtr, rhs: &PyObjPtr) -> Result<PyObjPtr, PyException> {
        let res= PyObject::mul(&lhs.get_ref(), &rhs.get_ref());
        match res.obj {
            Obj::Except(e) => Err(e),
            _ => Ok(res.to_ptr()),
        }
    }

    pub fn __div__(lhs: &PyObjPtr, rhs: &PyObjPtr) -> Result<PyObjPtr, PyException> {
        let res= PyObject::div(&lhs.get_ref(), &rhs.get_ref());
        match res.obj {
            Obj::Except(e) => Err(e),
            _ => Ok(res.to_ptr()),
        }
    }

    pub fn __eq__(lhs: &PyObjPtr, rhs: &PyObjPtr) -> bool {
        lhs.eq(rhs)
    }

    pub fn __ne__(lhs: &PyObjPtr, rhs: &PyObjPtr) -> bool {
        !PyObject::__eq__(lhs, rhs)
    }

    pub fn __pos__(obj: &PyObjPtr) -> Result<PyObjPtr, PyException> {
        Ok(obj.clone())
    }

    pub fn __neg__(obj: &PyObjPtr) -> Result<PyObjPtr, PyException> {
        let ret = match &obj.get_ref().obj {
            Obj::None => PyObject::none(),
            Obj::Bool(b) => PyObject::new_bool(!b).to_ptr(),
            Obj::Float(f) => PyObject::new_float(-f).to_ptr(),
            Obj::Int(i) => PyObject::new_int(i.clone().neg()).to_ptr(),
            _ => {
                return Err(PyException {
                    error: PyError::NotImplementedError,
                    msg: format!("Negation not implemented for {}", *obj.get_ref()),
                })
            }
        };
        Ok(ret.into())
    }

    pub fn __call__(&self, objs: &Vec<PyObjPtr>) -> Result<PyObjPtr, PyException> {
        match &self.obj {
            Obj::FuncPtr(fn_ptr) => Ok((fn_ptr.ptr)(objs)),
            _ => Err(PyException {
                error: PyError::TypeError,
                msg: format!("Type is not a function"),
            }),
        }
    }

    pub fn to_pyptr(self) -> Arc<Self> {
        Arc::from(self)
    }

}

impl PartialEq for PyObject {
    fn eq(&self, other: &Self) -> bool {
        match (&self.obj, &other.obj) {
            (Obj::None, Obj::None) => true,
            (Obj::Float(flt), other) => match other {
                Obj::Float(same) => *flt == *same,
                Obj::Int(i) => *flt == i.to_f64(),
                Obj::Bool(b) => *flt == f64::from(*b),
                _ => false,
            },
            (Obj::Int(i), other) => match other {
                Obj::Float(f) => i.to_f64() == *f,
                Obj::Int(same) => *i == *same,
                Obj::Bool(b) => *i == Integer::from(*b),
                _ => false,
            },
            (Obj::Bool(b), other) => match other {
                Obj::Float(f) => f64::from(*b) == *f,
                Obj::Int(i) => Integer::from(*b) == *i,
                Obj::Bool(same) => *b == *same,
                _ => false,
            },
            (Obj::Str(s1), Obj::Str(s2)) => s1 == s2,
            (Obj::Dict(_), _) | (_, Obj::Dict(_)) => false,
            (_, _) => false,
        }
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl PartialOrd for PyObject {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.lt(other) {
            return Some(std::cmp::Ordering::Less);
        }
        if self.gt(other) {
            return Some(std::cmp::Ordering::Greater);
        }
        if self.eq(other) {
            return Some(std::cmp::Ordering::Equal);
        }
        return None;
    }

    fn lt(&self, other: &Self) -> bool {
        let ret = match (&self.obj, &other.obj) {
            (Obj::Float(flt), other) => match other {
                Obj::Float(same) => *flt < *same,
                Obj::Int(i) => *flt < i.to_f64(),
                Obj::Bool(b) => *flt < f64::from(*b),
                _ => false,
            },
            (Obj::Int(i), other) => match other {
                Obj::Float(flt) => i.to_f64() < *flt,
                Obj::Int(same) => *i < *same,
                Obj::Bool(b) => *i < Integer::from(*b),
                _ => false,
            },
            (Obj::Bool(b), other) => match other {
                Obj::Float(f) => f64::from(*b) < *f,
                Obj::Int(i) => Integer::from(*b) < *i,
                Obj::Bool(same) => *b < *same,
                _ => false,
            },
            (Obj::Str(s1), Obj::Str(s2)) => s1 < s2,
            _ => false,
        };
        ret
    }

    fn gt(&self, other: &Self) -> bool {
        let ret = match (&self.obj, &other.obj) {
            (Obj::Float(flt), other) => match other {
                Obj::Float(same) => *flt > *same,
                Obj::Int(i) => *flt > i.to_f64(),
                Obj::Bool(b) => *flt > f64::from(*b),
                _ => false,
            },
            (Obj::Int(i), other) => match other {
                Obj::Float(flt) => i.to_f64() > *flt,
                Obj::Int(same) => *i > *same,
                Obj::Bool(b) => *i > Integer::from(*b),
                _ => false,
            },
            (Obj::Bool(b), other) => match other {
                Obj::Float(f) => f64::from(*b) > *f,
                Obj::Int(i) => Integer::from(*b) > *i,
                Obj::Bool(same) => *b > *same,
                _ => false,
            },
            (Obj::Str(s1), Obj::Str(s2)) => s1 > s2,
            _ => false,
        };
        ret
    }

    fn ge(&self, other: &Self) -> bool {
        self.gt(other) || self.eq(other)
    }

    fn le(&self, other: &Self) -> bool {
        self.lt(other) || self.eq(other)
    }
}

impl std::fmt::Display for PyObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.__str__())
    }
}

impl Default for Obj {
    fn default() -> Self {
        Obj::None
    }
}

impl Termination for Obj {
    fn report(self) -> std::process::ExitCode {
        match self {
            _ => ExitCode::SUCCESS,
        }
    }
}

impl<T: ToObj> From<T> for PyObject {
    fn from(value: T) -> Self {
        value.to_pyobj()
    }
}

impl core::hash::Hash for Obj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Obj::None => {}
            Obj::Bool(b) => b.hash(state),
            Obj::Float(f) => f.to_le_bytes().hash(state),
            Obj::Str(s) => s.hash(state),
            Obj::Int(i) => i.hash(state),
            Obj::FuncPtr(f) => f.hash(state),
            Obj::Except(e) => e.hash(state),
            Obj::List(v) => v.hash(state),
            Obj::Set(v) => v.hash(state),
            Obj::Tuple(v) => v.hash(state),
            Obj::Range(v) => v.hash(state),
            Obj::Dict(h) => PyUtils::hash_hashmap(h, state),
            Obj::Iter(a) => a.hash(state),
            Obj::Type(c) => c.hash(state),
            Obj::ClassInst(c) => c.hash(state),
            Obj::Code(c) => c.hash(state),
            Obj::FunctionObj(f) => f.hash(state),
            Obj::BuildClass => {},
        }
    }
}

// obj iter
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct PyObjIter {
    items: Vec<PyObjPtr>,
    index: usize,
}

impl PyObjIter {
    pub fn from(obj: &PyObjPtr) -> Option<Self> {
        let iter = match &obj.get_ref().obj {
            Obj::List(v) |
            Obj::Tuple(v) | 
            Obj::Set(v) => PyObjIter {
                items: v.clone(),
                index: 0,
            },
            Obj::Str(s) => {
                let items = s
                    .chars()
                    .map(|c| PyObject::new_str(c).to_ptr())
                    .collect();
                PyObjIter { items, index: 0 }
            }
            Obj::Dict(m) => {
                let items = m.keys().cloned().map(|k| k).collect();
                PyObjIter { items, index: 0 }
            }
            _ => return None,
        };
        Some(iter)
    }

    pub fn get_curr(&self) -> Option<PyObjPtr> {
        self.items.get(self.index).cloned()
    }

    pub fn get_items(self) -> Vec<PyObjPtr> {
        self.items
    }
}

impl Iterator for PyObjIter {
    type Item = PyObjPtr;
    fn next(&mut self) -> Option<Self::Item> {
        let out = self.get_curr();
        if out.is_some() {
            self.index += 1;
        }
        out
    }
}

// obj iter
#[derive(Debug, Clone, PartialEq)]
pub struct PyObjIntoIter {
    items: Vec<PyObjPtr>,
    index: usize,
}

impl PyObjIntoIter {
    fn from(obj: PyObjPtr) -> Option<Self> {
        let iter = match &obj.get_ref().obj {
            Obj::List(v) => {
                PyObjIntoIter {
                    items: v.clone(),
                    index: 0,
                } // not correct
            }
            Obj::Str(s) => {
                let items = s
                    .chars()
                    .map(|c| PyObject::new_str(c).to_ptr())
                    .collect();
                PyObjIntoIter { items, index: 0 }
            }
            Obj::Dict(m) => {
                let items = m.keys().cloned().map(|k| k).collect();
                PyObjIntoIter { items, index: 0 }
            }
            Obj::Range(r) => {
                let items = r.clone().to_vec();
                PyObjIntoIter { items, index: 0 }
            }
            _ => return None,
        };
        Some(iter)
    }

    fn get_curr(&self) -> Option<PyObjPtr> {
        self.items.get(self.index).cloned()
    }
}

impl Iterator for PyObjIntoIter {
    type Item = PyObjPtr;
    fn next(&mut self) -> Option<Self::Item> {
        let out = self.get_curr();
        if out.is_some() {
            self.index += 1;
        }
        out
    }
}

impl IntoIterator for PyObjPtr {
    type Item = PyObjPtr;
    type IntoIter = PyObjIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        PyObjIntoIter::from(self).expect("not able to iterate").into_iter()
    }
}

// Add this near the other iterator impls (after ObjIntoIter)
impl PyObject {
    pub fn iter_py(&self) -> Option<PyObjIter> {
        match &self.obj {
            Obj::List(v) => {
                Some(PyObjIter {
                    items: v.clone(),
                    index: 0,
                })
            }
            Obj::Tuple(v) | Obj::Set(v) => Some(PyObjIter {
                items: v.clone(),
                index: 0,
            }),
            Obj::Str(s) => {
                let items = s
                    .chars()
                    .map(|c| PyObject::new_str(c).to_ptr())
                    .collect();
                Some(PyObjIter { items, index: 0 })
            }
            Obj::Dict(m) => {
                let items = m.keys().cloned().map(|k| k).collect();
                Some(PyObjIter { items, index: 0 })
            }
            Obj::Range(r) => {
                let items = r.clone().to_vec();
                Some(PyObjIter { items, index: 0 })
            }
            _ => None,
        }
    }

    pub fn __getattr__(&self, field: &String) -> Result<PyObjPtr, PyException> {
        match &self.obj {
            Obj::ClassInst(inst) => {
                match inst.fields.get(field).cloned()  {
                    Some(obj) => {
                        return Ok(obj.clone());
                    }
                    None => { 
                        Err(PyException {
                            error: PyError::UndefinedVariableError,
                            msg: format!("no field \'{field}\' in obj {:?}", self),
                        })
                    }
                }
            }
            _ => {
                Err(PyException {
                    error: PyError::NotImplementedError,
                    msg: format!("cannot use __get_attr__ for {:?}", self),
                })
            }
        }
    }

    pub fn __set_attr__(&mut self, field: &String, val: PyObjPtr) -> Option<PyException> {
        match &mut self.obj {
            Obj::ClassInst(inst) => {
                match inst.fields.get_mut(field) {
                    Some(obj) => {
                        *obj = val;
                        None
                    }
                    None => Some(PyException {
                        error: PyError::UndefinedVariableError,
                        msg: format!("no field \'{field}\' in obj {:?}", self),
                    }),
                }
            }
            _ => None,
        }
    }
}

// Extension trait so PyObjPtr.iter() and PyObjPtr.into_obj_iter() are available
pub trait ArcObjIterExt {
    fn iter(&self) -> Option<PyObjIter>;
    fn into_obj_iter(self) -> Option<PyObjIntoIter>;
}

impl ArcObjIterExt for PyObjPtr {
    fn iter(&self) -> Option<PyObjIter> {
        // ObjIter::from takes PyObjPtr and returns Option<ObjIter>
        PyObjIter::from(&self)
    }

    fn into_obj_iter(self) -> Option<PyObjIntoIter> {
        PyObjIntoIter::from(self)
    }
}

pub trait ToObj: Sized + Clone {
    fn to_pyobj(self) -> PyObject;
    fn to_pyptr(self) -> PyObjPtr {
        self.to_pyobj().to_ptr()
    }
}

impl ToObj for Expression {
    fn to_pyobj(self) -> PyObject {
        match self {
            Expression::Atom(atom) => PyObject::from_atom(&atom),
            Expression::Operation(op, args) => match op {
                Op::List => {
                    let mut objs = vec![];
                    for a in args {
                        objs.push(a.to_pyptr());
                    }
                    objs.to_pyobj()
                }
                Op::Plus => {
                    let lhs = args.first().cloned().unwrap().to_pyobj();
                    let rhs = args.last().cloned().unwrap().to_pyobj();
                    let sum = PyObject::add(&lhs, &rhs);
                    sum
                }
                _ => PyObject::new_exception(PyException {
                    error: PyError::TypeError,
                    msg: format!("cannot convert op {:#?} with args {:#?} to Obj", op, args),
                }),
            },
            _ => PyObject::new_exception(PyException {
                error: PyError::TypeError,
                msg: format!("cannot convert {:#?} to Obj", self),
            }),
        }
    }
}

impl ToObj for PyException {
    fn to_pyobj(self) -> PyObject {
        PyObject::new_exception(self)
    }
}

impl ToObj for PyCodeObj {
    fn to_pyobj(self) -> PyObject {
        PyObject::new_codeobj(self)
    }
}
impl ToObj for Arc<PyCodeObj> {
    fn to_pyobj(self) -> PyObject {
        PyObject::new_codeobj_arc(self)
    }
}

impl ToObj for PyClassInst {
    fn to_pyobj(self) -> PyObject {
        PyObject::new_classinst(self)
    }
}

impl ToObj for PyTypeObj {
    fn to_pyobj(self) -> PyObject {
        PyObject::new_type(self)
    }
}

impl ToObj for FnPtr {
    fn to_pyobj(self) -> PyObject {
        PyObject::new_fnptr(self)
    }
}

impl ToObj for rug::Integer {
    fn to_pyobj(self) -> PyObject {
        PyObject::new_int(self)
    }

}

impl ToObj for PyObjIter {
    fn to_pyobj(self) -> PyObject {
        PyObject::new_iter(self)
    }
}

macro_rules! impl_to_obj_for_int {
    ($($ty:ty),+) => {
        $(
            impl ToObj for $ty {
                fn to_pyobj(self) -> PyObject {
                    PyObject::new_int(Integer::from(self))
                }
            }
        )+
    };
}
impl_to_obj_for_int!(i8, u8, u16, i16, u32, i32, u64, i64, usize);

macro_rules! impl_to_obj_for_float {
    ($($ty:ty),+) => {
        $(
            impl ToObj for $ty {
                fn to_pyobj(self) -> PyObject {
                    PyObject::new_float(self as f64)
                }
            }
        )+
    };
}
impl_to_obj_for_float!(f32, f64);

impl ToObj for bool {
    fn to_pyobj(self) -> PyObject {
        PyObject::new_bool(self)
    }
}
impl ToObj for String {
    fn to_pyobj(self) -> PyObject {
        PyObject::from_atom(&self)
    }
}
impl ToObj for &str {
    fn to_pyobj(self) -> PyObject {
        PyObject::from_atom(self)
    }
}

impl ToObj for Vec<PyObjPtr> {
    fn to_pyobj(self) -> PyObject {
        PyObject::new_list(self)
    }
}
