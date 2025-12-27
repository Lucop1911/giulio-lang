use std::io::{self, Write};

use crate::{Evaluator, Lexer, Parser, Tokens, interpreter::obj::Object};

pub fn repl(mut evaluator: Evaluator) {
    println!("Giulio-lang v0.1.0");
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
                eprintln!("Lex error: {:?}", e);
                continue;
            }
        };

        let tokens = Tokens::new(&token_vec);

        let program = match Parser::parse_tokens(tokens) {
            Ok((_, program)) => program,
            Err(e) => {
                eprintln!("Parse error: {:?}", e);
                continue;
            }
        };

        match evaluator.eval_program(program) {
            Object::Null => {}
            Object::Error(e) => eprintln!("{}", e),
            Object::String(s) => print!("{}", s),
            other => println!("{}", other),
        }
    }
}
