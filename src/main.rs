
pub mod pyrs_obj;
pub mod pyrs_parsing;
pub mod pyrs_std;
pub mod pyrs_error;
pub mod pyrs_userclass;
pub mod pyrs_utils;
pub mod pyrs_interpreter;

#[allow(unused_imports)]
use crate::{
    pyrs_interpreter::{Interpreter, InterpreterCommand},
    pyrs_obj::{Obj},
    pyrs_error::{PyException}, 
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
    use std::{
        ops::Index,
        collections::HashMap,
    };
    use pretty_assertions::{
        assert_eq
    };

    use super::*;

    struct EqTester
    {
        vars: HashMap<String, Obj>,
        funcs: HashMap<String, FnPtr>,
    }

    impl EqTester 
    {
        fn new() -> Self {
            EqTester { 
                vars: Obj::new_map(), 
                funcs: Funcs::get_std_map() 
            }
        }

        fn eval_eq(&mut self, expr: &Expression, result: &str)
        {
            let res = match expr.eval(&mut self.vars, &mut self.funcs) {
                Ok(val) => val,
                Err(e) => panic!("{e}"),
            };
            assert_eq!(res.to_string(), result);
        }
    }

    fn join_expr_strings(exprs: Vec<&Expression>) -> String
    {
        let mut res = String::new();
        for e in exprs {
            res.push_str(&e.to_string().as_str());
            res.push_str(" | ");
        }
        res.pop();
        res.pop();
        res.pop();
        res
    }

    #[test]
    fn parse() 
    {
        let s1 = Expression::from_line("1");
        let s2 = Expression::from_line("1 + 2 * 3");
        let s3 = Expression::from_line("(1 + 2) * 3");
        let s4 = Expression::from_line("print(100)");
        
        let final_str = join_expr_strings(vec![&s1, &s2, &s3, &s4]);
        let res_str = "Atom(1) | Op[+ Atom(1) Op[* Atom(2) Atom(3)]] | Op[* Op[+ Atom(1) Atom(2)] Atom(3)] | Func[print args[ Atom(100)]]";
        assert_eq!(final_str, res_str);
    }

    #[test]
    fn strlit_parse_eval() 
    {
        let s1 = Expression::from_line("\"smelly\"");
        assert_eq!(s1.to_string(), "Atom(smelly)");
        let s2 = Expression::from_line("\"smelly\" + \"poop\"");
        assert_eq!(s2.to_string(), "Op[+ Atom(smelly) Atom(poop)]");

        let mut eq = EqTester::new();
        eq.eval_eq(&s1, "smelly");
        eq.eval_eq(&s2, "smellypoop");
    }

    #[test]
    fn test_7() {
        let s = Expression::from_line(" print(\" y = \", 5) ");
        assert_eq!(s.to_string(), "Func[print args[ Atom( y = ) Atom(5)]]");
    }

    #[test]
    fn test_8() {
        let s = Expression::from_line("y = 5");
        assert_eq!(s.to_string(), "Op[= Ident(y) Atom(5)]");
    }

    #[test]
    fn test_9() {
        let s = Expression::from_line("print_ret(10, 100)");
        assert_eq!(
            s.to_string(),
            "Func[print_ret args[ Atom(10) Atom(100)]]"
        );

        let mut eq = EqTester::new();
        eq.eval_eq(&s, "10 100 ");
    }

    #[test]
    fn test_10() {
        let s = Expression::from_line(" \"la\" * 3");
        assert_eq!(s.to_string(), "Op[* Atom(la) Atom(3)]");

        let mut eq = EqTester::new();
        eq.eval_eq(&s, "lalala");
    }

    #[test]
    fn test_11() {
        let exprs = Expression::from_multiline("if 1:\n\t print(1) ");
        assert_eq!(exprs.len(), 2);
        let expr_results = vec!["Keyword[if conds[ Atom(1)] args[]]", "Func[print args[ Atom(1)]]"];
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
        let expr_results = vec!["Op[= Ident(x) Atom(2)]","Keyword[if conds[ Ident(x)] args[]]", "Func[print_ret args[ Ident(x)]]"];
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
        let s2 = Expression::from_line("1 > 0");
        let s3 = Expression::from_line("\"poop\" != 0");
        let s4 = Expression::from_line("1 == 0");
        let s5 = Expression::from_line("1.0 <= 0");
        let s6 = Expression::from_line("1 >= 0.0");

        let expr_str = join_expr_strings(vec![&s1, &s2, &s3, &s4, &s5, &s6]);
        let res_str = "Op[< Atom(1) Atom(0)] | Op[> Atom(1) Atom(0)] | Op[!= Atom(poop) Atom(0)] | Op[== Atom(1) Atom(0)] | Op[<= Atom(1.0) Atom(0)] | Op[>= Atom(1) Atom(0.0)]";

        assert_eq!(expr_str, res_str);

        let mut eq = EqTester::new();
        eq.eval_eq(&s1, "false");
        eq.eval_eq(&s2, "true");
        eq.eval_eq(&s3, "true");
        eq.eval_eq(&s4, "false");
        eq.eval_eq(&s5,"false");
        eq.eval_eq(&s6, "true");
        Ok(Obj::None)
    }

    #[test]
    fn assign() -> Result<Obj, PyException> 
    {
        let s1 = Expression::from_line("x = 2");
        let s2 = Expression::from_line("six = 6");
        let s3 = Expression::from_line("y = x");
        let s4 = Expression::from_line("z = 20 * 4");

        let expr_strs = join_expr_strings(vec![&s1, &s2, &s3, &s4]);
        let res_strs = "Op[= Ident(x) Atom(2)] | Op[= Ident(six) Atom(6)] | Op[= Ident(y) Ident(x)] | Op[= Ident(z) Op[* Atom(20) Atom(4)]]";
        assert_eq!(expr_strs, res_strs);

        let mut eq = EqTester::new();
        eq.eval_eq(&s1, "2");
        eq.eval_eq(&s2, "6");
        eq.eval_eq(&s3, "2");
        eq.eval_eq(&s4, "80");

        Ok(Obj::None)
    }

    #[test]
    fn while_test() -> Result<Obj, PyException> 
    {   
        let expr = Expression::from_multiline
        (r#"
        i = 0
        n1 = 0
        n2 = 1
        n3 = 0
        print("Fibbonacci: ")
        while i < 20:
            n3 = n1 + n2
            print("(", i, ") ", n3)
            n1 = n2
            n2 = n3
            i = i + 1
        "#);

        let ret_strs = vec![
            "None", 
            "Op[= Ident(i) Atom(0)]",
            "Op[= Ident(n1) Atom(0)]",
            "Op[= Ident(n2) Atom(1)]",
            "Op[= Ident(n3) Atom(0)]",
            "Func[print args[ Atom(Fibbonacci: )]]",
            "Keyword[while conds[ Op[< Ident(i) Atom(20)]] args[]]",
            "Op[= Ident(n3) Op[+ Ident(n1) Ident(n2)]]",
            "Func[print args[ Atom(() Ident(i) Atom() ) Ident(n3)]]",
            "Op[= Ident(n1) Ident(n2)]",
            "Op[= Ident(n2) Ident(n3)]",
            "Op[= Ident(i) Op[+ Ident(i) Atom(1)]]",
            "None"
        ];
        
        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();

        let idx_err= "[Bad Index]";

        let mut ret_objs: Vec<Obj> = vec![];
        let mut idx = 0;
        for e in expr {
            let obj = e.eval(&mut vars, &mut funcs)?;
            assert_eq!(e.to_string(), ret_strs.get(idx).unwrap_or(&idx_err).to_string());
            ret_objs.push(obj);
            idx += 1;
        }
        Ok(Obj::None)

    }

    #[test]
    fn nested_ifs() -> Result<Obj, PyException> 
    {
        assert!(false, "Test broken: looping??");

        let expr = Expression::from_multiline
        (r#"
        if True:
            print_ret("a: good")
            if False:
                print_ret("b: bad")
            if True:
                print_ret("c: good)
        print("d: good)
        "#);

        let ret_strs = vec![
            "None",
            "Keyword[while conds[ Atom(True)] args[]]",
            "Func[print_ret args[ Atom(a: good)]]",
            "Keyword[while conds[ Atom(False)] args[]]",
            "Func[print_ret args[ Atom(b: bad)]]",
            "Keyword[while conds[ Atom(True)] args[]]",
            "Func[print_ret args[ Atom(c: bad)]]",
            "Func[print_ret args[ Atom(d: good)]]",
            "None"
        ];

        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();

        let idx_err= "[Bad Index]";

        let mut ret_objs: Vec<Obj> = vec![];
        let mut idx = 0;
        for e in expr {
            let obj = e.eval(&mut vars, &mut funcs)?;
            assert_eq!(e.to_string(), ret_strs.get(idx).unwrap_or(&idx_err).to_string());
            ret_objs.push(obj);
            idx += 1;
        }
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
