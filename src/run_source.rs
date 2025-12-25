use crate::{Parser, Evaluator, Lexer, Tokens, interpreter::obj::Object};

pub fn run_source(input: &str, evaluator: &mut Evaluator) {
    // Lexing
    let token_vec = match Lexer::lex_tokens(input.as_bytes()) {
        Ok((_, t)) => t,
        Err(e) => {
            eprintln!("Lex error: {:?}", e);
            return;
        }
    };

    let tokens = Tokens::new(&token_vec);

    // Parsing
    let program = match Parser::parse_tokens(tokens) {
        Ok((_, program)) => program,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            return;
        }
    };

    // Evaluate
    let result = evaluator.eval_program(program);

    match result {
        Object::Null => {}
        Object::Error(e) => eprintln!("{}", e),
        Object::String(s) => print!("{}", s),
        other => println!("{}", other),
    }
}
