use crate::{
    pyrs_error::{PyError, PyException},
    pyrs_parsing::{Expression, Op},
    pyrs_std::{FnPtr, RangeObj},
    pyrs_userclass::{CustomClass},
    pyrs_codeobject::{CodeObj, FuncObj},
    pyrs_modules::PyModule,
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

    Null,
    None,

    Bool(bool),
    Float(f64),
    Str(String),
    Int(Integer),

    Function(FnPtr),

    Except(PyException),

    List(Arc<Mutex<Vec<Arc<Obj>>>>),  // [], mutable, ordered, duplicates, int indexing,
    Tuple(Vec<Arc<Obj>>), // (), immutable, ordered, duplicates, int indexing,
    Set(Vec<Arc<Obj>>),   // {}, mutable, unordered, no dupes, no indexing,
    Range(RangeObj),

    Dict(HashMap<Obj, Arc<Obj>>),

    Iter(ObjIter),

    CustomClass(CustomClass),

    Code(CodeObj),
    Func(FuncObj),

    Module(PyModule),
    // Binary
    // - bytes
    // - bytearray,
    // - memoryview,

    // Set
    // - frozenset

    // Mapping
    // - dict (HashMap)
}

pub trait PyObj: std::fmt::Debug + Clone {

    fn compare_op(lhs: &Arc<Self>, rhs: &Arc<Self>, op: &Op) -> bool {
        let ret = match op {
            Op::Eq => Self::__eq__(lhs, rhs),
            Op::Neq => Self::__ne__(lhs, rhs),
            Op::LessThan => Self::__lt__(lhs, rhs),
            Op::GreaterThan => Self::__gt__(lhs, rhs),
            Op::LessEq => Self::__le__(lhs, rhs),
            Op::GreaterEq => Self::__ge__(lhs, rhs),
            _ => return Self::__default__().__bool__(),
        };
        ret
    }

    fn __dict__(&self, _ident: &String) -> Option<&Arc<Obj>> {
        panic!();
    }

    fn __default__() -> Self {
        panic!()
    }

    fn __str__(&self) -> String {
        panic!()
    }

    fn __unpack__(self) -> Result<Vec<Arc<Obj>>, PyException> {
        Err(PyException {
            error: PyError::TypeError,
            msg: format!("Unable to deref the PyObj: {:?}", self),
        })
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    fn __int__(&self) -> isize {
        panic!();
    }

    fn __bool__(&self) -> bool {
        false
    }
    fn __len__(&self) -> usize {
        unimplemented!();
    }
    fn __lt__(_lhs: &Arc<Self>, _rhs: &Arc<Self>) -> bool {
        false
    }
    fn __gt__(_lhs: &Arc<Self>, _rhs: &Arc<Self>) -> bool {
        false
    }
    fn __le__(_lhs: &Arc<Self>, _rhs: &Arc<Self>) -> bool {
        false
    }
    fn __ge__(_lhs: &Arc<Self>, _rhs: &Arc<Self>) -> bool {
        false
    }
    fn __eq__(_lhs: &Arc<Self>, _rhs: &Arc<Self>) -> bool {
        false
    }
    fn __ne__(_lhs: &Arc<Self>, _rhs: &Arc<Self>) -> bool {
        false
    }

    fn __add__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        Err(PyException {
            error: PyError::TypeError,
            msg: format!("Unable to add the two PyObj types : {:?}, {:?}", lhs, rhs),
        })
    }

    fn __sub__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        Err(PyException {
            error: PyError::TypeError,
            msg: format!(
                "Unable to subtract the two PyObj types : {:?}, {:?}",
                lhs, rhs
            ),
        })
    }
    fn __mul__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        Err(PyException {
            error: PyError::TypeError,
            msg: format!(
                "Unable to multiply the two PyObj types : {:?}, {:?}",
                lhs, rhs
            ),
        })
    }
    fn __div__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        Err(PyException {
            error: PyError::TypeError,
            msg: format!(
                "Unable to divide the two PyObj types : {:?}, {:?}",
                lhs, rhs
            ),
        })
    }

    fn __pos__(obj: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        Ok(obj.clone())
    }

    fn __neg__(obj: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        Err(PyException {
            error: PyError::TypeError,
            msg: format!(" __neg__: not implemented for {:?}", obj),
        })
    }

    fn __call__(&self, objs: &Vec<Arc<Self>>) -> Result<Arc<Self>, PyException> {
        Err(PyException {
            error: PyError::TypeError,
            msg: format!(" __call__: not implemented for {:?}", objs),
        })
    }

    fn to_arc(self) -> Arc<Self> {
        Arc::from(self)
    }
}

