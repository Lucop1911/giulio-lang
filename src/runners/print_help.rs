pub fn print_help() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const GITHUB: &str = "https://github.com/Lucop1911/g-lang";
    const DOCUMENTATION: &str = "https://g-lang.vercel.app";

    print!("G-lang v");
    println!("{}", VERSION);
    println!("A dynamically-typed interpreted programming language\n");
    
    println!("USAGE:");
    println!("    gl [COMMAND] [OPTIONS]\n");
    
    println!("COMMANDS:");
    println!("    (no command)       Start the REPL (Read-Eval-Print Loop)");
    println!("    run <file>         Execute a .g file");
    println!("    check <file>       Lex and Parse to check a .g file for syntax errors\n");
    
    println!("OPTIONS:");
    println!("    -h, --help         Print this help message");
    println!("    -v, --version      Print version information\n");
    
    println!("EXAMPLES:");
    println!("    gl                    # Start REPL mode");
    println!("    gl run script.g     # Run a script");
    println!("    gl check script.g   # Check a file");
    println!("    gl --version          # Show version");
    println!("    gl --help             # Show this help\n");
    
    println!("REPL COMMANDS:");
    println!("    exit, quit         Exit the REPL\n");
    
    println!("For more information, visit: {}", DOCUMENTATION);
    println!("Github repo: {}", GITHUB)
}