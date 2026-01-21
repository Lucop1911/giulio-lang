use crate::interpreter::obj::Object;
use rand::Rng;

pub fn math_clamp(args: Vec<Object>) -> Result<Object, String> {
    match (&args[0], &args[1], &args[2]) {
        (Object::Integer(n), Object::Integer(min), Object::Integer(max)) => {
            if min > max { 
                return Err(" second field (min) cannot be greater than third field (max)".to_string())
            }

            Ok(Object::Integer(*n.clamp(min, max)))
        }
        _ => Err("clamp expects three integers (n, min, max)".to_string())
    }
}

pub fn math_random(args: Vec<Object>) -> Result<Object, String> {
    let mut rng = rand::rng();

    match args.as_slice() {
        [] => Ok(Object::Integer(rng.random_range(0..=10))),
        [Object::Integer(max)] => {
            if *max < 0 { 
                return Err("max must be non negative".to_string())
            }
            Ok(Object::Integer(rng.random_range(0..*max)))
        }
        [Object::Integer(min), Object::Integer(max)] => {
            if *max < *min {
                return Err("min must be lower than or equal to max".to_string())
            }
            Ok(Object::Integer(rng.random_range(*min..=*max)))
        }
        _ => Err("random_int() takes 0, 1, or 2 integer arguments".to_string()),
    }
}