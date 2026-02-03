use crate::{Parser, Lexer, Tokens};
use crate::parser_errors::{convert_nom_error, show_error_context};

pub fn run_check(input: &str) {
    // Lexing
    let token_vec = match Lexer::lex_tokens(input.as_bytes()) {
        Ok((_, t)) => t,
        Err(e) => {
            eprintln!("╭─ Check Failed ─────────────────────────────");
            eprintln!("│");
            eprintln!("│ Lexer Error:");
            eprintln!("│   {:?}", e);
            eprintln!("│");
            eprintln!("╰────────────────────────────────────────────");
            return;
        }
    };

    let tokens = Tokens::new(&token_vec);

    // Parsing
    let _ = match Parser::parse_tokens(tokens) {
        Ok((_, program)) => program,
        Err(e) => {
            eprintln!("╭─ Check Failed ─────────────────────────────");
            eprintln!("│");
            eprintln!("│ Parser Error:");
            
            // Extract better error information
            if let nom::Err::Error(err) | nom::Err::Failure(err) = &e {
                let parser_error = convert_nom_error(&e, "");
                eprintln!("│   {}", parser_error);
                eprintln!("│");
                eprintln!("│ {}", show_error_context(&err.input, 3));
            } else {
                eprintln!("│   Unexpected end of input");
            }
            
            eprintln!("│");
            eprintln!("╰────────────────────────────────────────────");
            return;
        }
    };

    println!("╭─ Check Passed ─────────────────────────────");
    println!("│");
    println!("│ ✓ No syntax errors found");
    println!("│");
    println!("╰────────────────────────────────────────────");
}