
#[allow(unused_imports)]
use crate::{
    pyrs_interpreter::{Interpreter, InterpreterCommand},
    pyrs_obj::{Obj, PyObj, ToObj},
    pyrs_error::{PyException}, 
    pyrs_parsing::{Expression, Token, Op, Keyword, Lexer},
    pyrs_std::{FnPtr, Funcs},
    pyrs_bytecode::{PyBytecode},
    pyrs_vm::{PyVM, IntrinsicFunc},
    pyrs_utils::{split_to_words},
};

#[cfg(test)]
mod tests {

    use std::{
        ops::Index,
        collections::HashMap,
        mem::size_of,
        sync::Arc,
    };

    use pretty_assertions::{assert_eq};
    use super::*;

    struct EqTester
    {
        vars: HashMap<String, Arc<Obj>>,
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
    fn memory_size_types()
    {
        assert_eq!(96, size_of::<Obj>(), "Obj size not 56 bytes");
        assert_eq!(24, size_of::<Token>(), "Token size not 24 bytes");
        assert_eq!(56, size_of::<Expression>(), "Expression size not 56 bytes");
        assert_eq!(104, size_of::<PyBytecode>(), "Bytecode size not 64 bytes");
        assert_eq!(304, size_of::<PyVM>(), "VirtualMachine size changed");
    }

    #[test]
    fn parse() 
    {
        let s1 = Expression::from_line("1");
        let s2 = Expression::from_line("1 + 2 * 3");
        let s3 = Expression::from_line("(1 + 2) * 3");
        let s4 = Expression::from_line("print(100)");
        let s5 = Expression::from_line("print(1, 2, \"5\")");
        let s6 = Expression::from_line("x=2");
        let s7 = Expression::from_line("x+=2");
        
        let final_str = join_expr_strings(vec![&s1, &s2, &s3, &s4, &s5, &s6, &s7]);
        let res_str = 
        "Atom(1) | \
        Op[+ Atom(1) Op[* Atom(2) Atom(3)]] | \
        Op[* Op[+ Atom(1) Atom(2)] Atom(3)] | \
        Call[print args[ Atom(100)]] | \
        Call[print args[ Atom(1) Atom(2) Atom(5)]] | \
        Op[= Ident(x) Atom(2)] | \
        Op[+= Ident(x) Atom(2)]";
        assert_eq!(final_str, res_str);
    }

    #[test]
    fn parse_underscore() 
    {
        let s1 = split_to_words("x.__str__()");
        let res_str = vec!["x", ".", "__str__", "(", ")"];
        assert_eq!(s1, res_str);
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
        assert_eq!(s.to_string(), "Call[print args[ Atom( y = ) Atom(5)]]");
    }

    #[test]
    fn test_8() {
        let s = Expression::from_line("y = 5");
        assert_eq!(s.to_string(), "Op[= Ident(y) Atom(5)]");
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
        dbg!(&exprs);
        assert_eq!(exprs.len(), 1);
        let expr_results = vec!["Keyword[if conds[ Atom(1)] args[ Call[print args[ Atom(1)]]]]"];
        for (idx, expr) in exprs.iter().enumerate() {
            assert_eq!(expr.to_string(), expr_results.index(idx).to_string());
        }
    }

    #[test]
    fn test_12() -> Result<Obj, PyException> {
        let exprs = Expression::from_multiline("x = 2\n if x:\n\t print_ret(x) ");
        assert_eq!(exprs.len(), 2);
        println!("Exprs: {:?}", exprs);

        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();
        let expr_results = vec!["Op[= Ident(x) Atom(2)]","Keyword[if conds[ Ident(x)] args[ Call[print_ret args[ Ident(x)]]]]"];
        let obj_results: Vec<Arc<Obj>> = vec![Obj::from(2usize), Obj::from(true), Obj::from("2 ")];
        
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
        eq.eval_eq(&s1, "False");
        eq.eval_eq(&s3, "True");
        eq.eval_eq(&s4, "False");
        eq.eval_eq(&s5,"False");
        eq.eval_eq(&s6, "True");
        Ok(Obj::None)
    }

    #[test]
    fn obj_equality()
    {
        assert_eq!(Obj::None, Obj::None);
        assert_eq!(&Obj::None, &Obj::None);
        assert_eq!(Obj::None.to_arc(), Obj::None.to_arc());
        assert_eq!(Obj::None.to_arc().as_ref(), &Obj::None);

        assert_eq!(Obj::Bool(true).to_arc(), Obj::Float(1.0).to_arc());
        assert_ne!(Obj::new_dict(), Obj::new_dict());

        let null_obj = Arc::new(Obj::Null);
        let null_ref = null_obj.clone();
        assert_eq!(null_obj.as_ref(), null_ref.as_ref());
        assert_eq!(null_obj, null_ref);
    }

