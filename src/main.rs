
pub mod pyrs_obj;
pub mod pyrs_parsing;
pub mod pyrs_std;
pub mod pyrs_userclass;
pub mod pyrs_utils;
pub mod pyrs_interpreter;

#[allow(unused_imports)]
use crate::{
    pyrs_interpreter::{Interpreter, InterpreterCommand},
    pyrs_obj::{Obj, PyException}, 
    pyrs_parsing::{Expression},
    pyrs_std::{FnPtr, Funcs}
};

fn main() {

    let args = std::env::args();
    let mut argv: Vec<String> = vec![];
    for a in args {
        argv.push(a);
    }

    let mut interp = Interpreter::new();
    let cmd = Interpreter::parse_args(&argv);
    match cmd {
        InterpreterCommand::Live => interp.live_interpret(),
        InterpreterCommand::AnyFile(file) => interp.interpret_file(file),
        InterpreterCommand::PyFile(py) => interp.interpret_file(py),
        InterpreterCommand::FromString(words) => interp.interpret_line(words),
        InterpreterCommand::Error(msg) => println!("{}", msg),
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Index;

    use super::*;

    #[test]
    fn test_1() {
        let s = Expression::from_line("1");
        assert_eq!(s.to_string(), "Atom(1)");
    }

    #[test]
    fn test_2() {
        let s = Expression::from_line("1 + 2 * 3");
        assert_eq!(s.to_string(), "Op[+ Atom(1) Op[* Atom(2) Atom(3)]]");
    }

    #[test]
    fn test_3() {
        let s = Expression::from_line("(1 + 2) * 3");
        assert_eq!(s.to_string(), "Op[* Op[+ Atom(1) Atom(2)] Atom(3)]");
    }

    #[test]
    fn test_4() {
        let s = Expression::from_line("print(100)");
        assert_eq!(s.to_string(), "Func[print, args[ Atom(100)]]");
    }

    #[test]
    fn test_5() -> Result<Obj, PyException> {
        let s = Expression::from_line("\"smelly\"");
        assert_eq!(s.to_string(), "Atom(smelly)");
        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();
        let res = s.eval(&mut vars, &mut funcs)?;
        assert_eq!(res.to_string(), "smelly");
        Ok(res)
    }

    #[test]
    fn test_6() -> Result<Obj, PyException> {
        let s = Expression::from_line("\"smelly\" + \"poop\"");
        assert_eq!(s.to_string(), "Op[+ Atom(smelly) Atom(poop)]");
        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();
        let res = s.eval(&mut vars, &mut funcs)?;
        assert_eq!(res.to_string(), "smellypoop");
        Ok(res)
    }

    #[test]
    fn test_7() {
        let s = Expression::from_line(" print(\" y = \", 5) ");
        assert_eq!(s.to_string(), "Func[print, args[ Atom( y = ) Atom(5)]]");
    }

    #[test]
    fn test_8() {
        let s = Expression::from_line("y = 5");
        assert_eq!(s.to_string(), "Op[= Ident(y) Atom(5)]");
    }

    #[test]
    fn test_9() -> Result<Obj, PyException> {
        let s = Expression::from_line("print_ret(10, 100)");
        assert_eq!(
            s.to_string(),
            "Func[print_ret, args[ Atom(10) Atom(100)]]"
        );
        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();
        let res = s.eval(&mut vars, &mut funcs)?;
        assert_eq!(res.to_string(), "10 100 ");
        Ok(res)
    }

    #[test]
    fn test_10() -> Result<Obj, PyException> {
        let s = Expression::from_line(" \"la\" * 3");
        assert_eq!(s.to_string(), "Op[* Atom(la) Atom(3)]");
        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();
        let res = s.eval(&mut vars, &mut funcs)?;
        assert_eq!(res.to_string(), "lalala");
        Ok(res)
    }

    #[test]
    fn test_11() {
        let exprs = Expression::from_multiline("if 1:\n\t print(1) ");
        assert_eq!(exprs.len(), 2);
        let expr_results = vec!["Keyword[if conds[ Atom(1)] args[]]", "Func[print, args[ Atom(1)]]"];
        for (idx, expr) in exprs.iter().enumerate() {
            assert_eq!(expr.to_string(), expr_results.index(idx).to_string());
        }
    }

    #[test]
    fn test_12() -> Result<Obj, PyException> {
        let exprs = Expression::from_multiline("x = 2\n if x:\n\t print_ret(x) ");
        assert_eq!(exprs.len(), 3);
        println!("Exprs: {:?}", exprs);

        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();
        let expr_results = vec!["Op[= Ident(x) Atom(2)]","Keyword[if conds[ Ident(x)] args[]]", "Func[print_ret, args[ Ident(x)]]"];
        let obj_results: Vec<Obj> = vec![Obj::Int(2), Obj::Bool(true), Obj::from_str("2 ")];
        
        for (idx, expr) in exprs.iter().enumerate() {
            println!("Evaluating: {expr}");
            assert_eq!(expr.to_string(), expr_results.index(idx).to_string());
            let obj = expr.eval(&mut vars, &mut funcs)?;
            println!("Obj: {}", obj.to_string());
            println!("vars: {:?}", vars);
            assert_eq!(obj, obj_results.index(idx).clone());
        }
        Ok(Obj::None)
    }

    #[test]
    fn equality() -> Result<Obj, PyException> 
    {
        let s1 = Expression::from_line("1 < 0");
        assert_eq!(s1.to_string(), "Op[< Atom(1) Atom(0)]");
        let s2 = Expression::from_line("1 > 0");
        assert_eq!(s2.to_string(), "Op[> Atom(1) Atom(0)]");
        let s3 = Expression::from_line("\"poop\" != 0");
        assert_eq!(s3.to_string(), "Op[!= Atom(poop) Atom(0)]");
        let s4 = Expression::from_line("1 == 0");
        assert_eq!(s4.to_string(), "Op[== Atom(1) Atom(0)]");
        let s5 = Expression::from_line("1.0 <= 0");
        assert_eq!(s5.to_string(), "Op[<= Atom(1.0) Atom(0)]");
        let s6 = Expression::from_line("1 >= 0.0");
        assert_eq!(s6.to_string(), "Op[>= Atom(1) Atom(0.0)]");

        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();

        let res1 = s1.eval(&mut vars, &mut funcs)?;
        assert_eq!(res1.to_string(), "false");
        let res2 = s2.eval(&mut vars, &mut funcs)?;
        assert_eq!(res2.to_string(), "true");
        let res3 = s3.eval(&mut vars, &mut funcs)?;
        assert_eq!(res3.to_string(), "true");
        let res4 = s4.eval(&mut vars, &mut funcs)?;
        assert_eq!(res4.to_string(), "false");
        let res5 = s5.eval(&mut vars, &mut funcs)?;
        assert_eq!(res5.to_string(), "false");
        let res6 = s6.eval(&mut vars, &mut funcs)?;
        assert_eq!(res6.to_string(), "true");
        Ok(Obj::None)
    }

    #[test]
    fn assign() -> Result<Obj, PyException> 
    {
        let s1 = Expression::from_line("x = 2");
        assert_eq!(s1.to_string(), "Op[= Ident(x) Atom(2)]");
        let s2 = Expression::from_line("six = 6");
        assert_eq!(s2.to_string(), "Op[= Ident(six) Atom(6)]");
        let s3 = Expression::from_line("x = y");
        assert_eq!(s3.to_string(), "Op[= Ident(x) Ident(y)]");

        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();



        Ok(Obj::None)
    }
    // TODO: 
    // - Nested if statements
    // - while loops

    /*
    Usage: cargo.exe test [OPTIONS] [TESTNAME] [-- [ARGS]...]

    Arguments:
    [TESTNAME]  If specified, only run tests containing   
                this string in their names
    [ARGS]...   Arguments for the test binary

    Options:
        --no-run
            Compile, but don't run tests
        --no-fail-fast
            Run all tests regardless of failure
        --future-incompat-report
            Outputs a future incompatibility report at the          end of the build
        --message-format <FMT>
            Error format [possible values: human, short,  
            json, json-diagnostic-short,
            json-diagnostic-rendered-ansi,
            json-render-diagnostics]
    -q, --quiet
            Display one character per test instead of one 
            line
    -v, --verbose...
            Use verbose output (-vv very verbose/build.rs 
            output)
        --color <WHEN>
            Coloring [possible values: auto, always,      
            never]
        --config <KEY=VALUE|PATH>
            Override a configuration value
    -Z <FLAG>
            Unstable (nightly-only) flags to Cargo, see   
            'cargo -Z help' for details
    -h, --help
            Print help

    Package Selection:
    -p, --package [<SPEC>]
            Package to run tests for
        --workspace
            Test all packages in the workspace
        --exclude <SPEC>
            Exclude packages from the test
        --all
            Alias for --workspace (deprecated)

    Target Selection:
        --lib
            Test only this package's library
        --bins
            Test all binaries
        --bin [<NAME>]
            Test only the specified binary
        --examples
            Test all examples
        --example [<NAME>]
            Test only the specified example
        --tests
            Test all targets that have `test = true` set  
        --test [<NAME>]
            Test only the specified test target
        --benches
            Test all targets that have `bench = true` set 
        --bench [<NAME>]
            Test only the specified bench target
        --all-targets
            Test all targets (does not include doctests)  
        --doc
            Test only this library's documentation        

    Feature Selection:
    -F, --features <FEATURES>
            Space or comma separated list of features to  
            activate
        --all-features
            Activate all available features
        --no-default-features
            Do not activate the `default` feature

    Compilation Options:
    -j, --jobs <N>
            Number of parallel jobs, defaults to # of     
            CPUs.
    -r, --release
            Build artifacts in release mode, with
            optimizations
        --profile <PROFILE-NAME>
            Build artifacts with the specified profile    
        --target [<TRIPLE>]
            Build for the target triple
        --target-dir <DIRECTORY>
            Directory for all generated artifacts
        --unit-graph
            Output build graph in JSON (unstable)
        --timings[=<FMTS>]
            Timing output formats (unstable) (comma       
            separated): html, json

    Manifest Options:
        --manifest-path <PATH>
            Path to Cargo.toml
        --lockfile-path <PATH>
            Path to Cargo.lock (unstable)
        --ignore-rust-version
            Ignore `rust-version` specification in        
            packages
        --locked
            Assert that `Cargo.lock` will remain unchanged      --offline
            Run without accessing the network
        --frozen
            Equivalent to specifying both --locked and    
            --offline

     */
}
