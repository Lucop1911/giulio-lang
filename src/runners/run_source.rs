use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::{Parser, Lexer, Tokens, runtime::obj::Object};
use crate::parser_errors::{convert_nom_error, show_error_context};
use crate::runtime::env::Environment;
use crate::runtime::module_registry::ModuleRegistry;
use crate::vm::compiler::Compiler;
use crate::vm::vm::VirtualMachine;

pub async fn run_source(input: &str) {
    let token_vec = match Lexer::lex_tokens(input.as_bytes()) {
        Ok((_, t)) => t,
        Err(e) => {
            eprintln!("╭─ Lexer Error ──────────────────────────────");
            eprintln!("│");
            eprintln!("│ {:?}", e);
            eprintln!("│");
            eprintln!("╰────────────────────────────────────────────");
            return;
        }
    };

    let tokens = Tokens::new(&token_vec);

    let mut program = match Parser::parse_tokens(tokens) {
        Ok((_, program)) => program,
        Err(e) => {
            eprintln!("╭─ Parser Error ─────────────────────────────");
            eprintln!("│");

            if let nom::Err::Error(err) | nom::Err::Failure(err) = &e {
                let parser_error = convert_nom_error(&e, "");
                eprintln!("│ {}", parser_error);
                eprintln!("│");
                eprintln!("│ {}", show_error_context(&err.input, 3));
            } else {
                eprintln!("│ Unexpected end of input");
            }

            eprintln!("│");
            eprintln!("╰────────────────────────────────────────────");
            return;
        }
    };

    let chunk = Compiler::compile_program(&mut program);
    let globals = Arc::new(Mutex::new(Environment::new_root()));
    let module_registry = Arc::new(Mutex::new(ModuleRegistry::new(PathBuf::from("."))));
    let mut vm = VirtualMachine::new(globals, module_registry);

    let result = vm.run(Arc::new(chunk)).await;

    match result {
        Ok(Object::Null) => {}
        Ok(Object::Error(e)) => {
            eprintln!("╭─ Runtime Error ────────────────────────────");
            eprintln!("│");
            eprintln!("│ {}", e);
            eprintln!("│");
            eprintln!("╰────────────────────────────────────────────");
        }
        Ok(Object::String(s)) => print!("{}", s),
        Ok(other) => println!("{}", other),
        Err(e) => {
            eprintln!("╭─ Runtime Error ────────────────────────────");
            eprintln!("│");
            eprintln!("│ {}", e);
            eprintln!("│");
            eprintln!("╰────────────────────────────────────────────");
        }
    }
}