impl Obj {
    pub fn from<T: ToObj>(arg: T) -> Arc<Obj> {
        arg.to_arc()
    }

    pub fn new_vec() -> Vec<Obj> {
        return vec![];
    }

    pub fn new_arc_vec() -> Vec<Arc<Obj>> {
        return vec![];
    }

    pub fn new_map() -> HashMap<String, Arc<Obj>> {
        return HashMap::new();
    }

    pub fn new_dict() -> Obj {
        Obj::Dict(HashMap::new())
    }

    pub fn is_num(&self) -> bool {
        match self {
            Obj::Float(_) | Obj::Int(_) => true,
            _ => false,
        }
    }

    pub fn from_str(s: &str) -> Obj {
        Obj::Str(s.to_string())
    }

    pub fn repr(&self) -> &str {
        unimplemented!();
    }

    pub fn from_atom(c: &str) -> Self {
        if let Ok(val) = Integer::from_str(c) {
            return Obj::Int(val);
        }
        if let Ok(val) = c.parse::<f64>() {
            return Obj::Float(val);
        } else {
            Obj::Str(c.to_string())
        }
    }

    pub fn is_iterable(&self) -> bool {
        match self {
            Obj::Set(_) | Obj::Str(_) | Obj::List(_) | Obj::Dict(_) | Obj::Tuple(_) => true,
            _ => false,
        }
    }

    pub fn iter_next(&mut self) -> Option<Arc<Obj>> {
        match self {
            Obj::Iter(i) => i.next(),
            _ => None,
        }
    }

