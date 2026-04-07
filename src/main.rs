use std::env;
use std::fs;

use g_lang::Evaluator;
use g_lang::runners::print_help::print_help;
use g_lang::runners::run_check::run_check;
use g_lang::runners::run_source::run_source;
use g_lang::runners::run_repl_mode::repl;

/// CLI entry point.
///
/// Dispatches to one of four execution modes based on the first CLI argument:
///
/// | Argument            | Mode                                  |
/// |---------------------|---------------------------------------|
/// | *(none)*            | REPL (interactive read-eval-print)    |
/// | `run <file>`        | Execute a `.g` script                 |
/// | `check <file>`      | Parse-only syntax validation          |
/// | `-v` / `--version`  | Print version and exit                |
/// | `-h` / `--help`     | Print usage and exit                  |
#[tokio::main]
async fn main() {
    let mut evaluator = Evaluator::default();
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(flag) if flag == "--version" || flag == "-version" || flag == "-v" || flag == "--v" => {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            println!("{}", VERSION);
        }

        Some(flag) if flag == "--help" || flag == "-help" || flag == "--h" || flag == "-h" => {
            print_help();
        }

        Some(flag) if flag == "check" => {
            if let Some(filename) = args.get(2) {
                if !filename.ends_with(".g") {
                    eprintln!("Error: File must have .g extension");
                    return;
                }
                let source = match fs::read_to_string(filename) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Could not read file {}: {}", filename, e);
                        return;
                    }
                };
                run_check(&source);
            }
        }

        Some(flag) if flag == "run" => {
            if let Some(filename) = args.get(2) {
                if !filename.ends_with(".g") {
                    eprintln!("Error: File must have .g extension");
                    return;
                }
                let source = match fs::read_to_string(filename) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Could not read file {}: {}", filename, e);
                        return;
                    }
                };

            run_source(&source, &mut evaluator).await;
            }
        }

        Some(arg) => {
            eprintln!("Unknown argument: {}", arg);
            eprintln!("Use --help for usage.");
        }

        None => repl(evaluator).await,
    }
}
