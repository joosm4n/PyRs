use crate::{
    pyrs_error::{PyError, PyException},
    pyrs_obj::{Obj, ToObj},
};
use std::{
    collections::HashMap, f32::consts::E, sync::Arc
};

use rug::Integer;

pub trait Import {
    fn get_name() -> &'static str;
    fn try_get(name: &str) -> Option<FnPtr>;
}

#[derive(Debug, Clone)]
pub struct FnPtr {
    pub ptr: fn(&Vec<Arc<Obj>>) -> Arc<Obj>,
    pub name: String,
}

impl PartialEq for FnPtr {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
    fn ne(&self, other: &Self) -> bool {
        self.name != other.name
    }
}
impl PartialOrd for FnPtr {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl std::fmt::Display for FnPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct Funcs {}

impl Funcs {
    pub fn get_std_map() -> HashMap<String, FnPtr> {
        let mut func_map: HashMap<String, FnPtr> = HashMap::new();
        func_map.insert(
            "print".to_string(),
            FnPtr {
                ptr: Funcs::print,
                name: "print".to_string(),
            },
        );
        func_map.insert(
            "print_ret".to_string(),
            FnPtr {
                ptr: Funcs::print_ret,
                name: "print_ret".to_string(),
            },
        );
        return func_map;
    }

    pub fn print(args: &Vec<Arc<Obj>>) -> Arc<Obj> {
        let mut msg = String::new();
        for arg in args {
            msg += &(format!("{} ", arg).as_str());
        }
        println!("{}", msg);
        Arc::from(Obj::None)
    }

    pub fn print_ret(args: &Vec<Arc<Obj>>) -> Arc<Obj> {
        let mut msg = String::new();
        for arg in args {
            msg += &(format!("{} ", arg).as_str());
        }
        println!("{}", msg);
        Arc::from(Obj::Str(msg))
    }

    pub fn bin(obj: &Obj) -> Arc<Obj> {
        // num.index_
        let s = match obj {
            Obj::Int(i) => format!("{:b}", i),
            _ => unimplemented!(),
        };
        Arc::from(Obj::Str(s))
    }

    pub fn float(obj: &Obj) -> Result<Obj, PyException> {
        let ret = match obj {
            Obj::Float(_) => obj.clone(),
            Obj::Int(i) => Obj::Float(i.to_f64()),
            Obj::Str(s) => match s.parse::<f64>() {
                Ok(f) => Obj::Float(f),
                Err(e) => {
                    return Err(PyException {
                        error: PyError::FloatParseError,
                        msg: format!("Failed to parse \"{s}\" to float. {e}"),
                    });
                }
            },
            _ => {
                return Err(PyException {
                    error: PyError::FloatParseError,
                    msg: format!("Unable to convert {obj} to float"),
                });
            }
        };
        Ok(ret)
    }

    // TODO: Implement -
    // abs, aiter, all, anext, any, ascii,
    // bin, bool, breakpoint, bytearray, bytes,
    // callable, chr, classmethod, compile, complex,
    // delattr, dict, dir, divmod,
    // enumerate, eval, exec,
    // filter, float, format, frozenset,
    // getattr, globals,
    // hasattr, hash, help, hex,
    // id, input, int, isinstance, issubclass, iter,
    // len, list, locals
    // map, max, memoryview, min,
    // next,
    // object, oct, open, ord,
    // pow, print, property,
    // range, repr, reversed, round,
    // set, setattr, slice, sorted, staticmethod, str, sum, super,
    // tuple, type,
    // vars,
    // zip,
    // __import__
}

#[derive(Debug, Clone)]
pub struct RangeObj 
{
    pub start: Option<Integer>,
    pub end: Option<Integer>,
    pub inc: Option<Integer>,
    one_arg: bool,
}

impl RangeObj 
{
    pub fn from(start_val: Option<Integer>, end_val: Option<Integer>, increment: Option<Integer>) -> Self
    {
        let only_one_arg = end_val.is_none();
        RangeObj { start: start_val, end: end_val, inc: increment, one_arg: only_one_arg }
    }

    pub fn to_vec(self) -> Vec<Arc<Obj>>
    {
        let mut objs = vec![];

        let start: Integer; let end: Integer; let inc: Integer;
        if self.one_arg {
            start = Integer::ZERO;
            end = self.start.unwrap_or(Integer::ZERO);
            inc = Integer::from(1);
        }
        else {
            start = self.start.unwrap_or(Integer::ZERO);
            end = self.end.unwrap_or(Integer::ZERO);
            inc = self.inc.unwrap_or(Integer::from(1));
        }
        
        if start < end {
            let mut curr = start;
            while curr < end {
                objs.push(curr.clone().to_arc());
                curr += inc.clone();
            }
        }
        else {
            let mut curr = start;
            while curr > end {
                objs.push(curr.clone().to_arc());
                curr += inc.clone();
            }
        }

        objs
    }
}

impl Import for Funcs {
    fn get_name() -> &'static str {
        return "std";
    }

