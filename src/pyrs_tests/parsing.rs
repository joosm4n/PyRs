#[allow(unused_imports)]
mod tests {

    use crate::pyrs_parser2::*;
    use crate::pyrs_utils::*;
    use std::sync::Arc;

    #[test]
    fn token_list() {
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

        let file_data = FileData::new(
            "/this/is/the/file-path".into(),
            "da-test-file".into(),
            contents,
        );
        let tokens = Parser::parse(&file_data);

        let raw_lines: Vec<_> = file_data.get_contents().lines().collect();
        let trimmed_lines: Vec<_> = raw_lines.iter().map(|s| s.trim()).collect();

        // println!("raw:{raw_lines:?}\n");
        // println!("trimmed:{trimmed_lines:?}");
        for t in tokens {
            println!("{t:?}");
        }
        println!("{}", file_data.get_contents_fmt());
        println!("{}", file_data.get_line_fmt(4, true).unwrap_or_default());

        panic!("This should fail, so i can see output.");
    }
}
