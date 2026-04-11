use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::{Lexer, Parser, Tokens, runtime::obj::Object};
use crate::parser_errors::{convert_nom_error, show_error_context};
use crate::runtime::env::Environment;
use crate::runtime::module_registry::ModuleRegistry;
use crate::vm::compiler::Compiler;
use crate::vm::vm::VirtualMachine;

pub async fn repl() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("g-lang v{}", VERSION);
    println!("Type 'exit' or 'quit' to quit\n");

    let globals = Arc::new(Mutex::new(Environment::new()));
    let module_registry = Arc::new(Mutex::new(ModuleRegistry::new(PathBuf::from("."))));
    let mut vm = VirtualMachine::new(globals, module_registry);

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

        let mut program = match Parser::parse_tokens(tokens) {
            Ok((_, program)) => program,
            Err(e) => {
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

        let chunk = Compiler::compile_program(&mut program);
        let result = vm.run(Arc::new(chunk)).await;

        match result {
            Ok(Object::Null) => {}
            Ok(Object::Error(e)) => eprintln!("{}", e),
            Ok(Object::String(s)) => print!("{}", s),
            Ok(other) => println!("{}", other),
            Err(e) => eprintln!("{}", e),
        }

        println!();
        io::stdout().flush().unwrap();
    }
}
