use crate::interpreter::obj::Object;
use num_traits::ToPrimitive;

// Method only
pub fn btofloat_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::Integer(n)) => {
            Ok(Object::Float(*n as f64)) // Wont fail as Integers are always i64
        }
        Some(Object::BigInteger(n)) => match n.to_f64() {
            Some(n) => Ok(Object::Float(n)),
            None => Err(format!(
                "to_float() cannot convert BigInteger to Float (overflow)"
            )),
        },
        Some(Object::String(str)) => match str.trim().parse::<f64>() {
            Ok(f) => Ok(Object::Float(f)),
            Err(_) => Err(format!("to_float() cannot convert '{}' to float", str)),
        },
        Some(o) => Err(format!(
            "to_float() expects integer, bigInteger, or string, got {}",
            o.type_name()
        )),
        None => Err(format!("to_float() expects 1 argument, got 0")),
    }
}
