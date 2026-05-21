:::mermaid
graph TB
    A["file.py ' x = 2 ... '" ]  -- fn split_to_lines() 
    --> B[Lines] -- fn split_to_words()
    --> C[Words] -- struct Lexer
    --> D[Tokens]  -- fn parse_expression()
    --> E[Expressions] -- fn PyBytecode::from_expr()
    --> F[PyBytecode]
:::