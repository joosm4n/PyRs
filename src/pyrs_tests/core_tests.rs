#[allow(unused_imports)]
use crate::{
    pyrs_bytecode::PyBytecode,
    pyrs_codeobject::{PyCodeObj, PyCompileCtx},
    pyrs_error::PyException,
    pyrs_interpreter::{Interpreter, InterpreterCommand},
    pyrs_obj::{Obj, ToObj},
    pyrs_parsing::{Expression, Keyword, Lexer, Op, Token},
    pyrs_pyobject::{AttrDict, PyObjPtr, PyObject},
    pyrs_serializer::{PyHeader, PySerializer},
    pyrs_std::{FnPtr, Funcs},
    pyrs_utils::{FromBytes, PyUtils},
    pyrs_vm::{IntrinsicFunc, PyFrame, PyVM},
};

#[cfg(test)]
mod tests {

    use std::{collections::HashMap, mem::size_of, ops::Index};

    use crate::{pyrs_error::PyPanicHandle, pyrs_interpreter::PyRsVersion};

    use super::*;
    use pretty_assertions::assert_eq;

    struct EqTester {
        vars: AttrDict,
        funcs: HashMap<String, FnPtr>,
    }

    impl EqTester {
        fn new() -> Self {
            EqTester {
                vars: PyObject::new_map(),
                funcs: Funcs::get_std_map(),
            }
        }

        fn eval_eq(&mut self, expr: &Expression, result: &str) {
            let res = match expr.eval(&mut self.vars, &mut self.funcs) {
                Ok(val) => val,
                Err(e) => panic!("{e}"),
            };
            assert_eq!(res.get_ref().to_string(), result);
        }
    }

    fn join_expr_strings(exprs: Vec<&Expression>) -> String {
        let mut res = String::new();
        for e in exprs {
            res.push_str(e.to_string().as_str());
            res.push_str(" | ");
        }
        res.pop();
        res.pop();
        res.pop();
        res
    }

    #[test]
    fn memory_size_types() {
        assert_eq!(56, size_of::<Obj>(), "Obj size changed");
        assert_eq!(24, size_of::<Token>(), "Token size changed");
        assert_eq!(56, size_of::<Expression>(), "Expression size changed");
        assert_eq!(2, size_of::<PyBytecode>(), "PyBytecode size changed");
        assert_eq!(136, size_of::<PyVM>(), "PyVirtualMachine size changed");
        assert_eq!(192, size_of::<PyCodeObj>(), "PyCodeObj size changed");
        assert_eq!(72, size_of::<PyFrame>(), "PyFrame size changed");
    }

    #[test]
    fn parse() {
        let s1 = Expression::from_line("1");
        let s2 = Expression::from_line("1 + 2 * 3");
        let s3 = Expression::from_line("(1 + 2) * 3");
        let s4 = Expression::from_line("print(100)");
        let s5 = Expression::from_line("print(1, 2, \"5\")");
        let s6 = Expression::from_line("x=2");
        let s7 = Expression::from_line("x+=2");

        let final_str = join_expr_strings(vec![&s1, &s2, &s3, &s4, &s5, &s6, &s7]);
        let res_str = "Atom(1) | \
        Op[+ Atom(1) Op[* Atom(2) Atom(3)]] | \
        Op[* Op[+ Atom(1) Atom(2)] Atom(3)] | \
        Call[print args[ Atom(100)]] | \
        Call[print args[ Atom(1) Atom(2) Atom(5)]] | \
        Op[= Ident(x) Atom(2)] | \
        Op[+= Ident(x) Atom(2)]";
        assert_eq!(final_str, res_str);
    }

    #[test]
    fn parse_underscore() {
        let s1 = PyUtils::split_to_words("x.__str__()");
        let res_str = vec!["x", ".", "__str__", "(", ")"];
        assert_eq!(s1, res_str);
    }

