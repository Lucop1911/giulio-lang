use std::io::{self, Write};
use crate::interpreter::obj::Object;

// Function only
pub fn binput_fn(args: Vec<Object>) -> Result<Object, String> {
    if args.len() > 1 { return Err("input() takes at most 1 argument".to_string())}

    match args.first() {
        Some(Object::String(s)) => { 
            print!("{}", s);
            io::stdout().flush().map_err(|e| e.to_string())?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            Ok(Object::String(input.trim_end().to_string()))
        }
        Some(Object::Null) | None=> {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            Ok(Object::String(input.trim_end().to_string()))
        }
        _ => Err("Invalid argument to input()".to_string())
    }
}