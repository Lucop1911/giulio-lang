use crate::interpreter::obj::Object;

pub fn bset_field_fn(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 3 {
        return Err(format!(
            "set_field() expects 3 arguments, got {}",
            args.len()
        ));
    }
    match (args[0].clone(), args[1].clone(), args[2].clone()) {
        (
            Object::Struct {
                name,
                mut fields,
                methods,
            },
            Object::String(field_name),
            new_value,
        ) => {
            fields.insert(field_name.clone(), new_value.clone());
            Ok(Object::Struct {
                name: name.clone(),
                fields,
                methods: methods.clone(),
            })
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
            Object::Struct {
                name: _,
                fields,
                methods: _,
            },
            Object::String(field_name),
        ) => match fields.get(&field_name) {
            Some(value) => Ok(value.clone()),
            None => Err(format!("get_field() field '{}' does not exist", field_name)),
        },
        (o, _) => Err(format!("get_field() expects struct, got {}", o.type_name())),
    }
}

pub fn bstruct_fields_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(Object::Struct {
            name: _,
            fields,
            methods: _,
        }) => {
            let field_names: Vec<Object> = fields.keys().cloned().map(Object::String).collect();
            Ok(Object::Array(field_names))
        }
        Some(o) => Err(format!("fields() expects struct, got {}", o.type_name())),
        None => Err(format!("fields() expects 1 argument, got 0")),
    }
}

pub fn bstruct_name_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(Object::Struct {
            name,
            fields: _,
            methods: _,
        }) => Ok(Object::String(name)),
        Some(o) => Err(format!("name() expects struct, got {}", o.type_name())),
        None => Err(format!("name() expects 1 argument, got 0")),
    }
}
