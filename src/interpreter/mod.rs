pub mod env;
pub mod eval;
pub mod obj;
pub mod builtins;
pub mod module_registry;
pub mod helpers;
pub mod eval_context;

#[cfg(test)]
mod tests {
    use nom::error::ErrorKind;
    use crate::ast::ast::Program;
    use crate::lexer::lexer::Lexer;
    use crate::lexer::token::Tokens;
    use crate::parser::parser::Parser;
    use crate::interpreter::eval::Evaluator;
    use crate::interpreter::obj::Object;

    pub fn parse_test_helper(input: &str) -> Program {
        let (remaining, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(remaining.len(), 0, "Lexer did not consume all input");
        let tokens_wrapper = Tokens::new(&tokens);
        let result = Parser::parse_tokens(tokens_wrapper);
        assert!(result.is_ok(), "Parser returned an error: {:?}", result.err());
        let (remaining_tokens, program) = result.unwrap();
        assert_eq!(remaining_tokens.token.len(), 0, "Parser did not consume all tokens including EOF");
        program
    }

    // New helper to parse programs that are expected to fail during parsing
    pub fn parse_program_expect_error(input: &str) -> nom::error::ErrorKind {
        let (remaining, tokens) = Lexer::lex_tokens(input.as_bytes()).unwrap();
        assert_eq!(remaining.len(), 0, "Lexer did not consume all input");
        let tokens_wrapper = Tokens::new(&tokens);
        let result = Parser::parse_tokens(tokens_wrapper);
        assert!(result.is_err(), "Parser was expected to return an error, but returned OK: {:?}", result.ok().unwrap());
        if let nom::Err::Failure(e) | nom::Err::Error(e) = result.err().unwrap() {
            e.code
        } else {
            panic!("Expected a Failure or Error from parser, but got Incomplete");
        }
    }

    pub async fn eval_test_helper(input: &str) -> Object {
        let program = parse_test_helper(input);
        let evaluator = Evaluator::default();
        evaluator.eval_program(program).await
    }

    #[tokio::test]
    async fn test_eval_integer_expression() {
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
            let evaluated = eval_test_helper(input).await;
            match evaluated {
                Object::Integer(i) => assert_eq!(i, expected, "input: {}", input),
                _ => panic!("Expected Integer, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[tokio::test]
    async fn test_eval_boolean_expression() {
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
            let evaluated = eval_test_helper(input).await;
            match evaluated {
                Object::Boolean(b) => assert_eq!(b, expected, "input: {}", input),
                _ => panic!("Expected Boolean, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[tokio::test]
    async fn test_bang_operator() {
        let tests = vec![
            ("!true", false),
            ("!false", true),
            ("!!true", true),
            ("!!false", false),
        ];

        for (input, expected) in tests {
            let evaluated = eval_test_helper(input).await;
            match evaluated {
                Object::Boolean(b) => assert_eq!(b, expected, "input: {}", input),
                _ => panic!("Expected Boolean, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[tokio::test]
    async fn test_if_else_expressions() {
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
            let evaluated = eval_test_helper(input).await;
            assert_eq!(evaluated, expected, "input: {}", input);
        }
    }

    #[tokio::test]
    async fn test_return_statements() {
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
            let evaluated = eval_test_helper(input).await;
            match evaluated {
                Object::Integer(i) => assert_eq!(i, expected, "input: {}", input),
                _ => panic!("Expected Integer, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[tokio::test]
    async fn test_let_statements() {
        let tests = vec![
            ("let a = 5; a", 5),
            ("let a = 5 * 5; a", 25),
            ("let a = 5; let b = a; b", 5),
            ("let a = 5; let b = a; let c = a + b + 5; c", 15),
        ];

        for (input, expected) in tests {
            let evaluated = eval_test_helper(input).await;
            match evaluated {
                Object::Integer(i) => assert_eq!(i, expected, "input: {}", input),
                _ => panic!("Expected Integer, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[tokio::test]
    async fn test_function_application() {
        let tests = vec![
            ("let identity = fn(x) { x }; identity(5)", 5),
            ("let identity = fn(x) { return x; }; identity(5)", 5),
            ("let double = fn(x) { x * 2 }; double(5)", 10),
            ("let add = fn(x, y) { x + y }; add(5, 5)", 10),
            ("let add = fn(x, y) { x + y }; add(5 + 5, add(5, 5))", 20),
            ("fn(x) { x }(5)", 5),
        ];

        for (input, expected) in tests {
            let evaluated = eval_test_helper(input).await;
            match evaluated {
                Object::Integer(i) => assert_eq!(i, expected, "input: {}", input),
                _ => panic!("Expected Integer, got {:?} for input: {}", evaluated, input),
            }
        }
    }

    #[tokio::test]
    async fn test_async_function_basic() {
        let input = r#"
            async fn main() {
                async fn async_identity(x) {
                    return x;
                }
                return await async_identity(10);
            }
            main();
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(10));
    }

    #[tokio::test]
    async fn test_await_expressions() {
        let input = r#"
            async fn main() {
                async fn add_one(x) {
                    return x + 1;
                }
                let res = await add_one(5);
                return await add_one(res);
            }
            main();
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(7));
    }

    #[tokio::test]
    async fn test_chained_async_calls() {
        let input = r#"
            async fn main() {
                async fn add_one(x) {
                    return x + 1;
                }
                async fn add_two(x) {
                    let one_added = await add_one(x);
                    return await add_one(one_added);
                }
                return await add_two(5);
            }
            main();
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(7));
    }

    #[tokio::test]
    async fn test_async_function_return_types() {
        let input_int = r#"
            async fn main() {
                async fn get_int() {
                    return 123;
                }
                return await get_int();
            }
            main();
        "#;
        assert_eq!(eval_test_helper(input_int).await, Object::Integer(123));

        let input_bool = r#"
            async fn main() {
                async fn get_bool() {
                    return true;
                }
                return await get_bool();
            }
            main();
        "#;
        assert_eq!(eval_test_helper(input_bool).await, Object::Boolean(true));

        let input_string = r#"
            async fn main() {
                async fn get_string() {
                    return "hello";
                }
                return await get_string();
            }
            main();
        "#;
        assert_eq!(eval_test_helper(input_string).await, Object::String("hello".to_string()));
    }
    #[tokio::test]
    async fn test_try_catch_no_throw() {
        let input = r#"
            let x = 0;
            try {
                x = 1;
            } catch (e) {
                x = 2;
            } finally {
                x = x + 1;
            }
            x
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(2));
    }

    #[tokio::test]
    async fn test_try_throw_catch() {
        let input = r#"
            let x = 0;
            try {
                throw "Error!";
                x = 1;
            } catch (e) {
                x = 2;
            } finally {
                x = x + 1;
            }
            x
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(3));
    }

    #[tokio::test]
    async fn test_try_throw_catch_with_exception_ident() {
        let input = r#"
            let err_msg = "";
            try {
                throw "Something went wrong";
            } catch (e) {
                err_msg = e;
            }
            err_msg
        "#;
        assert_eq!(eval_test_helper(input).await, Object::String("Something went wrong".to_string()));
    }

    #[tokio::test]
    async fn test_try_throw_no_catch_finally() {
        let input = r#"
            let x = 0;
            try {
                throw "Error!";
                x = 1;
            } finally {
                x = x + 1;
            }
            x
        "#;
        // The finally block should execute, and then the error should be re-thrown
        match eval_test_helper(input).await {
            Object::ThrownValue(obj) => assert_eq!(*obj, Object::String("Error!".to_string())),
            _ => panic!("Expected a ThrownValue"),
        }
    }

    #[tokio::test]
    async fn test_finally_overrides_return() {
        let input = r#"
            fn test_fn() {
                try {
                    return 1;
                } finally {
                    return 2;
                }
            }
            test_fn()
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(2));
    }

    #[tokio::test]
    async fn test_finally_overrides_thrown_value() {
        let input = r#"
            try {
                throw "Error from try";
            } finally {
                throw "Error from finally";
            }
        "#;
        match eval_test_helper(input).await {
            Object::ThrownValue(obj) => assert_eq!(*obj, Object::String("Error from finally".to_string())),
            _ => panic!("Expected ThrownValue"),
        }
    }

    #[tokio::test]
    async fn test_finally_not_overriding_thrown_value_if_not_thrown() {
        let input = r#"
            let result = try {
                1 + 1 // Removed semicolon
            } catch (e) {
                "Caught: " + e // Removed semicolon
            } finally {
                // This finally block does not throw or return
            };
            result
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(2));
    }

    #[tokio::test]
    async fn test_nested_try_catch() {
        let input = r#"
            let outer_status = "";
            try {
                try {
                    throw "Inner Error";
                } catch (e) {
                    outer_status = "Inner caught: " + e;
                } finally {
                    outer_status = outer_status + " (inner finally)";
                }
            } catch (e) {
                outer_status = outer_status + "Outer caught: " + e;
            } finally {
                outer_status = outer_status + " (outer finally)";
            }
            outer_status
        "#;
        assert_eq!(eval_test_helper(input).await, Object::String("Inner caught: Inner Error (inner finally) (outer finally)".to_string()));
    }

    #[tokio::test]
    async fn test_try_result_is_last_expression() {
        let input = r#"
            let result = try {
                1 + 1 // Removed semicolon
            } catch (e) {
                0
            };
            result
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(2));
    }

    #[tokio::test]
    async fn test_catch_result_is_last_expression() {
        let input = r#"
            let result = try {
                throw 1;
            } catch (e) {
                e + 1 // Removed semicolon
            };
            result
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(2));
    }

    #[tokio::test]
    async fn test_error_type_propagation() {
        let input = r#"
            let err = try {
                throw true;
            } catch (e) {
                e // Removed semicolon
            };
            err
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Boolean(true));

        let input_str = r#"
            let err = try {
                throw "some string";
            } catch (e) {
                e // Removed semicolon
            };
            err
        "#;
        assert_eq!(eval_test_helper(input_str).await, Object::String("some string".to_string()));

        let input_int = r#"
            let err = try {
                throw 123;
            } catch (e) {
                e // Removed semicolon
            };
            err
        "#;
        assert_eq!(eval_test_helper(input_int).await, Object::Integer(123));
    }

    #[tokio::test]
    async fn test_if_no_catch_or_finally_is_present() {
        let input = r#"
            let res = try { 1 }
            res
        "#;
        let err = parse_program_expect_error(input);
        assert_eq!(err, ErrorKind::Verify);
    }

    #[tokio::test]
    async fn test_try_throw_no_catch_no_finally() {
        let input = r#"
            try {
                throw "Critical Error";
            }
        "#;
        let err = parse_program_expect_error(input);
        assert_eq!(err, ErrorKind::Verify);
    }

    #[tokio::test]
    async fn test_closures() {
        let input = r#"
            let new_adder = fn(x) {
                fn(y) {
                    x + y
                }
            };
            let add_two = new_adder(2);
            add_two(3)
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(5));

        let input2 = r#"
            let make_multiplier = fn(x) {
                fn(y) {
                    x * y
                }
            };
            let double = make_multiplier(2);
            let triple = make_multiplier(3);
            let result = double(4) + triple(4);
            result
        "#;
        assert_eq!(eval_test_helper(input2).await, Object::Integer(20));

        let input3 = r#"
            let x = 10;
            let add_to_x = fn(y) {
                x + y
            };
            add_to_x(5)
        "#;
        assert_eq!(eval_test_helper(input3).await, Object::Integer(15));

        let input4 = r#"
            let create_counter = fn(start) {
                let count = start;
                fn() {
                    count = count + 1;
                    count
                }
            };
            let counter = create_counter(0);
            counter()
        "#;
        assert_eq!(eval_test_helper(input4).await, Object::Integer(1));
    }

    #[tokio::test]
    async fn test_recursion() {
        let input = r#"
            let factorial = fn(n) {
                if (n <= 1) {
                    1
                } else {
                    n * factorial(n - 1)
                }
            };
            factorial(5)
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(120));

        let input2 = r#"
            let sum_to = fn(n) {
                if (n == 0) {
                    0
                } else {
                    n + sum_to(n - 1)
                }
            };
            sum_to(10)
        "#;
        assert_eq!(eval_test_helper(input2).await, Object::Integer(55));

        let input3 = r#"
            let fib = fn(n) {
                if (n <= 1) {
                    n
                } else {
                    fib(n - 1) + fib(n - 2)
                }
            };
            fib(10)
        "#;
        assert_eq!(eval_test_helper(input3).await, Object::Integer(55));

        let input4 = r#"
            let countdown = fn(n) {
                if (n <= 0) {
                    0
                } else {
                    countdown(n - 1)
                }
            };
            countdown(10)
        "#;
        assert_eq!(eval_test_helper(input4).await, Object::Integer(0));
    }

    #[tokio::test]
    async fn test_fibonacci() {
        let input = r#"
            let fib = fn(n) {
                if (n == 0) {
                    0
                } else {
                    if (n == 1) {
                        1
                    } else {
                        fib(n - 1) + fib(n - 2)
                    }
                }
            };
            fib(0)
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(0));

        let input1 = r#"
            let fib = fn(n) {
                if (n == 0) {
                    0
                } else {
                    if (n == 1) {
                        1
                    } else {
                        fib(n - 1) + fib(n - 2)
                    }
                }
            };
            fib(1)
        "#;
        assert_eq!(eval_test_helper(input1).await, Object::Integer(1));

        let input2 = r#"
            let fib = fn(n) {
                if (n == 0) {
                    0
                } else {
                    if (n == 1) {
                        1
                    } else {
                        fib(n - 1) + fib(n - 2)
                    }
                }
            };
            fib(7)
        "#;
        assert_eq!(eval_test_helper(input2).await, Object::Integer(13));

        let input3 = r#"
            let fib = fn(n) {
                if (n == 0) {
                    0
                } else {
                    if (n == 1) {
                        1
                    } else {
                        fib(n - 1) + fib(n - 2)
                    }
                }
            };
            fib(15)
        "#;
        assert_eq!(eval_test_helper(input3).await, Object::Integer(610));
    }

    #[tokio::test]
    async fn test_mutual_recursion() {
        let input = r#"
            let is_even = fn(n) {
                if (n == 0) {
                    true
                } else {
                    is_odd(n - 1)
                }
            };
            let is_odd = fn(n) {
                if (n == 0) {
                    false
                } else {
                    is_even(n - 1)
                }
            };
            is_even(4)
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Boolean(true));

        let input2 = r#"
            let is_even = fn(n) {
                if (n == 0) {
                    true
                } else {
                    is_odd(n - 1)
                }
            };
            let is_odd = fn(n) {
                if (n == 0) {
                    false
                } else {
                    is_even(n - 1)
                }
            };
            is_odd(7)
        "#;
        assert_eq!(eval_test_helper(input2).await, Object::Boolean(true));
    }

    #[tokio::test]
    async fn test_higher_order_functions() {
        let input = r#"
            let apply_twice = fn(f, x) {
                f(f(x))
            };
            let add_one = fn(x) { x + 1 };
            apply_twice(add_one, 0)
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(2));

        let input2 = r#"
            let compose = fn(f, g) {
                fn(x) {
                    f(g(x))
                }
            };
            let double = fn(x) { x * 2 };
            let add_one = fn(x) { x + 1 };
            let double_then_add_one = compose(add_one, double);
            double_then_add_one(5)
        "#;
        assert_eq!(eval_test_helper(input2).await, Object::Integer(11));

        let input3 = r#"
            let apply_three_times = fn(f, x) {
                f(f(f(x)))
            };
            let square = fn(x) { x * x };
            apply_three_times(square, 2)
        "#;
        assert_eq!(eval_test_helper(input3).await, Object::Integer(256));
    }

    #[tokio::test]
    async fn test_closure_with_counter() {
        let input = r#"
            let make_counter = fn() {
                let count = 0;
                let increment = fn() {
                    count = count + 1;
                    count
                };
                increment
            };
            let counter = make_counter();
            let first = counter();
            let second = counter();
            let third = counter();
            first + second + third
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(6));
    }

    #[tokio::test]
    async fn test_nested_closures() {
        let input = r#"
            let outer = fn(x) {
                let middle = fn(y) {
                    let inner = fn(z) {
                        x + y + z
                    };
                    inner
                };
                middle
            };
            outer(1)(2)(3)
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(6));

        let input2 = r#"
            let factory = fn(x) {
                fn(y) {
                    fn(z) {
                        x * y + z
                    }
                }
            };
            factory(2)(3)(4)
        "#;
        assert_eq!(eval_test_helper(input2).await, Object::Integer(10));
    }

