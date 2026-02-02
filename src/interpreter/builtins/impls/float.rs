use num_traits::ToPrimitive;
use crate::interpreter::obj::Object;

// Method only
pub fn btofloat_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::Integer(n)) => {
            Ok(Object::Float(*n as f64)) // Wont fail as Integers are always i64
        }
        Some(Object::BigInteger(n)) => {
            match n.to_f64() {
                Some(n) => Ok(Object::Float(n)),
                None => Err("Could not convert BigInteger to Float".to_string())
            }
        }
        Some(Object::String(str)) => {
            match str.trim().parse::<f64>() {
                Ok(f) => Ok(Object::Float(f)),
                Err(_) => Err("Cannot convert string to float".to_string()),
            }
        }
        _ => Err("to_float() expects an integer, a bigInteger or a string".to_string())
    }
}