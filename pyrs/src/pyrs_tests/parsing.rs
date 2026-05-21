#![allow(unused_imports, unreachable_code)]

macro_rules! __func_name__ {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }};
}

use crate::{pyrs_parser2::*, pyrs_tokentypes::*, pyrs_utils::*};
use pretty_assertions::assert_eq;
use std::sync::Arc;

#[test]
fn parsing_core() -> Result<(), DynError> {
    let contents = String::from(
        r#"i = 0
n1 = 0
n2 = 1
n3 = 0
big = (n1 != 1)
v = big.x + \
    2
print("Fibbonacci: ")
while i < 20:
    n3 = n1 + n2
    print("(", i, ") ", n3)
    n1 = n2
    n2 = n3
    i = i + 1 # test comment

    if x:
        print(7)
"#,
    );

    let fd = Arc::new(FileData::new(
        "/this/is/the/file-path".into(),
        "da-test-file".into(),
        contents,
    ));
    let tokens = Parser::parse(fd.get_contents(), fd.clone())?;

    let expected_tokens = vec![
        Token::basic("i", &fd, TokenKind::Name),
        Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
        Token::basic("0", &fd, TokenKind::Number(NumLit::Dec)),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("n1", &fd, TokenKind::Name),
        Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
        Token::basic("0", &fd, TokenKind::Number(NumLit::Dec)),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("n2", &fd, TokenKind::Name),
        Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
        Token::basic("1", &fd, TokenKind::Number(NumLit::Dec)),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("n3", &fd, TokenKind::Name),
        Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
        Token::basic("0", &fd, TokenKind::Number(NumLit::Dec)),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("big", &fd, TokenKind::Name),
        Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
        Token::basic("(", &fd, TokenKind::Op(Op::LPAR_)),
        Token::basic("n1", &fd, TokenKind::Name),
        Token::basic("!=", &fd, TokenKind::Op(Op::NOTEQUAL_)),
        Token::basic("1", &fd, TokenKind::Number(NumLit::Dec)),
        Token::basic(")", &fd, TokenKind::Op(Op::RPAR_)),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("v", &fd, TokenKind::Name),
        Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
        Token::basic("big", &fd, TokenKind::Name),
        Token::basic(".", &fd, TokenKind::Op(Op::DOT_)),
        Token::basic("x", &fd, TokenKind::Name),
        Token::basic("+", &fd, TokenKind::Op(Op::PLUS_)),
        Token::basic("\\", &fd, TokenKind::NL),
        Token::basic("    ", &fd, TokenKind::Indent),
        Token::basic("2", &fd, TokenKind::Number(NumLit::Dec)),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("print", &fd, TokenKind::Name),
        Token::basic("(", &fd, TokenKind::Op(Op::LPAR_)),
        Token::basic("\"Fibbonacci: \"", &fd, TokenKind::String),
        Token::basic(")", &fd, TokenKind::Op(Op::RPAR_)),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("while", &fd, TokenKind::Name),
        Token::basic("i", &fd, TokenKind::Name),
        Token::basic("<", &fd, TokenKind::Op(Op::LESS_)),
        Token::basic("20", &fd, TokenKind::Number(NumLit::Dec)),
        Token::basic(":", &fd, TokenKind::Op(Op::COLON_)),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("    ", &fd, TokenKind::Indent),
        Token::basic("n3", &fd, TokenKind::Name),
        Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
        Token::basic("n1", &fd, TokenKind::Name),
        Token::basic("+", &fd, TokenKind::Op(Op::PLUS_)),
        Token::basic("n2", &fd, TokenKind::Name),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("    ", &fd, TokenKind::Indent),
        Token::basic("print", &fd, TokenKind::Name),
        Token::basic("(", &fd, TokenKind::Op(Op::LPAR_)),
        Token::basic("\"(\"", &fd, TokenKind::String),
        Token::basic(",", &fd, TokenKind::Op(Op::COMMA_)),
        Token::basic("i", &fd, TokenKind::Name),
        Token::basic(",", &fd, TokenKind::Op(Op::COMMA_)),
        Token::basic("\") \"", &fd, TokenKind::String),
        Token::basic(",", &fd, TokenKind::Op(Op::COMMA_)),
        Token::basic("n3", &fd, TokenKind::Name),
        Token::basic(")", &fd, TokenKind::Op(Op::RPAR_)),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("    ", &fd, TokenKind::Indent),
        Token::basic("n1", &fd, TokenKind::Name),
        Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
        Token::basic("n2", &fd, TokenKind::Name),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("    ", &fd, TokenKind::Indent),
        Token::basic("n2", &fd, TokenKind::Name),
        Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
        Token::basic("n3", &fd, TokenKind::Name),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("    ", &fd, TokenKind::Indent),
        Token::basic("i", &fd, TokenKind::Name),
        Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
        Token::basic("i", &fd, TokenKind::Name),
        Token::basic("+", &fd, TokenKind::Op(Op::PLUS_)),
        Token::basic("1", &fd, TokenKind::Number(NumLit::Dec)),
        Token::basic("# test comment", &fd, TokenKind::Commment),
        Token::basic("# test comment", &fd, TokenKind::NewLine),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("    ", &fd, TokenKind::Indent),
        Token::basic("if", &fd, TokenKind::Name),
        Token::basic("x", &fd, TokenKind::Name),
        Token::basic("{", &fd, TokenKind::Op(Op::LBRACE_)),
        Token::basic("print", &fd, TokenKind::Name),
        Token::basic("(", &fd, TokenKind::Op(Op::LPAR_)),
        Token::basic("7", &fd, TokenKind::Number(NumLit::Dec)),
        Token::basic(")", &fd, TokenKind::Op(Op::RPAR_)),
        Token::basic("}", &fd, TokenKind::Op(Op::RBRACE_)),
        Token::basic(";", &fd, TokenKind::Op(Op::SEMI_)),
        Token::basic("\n", &fd, TokenKind::NewLine),
        Token::basic("\n", &fd, TokenKind::EndMarker),
    ];
    assert_eq!(tokens, expected_tokens);

    for t in &tokens {
        println!("{t:?}");
    }
    println!("\n{}", fd.get_contents_fmt());
    println!("\n{}", fd.get_line_fmt(4, true).unwrap_or_default());
    println!("\n{}\n", tokens.get(3).as_ref().unwrap().get_line_fmt());
    println!("\n{}\n", tokens.last().as_ref().unwrap().get_line_fmt());

    // panic!("--- PYRS: INTENTIONAL_FAIL {} ---", __function__!());
    Ok(())
}

