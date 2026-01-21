use std::{
    collections::HashMap, process::{ExitCode, Termination}, sync::Arc
};
use crate::{
    pyrs_error::{PyException, PyError},
    pyrs_std::{FnPtr},
    pyrs_parsing::{Op},
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum Obj {
    None,
    Bool(bool),
    Float(f64),
    Str(String),
    Int(i64),

    Function(FnPtr),

    Except(PyException)
    //User(UserClass),

    // Numeric
    // - Int (Unlimited precision)
    // - Float (f64)
    // - Complex (f64, f64)

    // Boolean
    // - bool

    // Iterator
    // - containters

    // Sequence
    // - list
    // - tuple
    // - range

    // Text
    // - str

    // Binary
    // - bytes
    // - bytearray,
    // - memoryview,

    // Set
    // - set
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
}

impl Obj {

    pub fn from<T: ToObj>(arg: T) -> Arc<Obj> {
        arg.to_arc()
    }

    pub fn new_vec() -> Vec<Obj>
    {
        return vec![];
    }
    
    pub fn new_arc_vec() -> Vec<Arc<Obj>>
    {
        return vec![];
    }

    pub fn new_map() -> HashMap<String, Arc<Obj>> {
        return HashMap::new();
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
        if let Ok(val) = c.parse::<i64>() {
            Obj::Int(val)
        } else if let Ok(val) = c.parse::<f64>() {
            Obj::Float(val)
        } else {
            Obj::Str(c.to_string())
        }
    }

    

}

impl PyObj for Obj {

    fn __default__() -> Self {
        Obj::None
    }

    fn __bool__(&self) -> bool {
        let ret = match self {
            Obj::Bool(v) => *v,
            Obj::Float(v) => *v != 0f64,
            Obj::Int(v) => *v != 0i64,
            Obj::Str(v) => *v != "",
            _ => panic!(".__bool__() not implemented for: {:?}", self),
        };
        return ret;
    }

    fn __len__(&self) -> usize {
        unimplemented!();
    }

    fn __lt__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> bool {
        let ret = match (lhs.as_ref(), rhs.as_ref()) {
            (Obj::Float(flt), other) => match other {
                Obj::Float(same) => *flt < *same,
                Obj::Int(i) => *flt < (*i as f64),
                Obj::Bool(b) => *flt < f64::from(*b),
                _ => false,
            },
            (Obj::Int(i), other) => match other {
                Obj::Float(flt) => (*i as f64) < *flt,
                Obj::Int(same) => *i < *same,
                Obj::Bool(b) => *i < i64::from(*b),
                _ => false,
            },
            (Obj::Bool(b), other) => match other {
                Obj::Float(f) => f64::from(*b) < *f,
                Obj::Int(i) => i64::from(*i) < *i,
                Obj::Bool(same) => *b < *same,
                _ => false,
            },
            _ => false,
        };
        ret
    }

    fn __gt__(lhs: &Arc<Obj>, rhs: &Arc<Obj>) -> bool {
        let ret = match (lhs.as_ref(), rhs.as_ref()) {
            (Obj::Float(flt), other) => match other {
                Obj::Float(same) => *flt > *same,
                Obj::Int(i) => *flt > (*i as f64),
                Obj::Bool(b) => *flt > f64::from(*b),
                _ => false,
            },
            (Obj::Int(i), other) => match other {
                Obj::Float(flt) => (*i as f64) > *flt,
                Obj::Int(same) => *i > *same,
                Obj::Bool(b) => *i > i64::from(*b),
                _ => false,
            },
            (Obj::Bool(b), other) => match other {
                Obj::Float(f) => f64::from(*b) > *f,
                Obj::Int(i) => i64::from(*i) > *i,
                Obj::Bool(same) => *b > *same,
                _ => false,
            },
            _ => false,
        };
        ret
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
                Obj::Int(i) => *flt == *i as f64,
                Obj::Bool(b) => *flt == f64::from(*b),
                _ => false,
            },
            (Obj::Int(i), other) => match other {
                Obj::Float(f) => *i as f64 == *f,
                Obj::Int(same) => *i == *same,
                Obj::Bool(b) => *i == i64::from(*b),
                _ => false,
            },
            (Obj::Bool(b), other) => match other {
                Obj::Float(f) => f64::from(*b) == *f,
                Obj::Int(i) => i64::from(*b) == *i,
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
                    Obj::Int(v) => *v as f64,
                    _ => return err,
                };
                Obj::Float(dbl + val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => Obj::Int(int + v),
                Obj::Float(v) => Obj::Float(*int as f64 + v),
                _ => return err,
            },
            (Obj::Str(s), other) => match other {
                Obj::Str(v) => Obj::Str(format!("{s}{v}")),
                _ => return err,
            },
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
                    Obj::Int(v) => *v as f64,
                    _ => return err,
                };
                Obj::Float(dbl - val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => Obj::Int(int - v),
                Obj::Float(v) => Obj::Float(*int as f64 - v),
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
                    Obj::Int(v) => *v as f64,
                    _ => return err,
                };
                Obj::Float(dbl * val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => Obj::Int(int * v),
                Obj::Float(v) => Obj::Float(*int as f64 * v),
                _ => return err,
            },
            (Obj::Str(s), other) => match other {
                Obj::Int(v) => {
                    if *v >= 0 {
                        let mut mult = String::new();
                        for _i in 0..*v {
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
                    Obj::Int(v) => *v as f64,
                    _ => return type_err,
                };
                if val == 0f64 {
                    return zero_div_err
                }
                Obj::Float(dbl / val)
            }
            (Obj::Int(int), other) => match other {
                Obj::Int(v) => {
                    if *v == 0i64 {
                        return zero_div_err
                    }
                    Obj::Int(int / v)
                }
                Obj::Float(v) => {
                    if *v == 0f64 {
                        return zero_div_err
                    }
                    Obj::Float(*int as f64 / v)
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
            Obj::Int(i) => Obj::Int(-i),
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
}

impl std::fmt::Display for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Obj::None => write!(f, "None"),
            Obj::Bool(val) => write!(f, "{}", val),
            Obj::Float(val) => write!(f, "{}", val),
            Obj::Str(s) => write!(f, "{}", s),
            Obj::Int(val) => write!(f, "{}", val),
            Obj::Function(ptr) => write!(f, "{}", ptr),
            Obj::Except(e) => write!(f, "{}", e),
            //Obj::User(class) => write!(f, "{}", class),
            //t => write!(f, "{:?}", t),
        }
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
        Obj::Int(self)
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
        Obj::Int(self as i64)
    }
}
impl ToObj for i32 {
    fn to_obj(self) -> Obj {
        Obj::Int(self as i64)
    }
}
impl ToObj for i16 {
    fn to_obj(self) -> Obj {
        Obj::Int(self as i64)
    }
}
impl ToObj for i8 {
    fn to_obj(self) -> Obj {
        Obj::Int(self as i64)
    }
}
impl ToObj for u32 {
    fn to_obj(self) -> Obj {
        Obj::Int(self as i64)
    }
}
impl ToObj for u16 {
    fn to_obj(self) -> Obj {
        Obj::Int(self as i64)
    }
}
 
impl ToObj for u8 {
    fn to_obj(self) -> Obj {
        Obj::Int(self as i64)
    }
}