    #[test]
    fn parse_assign()
    {
        let s1 = Expression::from_line("x = 2");
        let s2 = Expression::from_line("six = 6");
        let s3 = Expression::from_line("y = x");
        let s4 = Expression::from_line("z = 20 * 4");
        let s5 = Expression::from_line("x += 2");
        let s6 = Expression::from_line("x /= 2");

        let expr_strs = join_expr_strings(vec![&s1, &s2, &s3, &s4, &s5, &s6]);
        let res_strs = "Op[= Ident(x) Atom(2)] | Op[= Ident(six) Atom(6)] | Op[= Ident(y) Ident(x)] | Op[= Ident(z) Op[* Atom(20) Atom(4)]] | Op[+= Ident(x) Atom(2)] | Op[/= Ident(x) Atom(2)]";
        assert_eq!(expr_strs, res_strs);
    }

    #[test]
    #[ignore]
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
            
            "Op[= Ident(i) Atom(0)]",
            "Op[= Ident(n1) Atom(0)]",
            "Op[= Ident(n2) Atom(1)]",
            "Op[= Ident(n3) Atom(0)]",
            "Call[print args[ Atom(Fibbonacci: )]]",
            "Keyword[while conds[ Op[< Ident(i) Atom(20)]] args[ \
            Op[= Ident(n3) Op[+ Ident(n1) Ident(n2)]] \
            Call[print args[ Atom(() Ident(i) Atom() ) Ident(n3)]] \
            Op[= Ident(n1) Ident(n2)] \
            Op[= Ident(n2) Ident(n3)] \
            Op[= Ident(i) Op[+ Ident(i) Atom(1)]]]]",
            "None"
        ];
        
        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();

        let idx_err= "[Bad Index]";

        let mut ret_objs: Vec<Arc<Obj>> = vec![];
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
        //panic!();
        let expr = Expression::from_multiline(
        "if True:\n\
         \tprint_ret(\"a: good\")\n\
         \tif False:\n\
         \t\tprint_ret(\"b: bad\")\n\
         \tif True:\n\
         \t\tprint_ret(\"c: good\")\n\
         \tprint(\"d: good\")"
        );

        let ret_strs = vec![
            r#"Keyword[if conds[ Keyword[True conds[] args[]]] args[ Call[print_ret args[ Atom(a: good)]] Keyword[if conds[ Keyword[False conds[] args[]]] args[ Call[print_ret args[ Atom(b: bad)]]]] Keyword[if conds[ Keyword[True conds[] args[]]] args[ Call[print_ret args[ Atom(c: good)]]]] Call[print args[ Atom(d: good)]]]]"#
        ];

        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();

        let idx_err= "[Bad Index]";

