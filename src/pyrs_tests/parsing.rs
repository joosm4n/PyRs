#[allow(unused_imports)]
#[cfg(test)]
mod tests {

    use crate::pyrs_parser2::*;
    use crate::pyrs_utils::*;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    #[test]
    fn item_list() {
        let contents = String::from(
            r#"i = 0
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

    if x { print(7) };
"#,
        );

        let file_data = Arc::new(FileData::new(
            "/this/is/the/file-path".into(),
            "da-test-file".into(),
            contents,
        ));
        let tokens = Parser::parse(file_data.get_contents(), file_data.clone());

        for t in tokens {
            println!("{t:?}");
        }
        println!("{}", file_data.get_contents_fmt());
        println!("{}", file_data.get_line_fmt(4, true).unwrap_or_default());

        panic!("This should fail, so i can see output.");
    }

    #[test]
    fn parse_items() {
        let fd = Arc::new(FileData::new("NOFILE".into(), "NOFILE".into(), "".into()));
        assert_eq!(
            Parser::_parse_test("x", fd.clone()),
            vec![Item::basic("x", fd.clone(), ItemKind::Ident)]
        );
        assert_eq!(
            Parser::_parse_test("{", fd.clone()),
            vec![Item::basic("{", fd.clone(), ItemKind::Operator)]
        );
        assert_eq!(
            Parser::_parse_test("1", fd.clone()),
            vec![Item::basic("1", fd.clone(), ItemKind::NumLiteral)]
        );
        assert_eq!(
            Parser::_parse_test("x = 2", fd.clone()),
            vec![
                Item::basic("x", fd.clone(), ItemKind::Ident),
                Item::basic("=", fd.clone(), ItemKind::Operator),
                Item::basic("2", fd.clone(), ItemKind::NumLiteral),
            ]
        );
    }

    #[test]
    fn lexing() {}
}
