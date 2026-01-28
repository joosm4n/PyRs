use std::{
    collections::HashMap, ops::{Add, Mul, Neg}, process::{ExitCode, Termination}, str::FromStr, sync::Arc
};
use crate::{
    pyrs_error::{PyException, PyError},
    pyrs_std::{FnPtr},
    pyrs_parsing::{Op},
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

    Function(FnPtr),

    Except(PyException),

    List(Vec<Arc<Obj>>), // [], mutable, ordered, duplicates, int indexing, 
    Tuple(Vec<Arc<Obj>>), // (), immutable, ordered, duplicates, int indexing,
    Set(Vec<Arc<Obj>>), // {}, mutable, unordered, no dupes, no indexing, 

    Dict(HashMap<Obj, Arc<Obj>>),

    //User(UserClass),

    // Iterator
    // - containters

    // Sequence
    // - range

    // Binary
    // - bytes
    // - bytearray,
    // - memoryview,

    // Set
    // - frozenset

    // Mapping
    // - dict (HashMap)
}
pub trait PyObj: std::fmt::Debug + Clone 
{
    fn compare_op(lhs: &Arc<Self>, rhs: &Arc<Self>, op: &Op) -> bool
    {
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

    fn __default__() -> Self {
        panic!()
    }

    fn __str__(&self) -> String {
        panic!()
    }

    fn __deref__(obj: &Arc<Self>) -> Result<Arc<Obj>, PyException> {
        Err(PyException{
            error: PyError::TypeError,
            msg: format!(
                "Unable to deref the PyObj: {:?}",
                obj
            ),
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
        Err(PyException{
            error: PyError::TypeError,
            msg: format!(
                "Unable to add the two PyObj types : {:?}, {:?}",
                lhs, rhs
            ),
        })
    }

    fn __sub__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        Err(PyException{
            error: PyError::TypeError,
            msg: format!(
                "Unable to subtract the two PyObj types : {:?}, {:?}",
                lhs, rhs
            ),
        })
    }
    fn __mul__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        Err(PyException{
            error: PyError::TypeError,
            msg: format!(
                "Unable to multiply the two PyObj types : {:?}, {:?}",
                lhs, rhs
            ),
        })
    }
    fn __div__(lhs: &Arc<Self>, rhs: &Arc<Self>) -> Result<Arc<Self>, PyException> {
        Err(PyException{
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
        Err(PyException{
            error: PyError::TypeError,
            msg: format!(" __neg__: not implemented for {:?}", obj),
        })
    }

    fn __call__(&self, objs: &Vec<Arc<Self>>) -> Result<Arc<Self>, PyException>
    {
        Err(PyException{
            error: PyError::TypeError,
            msg: format!(" __call__: not implemented for {:?}", objs),
        })
    }

    fn to_arc(self) -> Arc<Self>
    {
        Arc::from(self)
    }
}

impl Obj {

    pub fn from<T: ToObj>(arg: T) -> Arc<Obj> {
        arg.to_arc()
    }

    pub fn new_vec() -> Vec<Obj>
    {
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
            return Obj::Int(val)
        }
        if let Ok(val) = c.parse::<f64>() {
            return Obj::Float(val)
        } else {
            Obj::Str(c.to_string())
        }
    }

}

impl PyObj for Obj {

    fn __default__() -> Self {
        Obj::None
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
            Obj::List(vec) | 
            Obj::Tuple(vec) | 
            Obj::Set(vec) => vec.len() != 0usize,
            _ => panic!("TypeError: __bool__() not implemented for: {:?}", self),
        };
        return ret;
    }

    fn __str__(&self) -> String {
        match self {
            Obj::None => format!("None"),
            Obj::Bool(val) => format!("{}", val),
            Obj::Float(val) => format!("{}", val),
            Obj::Str(s) => format!("{}", s),
            Obj::Int(val) => format!("{}", val),
            Obj::Function(ptr) => format!("{}", ptr),
            Obj::Except(e) => format!("{}", e),
            Obj::List(objs) => {
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
            Obj::List(list) => list.len(),
            _ => panic!("TypeError: __len__() not implemented for: {:?}", self), 
        }
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

    fn __eq__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> bool {
        match (lhs.as_ref(), rhs.as_ref()) {
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
            (_, _) => false,
        }
    }

    fn __ne__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> bool {
        !Obj::__eq__(lhs, rhs)
    }

    fn __add__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> Result<Arc<Obj>, PyException> {
        let err = Err(PyException{
            error: PyError::TypeError,
            msg: format!("No valid way to add: {} and {}", lhs, rhs.clone(),),
        });

        let obj = match (lhs.as_ref(), rhs.as_ref()) {
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
                    let mut new_list = Vec::with_capacity(l1.len() + l2.len());
                    new_list.extend(l1.iter().cloned());
                    new_list.extend(l2.iter().cloned());    
                    Obj::List(new_list)
                }
                _ => {
                    let mut list_err = err.unwrap_err();
                    list_err.msg = format!("TypeError: can only concatenate list (not \"{:?}\") to list", other);
                    return Err(list_err);
                }
            }
            _ => return err,
        };
        Ok(obj.into())
    }

    fn __sub__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> Result<Arc<Obj>, PyException> {
        let err = Err(PyException{
            error: PyError::TypeError,
            msg: format!("No valid way to subtract: {} and {}", lhs, rhs.clone(),),
        });

        let obj = match (lhs.as_ref(), rhs.as_ref()) {
            (Obj::Float(dbl), other) => {
                let val = match other {
                    Obj::Float(v) => *v,
                    Obj::Int(v) => v.to_f64() ,
                    _ => return err,
                };
                Obj::Float(dbl - val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => Obj::Int(int.clone().add(v)),
                Obj::Float(v) => Obj::Float(int.to_f64() - v),
                _ => return err,
            },
            _ => return err,
        };
        Ok(obj.into())
    }

    fn __mul__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> Result<Arc<Obj>, PyException> {
        let err = Err(PyException{
            error: PyError::TypeError,
            msg: format!("No valid way to subtract: {} and {}", lhs, rhs.clone(),),
        });

        let obj = match (lhs.as_ref(), rhs.as_ref()) {
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
                        return Err(PyException{
                            error: PyError::TypeError,
                            msg: format!(" can't multiply sequence by non-int of type {}", lhs),
                        });
                    }
                }
                _ => return err,
            },
            _ => return err,
        };
        Ok(obj.into())
    }

    fn __div__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> Result<Arc<Obj>, PyException> {
        let type_err = Err(PyException{
            error: PyError::TypeError,
            msg: format!("No valid way to divide: {} and {}", lhs, rhs.clone(),),
        });
        let zero_div_err = Err(PyException{
            error: PyError::ZeroDivisionError,
            msg: format!(" tried to divide {lhs} by {rhs}"),
        });

        let obj = match (lhs.as_ref(), rhs.as_ref()) {
            (Obj::Float(dbl), other) => {
                let val = match other {
                    Obj::Float(v) => *v,
                    Obj::Int(v) => v.to_f64(),
                    _ => return type_err,
                };
                if val == 0f64 {
                    return zero_div_err
                }
                Obj::Float(dbl / val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => {
                    if *v == Integer::ZERO {
                        return zero_div_err
                    }
                    Obj::Int(int.clone().div_exact(v))
                }
                Obj::Float(v) => {
                    if *v == 0f64 {
                        return zero_div_err
                    }
                    Obj::Float(int.to_f64() / v)
                }
                _ => return type_err,
            },
            _ => return type_err,
        };
        Ok(obj.into())
    }

    fn __pos__(obj: &Arc<Obj>) -> Result<Arc<Obj>, PyException> {
        Ok(Arc::from(obj.clone()))
    }

    fn __neg__(obj: &Arc<Obj>) -> Result<Arc<Obj>, PyException> {
        let ret= match obj.as_ref() {
            Obj::None => Obj::None,
            Obj::Bool(b) => Obj::Bool(!b),
            Obj::Float(f) => Obj::Float(-f),
            Obj::Int(i) => Obj::Int(i.clone().neg()),
            _ => return Err(PyException{
                error: PyError::NotImplementedError, 
                msg: format!("Negation not implemented for {}", obj), 
            }),
        };
        Ok(ret.into())
    }

    fn __call__(&self, objs: &Vec<Arc<Obj>>) -> Result<Arc<Obj>, PyException> {
        match self {
            Obj::Function(fn_ptr) => {
                Ok((fn_ptr.ptr)(objs))
            }
            _ => Err( PyException { error: PyError::TypeError, msg: format!("Type is not a function") }),
        }
    }

    fn to_arc(self) -> Arc<Self> {
        Arc::from(self)
    }
}

