use crate::lexer::token::SpannedTokens;
use crate::parser::parser_errors::{convert_nom_error, show_error_context};
use crate::vm::compiler::compute_slots::compute_slots;
use crate::parser::parser::Parser;
use crate::lexer::lexer::Lexer;

pub fn run_check(input: &str) {
    let spanned_tokens = match Lexer::lex_tokens(input.as_bytes()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("╭─ Check Failed ─────────────────────────────");
            eprintln!("│");
            eprintln!("│ Lexer Error:");
            eprintln!("│   {}", e);
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
            eprintln!("╭─ Check Failed ─────────────────────────────");
            eprintln!("│");
            eprintln!("│ Parser Error:");

            if let nom::Err::Error(err) | nom::Err::Failure(err) = &e {
                let remaining_count = err.input.token.len();
                let total_count = tokens.token.len();
                let error_index = total_count - remaining_count;
                let parser_error = convert_nom_error(&e, "", &spanned_tokens, error_index);
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

    compute_slots(&mut program);

    println!("╭─ Check Passed ─────────────────────────────");
    println!("│");
    println!("│ ✓ No syntax errors found");
    println!("│");
    println!("╰────────────────────────────────────────────");
}
