use std::io::{self, Write};

use giulio_lang::{Evaluator, Lexer, Parser, Tokens, interpreter::obj::Object};

fn main() {
    println!("Giulio-lang v0.1.0");
    println!("Type 'exit' to quit\n");

    let mut evaluator = Evaluator::new();

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

        // Lexing
        let token_vec = match Lexer::lex_tokens(input.as_bytes()) {
            Ok((_, t)) => t,
            Err(e) => {
                eprintln!("Lex error: {:?}", e);
                continue;
            }
        };

        // Pass a slice of the vector to Tokens
        let tokens = Tokens::new(&token_vec);

        // Parsing
        let program = match Parser::parse_tokens(tokens) {
            Ok((_, program)) => program,
            Err(e) => {
                eprintln!("Parse error: {:?}", e);
                continue;
            }
        };

        // Evaluate
        let result = evaluator.eval_program(program);

        // Handle result
        match result {
            Object::Null => {}
            Object::Error(e) => eprintln!("{}", e),
            Object::String(s) => print!("{}", s),
            other => println!("{}", other),
        }

        println!();
        io::stdout().flush().unwrap();
    }
}
