use crate::interpreter::obj::Object;

// Function only
pub fn bprint_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::String(t)) => {
            print!("{}", t);
            Ok(Object::Null)
        }
        Some(o) => {
            print!("{}", o);
            Ok(Object::Null)
        }
        _ => Err("invalid arguments for print".to_string()),
    }
}

// Function only
pub fn bprintln_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::String(t)) => {
            println!("{}", t);
            Ok(Object::Null)
        }
        Some(o) => {
            println!("{}", o);
            Ok(Object::Null)
        }
        _ => Err("Invalid arguments for println".to_string())
    }
}