#[test]
fn parsing_basic_tokens() -> Result<(), DynError> {
    let fd = Arc::new(FileData::new("NOFILE".into(), "NOFILE".into(), "".into()));
    assert_eq!(
        Parser::_parse_test("x")?,
        vec![Token::basic("x", &fd, TokenKind::Name)]
    );
    assert_eq!(
        Parser::_parse_test("{")?,
        vec![Token::basic("{", &fd, TokenKind::Op(Op::LBRACE_))]
    );
    assert_eq!(
        Parser::_parse_test("1")?,
        vec![Token::basic("1", &fd, TokenKind::Number(NumLit::Dec))]
    );
    assert_eq!(
        Parser::_parse_test("x = 2")?,
        vec![
            Token::basic("x", &fd, TokenKind::Name),
            Token::basic("=", &fd, TokenKind::Op(Op::EQUAL_)),
            Token::basic("2", &fd, TokenKind::Number(NumLit::Dec)),
        ]
    );
    Ok(())
}

#[test]
fn parsing_number_literals() -> Result<(), DynError> {
    let fd = Arc::new(FileData::new("NOFILE".into(), "NOFILE".into(), "".into()));

    assert_eq!(
        Parser::_parse_test("1 1_0 1.0 10.0_0 0b1 0xa 0o7 0_0:")?,
        vec![
            Token::basic("1", &fd, TokenKind::Number(NumLit::Dec)),
            Token::basic("1_0", &fd, TokenKind::Number(NumLit::Dec)),
            Token::basic("1.0", &fd, TokenKind::Number(NumLit::Dec)),
            Token::basic("10.0_0", &fd, TokenKind::Number(NumLit::Dec)),
            Token::basic("0b1", &fd, TokenKind::Number(NumLit::Bin)),
            Token::basic("0xa", &fd, TokenKind::Number(NumLit::Hex)),
            Token::basic("0o7", &fd, TokenKind::Number(NumLit::Oct)),
            Token::basic("0_0", &fd, TokenKind::Number(NumLit::Zero)),
            Token::basic(":", &fd, TokenKind::Op(Op::COLON_)),
        ]
    );

    assert_eq!(
        Parser::_parse_test("0_x")
            .unwrap_err()
            .downcast_ref::<ParserError>()
            .unwrap(),
        &ParserError::empty(),
    );
    assert_eq!(
        Parser::_parse_test("0_02")
            .unwrap_err()
            .downcast_ref::<ParserError>()
            .unwrap(),
        &ParserError::empty(),
    );

    Ok(())
}
