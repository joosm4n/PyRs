#[allow(unused_imports, unreachable_code)]
#[cfg(test)]
mod tests {

    use crate::{pyrs_parser2::*, pyrs_tokentypes::*, pyrs_utils::*};
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    #[test]
    fn item_list() -> Result<(), DynError> {
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

    if x { print(7) };
"#,
        );

        let file_data = Arc::new(FileData::new(
            "/this/is/the/file-path".into(),
            "da-test-file".into(),
            contents,
        ));
        let tokens = Parser::parse(file_data.get_contents(), file_data.clone())?;

        for t in &tokens {
            println!("{t:?}");
        }
        println!("\n{}", file_data.get_contents_fmt());
        println!("\n{}", file_data.get_line_fmt(4, true).unwrap_or_default());
        println!("\n{}\n", tokens.get(3).as_ref().unwrap().get_line_fmt());
        println!("\n{}\n", tokens.last().as_ref().unwrap().get_line_fmt());

        panic!("This should fail, so i can see output.");
        Ok(())
    }

    #[test]
    fn basic_tokens() -> Result<(), DynError> {
        let fd = Arc::new(FileData::new("NOFILE".into(), "NOFILE".into(), "".into()));
        assert_eq!(
            Parser::_parse_test("x", &fd),
            vec![Token::basic("x", &fd, TokenKind::Name)]
        );
        assert_eq!(
            Parser::_parse_test("{", &fd),
            vec![Token::basic("{", &fd, TokenKind::Op(Op::LBRACE))]
        );
        assert_eq!(
            Parser::_parse_test("1", &fd),
            vec![Token::basic("1", &fd, TokenKind::Number)]
        );
        assert_eq!(
            Parser::_parse_test("x = 2", &fd),
            vec![
                Token::basic("x", &fd, TokenKind::Name),
                Token::basic("=", &fd, TokenKind::Op(Op::EQUAL)),
                Token::basic("2", &fd, TokenKind::Number),
            ]
        );
        Ok(())
    }

    // TODO: Finish
    #[test]
    fn parsing_number_literals() -> Result<(), DynError> {
        let fd = Arc::new(FileData::new("NOFILE".into(), "NOFILE".into(), "".into()));
        assert_eq!(
            Parser::_parse_test("1", &fd),
            vec![Token::basic("1", &fd, TokenKind::Number)]
        );
        assert_eq!(
            Parser::_parse_test("1_0", &fd),
            vec![Token::basic("1_0", &fd, TokenKind::Number)]
        );
        assert_eq!(
            Parser::_parse_test("1.0", &fd),
            vec![Token::basic("1.0", &fd, TokenKind::Number)]
        );
        assert_eq!(
            Parser::_parse_test("10.0_0", &fd),
            vec![Token::basic("10.0_0", &fd, TokenKind::Number),]
        );
        assert_eq!(
            Parser::_parse_test("0b1", &fd),
            vec![Token::basic("0b1", &fd, TokenKind::Number)]
        );
        assert_eq!(
            Parser::_parse_test("0xa", &fd),
            vec![Token::basic("0xa", &fd, TokenKind::Number)]
        );
        assert_eq!(
            Parser::_parse_test("0o7", &fd),
            vec![Token::basic("0o7", &fd, TokenKind::Number)]
        );
        assert_eq!(
            Parser::_parse_test("0_0", &fd),
            vec![Token::basic("0_0", &fd, TokenKind::Number),]
        );

        Ok(())
    }
}
