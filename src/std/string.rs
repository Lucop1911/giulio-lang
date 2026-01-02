use crate::interpreter::obj::Object;

pub fn string_join(args: Vec<Object>) -> Result<Object, String> {
    match (&args[0], &args[1]) {
        (Object::Array(arr), Object::String(separator)) => {
            let strings: Result<Vec<String>, String> = arr.iter().map(|obj| {
                match obj {
                    Object::String(s) => Ok(s.clone()),
                    _ => Err("join expects an array of strings".to_string()),
                }
            }).collect();
            
            match strings {
                Ok(strs) => Ok(Object::String(strs.join(separator))),
                Err(e) => Err(e),
            }
        }
        _ => Err("join expects an array and a string".to_string()),
    }
}