    #[allow(unreachable_code, unused_variables)]
    fn try_get<'a>(word: &'a str) -> Option<FnPtr> {
        return None;
        match word {
            "print" => Some(FnPtr {
                ptr: Funcs::print,
                name: "print".to_string(),
            }),
            "print_ret" => Some(FnPtr {
                ptr: Funcs::print_ret,
                name: "print_ret".to_string(),
            }),
            _ => None,
        }
    }
}



pub struct Maths {}

impl Import for Maths {
    fn get_name() -> &'static str {
        "maths"
    }
    fn try_get(name: &str) -> Option<FnPtr> {
        match name {
            "sin" => Some(FnPtr {
                ptr: Maths::sin,
                name: "sin".to_string(),
            }),
            "cos" => Some(FnPtr {
                ptr: Maths::cos,
                name: "cos".to_string(),
            }),
            "tan" => Some(FnPtr {
                ptr: Maths::tan,
                name: "tan".to_string(),
            }),
            "sqrt" => Some(FnPtr {
                ptr: Maths::sqrt,
                name: "sqrt".to_string(),
            }),
            "abs" => Some(FnPtr {
                ptr: Maths::abs,
                name: "abs".to_string(),
            }),
            "ln" => Some(FnPtr {
                ptr: Maths::ln,
                name: "ln".to_string(),
            }),
            "log10" => Some(FnPtr {
                ptr: Maths::log10,
                name: "log10".to_string(),
            }),
            "exp" => Some(FnPtr {
                ptr: Maths::exp,
                name: "exp".to_string(),
            }),
            _ => None,
        }
    }
}

#[allow(dead_code)]
impl Maths {
    pub fn sin(args: &Vec<Arc<Obj>>) -> Arc<Obj> {
        if args.len() != 1 {
            panic!("[Type Error] Func{{sin}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg.as_ref() {
            Obj::Float(d) => *d,
            Obj::Int(i) => i.to_f64(),
            _ => panic!(
                "[Type Error] Func{{sin}} only takes a number types: {:?}",
                arg
            ),
        };
        Arc::from(Obj::Float(val.sin()))
    }

    pub fn cos(args: &Vec<Arc<Obj>>) -> Arc<Obj> {
        if args.len() != 1 {
            panic!("[Type Error] Func{{cos}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg.as_ref() {
            Obj::Float(d) => *d,
            Obj::Int(i) => i.to_f64(),
            _ => panic!(
                "[Type Error] Func{{cos}} only takes a number types: {:?}",
                arg
            ),
        };
        Arc::from(Obj::Float(val.cos()))
    }

    pub fn tan(args: &Vec<Arc<Obj>>) -> Arc<Obj> {
        if args.len() != 1 {
            panic!("[Type Error] Func{{tan}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg.as_ref() {
            Obj::Float(d) => *d,
            Obj::Int(i) => i.to_f64(),
            _ => panic!(
                "[Type Error] Func{{tan}} only takes a number types: {:?}",
                arg
            ),
        };
        Arc::from(Obj::Float(val.tan()))
    }

    pub fn sqrt(args: &Vec<Arc<Obj>>) -> Arc<Obj> {
        if args.len() != 1 {
            panic!("[Type Error] Func{{sqrt}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg.as_ref() {
            Obj::Float(d) => *d,
            Obj::Int(i) => i.to_f64(),
            _ => panic!(
                "[Type Error] Func{{sqrt}} only takes a number types: {:?}",
                arg
            ),
        };
        Arc::from(Obj::Float(val.sqrt()))
    }

    pub fn abs(args: &Vec<Arc<Obj>>) -> Arc<Obj> {
        if args.len() != 1 {
            panic!("[Type Error] Func{{abs}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg.as_ref() {
            Obj::Float(d) => *d,
            Obj::Int(i) => i.to_f64(),
            _ => panic!(
                "[Type Error] Func{{abs}} only takes a number types: {:?}",
                arg
            ),
        };
        Arc::from(Obj::Float(val.abs()))
    }

    pub fn ln(args: &Vec<Arc<Obj>>) -> Arc<Obj> {
        if args.len() != 1 {
            panic!("[Type Error] Func{{ln}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg.as_ref() {
            Obj::Float(d) => *d,
            Obj::Int(i) => i.to_f64(),
            _ => panic!(
                "[Type Error] Func{{ln}} only takes a number types: {:?}",
                arg
            ),
        };
        Arc::from(Obj::Float(val.ln()))
    }

    pub fn log10(args: &Vec<Arc<Obj>>) -> Arc<Obj> {
        if args.len() != 1 {
            panic!("[Type Error] Func{{log10}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg.as_ref() {
            Obj::Float(d) => *d,
            Obj::Int(i) => i.to_f64(),
            _ => panic!(
                "[Type Error] Func{{log10}} only takes a number types: {:?}",
                arg
            ),
        };
        Arc::from(Obj::Float(val.log10()))
    }

    pub fn exp(args: &Vec<Arc<Obj>>) -> Arc<Obj> {
        if args.len() != 1 {
            panic!("[Type Error] Func{{exp}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg.as_ref() {
            Obj::Float(d) => *d,
            Obj::Int(i) => i.to_f64(),
            _ => panic!(
                "[Type Error] Func{{exp}} only takes a number types: {:?}",
                arg
            ),
        };
        Arc::from(Obj::Float(val.exp()))
    }
}
