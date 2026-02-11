pub mod env;
pub mod eval;
pub mod obj;
pub mod builtins;
pub mod module_registry;
pub mod helpers;

#[cfg(test)]
mod tests {
    use crate::ast::ast::Program;
    use crate::lexer::lexer::Lexer;
    use crate::lexer::token::Tokens;
    use crate::parser::parser::Parser;
    use crate::interpreter::eval::Evaluator;
    use crate::interpreter::obj::Object;

    fn parse_test_helper(input: &str) -> Program {
        let (remaining, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(remaining.len(), 0, "Lexer did not consume all input");
        let tokens_wrapper = Tokens::new(&tokens);
        let result = Parser::parse_tokens(tokens_wrapper);
        assert!(result.is_ok(), "Parser returned an error: {:?}", result.err());
        let (remaining_tokens, program) = result.unwrap();
        assert_eq!(remaining_tokens.token.len(), 0, "Parser did not consume all tokens including EOF");
        program
    }

    fn eval_test_helper(input: &str) -> Object {
        let program = parse_test_helper(input);
        let mut evaluator = Evaluator::default();
        evaluator.eval_program(program)
    }

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![
            ("5", 5),
            ("10", 10),
            ("-5", -5),
            ("-10", -10),
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("3 * (3 * 3) + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];

        for (input, expected) in tests {
            let evaluated = eval_test_helper(input);
            match evaluated {
                Object::Integer(i) => assert_eq!(i, expected, "input: {}", input),
                _ => panic!("Expected Integer, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = vec![
            ("true", true),
            ("false", false),
            ("1 < 2", true),
            ("1 > 2", false),
            ("1 < 1", false),
            ("1 > 1", false),
            ("1 == 1", true),
            ("1 != 1", false),
            ("1 == 2", false),
            ("1 != 2", true),
            ("true == true", true),
            ("false == false", true),
            ("true == false", false),
            ("true != false", true),
            ("false != true", true),
            ("(1 < 2) == true", true),
            ("(1 < 2) == false", false),
            ("(1 > 2) == true", false),
            ("(1 > 2) == false", true),
        ];

        for (input, expected) in tests {
            let evaluated = eval_test_helper(input);
            match evaluated {
                Object::Boolean(b) => assert_eq!(b, expected, "input: {}", input),
                _ => panic!("Expected Boolean, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = vec![
            ("!true", false),
            ("!false", true),
            ("!!true", true),
            ("!!false", false),
        ];

        for (input, expected) in tests {
            let evaluated = eval_test_helper(input);
            match evaluated {
                Object::Boolean(b) => assert_eq!(b, expected, "input: {}", input),
                _ => panic!("Expected Boolean, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[test]
    fn test_if_else_expressions() {
        let tests = vec![
            ("if (true) { 10 }", Object::Integer(10)),
            ("if (false) { 10 }", Object::Null),
            ("if (1 == 1) { 10 }", Object::Integer(10)),
            ("if (1 < 2) { 10 }", Object::Integer(10)),
            ("if (1 > 2) { 10 }", Object::Null),
            ("if (1 > 2) { 10 } else { 20 }", Object::Integer(20)),
            ("if (1 < 2) { 10 } else { 20 }", Object::Integer(10)),
        ];

        for (input, expected) in tests {
            let evaluated = eval_test_helper(input);
            assert_eq!(evaluated, expected, "input: {}", input);
        }
    }

    #[test]
    fn test_return_statements() {
        let tests = vec![
            ("return 10;", 10),
            ("return 10; 9;", 10),
            ("return 2 * 5; 9;", 10),
            ("9; return 2 * 5; 9;", 10),
            (
                "if (10 > 1) {
                    if (10 > 1) {
                        return 10;
                    }
                    return 1;
                }",
                10,
            ),
        ];

        for (input, expected) in tests {
            let evaluated = eval_test_helper(input);
            match evaluated {
                Object::Integer(i) => assert_eq!(i, expected, "input: {}", input),
                _ => panic!("Expected Integer, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = vec![
            ("let a = 5; a", 5),
            ("let a = 5 * 5; a", 25),
            ("let a = 5; let b = a; b", 5),
            ("let a = 5; let b = a; let c = a + b + 5; c", 15),
        ];

        for (input, expected) in tests {
            let evaluated = eval_test_helper(input);
            match evaluated {
                Object::Integer(i) => assert_eq!(i, expected, "input: {}", input),
                _ => panic!("Expected Integer, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[test]
    fn test_function_application() {
        let tests = vec![
            ("let identity = fn(x) { x }; identity(5)", 5),
            ("let identity = fn(x) { return x; }; identity(5)", 5),
            ("let double = fn(x) { x * 2 }; double(5)", 10),
            ("let add = fn(x, y) { x + y }; add(5, 5)", 10),
            ("let add = fn(x, y) { x + y }; add(5 + 5, add(5, 5))", 20),
            ("fn(x) { x }(5)", 5),
        ];

        for (input, expected) in tests {
            let evaluated = eval_test_helper(input);
            match evaluated {
                Object::Integer(i) => assert_eq!(i, expected, "input: {}", input),
                _ => panic!("Expected Integer, got {:?} for input: {}", evaluated, input),
            }
        }
    }
}