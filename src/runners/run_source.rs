use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::lexer::lexer::Lexer;
use crate::lexer::token::SpannedTokens;
use crate::parser::parser::Parser;
use crate::parser::parser_errors::{convert_nom_error, show_error_context};
use crate::vm::obj::Object;
use crate::vm::runtime::env::Environment;
use crate::vm::runtime::module_registry::ModuleRegistry;
use crate::vm::compiler::Compiler;
use crate::vm::vm::VirtualMachine;

pub async fn run_source(input: &str) {
    let spanned_tokens = match Lexer::lex_tokens(input.as_bytes()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("╭─ Lexer Error ──────────────────────────────");
            eprintln!("│");
            eprintln!("│ {}", e);
            eprintln!("│");
            eprintln!("╰────────────────────────────────────────────");
            return;
        }
    };

    let spanned = SpannedTokens::new(&spanned_tokens);
    let (tokens, _) = spanned.to_tokens_with_offset();

    let mut program = match Parser::parse_tokens(tokens) {
        Ok((_, program)) => program,
        Err(e) => {
            eprintln!("╭─ Parser Error ─────────────────────────────");
            eprintln!("│");

            if let nom::Err::Error(err) | nom::Err::Failure(err) = &e {
                let remaining_count = err.input.token.len();
                let total_count = tokens.token.len();
                let error_index = total_count - remaining_count;
                let parser_error = convert_nom_error(&e, "", &spanned_tokens, error_index);
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

    let chunk = match Compiler::compile_program(&mut program) {
        Ok(chunk) => chunk,
        Err(e) => {
            eprintln!("╭─ Compiler Error ───────────────────────────");
            eprintln!("│");
            eprintln!("│ {}", e);
            eprintln!("│");
            eprintln!("╰────────────────────────────────────────────");
            return;
        }
    };
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
