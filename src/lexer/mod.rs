pub mod lexer;
pub mod token;

#[cfg(test)]
mod tests {
    use super::lexer::Lexer;
    use super::token::Token;
    use num_bigint::BigInt;

    #[test]
    fn test_simple_tokens() {
        let input = "=+(){},;";
        let expected_tokens = vec![
            Token::Assign,
            Token::Plus,
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::RBrace,
            Token::Comma,
            Token::SemiColon,
            Token::EOF,
        ];

        let (_, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_let_statement() {
        let input = "let five = 5;";
        let expected_tokens = vec![
            Token::Let,
            Token::Ident("five".to_string()),
            Token::Assign,
            Token::IntLiteral(5),
            Token::SemiColon,
            Token::EOF,
        ];

        let (_, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_function_declaration() {
        let input = "fn add(x, y) { x + y; }";
        let expected_tokens = vec![
            Token::Function,
            Token::Ident("add".to_string()),
            Token::LParen,
            Token::Ident("x".to_string()),
            Token::Comma,
            Token::Ident("y".to_string()),
            Token::RParen,
            Token::LBrace,
            Token::Ident("x".to_string()),
            Token::Plus,
            Token::Ident("y".to_string()),
            Token::SemiColon,
            Token::RBrace,
            Token::EOF,
        ];

        let (_, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_operators() {
        let input = "+ - * / % == != > < >= <= && || !";
        let expected_tokens = vec![
            Token::Plus,
            Token::Minus,
            Token::Multiply,
            Token::Divide,
            Token::Modulo,
            Token::Equal,
            Token::NotEqual,
            Token::GreaterThan,
            Token::LessThan,
            Token::GreaterThanEqual,
            Token::LessThanEqual,
            Token::And,
            Token::Or,
            Token::Not,
            Token::EOF,
        ];

        let (_, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_literals() {
        let input = r#"
            123;
            123.45;
            "hello world";
            "hello\nworld";
            true;
            false;
            null;
            9999999999999999999; // BigInt
        "#;
        let expected_tokens = vec![
            Token::IntLiteral(123),
            Token::SemiColon,
            Token::FloatLiteral(123.45),
            Token::SemiColon,
            Token::StringLiteral("hello world".to_string()),
            Token::SemiColon,
            Token::StringLiteral("hello\nworld".to_string()),
            Token::SemiColon,
            Token::BoolLiteral(true),
            Token::SemiColon,
            Token::BoolLiteral(false),
            Token::SemiColon,
            Token::NullLiteral,
            Token::SemiColon,
            Token::BigIntLiteral(BigInt::parse_bytes(b"9999999999999999999", 10).unwrap()),
            Token::SemiColon,
            Token::EOF,
        ];

        let (_, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_if_else_statement() {
        let input = "if (x < y) { return true; } else { return false; }";
        let expected_tokens = vec![
            Token::If,
            Token::LParen,
            Token::Ident("x".to_string()),
            Token::LessThan,
            Token::Ident("y".to_string()),
            Token::RParen,
            Token::LBrace,
            Token::Return,
            Token::BoolLiteral(true),
            Token::SemiColon,
            Token::RBrace,
            Token::Else,
            Token::LBrace,
            Token::Return,
            Token::BoolLiteral(false),
            Token::SemiColon,
            Token::RBrace,
            Token::EOF,
        ];

        let (_, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_string_with_escapes() {
        let input = r#""hello \"world\" \\ \n \r \t""#;
        let expected_tokens = vec![
            Token::StringLiteral("hello \"world\" \\ \n \r \t".to_string()),
            Token::EOF,
        ];
        let (_, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_unterminated_string() {
        let input = "\"hello";
        assert!(Lexer::lex_tokens(input.as_bytes()).is_err());
    }

    #[test]
    fn test_comments() {
        let input = r#"
            let x = 5; // this is a comment
            // another comment
            let y = 10;
        "#;
        let expected_tokens = vec![
            Token::Let,
            Token::Ident("x".to_string()),
            Token::Assign,
            Token::IntLiteral(5),
            Token::SemiColon,
            Token::Let,
            Token::Ident("y".to_string()),
            Token::Assign,
            Token::IntLiteral(10),
            Token::SemiColon,
            Token::EOF,
        ];

        let (_, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(tokens, expected_tokens);
    }
}