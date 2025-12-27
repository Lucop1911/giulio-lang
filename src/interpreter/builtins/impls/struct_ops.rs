use crate::interpreter::obj::Object;

pub fn bset_field_fn(args: Vec<Object>) -> Result<Object, String> {
    match (args[0].clone(), args[1].clone(), args[2].clone()) {
        (Object::Struct { name, mut fields, methods }, Object::String(field_name), new_value) => {
            fields.insert(field_name.clone(), new_value.clone());
            Ok(Object::Struct {
                name: name.clone(),
                fields,
                methods: methods.clone(),
            })
        }
        _ => Err("set_field expects (struct, field_name, value)".to_string()),
    }
}