    pub fn add(lhs: &Obj, rhs: &Obj) -> Obj {
        let err = Obj::Except(PyException {
            error: PyError::TypeError,
            msg: format!("No valid way to add: {} and {}", lhs, rhs.clone(),),
        });

        let obj = match (lhs, rhs) {
            (Obj::Float(dbl), other) => {
                let val = match other {
                    Obj::Float(v) => *v,
                    Obj::Int(v) => v.to_f64(),
                    _ => return err,
                };
                Obj::Float(dbl + val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => Obj::Int(int.clone().add(v)),
                Obj::Float(v) => Obj::Float(int.to_f64() + v),
                _ => return err,
            },
            (Obj::Str(s), other) => match other {
                Obj::Str(v) => Obj::Str(format!("{s}{v}")),
                _ => return err,
            },
            (Obj::List(l1), other) => match other {
                Obj::List(l2) => {
                    let l1_mut = l1.lock().expect("Unable to lock l1");
                    let l2_mut = l2.lock().expect("Unable to lock l2");
                    let mut new_list = Vec::with_capacity(l1_mut.len() + l2_mut.len());
                    new_list.extend(l1_mut.iter().cloned());
                    new_list.extend(l2_mut.iter().cloned());
                    Obj::List(Mutex::new(new_list).into())
                }
                _ => {
                    return Obj::Except(PyException {
                        error: PyError::TypeError,
                        msg: format!(
                            "TypeError: can only concatenate list (not \"{:?}\") to list",
                            other
                        ),
                    });
                }
            },
            _ => return err,
        };
        obj
    }

    pub fn sub(lhs: &Obj, rhs: &Obj) -> Obj {
        let err = Obj::Except(PyException {
            error: PyError::TypeError,
            msg: format!("No valid way to subtract: {} and {}", lhs, rhs.clone(),),
        });

        let obj = match (lhs, rhs) {
            (Obj::Float(dbl), other) => {
                let val = match other {
                    Obj::Float(v) => *v,
                    Obj::Int(v) => v.to_f64(),
                    _ => return err,
                };
                Obj::Float(dbl - val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => Obj::Int(int.clone().sub(v)),
                Obj::Float(v) => Obj::Float(int.to_f64() - v),
                _ => return err,
            },
            _ => return err,
        };
        obj
    }

    pub fn mul(lhs: &Obj, rhs: &Obj) -> Obj {
        let err = Obj::Except(PyException {
            error: PyError::TypeError,
            msg: format!("No valid way to subtract: {} and {}", lhs, rhs.clone(),),
        });

        let obj = match (lhs, rhs) {
            (Obj::Float(dbl), other) => {
                let val = match other {
                    Obj::Float(v) => *v,
                    Obj::Int(v) => v.to_f64(),
                    _ => return err,
                };
                Obj::Float(dbl * val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => Obj::Int(int.clone().mul(v)),
                Obj::Float(v) => Obj::Float(int.to_f64() * v),
                _ => return err,
            },
            (Obj::Str(s), other) => match other {
                Obj::Int(v) => {
                    if *v >= 0 {
                        let mut mult = String::new();
                        for _i in 0..v.to_u64().unwrap() {
                            mult = format!("{mult}{s}");
                        }
                        Obj::Str(mult)
                    } else {
                        return Obj::Except(PyException {
                            error: PyError::TypeError,
                            msg: format!(" can't multiply sequence by non-int of type {}", lhs),
                        });
                    }
                }
                _ => return err,
            },
            _ => return err,
        };
        obj
    }

    pub fn div(lhs: &Obj, rhs: &Obj) -> Obj {
        let type_err = Obj::Except(PyException {
            error: PyError::TypeError,
            msg: format!("No valid way to divide: {} and {}", lhs, rhs.clone(),),
        });
        let zero_div_err = Obj::Except(PyException {
            error: PyError::ZeroDivisionError,
            msg: format!(" tried to divide {lhs} by {rhs}"),
        });

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
                Obj::Float(dbl / val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => {
                    if *v == Integer::ZERO {
                        return zero_div_err;
                    }
                    Obj::Float(int.to_f64() / v.to_f64())
                }
                Obj::Float(v) => {
                    if *v == 0f64 {
                        return zero_div_err;
                    }
                    Obj::Float(int.to_f64() / v)
                }
                _ => return type_err,
            },
            _ => return type_err,
        };
        obj.into()
    }
}

impl PyObj for Obj {
    fn __default__() -> Self {
        Obj::None
    }

    fn __dict__(&self, field: &String) -> Option<&Arc<Obj>> {
        match self {
            Obj::CustomClass(o) => {
                Some(&o.fields[field])
            },
            _ => None,
        }
    }

    fn __int__(&self) -> isize {
        match self {
            Obj::Bool(v) => *v as isize,
            Obj::Int(v) => v.to_isize_wrapping(),
            Obj::Float(v) => *v as isize,
            _ => panic!(),
        }
    }

    fn __bool__(&self) -> bool {
        let ret = match self {
            Obj::None => false,
            Obj::Bool(v) => *v,
            Obj::Float(v) => *v != 0f64,
            Obj::Int(v) => *v != Integer::ZERO,
            Obj::Str(v) => *v != "",
            Obj::Tuple(vec) | Obj::Set(vec) => vec.len() != 0usize,
            Obj::List(vec) => { 
                let locked = vec.lock().expect("Unable to lock list");
                locked.len() != 0usize 
            }
            _ => panic!("TypeError: __bool__() not implemented for: {:?}", self),
        };
        return ret;
    }

    fn __unpack__(self) -> Result<Vec<Arc<Obj>>, PyException> {
        if self.is_iterable() {
            Ok(match self {
                Obj::Set(vec) |
                Obj::Tuple(vec) => vec, 
                Obj::List(vec) => {
                    let lock = vec.lock().expect("Unable to lock list");
                    lock.clone()
                }
                Obj::Range(range) => range.to_vec(),
                Obj::Dict(dict) => { 
                    dict.into_iter()
                    .map(|(key, _) | Arc::new(key))
                    .collect()
                },
                _ => unreachable!(),
            })
        }
        else {
            Err(PyException { 
                error: PyError::TypeError, 
                msg: format!("Cannot unpack a non iterable type: {:?}", self) 
            })
        }
    }

    fn __str__(&self) -> String {
        match self {
            Obj::Null => format!(""),
            Obj::None => format!("None"),
            Obj::Bool(val) => match val {
                true => format!("True"),
                false => format!("False"),
            },
            Obj::Float(val) => format!("{}", val),
            Obj::Str(s) => format!("{}", s),
            Obj::Int(val) => format!("{}", val),
            Obj::Function(ptr) => format!("{}", ptr),
            Obj::Except(e) => format!("{}", e),
            Obj::List(v) => {
                let objs = &*v.lock().expect("Unable to lock list");
                let mut list = String::from("[");
                for o in objs {
                    list.push_str(o.__repr__().as_str());
                    list.push(',');
                    list.push(' ');
                }
                list.pop();
                list.pop();
                list.push_str("]");
                format!("{}", list)
            }
            Obj::Tuple(objs) => {
                let mut tuple = String::from("(");
                for o in objs {
                    tuple.push_str(o.__repr__().as_str());
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
                    set.push_str(o.__repr__().as_str());
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
                    map.push_str(key.__repr__().as_str());
                    map.push(':');
                    map.push_str(value.__repr__().as_str());
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
                r
            }
            Obj::Iter(iter) => {
                format!("Iter[ {:#?} {} ]", iter.items, iter.index)
            }
            Obj::CustomClass(class ) => {
                format!("<class \'__main__.{}\'>", class.name)
            }
            Obj::Func(func) => {
                format!("<function {:?} >", func)
            }
            Obj::Module(module) => {
                format!("<module {} >", module.name)
            }
        }
    }

    fn __repr__(&self) -> String {
        match self {
            Obj::Str(s) => format!("\'{}\'", s),
            _ => self.__str__(),
        }
    }

    fn __len__(&self) -> usize {
        match self {
            Obj::List(v) => {
                let list = v.lock().expect("Unable to lock list");
                list.len()
            }
            _ => panic!("TypeError: __len__() not implemented for: {:?}", self),
        }
    }

    fn compare_op(lhs: &Arc<Obj>, rhs: &Arc<Obj>, op: &Op) -> bool {
        let ret = match op {
            Op::Eq => lhs.eq(rhs),
            Op::Neq => lhs.ne(rhs),
            Op::LessThan => lhs.lt(rhs),
            Op::GreaterThan => lhs.gt(rhs),
            Op::LessEq => lhs.le(rhs),
            Op::GreaterEq => lhs.ge(rhs),
            _ => return Obj::__default__().__bool__(),
        };
        ret
    }

    fn __lt__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> bool {
        lhs.as_ref().lt(rhs.as_ref())
    }

    fn __gt__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> bool {
        lhs.as_ref().gt(rhs.as_ref())
    }

    fn __le__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> bool {
        Obj::__lt__(lhs, rhs) || Obj::__eq__(lhs, rhs)
    }
    fn __ge__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> bool {
        Obj::__gt__(lhs, rhs) || Obj::__eq__(lhs, rhs)
    }

    fn __add__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        match Obj::add(lhs.as_ref(), rhs.as_ref()) {
            Obj::Except(e) => Err(e),
            o => Ok(o.into()),
        }
    }

    fn __sub__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        match Obj::sub(lhs.as_ref(), rhs.as_ref()) {
            Obj::Except(e) => Err(e),
            o => Ok(o.into()),
        }
    }

    fn __mul__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        match Obj::mul(lhs.as_ref(), rhs.as_ref()) {
            Obj::Except(e) => Err(e),
            o => Ok(o.into()),
        }
    }

