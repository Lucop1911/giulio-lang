use crate::{Parser, Evaluator, Lexer, Tokens, interpreter::obj::Object};
use crate::parser_errors::{convert_nom_error, show_error_context};

pub async fn run_source(input: &str, evaluator: &mut Evaluator) {
    // Lexing
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

    // Parsing
    let program = match Parser::parse_tokens(tokens) {
        Ok((_, program)) => program,
        Err(e) => {
            eprintln!("╭─ Parser Error ─────────────────────────────");
            eprintln!("│");
            
            // Extract better error information
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

    // Evaluate
    let result = evaluator.eval_program(program).await;

    match result {
        Object::Null => {}
        Object::Error(e) => {
            eprintln!("╭─ Runtime Error ────────────────────────────");
            eprintln!("│");
            eprintln!("│ {}", e);
            eprintln!("│");
            eprintln!("╰────────────────────────────────────────────");
        }
        Object::String(s) => print!("{}", s),
        _ => {},
    }
}