impl PartialEq for Obj
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
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
            (Obj::Dict(_), _) |
            (_, Obj::Dict(_)) => {
                false
            }
            (_, _) => false,
        }
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}


impl PartialOrd for Obj
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.lt(other) { return Some(std::cmp::Ordering::Less); }
        if self.gt(other) { return Some(std::cmp::Ordering::Greater); }
        if self.eq(other) { return Some(std::cmp::Ordering::Equal); }
        return None
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

impl Termination for Obj {
    fn report(self) -> std::process::ExitCode {
        ExitCode::SUCCESS
    }
}

pub trait ToObj : Sized {
    fn to_obj(self) -> Obj {
        PyObj::__default__()
    }
    fn to_arc(self) -> Arc<Obj> {
        self.to_obj().into()
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

impl ToObj for bool {
    fn to_obj(self) -> Obj {
        Obj::Bool(self)
    }
}

impl ToObj for f64 {
    fn to_obj(self) -> Obj {
        Obj::Float(self)
    }
}
impl ToObj for f32 {
    fn to_obj(self) -> Obj {
        Obj::Float(self as f64)
    }
}
impl ToObj for i64 {
    fn to_obj(self) -> Obj {
        Obj::Int(Integer::from(self))
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

impl ToObj for usize {
    fn to_obj(self) -> Obj {
        Obj::Int(self.into())
    }
}
impl ToObj for i32 {
    fn to_obj(self) -> Obj {
        Obj::Int(self.into())
    }
}
impl ToObj for i16 {
    fn to_obj(self) -> Obj {
        Obj::Int(self.into())
    }
}
impl ToObj for i8 {
    fn to_obj(self) -> Obj {
        Obj::Int(self.into())
    }
}
impl ToObj for u32 {
    fn to_obj(self) -> Obj {
        Obj::Int(self.into())
    }
}
impl ToObj for u16 {
    fn to_obj(self) -> Obj {
        Obj::Int(self.into())
    }
}
 
impl ToObj for u8 {
    fn to_obj(self) -> Obj {
        Obj::Int(self.into())
    }
}