    fn __div__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        match Obj::div(lhs.as_ref(), rhs.as_ref()) {
            Obj::Except(e) => Err(e),
            o => Ok(o.into()),
        }
    }

    fn __eq__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> bool {
        lhs.eq(rhs)
    }

    fn __ne__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> bool {
        !Obj::__eq__(lhs, rhs)
    }

    fn __pos__(obj: &Arc<Obj>) -> Result<Arc<Obj>, PyException> {
        Ok(Arc::from(obj.clone()))
    }

    fn __neg__(obj: &Arc<Obj>) -> Result<Arc<Obj>, PyException> {
        let ret = match obj.as_ref() {
            Obj::None => Obj::None,
            Obj::Bool(b) => Obj::Bool(!b),
            Obj::Float(f) => Obj::Float(-f),
            Obj::Int(i) => Obj::Int(i.clone().neg()),
            _ => {
                return Err(PyException {
                    error: PyError::NotImplementedError,
                    msg: format!("Negation not implemented for {}", obj),
                })
            }
        };
        Ok(ret.into())
    }

    fn __call__(&self, objs: &Vec<Arc<Obj>>) -> Result<Arc<Obj>, PyException> {
        match self {
            Obj::Function(fn_ptr) => Ok((fn_ptr.ptr)(objs)),
            _ => Err(PyException {
                error: PyError::TypeError,
                msg: format!("Type is not a function"),
            }),
        }
    }

    fn to_arc(self) -> Arc<Self> {
        Arc::from(self)
    }
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Obj::Null, Obj::Null) |
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