    #[test]
    fn strlit_parse_eval() {
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
    fn test_12() {
        let exprs = Expression::from_multiline("x = 2\n if x:\n\t print_ret(x) ");
        assert_eq!(exprs.len(), 2);
        println!("Exprs: {:?}", exprs);

        let mut vars = PyObject::new_map();
        let mut funcs = Funcs::get_std_map();
        let expr_results = vec![
            "Op[= Ident(x) Atom(2)]",
            "Keyword[if conds[ Ident(x)] args[ Call[print_ret args[ Ident(x)]]]]",
        ];
        let obj_results = vec![
            PyObject::from(2usize),
            PyObject::from(true),
            PyObject::from("2 "),
        ];

        for (idx, expr) in exprs.iter().enumerate() {
            println!("Evaluating: {expr}");
            assert_eq!(expr.to_string(), expr_results.index(idx).to_string());
            let obj = expr.eval(&mut vars, &mut funcs).handle_panic();
            println!("Obj: {}", *obj.get_ref());
            println!("vars: {:?}", vars);
            assert_eq!(obj, obj_results.index(idx).clone());
        }
    }

    #[test]
    fn equality() {
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
        eq.eval_eq(&s5, "False");
        eq.eval_eq(&s6, "True");
    }

    #[test]
    fn obj_equality() {
        assert_eq!(PyObjPtr::none(), PyObjPtr::none());
        assert_eq!(&PyObjPtr::none(), &PyObjPtr::none());

        assert_eq!(true.to_pyptr(), 1.0.to_pyptr());
        assert_ne!(PyObject::empty_dict(), PyObject::empty_dict());
    }

    #[test]
    fn parse_assign() {
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
    fn while_test() {
        let expr = Expression::from_multiline(
            r#"
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
        "#,
        );

        let ret_strs = [
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
            "None",
        ];

        let mut vars = PyObject::new_map();
        let mut funcs = Funcs::get_std_map();

        let idx_err = "[Bad Index]";

        let mut ret_objs = vec![];
        for (idx, e) in expr.iter().enumerate() {
            let obj = e.eval(&mut vars, &mut funcs).handle_panic();
            assert_eq!(
                e.to_string(),
                ret_strs.get(idx).unwrap_or(&idx_err).to_string()
            );
            ret_objs.push(obj);
        }
    }

    #[test]
    fn nested_ifs() {
        //panic!();
        let expr = Expression::from_multiline(
            "if True:\n\
         \tprint_ret(\"a: good\")\n\
         \tif False:\n\
         \t\tprint_ret(\"b: bad\")\n\
         \tif True:\n\
         \t\tprint_ret(\"c: good\")\n\
         \tprint(\"d: good\")",
        );

        let ret_strs = [
            r#"Keyword[if conds[ Keyword[True conds[] args[]]] args[ Call[print_ret args[ Atom(a: good)]] Keyword[if conds[ Keyword[False conds[] args[]]] args[ Call[print_ret args[ Atom(b: bad)]]]] Keyword[if conds[ Keyword[True conds[] args[]]] args[ Call[print_ret args[ Atom(c: good)]]]] Call[print args[ Atom(d: good)]]]]"#,
        ];

        let mut vars = PyObject::new_map();
        let mut funcs = Funcs::get_std_map();

        let idx_err = "[Bad Index]";

        let mut ret_objs = vec![];
        for (idx, e) in expr.iter().enumerate() {
            let obj = e.eval(&mut vars, &mut funcs).handle_panic();
            assert_eq!(
                e.to_string(),
                ret_strs.get(idx).unwrap_or(&idx_err).to_string()
            );
            ret_objs.push(obj);
        }
    }

    #[test]
    fn if_elif_else_expr() {
        //panic!();
        let expr = Expression::from_multiline(
            "if False:\n\
         \tprint_ret(\"a: bad\")\n\
         elif True:\n\
         \tprint_ret(\"b: good\")\n\
         if False:\n\
         \tprint_ret(\"c: good\")\n\
         else:\n\
         \tprint(\"d: good\")",
        );

        let ret_strs = [
            r#"Keyword[if conds[ Keyword[False conds[] args[]]] args[ Call[print_ret args[ Atom(a: bad)]] Keyword[elif conds[ Keyword[True conds[] args[]]] args[]] Call[print_ret args[ Atom(b: good)]]]]"#,
            r#"Keyword[if conds[ Keyword[False conds[] args[]]] args[ Call[print_ret args[ Atom(c: good)]] Keyword[else conds[] args[]] Call[print args[ Atom(d: good)]]]]"#,
        ];

        let mut vars = PyObject::new_map();
        let mut funcs = Funcs::get_std_map();

        let idx_err = "[Bad Index]";

        let mut ret_objs = vec![];
        for (idx, e) in expr.iter().enumerate() {
            let obj = e.eval(&mut vars, &mut funcs).handle_panic();
            assert_eq!(
                e.to_string(),
                ret_strs.get(idx).unwrap_or(&idx_err).to_string()
            );
            ret_objs.push(obj);
        }
    }

    #[test]
    fn bytecode_manual() {
        let code = vec![
            PyBytecode::Resume,
            PyBytecode::LoadConst(0),
            PyBytecode::StoreFast(0),
            PyBytecode::PushNull,
            PyBytecode::LoadFast(0),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
        ];

        let code_obj = PyCodeObj {
            name: "__test_bytecode_manual__".into(),
            consts: vec![5.to_pyptr()],
            varnames: vec!["x".into()],
            names: vec![],
            bytecode: code,
            num_varnames: 1,
            num_consts: 1,
            num_names: 0,
            globals: AttrDict::new(),
        };

        println!("Instruction Queue: ");
        println!("{}", PyBytecode::to_string(&code_obj.bytecode));
        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.execute(code_obj);
    }

    #[test]
    fn bytecode_from_expr() {
        let expr = Expression::from_multiline("x = 2\n if x:\n\t print(x) ");
        let mut code = PyCompileCtx::new("__test_bytecode_from_expr__".into());
        for e in expr {
            PyBytecode::from_expr(e, &mut code);
        }
        let code_obj = code.finish();
        assert_eq!(
            code_obj.num_varnames, 1,
            "varnames: {:?}",
            code_obj.varnames
        );

        let inst_str = PyBytecode::to_string(&code_obj.bytecode);
        println!("Instructions:\n{}", inst_str);
        assert_eq!(
            format!("{:?}", &code_obj.bytecode),
            r#"[Resume, LoadConst(0), StoreFast(0), LoadFast(0), PopJumpIfFalse(4), LoadGlobal(0), LoadFast(0), CallFunction(1), JumpForward(0)]"#
        );

        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.execute(code_obj);
    }

    #[test]
    fn bytecode_while_loop() {
        let code_obj = PyBytecode::from_string(
            r#"x = 0
        while x < 3:
	        print(x)
	        x += 1
        "#,
        );
        println!(
            "Instructions:\n{}",
            PyBytecode::to_string(&code_obj.bytecode)
        );
        assert_eq!(format!("{:?}", &code_obj.bytecode), r#"[Resume, LoadConst(0), StoreFast(0), LoadFast(0), LoadConst(1), CompareOp(LessThan), PopJumpIfFalse(9), LoadGlobal(0), LoadFast(0), CallFunction(1), LoadFast(0), LoadConst(2), BinaryAdd, StoreFast(0), JumpBackward(12), LoadConst(3)]"#.to_string());

        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.execute(code_obj);
    }

    #[test]
    fn bytecode_handwritten() {
        let code = vec![
            PyBytecode::Resume,
            PyBytecode::LoadConst(0),
            PyBytecode::StoreFast(0),
            PyBytecode::NOP,
            PyBytecode::LoadFast(0),
            PyBytecode::LoadConst(1),
            PyBytecode::CompareOp(Op::LessThan),
            PyBytecode::PopJumpIfFalse(8),
            PyBytecode::PushNull,
            PyBytecode::LoadFast(0),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::LoadFast(0),
            PyBytecode::LoadConst(2),
            PyBytecode::BinaryAdd,
            PyBytecode::StoreFast(0),
            PyBytecode::JumpBackward(12),
            PyBytecode::NOP,
        ];

        let code_obj = PyCodeObj {
            name: "__test_bytecode_handwritten__".into(),
            bytecode: code,
            consts: vec![0.to_pyptr(), 3.to_pyptr(), 1.to_pyptr()],
            varnames: vec!["x".into()],
            names: vec![],
            num_consts: 3,
            num_varnames: 1,
            num_names: 0,
            globals: AttrDict::new(),
        };

        let mut vm = PyVM::new();
        vm.execute(code_obj);
    }

    #[test]
    #[ignore]
    fn bytecode_from_file() {
        let code_obj = Interpreter::compile_file("compile_test_1.py").unwrap();
        println!(
            "Bytecode from file:\n{}",
            PyBytecode::to_string(&code_obj.bytecode)
        );

        let expected = vec![
            PyBytecode::Resume,
            PyBytecode::LoadConst(0),
            PyBytecode::MakeFunction,
            PyBytecode::StoreFast(0),
            PyBytecode::LoadConst(1),
            PyBytecode::MakeFunction,
            PyBytecode::StoreFast(1),
            PyBytecode::LoadConst(2),
            PyBytecode::MakeFunction,
            PyBytecode::StoreFast(2),
            PyBytecode::LoadConst(3),
            PyBytecode::LoadFast(0),
            PyBytecode::CallFunction(1),
            PyBytecode::StoreFast(3),
            PyBytecode::PushNull,
            PyBytecode::LoadFast(3),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::LoadConst(4),
            PyBytecode::LoadName(1),
            PyBytecode::CallFunction(1),
            PyBytecode::StoreFast(4),
            PyBytecode::PushNull,
            PyBytecode::LoadFast(4),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
            PyBytecode::LoadName(2),
            PyBytecode::CallFunction(0),
            PyBytecode::StoreFast(5),
            PyBytecode::PushNull,
            PyBytecode::LoadFast(5),
            PyBytecode::CallInstrinsic1(IntrinsicFunc::Print),
        ];

        let expected_codeobj = PyCodeObj {
            name: "compile_test_1".into(),
            bytecode: expected,
            consts: vec!["sum_a".to_pyptr(), 1.to_pyptr()],
            varnames: vec![],
            names: vec![],
            num_consts: 2,
            num_varnames: 0,
            num_names: 0,
            globals: AttrDict::new(),
        };

        assert_eq!(&code_obj, &expected_codeobj);

        let mut vm = PyVM::new();
        vm.execute(code_obj);
    }

    #[test]
    fn module_from_file() {
        let module = Interpreter::compile_file("tests/module_test_1.py").unwrap();
        println!("{:#?}", module);
    }

    #[test]
    fn module_import() {
        let src = "import module_test_1\n \
            module_test_1.mod_fn1()";

        let exprs = Expression::from_multiline(src);
        dbg!(&exprs);
        let mut code = PyCompileCtx::new("__test_module_import__".into());
        for e in exprs {
            PyBytecode::from_expr(e, &mut code);
        }
        let code_obj = code.finish();
        println!("code: \n{}", PyBytecode::to_string(&code_obj.bytecode));

        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.append_working_dir("tests");
        vm.execute(code_obj);

        panic!();
    }

    use std::{process::Command, time::Instant};

    #[test]
    #[ignore]
    fn speed_test() {
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
    fn list() {
        let line1 = Expression::from_line("x = [2, 3, 4]");
        assert_eq!(
            line1.to_string(),
            "Op[= Ident(x) Op[list Atom(2) Atom(3) Atom(4)]]".to_string()
        );

        let line2 = Expression::from_line("print(x + [\"add\", \"none\"])");
        assert_eq!(
            line2.to_string(),
            "Call[print args[ Op[+ Ident(x) Op[list Atom(add) Atom(none)]]]]"
        );

        let mut bytecode = PyCompileCtx::new("__test_list__".into());
        PyBytecode::from_expr(line1, &mut bytecode);
        PyBytecode::from_expr(line2, &mut bytecode);

        let code_obj = bytecode.finish();
        assert_eq!(format!("{:?}", &code_obj.bytecode), r#"[Resume, BuildList(0), LoadConst(0), ListAppend(0), StoreFast(0), LoadGlobal(0), LoadFast(0), BuildList(0), LoadConst(1), ListAppend(0), BinaryAdd, CallFunction(1)]"#.to_string());

        let mut vm = PyVM::new();
        // vm.set_debug_mode(true);
        vm.execute(code_obj);
    }

    #[test]
    fn definition() {
        let line1 = Expression::from_multiline("def go(a):\n\tprint(1)\ngo()");

        let expr_strs = join_expr_strings(vec![&line1[0], &line1[1]]);
        let res_strs = "Keyword[def conds[ Ident(go) Ident(a)] args[ Call[print args[ Atom(1)]]]] | Call[go args[]]";
        assert_eq!(expr_strs, res_strs);
    }

    #[test]
    fn bytecode_if_elif_else() {
        //panic!();
        let code_obj = PyBytecode::from_string(
            "if False:\n\
            \tprint(\"a: bad\")\n\
            elif False:\n\
            \tprint(\"b: good\")\n\
            elif True:\n\
            \tprint(\"e: good\")\n\
            if False:\n\
            \tprint(\"c: good\")\n\
            else:\n\
            \tprint(\"d: good\")",
        );

        println!("{}", PyBytecode::to_string(&code_obj.bytecode));
        let instructions = vec![
            PyBytecode::Resume,
            PyBytecode::LoadConst(0),
            PyBytecode::PopJumpIfFalse(4),
            PyBytecode::LoadGlobal(IntrinsicFunc::Print as u8),
            PyBytecode::LoadConst(1),
            PyBytecode::CallFunction(1),
            PyBytecode::JumpForward(11),
            PyBytecode::LoadConst(0),
            PyBytecode::PopJumpIfFalse(4),
            PyBytecode::LoadGlobal(IntrinsicFunc::Print as u8),
            PyBytecode::LoadConst(2),
            PyBytecode::CallFunction(1),
            PyBytecode::JumpForward(6),
            PyBytecode::LoadConst(3),
            PyBytecode::PopJumpIfFalse(4),
            PyBytecode::LoadGlobal(IntrinsicFunc::Print as u8),
            PyBytecode::LoadConst(4),
            PyBytecode::CallFunction(1),
            PyBytecode::JumpForward(0),
            PyBytecode::LoadConst(0),
            PyBytecode::PopJumpIfFalse(4),
            PyBytecode::LoadGlobal(IntrinsicFunc::Print as u8),
            PyBytecode::LoadConst(5),
            PyBytecode::CallFunction(1),
            PyBytecode::JumpForward(2),
            PyBytecode::LoadGlobal(IntrinsicFunc::Print as u8),
            PyBytecode::LoadConst(6),
            PyBytecode::CallFunction(1),
        ];
        assert_eq!(
            PyBytecode::to_string(&code_obj.bytecode),
            PyBytecode::to_string(&instructions)
        );

        let mut vm = PyVM::new();
        vm.execute(code_obj);
    }

    #[test]
    fn function_definition_bytecode() {
        let code_obj = PyBytecode::from_string(
            "def add(x, y):\n\
             \treturn x + y\n\
            result = add(5, 3)",
        );

        println!(
            "Function definition bytecode:\n{}",
            PyBytecode::to_string(&code_obj.bytecode)
        );

        let mut vm = PyVM::new();
        vm.execute(code_obj);
    }

    #[test]
    #[ignore]
    fn function_with_default_args() {
        let expr =
            Expression::from_multiline("def greet(name, msg=\"Hello\"):\n\tprint(msg, name)");
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
        let mut code = PyCompileCtx::new("__test_bytecode_unary__".into());
        PyBytecode::from_expr(Expression::from_line("-42"), &mut code);

        let code_obj = code.finish();
        let expected = vec![
            PyBytecode::Resume,
            PyBytecode::LoadConst(0),
            PyBytecode::UnaryNegative,
        ];

        assert_eq!(
            PyBytecode::to_string(&code_obj.bytecode),
            PyBytecode::to_string(&expected)
        );
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

        let mut vs = PyObject::new_map();
        let mut fns = Funcs::get_std_map();

        for (expr_str, expected) in comparisons {
            let expr = Expression::from_line(expr_str);
            assert_eq!(
                expr.eval(&mut vs, &mut fns).unwrap().get_ref().to_string(),
                expected,
                "{}",
                expr.to_string()
            );
        }
    }

    #[test]
    fn ops_tuple() {
        let tuple_expr = Expression::from_line("(1, 2, 3)");
        println!("Tuple expression: {}", tuple_expr);

        let mut code = PyCompileCtx::new("__test_ops_tuple__".into());
        PyBytecode::from_expr(tuple_expr, &mut code);
        let code_obj = code.finish();
        println!(
            "Tuple bytecode:\n {}",
            PyBytecode::to_string(&code_obj.bytecode)
        );
    }

    #[test]
    fn ops_set() {
        let tuple_expr = Expression::from_line("{1, 2, 3}");
        println!("Tuple expression: {}", tuple_expr);

        let mut code = PyCompileCtx::new("__test_ops_set__".into());
        PyBytecode::from_expr(tuple_expr, &mut code);

        let code_obj = code.finish();
        println!(
            "Tuple bytecode: {}",
            PyBytecode::to_string(&code_obj.bytecode)
        );
    }

    #[test]
    fn ops_dot() {
        let expr1 = Expression::from_line("a.x");
        assert_eq!(&expr1.to_string(), "Op[. Ident(a) Ident(x)]");

        let expr2 = Expression::from_line("a.x()");
        assert_eq!(&expr2.to_string(), "Op[. Ident(a) Call[x args[]]]");
    }

    #[test]
    fn for_loop_parsing() {
        let source_code = "v = [1, 2, 3]\n\
            for i in v:\n\
                \tprint(i)";

        let for_expr = Expression::from_multiline(source_code);

        assert_eq!(for_expr.len(), 2);
        println!("For loop: {}", for_expr[1]);

        /*
                bytecode:
        (0)             LoadConst(0)
        (1)             LoadConst(1)
        (2)             LoadConst(2)
        (3)             BuildList(3)
        (4)             StoreFast(0)
        (5)             LoadName(0)
        (6)             GetIter
        (7)             ForIter(5)
        (8)             StoreFast(1)
        (9)             PushNull
        (10)            LoadFast(1)
        (11)            CallInstrinsic1(Print)
        (12)            JumpBackward(6)
        <end __temp_bytecode_ThreadId(13)_1771182827630945031__>
         */
        match &for_expr[0] {
            Expression::Operation(Op::Equals, args) => {
                assert_eq!(args[0], Expression::Ident("v".into()));
                assert_eq!(
                    args[1],
                    Expression::Operation(
                        Op::List,
                        vec![
                            Expression::Atom("1".into()),
                            Expression::Atom("2".into()),
                            Expression::Atom("3".into())
                        ]
                    )
                );
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

        let code_obj = PyBytecode::from_string(source_code);
        println!("code: \n{}", PyBytecode::to_string(&code_obj.bytecode));

        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.execute(code_obj);
    }

    #[test]
    fn nested_list() {
        let nested_list = Expression::from_line("[[1, 2], [3, 4]]");
        assert_eq!(
            nested_list.to_string(),
            "Op[list Op[list Atom(1) Atom(2)] Op[list Atom(3) Atom(4)]]"
        );

        let mut code = PyCompileCtx::new("__test_nested_list__".into());
        PyBytecode::from_expr(nested_list, &mut code);

        let code_obj = code.finish();
        // Should have multiple BuildList instructions
        let build_list_count = code_obj
            .bytecode
            .iter()
            .filter(|inst| matches!(inst, PyBytecode::BuildList(_)))
            .count();
        assert_eq!(build_list_count, 1); // Two inner lists + one outer list
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
    fn parse_precedence_complex_maths() {
        let code_obj = PyBytecode::from_string("2 + 3 * 4 - 5 / 2");
        println!("code: \n{}", PyBytecode::to_string(&code_obj.bytecode));

        let mut vm = PyVM::new();
        vm.execute(code_obj);

        // let expected = vec![vec![11.5.to_arc()]];
        // assert_eq!(stack, &expected);
    }

    #[test]
    #[ignore]
    fn variable_scoping() {
        // Test variable assignment and retrieval
        let code_obj = PyBytecode::from_string(
            "x = 10\n\
             y = x * 2\n\
            print(y)",
        );

        let mut vm = PyVM::new();
        vm.execute(code_obj);
        panic!();
    }

    #[test]
    fn intrinsic_functions() {
        // Test that intrinsic functions are properly identified
        assert!(IntrinsicFunc::try_get("print").is_some());
        assert!(IntrinsicFunc::try_get("input").is_some());
        assert!(IntrinsicFunc::try_get("nonexistent").is_none());
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
        let _build_tuple = PyBytecode::BuildTuple(3);
        let _build_map = PyBytecode::BuildMap;
        let _list_append = PyBytecode::ListAppend;
        let _for_iter = PyBytecode::ForIter;
        let _get_iter = PyBytecode::GetIter;
    }

    #[test]
    fn expression_none_handling() {
        let empty_expr = Expression::None;
        assert_eq!(empty_expr.to_string(), "None");

        let mut code = PyCompileCtx::new("__test_expression_none_handling__".into());
        PyBytecode::from_expr(empty_expr, &mut code);

        let code_obj = code.finish();
        assert_eq!(code_obj.bytecode, vec![PyBytecode::Resume]);
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
        use crate::pyrs_utils::PyUtils;

        // Test str_starts_with
        assert!(PyUtils::str_starts_with("123abc", char::is_numeric));
        assert!(!PyUtils::str_starts_with("abc123", char::is_numeric));

        // Test trim_first_and_last
        assert_eq!(PyUtils::trim_first_and_last("\"hello\""), "hello");
        assert_eq!(PyUtils::trim_first_and_last("'world'"), "world");

        // Test get_indent
        assert_eq!(PyUtils::get_indent("    hello"), 4);
        assert_eq!(PyUtils::get_indent("\thello"), 4);
        assert_eq!(PyUtils::get_indent("    \thello"), 8); // 4 spaces + 1 tab
        assert_eq!(PyUtils::get_indent("hello"), 0);
    }

    #[test]
    fn split_to_words_comprehensive() {
        // Test basic splitting
        let words = PyUtils::split_to_words("hello world");
        assert_eq!(words, vec!["hello", "world"]);

        // Test operators
        let words = PyUtils::split_to_words("x=5");
        assert_eq!(words, vec!["x", "=", "5"]);

        let words = PyUtils::split_to_words("x==y");
        assert_eq!(words, vec!["x", "==", "y"]);

        let words = PyUtils::split_to_words("x!=y");
        assert_eq!(words, vec!["x", "!=", "y"]);

        // Test string literals
        let words = PyUtils::split_to_words("print(\"hello world\")");
        assert_eq!(words, vec!["print", "(", "\"hello world\"", ")"]);

        // Test mixed content
        let words = PyUtils::split_to_words("if x >= 10:");
        assert_eq!(words, vec!["if", "x", ">=", "10", ":"]);
    }

    #[test]
    fn utils_from_bytes() {
        let bytes1: Vec<u8> = vec![1, 0, 0, 0, 0, 0, 0, 0];
        let val1 = usize::from_bytes_le(bytes1.as_slice()).unwrap();
        assert_eq!(val1, 1usize);

        let num2 = 42069u32;
        let bytes2: Vec<u8> = num2.to_le_bytes().to_vec();
        let val2 = u32::from_bytes_le(bytes2.as_slice()).unwrap();
        assert_eq!(val2, num2);
    }

    #[test]

    fn complex_if_elif_else_evaluation() {
        let code_obj = PyBytecode::from_string(
            "x = 15\n\
             if x < 10:\n\
             \tresult = \"small\"\n\
             elif x < 20:\n\
             \tresult = \"medium\"\n\
             else:\n\
             \tresult = \"large\"",
        );

        let expected = vec![
            PyBytecode::Resume,
            PyBytecode::LoadConst(0),
            PyBytecode::StoreFast(0),
            PyBytecode::LoadFast(0),
            PyBytecode::LoadConst(1),
            PyBytecode::CompareOp(Op::LessThan),
            PyBytecode::PopJumpIfFalse(3),
            PyBytecode::LoadConst(2),
            PyBytecode::StoreFast(1),
            PyBytecode::JumpForward(8),
            PyBytecode::LoadFast(0),
            PyBytecode::LoadConst(3),
            PyBytecode::CompareOp(Op::LessThan),
            PyBytecode::PopJumpIfFalse(3),
            PyBytecode::LoadConst(4),
            PyBytecode::StoreFast(1),
            PyBytecode::JumpForward(2),
            PyBytecode::LoadConst(5),
            PyBytecode::StoreFast(1),
        ];

        assert_eq!(
            PyBytecode::to_string(&code_obj.bytecode),
            PyBytecode::to_string(&expected)
        );

        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.execute(code_obj);
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
             \ti = i + 1",
        );

        // Just test that it parses correctly
        assert!(expr.len() >= 2); // At least assignment and while loop

        // Test bytecode generation doesn't crash
        let mut code = PyCompileCtx::new("__test_nested_while_loops__".into());
        for e in expr {
            PyBytecode::from_expr(e, &mut code);
        }

        let code_obj = code.finish();
        println!(
            "Nested while loops bytecode:\n{}",
            PyBytecode::to_string(&code_obj.bytecode)
        );

        let mut vm = PyVM::new();
        vm.execute(code_obj);
    }

    #[test]
    fn list_concat() {
        let list_ops = [
            ("[1, 2] + [3, 4]", "[1, 2, 3, 4]"),
            // Add more list operations as they get implemented
        ];

        for (i, (expr_str, expected)) in list_ops.iter().enumerate() {
            println!("Line: {}", expr_str);
            let exprs = Expression::from_multiline(expr_str);
            let expr = exprs.first().unwrap();
            let obj = expr.clone().to_pyobj();
            assert_eq!(&obj.__str__(), expected, "expr(#{i}) {}", expr.to_string());
        }
    }

    #[test]
    fn iteration() {
        let list = vec![1.to_pyptr(), 2.to_pyptr()].to_pyptr();
        for x in list {
            println!("{}", *x.get_ref());
        }

        let list = vec![1.to_pyptr(), 2.to_pyptr()].to_pyptr();
        for mut x in &mut list.into_iter() {
            x = PyObject::__add__(&x, &2.to_pyptr()).handle_panic();
            println!("{}", *x.get_ref());
        }
    }

    #[test]
    fn parse_pratt_tests() {
        let s = Expression::from_line("1");
        assert_eq!(s.to_string(), "Atom(1)");

        let s = Expression::from_line("1 + 2 * 3");
        assert_eq!(s.to_string(), "Op[+ Atom(1) Op[* Atom(2) Atom(3)]]");

        let s = Expression::from_line("a + b * c * d + e");
        assert_eq!(
            s.to_string(),
            "Op[+ Op[+ Ident(a) Op[* Op[* Ident(b) Ident(c)] Ident(d)]] Ident(e)]"
        );

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

    #[test]
    fn serialize_header() {
        let header = PyHeader {
            name: "__test_serialize_header__".into(),
            time: PyUtils::curr_time(),
            version: PyRsVersion::get(),
            internal_filename: PyUtils::curr_dir(),
        };

        let bytes = header.seralize();

        let _r = std::fs::write("tests/hex.lib", bytes.clone());

        let deserial = PyHeader::deserialize(&bytes);
        assert_eq!(header, deserial);
    }

    #[test]
    fn serialize_bytecode_map() {
        let c = PyCodeObj::new(
            "__test_serialize_bytecode_map__",
            vec![
                PyBytecode::LoadConst(0),
                PyBytecode::LoadConst(1),
                PyBytecode::BinaryAdd,
            ],
        );
        let bytes = PySerializer::seralize_codeobj(&c);

        let _ = std::fs::write("tests/hex.lib", bytes.clone());
        let deserial = PySerializer::deserialize_codeobj(bytes);

        println!("{:#?}", deserial);
    }

    #[test]
    fn attr_get() {
        let mut c = PyCompileCtx::new("__test_calling__".into());
        let a = Expression::from_line("v = 1");
        dbg!(&a);
        PyBytecode::from_expr(a, &mut c);
        let b = Expression::from_line("v.x");
        dbg!(&b);
        PyBytecode::from_expr(b, &mut c);

        let codeobj = c.finish();
        dbg!(&codeobj);

        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.execute(codeobj);
    }

    #[test]
    fn classobj() {
        let code_obj =
            PyBytecode::from_string("class vec2:\n\tx=0\nv = vec2\nv.x = 10\ny = vec2\nprint(y.x)");
        dbg!(&code_obj);

        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.execute(code_obj);
        panic!();
    }

    #[test]
    fn calling_simple() {
        let code_obj =
            PyBytecode::from_string("class vec2:\n\tx = 0\nx = vec2\nx.x = 1\nprint(x.x)");
        dbg!(&code_obj);

        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.execute(code_obj);
    }

    #[test]
    fn calling_medium() {
        let code_obj = Interpreter::compile_file("tests/class_test_1.py").unwrap();
        let mut vm = PyVM::new();
        vm.set_debug_mode(true);
        vm.execute(code_obj);

        //panic!();
    }

    #[test]
    fn pretty_print() {
        let code_obj = PyCodeObj {
            name: "__test_bytecode_manual__".into(),
            consts: vec![5.to_pyptr()],
            varnames: vec!["x".into()],
            names: vec!["print".into()],
            bytecode: vec![],
            num_varnames: 1,
            num_consts: 1,
            num_names: 1,
            globals: AttrDict::new(),
        };
        code_obj.pretty_format();
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
