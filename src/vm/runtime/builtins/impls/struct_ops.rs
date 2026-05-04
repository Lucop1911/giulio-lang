use crate::vm::obj::Object;

pub fn bset_field_fn(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 3 {
        return Err(format!(
            "set_field() expects 3 arguments, got {}",
            args.len()
        ));
    }
    match (args[0].clone(), args[1].clone(), args[2].clone()) {
        (
            Object::Struct(mut s),
            Object::String(field_name),
            new_value,
        ) => {
            s.fields.insert(field_name.clone(), new_value.clone());
            Ok(Object::Struct(s))
        }
        (o, _, _) => Err(format!("set_field() expects struct, got {}", o.type_name())),
    }
}

pub fn bget_field_fn(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 2 {
        return Err(format!(
            "get_field() expects 2 arguments, got {}",
            args.len()
        ));
    }
    match (args[0].clone(), args[1].clone()) {
        (
            Object::Struct(s),
            Object::String(field_name),
        ) => match s.fields.get(&field_name) {
            Some(value) => Ok(value.clone()),
            None => Err(format!("get_field() field '{}' does not exist", field_name)),
        },
        (o, _) => Err(format!("get_field() expects struct, got {}", o.type_name())),
    }
}

pub fn bstruct_fields_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(Object::Struct(s)) => {
            let field_names: Vec<Object> = s.fields.keys().cloned().map(Object::String).collect();
            Ok(Object::Array(Box::new(field_names)))
        }
        Some(o) => Err(format!("fields() expects struct, got {}", o.type_name())),
        None => Err("fields() expects 1 argument, got 0".to_string()),
    }
}

pub fn bstruct_name_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(Object::Struct(s)) => Ok(Object::String(s.name)),
        Some(o) => Err(format!("name() expects struct, got {}", o.type_name())),
        None => Err("name() expects 1 argument, got 0".to_string()),
    }
}