impl PartialOrd for Obj {
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
        let ret = match (self, other) {
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
        let ret = match (self, other) {
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

impl std::fmt::Display for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.__str__())
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
            Obj::Null => ExitCode::FAILURE,
            _ => ExitCode::SUCCESS,
        }
    }
}

impl<T :ToObj> From<T> for Obj {
    fn from(value: T) -> Self {
        value.to_obj()
    }
}

// obj iter
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ObjIter {
    items: Vec<Arc<Obj>>,
    index: usize,
}

impl ObjIter 
{
    pub fn from(obj: &Arc<Obj>) -> Option<Self> {
        let iter = match obj.as_ref() {
            Obj::List(v) => {
                let list = v.lock().expect("Unable to lock list");
                ObjIter {
                    items: list.clone(),
                    index: 0,
                }
            }
            Obj::Tuple(v) | Obj::Set(v) => {
                ObjIter {
                    items: v.clone(),
                    index: 0,
                }
            }
            Obj::Str(s) => {
                let items = s
                    .chars()
                    .map(|c| Arc::new(Obj::Str(c.to_string())))
                    .collect();
                ObjIter { items, index: 0 }
            }
            Obj::Dict(m) => {
                let items = m.keys().cloned().map(|k| Arc::new(k)).collect();
                ObjIter { items, index: 0 }
            }
            _ => return None,
        };
        Some(iter)
    }

    pub fn get_curr(&self) -> Option<Arc<Obj>> {
        self.items.get(self.index).cloned()
    }

    pub fn get_items(self) -> Vec<Arc<Obj>>
    {
        self.items
    }
}

impl Iterator for ObjIter {
    type Item = Arc<Obj>;
    fn next(&mut self) -> Option<Self::Item> {
        let out = self.get_curr();
        if out.is_some() {
            self.index += 1;
        }
        out
    }
}

// obj iter
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ObjIntoIter {
    items: Vec<Arc<Obj>>,
    index: usize,
}

impl ObjIntoIter {
    fn from(obj: Arc<Obj>) -> Option<Self> {
        let iter = match obj.as_ref() {
            Obj::List(v) => {
                let list = v.lock().expect("Unable to lock list");
                ObjIntoIter { items: list.clone(), index: 0 } // not correct
            }
            Obj::Str(s) => {
                let items = s
                    .chars()
                    .map(|c| Arc::new(Obj::Str(c.to_string())))
                    .collect();
                ObjIntoIter { items, index: 0 }
            }
            Obj::Dict(m) => {
                let items = m.keys().cloned().map(|k| Arc::new(k)).collect();
                ObjIntoIter { items, index: 0 }
            }
            _ => return None,
        };
        Some(iter)
    }

    fn get_curr(&self) -> Option<Arc<Obj>> {
        self.items.get(self.index).cloned()
    }
}

impl Iterator for ObjIntoIter {
    type Item = Arc<Obj>;
    fn next(&mut self) -> Option<Self::Item> {
        let out = self.get_curr();
        if out.is_some() {
            self.index += 1;
        }
        out
    }
}

impl IntoIterator for Obj {
    type Item = Arc<Obj>;
    type IntoIter = ObjIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        ObjIntoIter::from(Arc::new(self)).expect("Not Iterable")
    }
}

