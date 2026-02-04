use std::io::{self, Write};

use crate::{Evaluator, Lexer, Parser, Tokens, interpreter::obj::Object};
use crate::parser_errors::{convert_nom_error, show_error_context};

pub fn repl(mut evaluator: Evaluator) {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("Giulio-lang v{}", VERSION);
    println!("Type 'exit' or 'quit' to quit\n");

    loop {
        print!(">> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Failed to read input");
            continue;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "exit" || trimmed == "quit" {
            println!("Goodbye!");
            break;
        }

        let token_vec = match Lexer::lex_tokens(input.as_bytes()) {
            Ok((_, t)) => t,
            Err(e) => {
                eprintln!("Lexer Error: {:?}", e);
                continue;
            }
        };

        let tokens = Tokens::new(&token_vec);

        let program = match Parser::parse_tokens(tokens) {
            Ok((_, program)) => program,
            Err(e) => {
                // Extract better error information
                if let nom::Err::Error(err) | nom::Err::Failure(err) = &e {
                    let parser_error = convert_nom_error(&e, "");
                    eprintln!("Parser Error: {}", parser_error);
                    eprintln!("{}", show_error_context(&err.input, 3));
                } else {
                    eprintln!("Parser Error: Unexpected end of input");
                }
                continue;
            }
        };

        match evaluator.eval_program(program) {
            Object::Null => {}
            Object::Error(e) => eprintln!("{}", e),
            Object::String(s) => print!("{}", s),
            other => println!("{}", other),
        }
        
        println!();
        io::stdout().flush().unwrap();
    }
}