use crate::interpreter::obj::Object;

pub fn btype_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(obj) => Ok(Object::String(obj.type_name())),
        _ => Err("type() requires one argument".to_string()),
    }
}