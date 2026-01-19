use crate::{Parser, Lexer, Tokens};

pub fn run_check(input: &str) {
    // Lexing
    let token_vec = match Lexer::lex_tokens(input.as_bytes()) {
        Ok((_, t)) => t,
        Err(e) => {
            eprintln!("Check finished. \nErrors were found.\n");
            eprintln!("Lex error: {:?}", e);
            return;
        }
    };

    let tokens = Tokens::new(&token_vec);

    // Parsing
    let _ = match Parser::parse_tokens(tokens) {
        Ok((_, program)) => program,
        Err(e) => {
            eprintln!("Check finished. \nErrors were found.\n");
            eprintln!("Parse error: {:?}", e);
            return;
        }
    };

    println!("Check Finished");
    println!("No Errors were found");
}