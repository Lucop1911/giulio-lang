use crate::interpreter::obj::Object;

// Function only
pub fn bprint_fn(args: Vec<Object>) -> Result<Object, String> {
    for (i, obj) in args.iter().enumerate() {
        if i > 0 {
            print!("");
        }
        print!("{}", obj);
    }
    Ok(Object::Null)
}

// Function only
pub fn bprintln_fn(args: Vec<Object>) -> Result<Object, String> {
    for (i, obj) in args.iter().enumerate() {
        if i > 0 {
            print!("");
        }
        print!("{}", obj);
    }
    println!();
    Ok(Object::Null)
}

