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

pub fn bget_field_fn(args: Vec<Object>) -> Result<Object, String> {
    match (args[0].clone(), args[1].clone()) {
        (Object::Struct { name: _, fields, methods: _ }, Object::String(field_name)) => {
            match fields.get(&field_name) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("field '{}' does not exist", field_name)),
            }
        }
        _ => Err("get_field expects (struct, field_name)".to_string()),
    }
}

pub fn bstruct_fields_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(Object::Struct { name: _, fields, methods: _ }) => {
            let field_names: Vec<Object> = fields.keys().cloned().map(Object::String).collect();
            Ok(Object::Array(field_names))
        }
        _ => Err("fields() requires a struct".to_string()),
    }
}

pub fn bstruct_name_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(Object::Struct { name, fields: _, methods: _ }) => {
            Ok(Object::String(name))
        }
        _ => Err("name() requires a struct".to_string()),
    }
}