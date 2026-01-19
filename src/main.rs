use std::env;
use std::fs;

use giulio_lang::Evaluator;
use giulio_lang::runners::print_help::print_help;
use giulio_lang::runners::run_check::run_check;
use giulio_lang::runners::run_source::run_source;
use giulio_lang::runners::run_repl_mode::repl;


fn main() {
    let mut evaluator = Evaluator::default();
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(flag) if flag == "--version" || flag == "-version" || flag == "-v" || flag == "--v" => {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            println!("{}", VERSION);
            return;
        }

        Some(flag) if flag == "--help" || flag == "-help" || flag == "--h" || flag == "-h" => {
            print_help();
            return;
        }

        Some(flag) if flag == "check" => {
            if let Some(filename) = args.get(2) {
                let source = match fs::read_to_string(filename) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Could not read file {}: {}", filename, e);
                        return;
                    }
                };
                run_check(&source);
                return;
            }
        }

        Some(flag) if flag == "run" => {
            if let Some(filename) = args.get(2) {
                let source = match fs::read_to_string(filename) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Could not read file {}: {}", filename, e);
                        return;
                    }
                };

            run_source(&source, &mut evaluator);
            return;
            }
        }

        Some(arg) => {
            eprintln!("Unknown argument: {}", arg);
            eprintln!("Use --help for usage.");
            return;
        }

        None => repl(evaluator),
    }
}