    #[tokio::test]
    async fn test_multi_let() {
        let input = r#"
            let (a, b) = (10, 20);
            a + b
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(30));

        let input2 = r#"
            let (a, b) = (1, 2);
            a * b
        "#;
        assert_eq!(eval_test_helper(input2).await, Object::Integer(2));

        let input3 = r#"
            let (x, y, z) = (5, 10, 15);
            x + y + z
        "#;
        assert_eq!(eval_test_helper(input3).await, Object::Integer(30));

        let input4 = r#"
            let a = 5;
            let (b, c) = (10, 20);
            a + b + c
        "#;
        assert_eq!(eval_test_helper(input4).await, Object::Integer(35));
    }

    #[tokio::test]
    async fn test_tuple_assign() {
        let input = r#"
            let a = 0;
            let b = 0;
            (a, b) = (10, 20);
            a
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(10));

        let input2 = r#"
            let a = 0;
            let b = 0;
            (a, b) = (10, 20);
            b
        "#;
        assert_eq!(eval_test_helper(input2).await, Object::Integer(20));

        let input3 = r#"
            let a = 0;
            let b = 0;
            let c = 0;
            (a, b, c) = (1, 2, 3);
            a + b + c
        "#;
        assert_eq!(eval_test_helper(input3).await, Object::Integer(6));
    }

    #[tokio::test]
    async fn test_swap_variables() {
        let input = r#"
            let a = 10;
            let b = 20;
            (a, b) = (b, a);
            a
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(20));

        let input2 = r#"
            let a = 10;
            let b = 20;
            (a, b) = (b, a);
            b
        "#;
        assert_eq!(eval_test_helper(input2).await, Object::Integer(10));

        let input3 = r#"
            let x = 1;
            let y = 2;
            let z = 3;
            (x, y, z) = (z, x, y);
            x
        "#;
        assert_eq!(eval_test_helper(input3).await, Object::Integer(3));

        let input4 = r#"
            let x = 1;
            let y = 2;
            let z = 3;
            (x, y, z) = (z, x, y);
            y
        "#;
        assert_eq!(eval_test_helper(input4).await, Object::Integer(1));

        let input5 = r#"
            let x = 1;
            let y = 2;
            let z = 3;
            (x, y, z) = (z, x, y);
            z
        "#;
        assert_eq!(eval_test_helper(input5).await, Object::Integer(2));
    }

    #[tokio::test]
    async fn test_destructuring_for_in_loop() {
        let input = r#"
            let sum = 0;
            for ((a, b) in [[1, 2], [3, 4], [5, 6]]) {
                sum = sum + a + b;
            }
            sum
        "#;
        assert_eq!(eval_test_helper(input).await, Object::Integer(21));

        let input2 = r#"
            let first_sum = 0;
            let second_sum = 0;
            for ((a, b) in [[1, 2], [3, 4]]) {
                first_sum = first_sum + a;
                second_sum = second_sum + b;
            }
            first_sum + second_sum
        "#;
        assert_eq!(eval_test_helper(input2).await, Object::Integer(10));

        let input3 = r#"
            let first = 0;
            let second = 0;
            for ((num, _) in [[1, 2], [3, 4]]) {
                if (first == 0) {
                    first = num;
                } else {
                    second = num;
                }
            }
            first + second
        "#;
        assert_eq!(eval_test_helper(input3).await, Object::Integer(4));

        let input4 = r#"
            let sum = 0;
            for ((x, y, z) in [[1, 2, 3], [4, 5, 6]]) {
                sum = sum + x + y + z;
            }
            sum
        "#;
        assert_eq!(eval_test_helper(input4).await, Object::Integer(21));
    }

}