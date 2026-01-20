use crate::pyrs_obj::Obj;
use std::collections::HashMap;

pub trait Import {
    fn get_name() -> &'static str;
    fn try_get(name: &str) -> Option<FnPtr>;
}

#[derive(Debug, Clone)]
pub struct FnPtr {
    pub ptr: fn(&Vec<Obj>) -> Obj,
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

    pub fn print(args: &Vec<Obj>) -> Obj {
        let mut msg = String::new();
        for arg in args {
            msg += &(format!("{} ", arg).as_str());
        }
        println!("{}", msg);
        Obj::None
    }

    pub fn print_ret(args: &Vec<Obj>) -> Obj {
        let mut msg = String::new();
        for arg in args {
            msg += &(format!("{} ", arg).as_str());
        }
        println!("{}", msg);
        Obj::Str(msg)
    }

    pub fn bin(obj: &Obj) -> Obj {
        // num.index_
        let s = match obj {
            Obj::Int(i) => format!("{:b}", i),
            _ => unimplemented!(),
        };
        Obj::Str(s)
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

impl Import for Funcs {
    fn get_name() -> &'static str {
        return "std";
    }

    fn try_get<'a>(word: &'a str) -> Option<FnPtr> {
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

// implement 'if' statements

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
    pub fn sin(args: &Vec<Obj>) -> Obj {
        if args.len() != 1 {
            panic!("[Type Error] Func{{sin}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg {
            Obj::Float(d) => *d,
            Obj::Int(i) => *i as f64,
            _ => panic!(
                "[Type Error] Func{{sin}} only takes a number types: {:?}",
                arg
            ),
        };
        Obj::Float(val.sin())
    }

    pub fn cos(args: &Vec<Obj>) -> Obj {
        if args.len() != 1 {
            panic!("[Type Error] Func{{cos}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg {
            Obj::Float(d) => *d,
            Obj::Int(i) => *i as f64,
            _ => panic!(
                "[Type Error] Func{{cos}} only takes a number types: {:?}",
                arg
            ),
        };
        Obj::Float(val.cos())
    }

    pub fn tan(args: &Vec<Obj>) -> Obj {
        if args.len() != 1 {
            panic!("[Type Error] Func{{tan}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg {
            Obj::Float(d) => *d,
            Obj::Int(i) => *i as f64,
            _ => panic!(
                "[Type Error] Func{{tan}} only takes a number types: {:?}",
                arg
            ),
        };
        Obj::Float(val.tan())
    }

    pub fn sqrt(args: &Vec<Obj>) -> Obj {
        if args.len() != 1 {
            panic!("[Type Error] Func{{sqrt}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg {
            Obj::Float(d) => *d,
            Obj::Int(i) => *i as f64,
            _ => panic!(
                "[Type Error] Func{{sqrt}} only takes a number types: {:?}",
                arg
            ),
        };
        Obj::Float(val.sqrt())
    }

    pub fn abs(args: &Vec<Obj>) -> Obj {
        if args.len() != 1 {
            panic!("[Type Error] Func{{abs}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg {
            Obj::Float(d) => *d,
            Obj::Int(i) => *i as f64,
            _ => panic!(
                "[Type Error] Func{{abs}} only takes a number types: {:?}",
                arg
            ),
        };
        Obj::Float(val.abs())
    }

    pub fn ln(args: &Vec<Obj>) -> Obj {
        if args.len() != 1 {
            panic!("[Type Error] Func{{ln}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg {
            Obj::Float(d) => *d,
            Obj::Int(i) => *i as f64,
            _ => panic!(
                "[Type Error] Func{{ln}} only takes a number types: {:?}",
                arg
            ),
        };
        Obj::Float(val.ln())
    }

    pub fn log10(args: &Vec<Obj>) -> Obj {
        if args.len() != 1 {
            panic!("[Type Error] Func{{log10}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg {
            Obj::Float(d) => *d,
            Obj::Int(i) => *i as f64,
            _ => panic!(
                "[Type Error] Func{{log10}} only takes a number types: {:?}",
                arg
            ),
        };
        Obj::Float(val.log10())
    }

    pub fn exp(args: &Vec<Obj>) -> Obj {
        if args.len() != 1 {
            panic!("[Type Error] Func{{exp}} only takes 1 argument");
        }
        let arg = args.first().unwrap();

        let val = match arg {
            Obj::Float(d) => *d,
            Obj::Int(i) => *i as f64,
            _ => panic!(
                "[Type Error] Func{{exp}} only takes a number types: {:?}",
                arg
            ),
        };
        Obj::Float(val.exp())
    }
}
