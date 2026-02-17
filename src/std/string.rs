use crate::errors::RuntimeError;
use crate::interpreter::obj::Object;

pub fn string_join(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match (&args[0], &args[1]) {
        (Object::Array(arr), Object::String(separator)) => {
            let strings: Result<Vec<String>, RuntimeError> = arr
                .iter()
                .map(|obj| match obj {
                    Object::String(s) => Ok(s.clone()),
                    o => Err(RuntimeError::TypeMismatch {
                        expected: "string".to_string(),
                        got: o.type_name(),
                    }),
                })
                .collect();

            match strings {
                Ok(strs) => Ok(Object::String(strs.join(separator))),
                Err(e) => Err(e),
            }
        }
        _ => Err(RuntimeError::TypeMismatch {
            expected: "array, string".to_string(),
            got: "invalid arguments".to_string(),
        }),
    }
}