        let mut ret_objs: Vec<Arc<Obj>> = vec![];
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
    fn if_elif_else_expr() -> Result<Obj, PyException> 
    {
        //panic!();
        let expr = Expression::from_multiline(
        "if False:\n\
         \tprint_ret(\"a: bad\")\n\
         elif True:\n\
         \tprint_ret(\"b: good\")\n\
         if False:\n\
         \tprint_ret(\"c: good\")\n\
         else:\n\
         \tprint(\"d: good\")"
        );

        let ret_strs = vec![
            r#"Keyword[if conds[ Keyword[False conds[] args[]]] args[ Call[print_ret args[ Atom(a: bad)]] Keyword[elif conds[ Keyword[True conds[] args[]]] args[]] Call[print_ret args[ Atom(b: good)]]]]"#,
            r#"Keyword[if conds[ Keyword[False conds[] args[]]] args[ Call[print_ret args[ Atom(c: good)]] Keyword[else conds[] args[]] Call[print args[ Atom(d: good)]]]]"#,
        ];

        let mut vars = Obj::new_map();
        let mut funcs = Funcs::get_std_map();

        let idx_err= "[Bad Index]";

        let mut ret_objs: Vec<Arc<Obj>> = vec![];
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
    fn bytecode_manual() 
    {
        let code = vec![
            PyBytecode::LoadConst(Obj::Int(5.into())),
            PyBytecode::StoreName("x".to_string()),
            PyBytecode::LoadConst(Obj::Null.into()),
            PyBytecode::LoadName("x".to_string()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
        ];
        println!("Instruction Queue: ");
        println!("{}", PyBytecode::to_string(&code));
        let mut vm = PyVM::new();
        vm.execute(code);
    }

    #[test]
    fn bytecode_from_expr()
    {
        let expr = Expression::from_multiline("x = 2\n if x:\n\t print(x) ");
        let mut code = vec![];
        for e in expr {
            PyBytecode::from_expr(e, &mut code);
        }
        println!("Instructions:\n{}", PyBytecode::to_string(&code));
        assert_eq!(format!("{:?}", code), r#"[LoadConst(Int(2)), StoreName("x"), LoadName("x"), PopJumpIfFalse(3), PushNull, LoadName("x"), CallInstrinsic1(Print)]"#);
        
        let mut vm = PyVM::new();
        vm.execute(code);
    }

    #[test]
    fn bytecode_while_loop()
    {
        let code = PyBytecode::from_str
        (r#"x = 0
        while x < 3:
	        print(x)
	        x += 1
        "#);
        println!("Instructions:\n{}", PyBytecode::to_string(&code));
        assert_eq!(format!("{:?}", code), r#"[LoadConst(Int(0)), StoreName("x"), LoadName("x"), LoadConst(Int(3)), CompareOp(LessThan), PopJumpIfFalse(8), PushNull, LoadName("x"), CallInstrinsic1(Print), LoadName("x"), LoadConst(Int(1)), BinaryAdd, StoreName("x"), JumpBackward(12), LoadConst(None)]"#.to_string());
        
        let mut vm = PyVM::new();
        vm.execute(code);

    }

    #[test]
    fn handwritten_bytecode()
    {
        let code = vec![
            PyBytecode::LoadConst(Obj::Int(0.into())),
            PyBytecode::StoreName("x".to_string()),
            PyBytecode::NOP,
            PyBytecode::LoadName("x".to_string()), 
            PyBytecode::LoadConst(Obj::Int(3.into())), 
            PyBytecode::CompareOp(Op::LessThan), 
            PyBytecode::PopJumpIfFalse(8),
            PyBytecode::PushNull,
            PyBytecode::LoadName("x".to_string()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::LoadName("x".to_string()),
            PyBytecode::LoadConst(Obj::Int(1.into())),
            PyBytecode::BinaryAdd,
            PyBytecode::StoreName("x".to_string()),
            PyBytecode::JumpBackward(12),
            PyBytecode::NOP,
        ];
        let mut vm = PyVM::new();
        vm.execute(code);
    }

    #[test]
    fn bytecode_from_file()
    {
        let code = Interpreter::compile_file("src/pyrs_tests/compile_test_1.py").unwrap();
        println!("Bytecode from file:\n{}", PyBytecode::to_string(&code));
        let expected = vec![
            PyBytecode::LoadConst("sum_a".into()),
            PyBytecode::LoadConst(3.into()),
            PyBytecode::MakeFunction,
            PyBytecode::JumpForward(27),
            PyBytecode::StoreName("a".into()),
            PyBytecode::LoadConst(0.into()),
            PyBytecode::StoreName("s".into()),
            PyBytecode::PushNull,
            PyBytecode::LoadConst(0.into()),
            PyBytecode::LoadName("a".into()),
            PyBytecode::LoadConst(1.into()),
            PyBytecode::BinaryAdd,
            PyBytecode::LoadConst(1.into()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Range),
            PyBytecode::StoreName("r".into()),
            PyBytecode::PushNull,
            PyBytecode::LoadName("r".into()),
            PyBytecode::UnpackSequence,
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::LoadName("r".into()),
            PyBytecode::GetIter,
            PyBytecode::ForIter(6),
            PyBytecode::StoreName("v".into()),
            PyBytecode::LoadName("s".into()),
            PyBytecode::LoadName("v".into()),
            PyBytecode::BinaryAdd,
            PyBytecode::StoreName("s".into()),
            PyBytecode::JumpBackward(7),
            PyBytecode::LoadName("s".into()),
            PyBytecode::ReturnValue,
            PyBytecode::ReturnValue,
            PyBytecode::LoadConst("choice".into()),
            PyBytecode::LoadConst(34.into()),
            PyBytecode::MakeFunction,
            PyBytecode::JumpForward(11),
            PyBytecode::StoreName("s".into()),
            PyBytecode::LoadName("s".into()),
            PyBytecode::LoadConst("loop".into()),
            PyBytecode::CompareOp(Op::Eq),
            PyBytecode::PopJumpIfFalse(3),
            PyBytecode::LoadConst(true.into()),
            PyBytecode::ReturnValue,
            PyBytecode::JumpForward(2),
            PyBytecode::LoadConst(false.into()),
            PyBytecode::ReturnValue,
            PyBytecode::ReturnValue,
            PyBytecode::LoadConst("empty".into()),
            PyBytecode::LoadConst(49.into()),
            PyBytecode::MakeFunction,
            PyBytecode::JumpForward(2),
            PyBytecode::NOP,
            PyBytecode::ReturnValue,
            PyBytecode::LoadConst(5.into()),
            PyBytecode::LoadConst("sum_a".into()),
            PyBytecode::CallFunction(1),
            PyBytecode::StoreName("x".into()),
            PyBytecode::PushNull,
            PyBytecode::LoadName("x".into()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::LoadConst("loop".into()),
            PyBytecode::LoadConst("choice".into()),
            PyBytecode::CallFunction(1),
            PyBytecode::StoreName("y".into()),
            PyBytecode::PushNull,
            PyBytecode::LoadName("y".into()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::LoadConst("empty".into()),
            PyBytecode::CallFunction(0),
            PyBytecode::StoreName("z".into()),
            PyBytecode::PushNull,
            PyBytecode::LoadName("z".into()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print)
        ];
        assert_eq!(&code, &expected);

        let mut vm = PyVM::new();
        vm.execute(code);
    }

    #[test]
    fn module_from_file()
    {
        let module = Interpreter::compile_module("src/pyrs_tests/module_test_1.py").unwrap();
        println!("{:#?}", module);
        panic!();
    }

    #[test]
    fn module_import()
    {
        let src = 
        "import module_test_1\n \
        module_test_1.mod_fn1()";
        let exprs = Expression::from_multiline(src);
        dbg!(&exprs);
        let mut code = vec![];
        for e in exprs {
            PyBytecode::from_expr(e, &mut code);
        }
        println!("code: \n{}", PyBytecode::to_string(&code));
        
        let mut vm = PyVM::new();
        vm.append_working_dir("src/pyrs_tests");
        vm.execute(code);

        panic!();
    }

    use std::{
        time::Instant,
        process::Command,
    };    

    #[test]
    #[ignore]
    fn speed_test()
    {
        let pyrs_start = Instant::now();
        let pyrs_output = Command::new("Pyrs.exe")
        .arg("speed.py")
        .output()
        .expect("Failed to run \"Pyrs.exe speed.py\" ");
        let pyrs_duration = pyrs_start.elapsed();
        {
            let pyrs_stdout = str::from_utf8(&pyrs_output.stdout).expect("Not valid UTF-8");
            println!("Status Pyrs: success");
            println!("Stdout Pyrs: \n{}", pyrs_stdout);
        }

        let cpython_start = Instant::now();
        let cpython_output = Command::new("python3")
        .arg("speed.py")
            .output()
            .expect("Failed to run \"python3 speed.py\" ");
        
        let cpython_duration = cpython_start.elapsed();
        {
            let cpython_stdout = str::from_utf8(&cpython_output.stdout).expect("Not valid UTF-8");
            println!("Status CPython: success");
            println!("Stdout CPython: \n{}", cpython_stdout);
        }
        
        println!("pyrs: ");
        println!("Time elapsed: {:?}", pyrs_duration);
        println!("ms: {}", pyrs_duration.as_millis());
        
        println!("cpython: ");
        println!("ms: {}", cpython_duration.as_millis());
        println!("Time elapsed: {:?}", cpython_duration);
        
    }

    #[test]
    fn list()
    {
        let line1 = Expression::from_line("x = [2, 3, 4]");
        assert_eq!(line1.to_string(), "Op[= Ident(x) Op[list Atom(2) Atom(3) Atom(4)]]".to_string());
    
        let line2 = Expression::from_line("print(x + [\"add\", \"none\"])");
        assert_eq!(line2.to_string(), "Call[print args[ Op[+ Ident(x) Op[list Atom(add) Atom(none)]]]]");

        let mut bytecode = vec![];
        PyBytecode::from_expr(line1, &mut bytecode);
        PyBytecode::from_expr(line2, &mut bytecode);

        assert_eq!(format!("{:?}", bytecode), r#"[LoadConst(Int(2)), LoadConst(Int(3)), LoadConst(Int(4)), BuildList(3), StoreName("x"), PushNull, LoadName("x"), LoadConst(Str("add")), LoadConst(Str("none")), BuildList(2), BinaryAdd, CallInstrinsic1(Print)]"#.to_string());
        let mut vm = PyVM::new();
        vm.execute(bytecode);
    }

    #[test]
    fn definition()
    {
        let line1 = Expression::from_multiline("def go(a):\n\tprint(1)\ngo()");

        let expr_strs = join_expr_strings(vec![&line1[0], &line1[1]]);
        let res_strs = "Keyword[def conds[ Ident(go) Ident(a)] args[ Call[print args[ Atom(1)]]]] | Call[go args[]]";
        assert_eq!(expr_strs, res_strs);
    }

    #[test]
    fn bytecode_if_elif_else()
    {
        //panic!();
        let code = PyBytecode::from_str(
            "if False:\n\
            \tprint(\"a: bad\")\n\
            elif False:\n\
            \tprint(\"b: good\")\n\
            elif True:\n\
            \tprint(\"e: good\")\n\
            if False:\n\
            \tprint(\"c: good\")\n\
            else:\n\
            \tprint(\"d: good\")"
        );

        println!("{}", PyBytecode::to_string(&code));
        let instructions = vec![
            PyBytecode::LoadConst(false.to_obj()),
            PyBytecode::PopJumpIfFalse(4),
            PyBytecode::PushNull,
            PyBytecode::LoadConst("a: bad".to_obj()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::JumpForward(12),
            PyBytecode::LoadConst(false.to_obj()),
            PyBytecode::PopJumpIfFalse(4),
            PyBytecode::PushNull,
            PyBytecode::LoadConst("b: good".to_obj()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::JumpForward(6),
            PyBytecode::LoadConst(true.to_obj()),
            PyBytecode::PopJumpIfFalse(4),
            PyBytecode::PushNull,
            PyBytecode::LoadConst("e: good".to_obj()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::JumpForward(0),
            PyBytecode::LoadConst(false.to_obj()),
            PyBytecode::PopJumpIfFalse(4),
            PyBytecode::PushNull,
            PyBytecode::LoadConst("c: good".to_obj()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::JumpForward(3),
            PyBytecode::PushNull,
            PyBytecode::LoadConst("d: good".to_obj()),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
        ];
        assert_eq!(PyBytecode::to_string(&code), PyBytecode::to_string(&instructions));
        //for i in 0..code.len() {
        //    assert_eq!(code[i], instructions[i], "Instruction ({i})");
        //}
        let mut vm = PyVM::new();
        vm.execute(code);

    }

    #[test]
    fn function_definition_bytecode() {
        let code = PyBytecode::from_str(
            "def add(x, y):\n\
             \treturn x + y\n\
            result = add(5, 3)"
        );
        
        println!("Function definition bytecode:\n{}", PyBytecode::to_string(&code));
        
        // Test that function definition generates proper bytecode
        assert!(code.iter().any(|inst| matches!(inst, PyBytecode::MakeFunction)));
        assert!(code.iter().any(|inst| matches!(inst, PyBytecode::ReturnValue)));
        assert!(code.iter().any(|inst| matches!(inst, PyBytecode::JumpForward(_))));
        
        let mut vm = PyVM::new();
        vm.execute(code);
    }

    #[test]
    #[ignore]
    fn function_with_default_args() {
        let expr = Expression::from_multiline("def greet(name, msg=\"Hello\"):\n\tprint(msg, name)");
        assert_eq!(expr.len(), 1);
        
        let expected = "Keyword[def conds[ Ident(greet) Ident(name) Op[= Ident(msg) Atom(Hello)]] args[ Call[print args[ Ident(msg) Ident(name)]]]]";
        assert_eq!(expr[0].to_string(), expected);
    }

    #[test]
    fn unary_operations() {
        let pos_expr = Expression::from_line("+5");
        let neg_expr = Expression::from_line("-10");
        
        assert_eq!(pos_expr.to_string(), "Op[+ Atom(5)]");
        assert_eq!(neg_expr.to_string(), "Op[- Atom(10)]");
    }

    #[test]
    fn bytecode_unary() {
        let mut code = vec![];
        PyBytecode::from_expr(Expression::from_line("-42"), &mut code);
        
        let expected = vec![
            PyBytecode::LoadConst(Obj::Int(42.into())),
            PyBytecode::UnaryNegative
        ];

        assert_eq!(PyBytecode::to_string(&code), PyBytecode::to_string(&expected));
    }

    #[test]
    fn ops_compare() {
        let comparisons = vec![
            ("5 < 10", "True"),
            ("10 > 5", "True"), 
            ("5 <= 5", "True"),
            ("5 >= 5", "True"),
            ("5 == 5", "True"),
            ("5 != 4", "True"),
            ("\"abc\" < \"def\"", "True"),
            ("\"xyz\" > \"abc\"", "True"),
        ];
        
        let mut vs = Obj::new_map();
        let mut fns = Funcs::get_std_map();

        for (expr_str, expected) in comparisons {
            let expr = Expression::from_line(expr_str);
            assert_eq!(expr.eval(&mut vs, &mut fns).unwrap().to_string(), expected, "{}", expr.to_string());
        }
    }

    #[test]
    fn ops_tuple() 
    {
        let tuple_expr = Expression::from_line("(1, 2, 3)");
        println!("Tuple expression: {}", tuple_expr);
        
        let mut bytecode = vec![];
        PyBytecode::from_expr(tuple_expr, &mut bytecode);
        println!("Tuple bytecode: {:?}", bytecode);
    }

    #[test]
    fn ops_set()
    {
        let tuple_expr = Expression::from_line("{1, 2, 3}");
        println!("Tuple expression: {}", tuple_expr);
        
        let mut bytecode = vec![];
        PyBytecode::from_expr(tuple_expr, &mut bytecode);
        println!("Tuple bytecode: {:?}", bytecode);
    }

    #[test]
    fn ops_dot()
    {
        let expr1 = Expression::from_line("a.x");
        assert_eq!(&expr1.to_string(), "Op[. Ident(a) Ident(x)]");

        let expr2 = Expression::from_line("a.x()");
        assert_eq!(&expr2.to_string(), "Op[. Ident(a) Call[x args[]]]");
    }

    #[test]
    fn for_loop_parsing() {
        let source_code = 
            "v = [1, 2, 3]\n\
            for i in v:\n\
                \tprint(i)";

        let for_expr = Expression::from_multiline(source_code);
        
        assert_eq!(for_expr.len(), 2);
        println!("For loop: {}", for_expr[1]);
        
        match &for_expr[0] {
            Expression::Operation(Op::Equals, args) => {
                assert_eq!(args[0], Expression::Ident("v".into()));
                assert_eq!(args[1], Expression::Operation(Op::List, vec![Expression::Atom("1".into()), Expression::Atom("2".into()), Expression::Atom("3".into())]));
            }
            _ => panic!("Expected assign operation"),
        }

        // Check that it parses as a for keyword with proper structure
        match &for_expr[1] {
            Expression::Keyword(Keyword::For, conds, body) => {
                assert!(!conds.is_empty(), "For loop should have conditions");
                assert!(!body.is_empty(), "For loop should have body");
            }
            _ => panic!("Expected for loop keyword expression"),
        }

        let code = PyBytecode::from_str(source_code);
        println!("code: \n{}", PyBytecode::to_string(&code));
        
        let mut vm = PyVM::new();
        vm.execute(code);

    }

    #[test]
    fn nested_list() {
        let nested_list = Expression::from_line("[[1, 2], [3, 4]]");
        assert_eq!(nested_list.to_string(), "Op[list Op[list Atom(1) Atom(2)] Op[list Atom(3) Atom(4)]]");
        
        let mut bytecode = vec![];
        PyBytecode::from_expr(nested_list, &mut bytecode);
        
        // Should have multiple BuildList instructions
        let build_list_count = bytecode.iter().filter(|inst| matches!(inst, PyBytecode::BuildList(_))).count();
        assert_eq!(build_list_count, 3); // Two inner lists + one outer list
    }

    #[test]
    #[ignore]
    fn error_bytecode_generation() {
        // Test that unsupported operations generate error bytecode
        let mut bytecode = vec![];
        let invalid_expr = Expression::Operation(Op::Dot, vec![
            Expression::Atom("obj".to_string()),
            Expression::Atom("method".to_string())
        ]);
        
        PyBytecode::from_expr(invalid_expr, &mut bytecode);
        
        // Should generate an Error bytecode
        assert!(bytecode.iter().any(|inst| matches!(inst, PyBytecode::Error(_))));
    }

    #[test]
    fn parse_precedence_simple() {
        let e = Expression::from_line("1 + 2 * 3");
        assert_eq!(e.to_string(), "Op[+ Atom(1) Op[* Atom(2) Atom(3)]]");
    }

    #[test]
    fn parse_precedence_complex() {
        let e = Expression::from_line("2 + 3 * 4 - 5 / 2");
        let expected = "Op[- Op[+ Atom(2) Op[* Atom(3) Atom(4)]] Op[/ Atom(5) Atom(2)]]";
        assert_eq!(e.to_string(), expected);
    }

    #[test]
    fn parse_precedence_parentheses_override() {
        let e = Expression::from_line("(2 + 3) * 4 - 5 / 2");
        let expected = "Op[- Op[* Op[+ Atom(2) Atom(3)] Atom(4)] Op[/ Atom(5) Atom(2)]]";
        assert_eq!(e.to_string(), expected);
    }

    #[test]
    fn parse_precedence_complex_maths() 
    {
        let code = PyBytecode::from_str("2 + 3 * 4 - 5 / 2");
        println!("code: \n{}", PyBytecode::to_string(&code));
        let mut vm = PyVM::new();
        vm.execute(code);

        let stack = vm.view_stack();
        let expected = vec![vec![11.5.to_arc()]];
        assert_eq!(stack, &expected);
    }

    #[test]
    #[ignore]
    fn variable_scoping() {
        // Test variable assignment and retrieval
        let code = PyBytecode::from_str(
            "x = 10\n\
             y = x * 2\n\
             print(y)"
        );
        
        let mut vm = PyVM::new();
        vm.execute(code);
        let v = vm.get_vars();
        let vars = &v[0];
        
        // Check that variables are stored correctly
        assert!(vars.contains_key("x"));
        assert!(vars.contains_key("y"));
        assert_eq!(vars["y"].to_string(), "20");
    }

    #[test]
    fn intrinsic_functions() {
        // Test that intrinsic functions are properly identified
        assert!(IntrinsicFunc::try_get("print").is_some());
        assert!(IntrinsicFunc::try_get("input").is_some());
        assert!(IntrinsicFunc::try_get("nonexistent").is_none());
        
        // Test intrinsic function bytecode generation
        let print_expr = Expression::from_line("print(\"Hello World\")");
        let mut bytecode = vec![];
        PyBytecode::from_expr(print_expr, &mut bytecode);
        
        assert!(bytecode.iter().any(|inst| matches!(inst, PyBytecode::CallInstrinsic1(IntrinsicFunc::Print))));
    }

    #[test]
    fn multiline_string_parsing() {
        // Test parsing of strings with quotes
        let single_quote = Expression::from_line("'single quoted'");
        let double_quote = Expression::from_line("\"double quoted\"");
        
        assert_eq!(single_quote.to_string(), "Atom(single quoted)");
        assert_eq!(double_quote.to_string(), "Atom(double quoted)");
        
        let mut eq = EqTester::new();
        eq.eval_eq(&single_quote, "single quoted");
        eq.eval_eq(&double_quote, "double quoted");
    }

    #[test]
    fn bytecode_instruction_enum_coverage() {
        // Test that all enum variants can be created
        let _nop = PyBytecode::NOP;
        let _pop_top = PyBytecode::PopTop;
        let _copy = PyBytecode::Copy(1);
        let _swap = PyBytecode::Swap(2);
        let _unary_neg = PyBytecode::UnaryNegative;
        let _unary_not = PyBytecode::UnaryNot;
        let _unary_inv = PyBytecode::UnaryInvert;
        let _to_bool = PyBytecode::ToBool;
        let _binary_xor = PyBytecode::BinaryXOR;
        let _load_global = PyBytecode::LoadGlobal;
        let _store_global = PyBytecode::StoreGlobal;
        let _call_intrinsic2 = PyBytecode::CallInstrinsic2(IntrinsicFunc::Print);
        let _jump_if_false = PyBytecode::JumpIfFalse;
        let _jump_absolute = PyBytecode::JumpAbsolute;
        let _build_tuple = PyBytecode::BuildTuple(3);
        let _build_map = PyBytecode::BuildMap;
        let _list_append = PyBytecode::ListAppend;
        let _for_iter = PyBytecode::ForIter;
        let _get_iter = PyBytecode::GetIter;
        let _new_stack = PyBytecode::NewStack;
        let _destroy_stack = PyBytecode::DestroyStack;
    }

    #[test]
    fn expression_none_handling() {
        let empty_expr = Expression::None;
        assert_eq!(empty_expr.to_string(), "None");
        
        let mut bytecode = vec![];
        PyBytecode::from_expr(empty_expr, &mut bytecode);
        
        // Should not generate any bytecode for None expression
        assert!(bytecode.is_empty());
    }

    #[test]
    fn token_equality() {
        // Test Token PartialEq implementation
        let token1 = Token::Ident("test");
        let token2 = Token::Ident("test");
        let token3 = Token::Ident("different");
        
        assert_eq!(token1, token2);
        assert_ne!(token1, token3);
        
        let atom1 = Token::Atom("123");
        let atom2 = Token::Atom("123");
        assert_eq!(atom1, atom2);
        
        let op1 = Token::Op(Op::Plus);
        let op2 = Token::Op(Op::Plus);
        assert_eq!(op1, op2);
    }

    #[test]
    fn operator_display() {
        // Test Op Display implementation
        assert_eq!(format!("{}", Op::Plus), "+");
        assert_eq!(format!("{}", Op::Minus), "-");
        assert_eq!(format!("{}", Op::Asterisk), "*");
        assert_eq!(format!("{}", Op::ForwardSlash), "/");
        assert_eq!(format!("{}", Op::Equals), "=");
        assert_eq!(format!("{}", Op::Eq), "==");
        assert_eq!(format!("{}", Op::Neq), "!=");
        assert_eq!(format!("{}", Op::LessThan), "<");
        assert_eq!(format!("{}", Op::GreaterThan), ">");
        assert_eq!(format!("{}", Op::LessEq), "<=");
        assert_eq!(format!("{}", Op::GreaterEq), ">=");
    }

    #[test]
    fn keyword_display() {
        // Test Keyword Display implementation
        assert_eq!(format!("{}", Keyword::If), "if");
        assert_eq!(format!("{}", Keyword::Elif), "elif");
        assert_eq!(format!("{}", Keyword::Else), "else");
        assert_eq!(format!("{}", Keyword::For), "for");
        assert_eq!(format!("{}", Keyword::While), "while");
        assert_eq!(format!("{}", Keyword::Def), "def");
        assert_eq!(format!("{}", Keyword::True), "True");
        assert_eq!(format!("{}", Keyword::False), "False");
    }

    #[test]
    fn utils_string_functions() {
        use crate::pyrs_utils::*;
        
        // Test str_starts_with
        assert!(str_starts_with("123abc", char::is_numeric));
        assert!(!str_starts_with("abc123", char::is_numeric));
        
        // Test trim_first_and_last
        assert_eq!(trim_first_and_last("\"hello\""), "hello");
        assert_eq!(trim_first_and_last("'world'"), "world");
        
        // Test get_indent
        assert_eq!(get_indent("    hello"), 4);
        assert_eq!(get_indent("\thello"), 4);
        assert_eq!(get_indent("    \thello"), 8); // 4 spaces + 1 tab
        assert_eq!(get_indent("hello"), 0);
    }

    #[test]
    fn split_to_words_comprehensive() {
        use crate::pyrs_utils::split_to_words;
        
        // Test basic splitting
        let words = split_to_words("hello world");
        assert_eq!(words, vec!["hello", "world"]);
        
        // Test operators
        let words = split_to_words("x=5");
        assert_eq!(words, vec!["x", "=", "5"]);
        
        let words = split_to_words("x==y");
        assert_eq!(words, vec!["x", "==", "y"]);
        
        let words = split_to_words("x!=y");
        assert_eq!(words, vec!["x", "!=", "y"]);
        
        // Test string literals
        let words = split_to_words("print(\"hello world\")");
        assert_eq!(words, vec!["print", "(", "\"hello world\"", ")"]);
        
        // Test mixed content
        let words = split_to_words("if x >= 10:");
        assert_eq!(words, vec!["if", "x", ">=", "10", ":"]);
    }

    #[test]
    #[ignore]
    fn bytecode_conversion() {
        // Test PyBytecode to u8 conversion
        let nop: u8 = PyBytecode::NOP.into();
        let load_const: u8 = PyBytecode::LoadConst(Obj::None).into();
        
        // These should be different values
        assert_ne!(nop, load_const);
    }

    #[test]

    fn complex_if_elif_else_evaluation() {
        let code = PyBytecode::from_str(
            "x = 15\n\
             if x < 10:\n\
             \tresult = \"small\"\n\
             elif x < 20:\n\
             \tresult = \"medium\"\n\
             else:\n\
             \tresult = \"large\""
        );
        
        let expected = vec![
            PyBytecode::LoadConst(15.to_obj()),
            PyBytecode::StoreName("x".into()),
            PyBytecode::LoadName("x".into()),
            PyBytecode::LoadConst(10.to_obj()),
            PyBytecode::CompareOp(Op::LessThan),
            PyBytecode::PopJumpIfFalse(3),
            PyBytecode::LoadConst("small".to_obj()),
            PyBytecode::StoreName("result".into()),
            PyBytecode::JumpForward(9),
            PyBytecode::LoadName("x".into()),
            PyBytecode::LoadConst(20.to_obj()),
            PyBytecode::CompareOp(Op::LessThan),
            PyBytecode::PopJumpIfFalse(3),
            PyBytecode::LoadConst("medium".to_obj()),
            PyBytecode::StoreName("result".into()),
            PyBytecode::JumpForward(2),
            PyBytecode::LoadConst("large".to_obj()),
            PyBytecode::StoreName("result".into()),
        ];

        assert_eq!(PyBytecode::to_string(&code), PyBytecode::to_string(&expected));
        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.execute(code);
        let vars = vm.get_vars();
        
        let mut expected_vars = HashMap::new();
        expected_vars.insert("result".to_string(), "medium".to_arc());
        expected_vars.insert("x".to_string(), 15.to_arc());
        
        assert_eq!(vars[0]["result"], expected_vars["result"]);
    }

    #[test]
    fn nested_while_loops() {
        let expr = Expression::from_multiline(
            "i = 0\n\
             while i < 3:\n\
             \tj = 0\n\
             \twhile j < 2:\n\
             \t\tprint(i, j)\n\
             \t\tj = j + 1\n\
             \ti = i + 1"
        );
        
        // Just test that it parses correctly
        assert!(expr.len() >= 2); // At least assignment and while loop
        
        // Test bytecode generation doesn't crash
        let mut bytecode = vec![];
        for e in expr {
            PyBytecode::from_expr(e, &mut bytecode);
        }
        
        println!("Nested while loops bytecode:\n{}", PyBytecode::to_string(&bytecode));
    }

    #[test] 
    fn list_concat() {
        let list_ops = vec![
            ("[1, 2] + [3, 4]", "[1, 2, 3, 4]"),
            // Add more list operations as they get implemented
        ];
        
        for (i, (expr_str, expected)) in list_ops.iter().enumerate() {
            println!("Line: {}", expr_str);
            let exprs = Expression::from_multiline(expr_str);
            let expr = exprs.first().unwrap();
            let obj = expr.clone().to_obj();
            assert_eq!(&obj.__str__(), expected, "expr(#{i}) {}", expr.to_string());
        }
    }

    #[test]
    fn iteration() 
    {
        let list = vec![1.to_arc(), 2.to_arc()].to_obj();
        for x in list {
            println!("{}", x);
        }

        let list = vec![1.to_arc(), 2.to_arc()].to_obj();
        for mut x in &mut list.into_iter() {
            x = Obj::add(x.as_ref(), &2.to_obj()).to_arc();
            println!("{}", x);
        }
    }

    #[test]
    fn parse_pratt_tests() {
        let s = Expression::from_line("1");
        assert_eq!(s.to_string(), "Atom(1)");

        let s = Expression::from_line("1 + 2 * 3");
        assert_eq!(s.to_string(), "Op[+ Atom(1) Op[* Atom(2) Atom(3)]]");

        let s = Expression::from_line("a + b * c * d + e");
        assert_eq!(s.to_string(), "Op[+ Op[+ Ident(a) Op[* Op[* Ident(b) Ident(c)] Ident(d)]] Ident(e)]");

        let s = Expression::from_line("f . g . h");
        assert_eq!(s.to_string(), "Op[. Ident(f) Op[. Ident(g) Ident(h)]]");

        let s = Expression::from_line(" 1 + 2 + f . g . h * 3 * 4");
        assert_eq!(
            s.to_string(),
            "Op[+ Op[+ Atom(1) Atom(2)] Op[* Op[* Op[. Ident(f) Op[. Ident(g) Ident(h)]] Atom(3)] Atom(4)]]",
        );
        // "(+ (+ 1 2) (* (* (. f (. g h)) 3) 4))"

        let s = Expression::from_line("--1 * 2");
        assert_eq!(s.to_string(), "Op[* Op[- Op[- Atom(1)]] Atom(2)]");

        let s = Expression::from_line("--f . g");
        assert_eq!(s.to_string(), "Op[- Op[- Op[. Ident(f) Ident(g)]]]");

        let s = Expression::from_line("(((0)))");
        assert_eq!(s.to_string(), "Atom(0)");

        let s = Expression::from_line("x[0][1]");
        assert_eq!(s.to_string(), "Op[[ Op[[ Ident(x) Atom(0)] Atom(1)]");

    }

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
