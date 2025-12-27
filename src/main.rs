use std::io::{self, Write};
use std::fs;
use std::env;

use giulio_lang::run_source::run_source;
use giulio_lang::{Evaluator, Lexer, Parser, Tokens, interpreter::obj::Object};

fn main() {
    let mut evaluator = Evaluator::default();
    let args: Vec<String> = env::args().collect();
    // FILE MODE
    if args.get(1).unwrap_or(&"".to_string()) == &"run" {
        let filename = &args[2];
        if filename.ends_with(".giu") {
            let source = match fs::read_to_string(filename) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Could not read file {}: {}", filename, e);
                    return;
                }
            };

            run_source(&source, &mut evaluator);
            return;
        } else {
            println!("Not a giulio-lang file!");
            return;
        }
    }


    // REPL MODE
    println!("Giulio-lang v0.1.0");
    println!("Type 'exit' to quit\n");

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