// Add this near the other iterator impls (after ObjIntoIter)
impl Obj {
    pub fn iter_py(&self) -> Option<ObjIter> {
        match self {
            Obj::List(v) => {
                let list = v.lock().expect("Unable to lock list");
                Some(ObjIter {
                    items: list.clone(),
                    index: 0,
                })
            }
            Obj::Tuple(v) | Obj::Set(v) => Some(ObjIter {
                items: v.clone(),
                index: 0,
            }),
            Obj::Str(s) => {
                let items = s
                    .chars()
                    .map(|c| Arc::new(Obj::Str(c.to_string())))
                    .collect();
                Some(ObjIter { items, index: 0 })
            }
            Obj::Dict(m) => {
                let items = m.keys().cloned().map(|k| Arc::new(k)).collect();
                Some(ObjIter { items, index: 0 })
            }
            _ => None,
        }
    }
}

// Extension trait so Arc<Obj>.iter() and Arc<Obj>.into_obj_iter() are available
pub trait ArcObjIterExt {
    fn iter(&self) -> Option<ObjIter>;
    fn into_obj_iter(self) -> Option<ObjIntoIter>;
}

impl ArcObjIterExt for Arc<Obj> {
    fn iter(&self) -> Option<ObjIter> {
        // ObjIter::from takes Arc<Obj> and returns Option<ObjIter>
        ObjIter::from(&self)
    }

    fn into_obj_iter(self) -> Option<ObjIntoIter> {
        ObjIntoIter::from(self)
    }
}

pub trait ToObj: Sized + Clone {
    fn to_obj(self) -> Obj {
        PyObj::__default__()
    }
    fn to_arc(self) -> Arc<Obj> {
        self.to_obj().into()
    }
}

impl ToObj for Expression {
    fn to_arc(self) -> Arc<Obj> {
        self.to_obj().into()
    }

    fn to_obj(self) -> Obj {
        match self {
            Expression::Atom(atom) => Obj::from_atom(&atom),
            Expression::Operation(op, args) => match op {
                Op::List => {
                    let mut objs = vec![];
                    for a in args {
                        objs.push(a.to_arc());
                    }
                    objs.to_obj()
                }
                Op::Plus => {
                    let lhs = args.first().cloned().unwrap().to_obj();
                    let rhs = args.last().cloned().unwrap().to_obj();
                    let sum = Obj::add(&lhs, &rhs);
                    sum
                }
                _ => Obj::Except(PyException {
                    error: PyError::TypeError,
                    msg: format!("cannot convert op {:#?} with args {:#?} to Obj", op, args),
                }),
            },
            _ => Obj::Except(PyException {
                error: PyError::TypeError,
                msg: format!("cannot convert {:#?} to Obj", self),
            }),
        }
    }
}

impl ToObj for PyException {
    fn to_obj(self) -> Obj {
        Obj::Except(self)
    }
    fn to_arc(self) -> Arc<Obj> {
        Obj::Except(self).into()
    }
}

impl ToObj for rug::Integer {
    fn to_obj(self) -> Obj {
        Obj::Int(self)
    }
    fn to_arc(self) -> Arc<Obj> {
        self.to_obj().into()
    }
}

macro_rules! impl_to_obj_for_int {
    ($($ty:ty),+) => {
        $(
            impl ToObj for $ty {
                fn to_obj(self) -> Obj {
                    Obj::Int(Integer::from(self))
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
                fn to_obj(self) -> Obj {
                    Obj::Float(self as f64)
                }
            }
        )+
    };
}
impl_to_obj_for_float!(f32, f64);

impl ToObj for bool {
    fn to_obj(self) -> Obj {
        Obj::Bool(self)
    }
}
impl ToObj for String {
    fn to_obj(self) -> Obj {
        Obj::from_atom(&self)
    }
}
impl ToObj for &str {
    fn to_obj(self) -> Obj {
        Obj::from_atom(self)
    }
}

impl ToObj for Vec<Arc<Obj>> {
    fn to_obj(self) -> Obj {
        Obj::List(Arc::new(Mutex::new(self)))
    }
}
