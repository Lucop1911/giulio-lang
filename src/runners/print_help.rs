pub fn print_help() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const GITHUB: &str = "https://github.com/Lucop1911/giulio-lang";
    const DOCUMENTATION: &str = "https://giulio-lang.vercel.app";

    print!("Giulio-lang v");
    println!("{}", VERSION);
    println!("A dynamically-typed interpreted programming language\n");
    
    println!("USAGE:");
    println!("    giulio-lang [COMMAND] [OPTIONS]\n");
    
    println!("COMMANDS:");
    println!("    (no command)       Start the REPL (Read-Eval-Print Loop)");
    println!("    run <file>         Execute a .giu file");
    println!("    check <file>       Lex and Parse to check a .giu file for syntax errors\n");
    
    println!("OPTIONS:");
    println!("    -h, --help         Print this help message");
    println!("    -v, --version      Print version information\n");
    
    println!("EXAMPLES:");
    println!("    giulio-lang                    # Start REPL mode");
    println!("    giulio-lang run script.giu     # Run a script");
    println!("    giulio-lang check script.giu   # Check a file");
    println!("    giulio-lang --version          # Show version");
    println!("    giulio-lang --help             # Show this help\n");
    
    println!("REPL COMMANDS:");
    println!("    exit, quit         Exit the REPL\n");
    
    println!("For more information, visit: {}", DOCUMENTATION);
    println!("Github repo: {}", GITHUB)
}