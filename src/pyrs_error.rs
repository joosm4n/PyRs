
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyException
{
    pub error: PyError,
    pub msg: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PyError 
{
    ArithmeticError,
    IndexError,
    KeyError,
    IndentationError,
    TypeError,
    NotImplementedError,
    ZeroDivisionError,
    UndefinedVariableError,
    FloatParseError,
    StackError,
    SyntaxError,
    FileError,
}

impl PyException
{
    pub fn print(&self) {
        println!("{self}");   
    }
}

impl std::fmt::Display for PyException
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error, self.msg)
    }
}

impl std::process::Termination for PyException
{
    fn report(self) -> std::process::ExitCode {
        std::process::ExitCode::from(self.error as u8)
    }
}

pub trait PyPanicHandle<T> {
    fn handle(self) -> T;
}
impl<T> PyPanicHandle<T> for Result<T, PyException> {
    fn handle(self) -> T {
        match self {
            Ok(s) => s,
            Err(e) => panic!("{e}"),
        }
    }
}
