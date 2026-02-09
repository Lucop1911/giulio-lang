use std::fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file, write, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::interpreter::obj::Object;
use crate::errors::RuntimeError;

pub fn io_read_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match read_to_string(path) {
                Ok(text) => Ok(Object::String(text)),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not read from file: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_create_dir(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match create_dir_all(path) {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not create directory: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_delete_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match remove_file(path) {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not delete file: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_delete_dir(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match remove_dir_all(path) {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not delete directory: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_write_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let mut args = args.into_iter();

    match (args.next(), args.next()) {
        (Some(Object::String(path)), Some(Object::String(content))) => {
            match write(path, content) {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not write to file: {}", e)))
            }
        }
        (Some(Object::String(_)), Some(o)) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        (Some(o), _) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        _ => Err(RuntimeError::WrongNumberOfArguments { min: 2, max: 2, got: 0 }),
    }
}

pub fn io_append_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let mut args = args.into_iter();
    
    match (args.next(), args.next()) {
        (Some(Object::String(path)), Some(Object::String(content))) => {
             let result = OpenOptions::new()
             .create(true)
             .append(true)
             .open(path)
             .and_then(|mut file| file.write_all(content.as_bytes()));

            match result {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not append to file: {}", e)))
            }
        }
        (Some(Object::String(_)), Some(o)) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        (Some(o), _) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        _ => Err(RuntimeError::WrongNumberOfArguments { min: 2, max: 2, got: 0 }),
    }
}

pub fn io_exists(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            let path = Path::new(path);
            Ok(Object::Boolean(path.exists()))
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_is_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            let path = Path::new(path);
            Ok(Object::Boolean(path.is_file()))
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_is_dir(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            let path = Path::new(path);
            Ok(Object::Boolean(path.is_dir()))
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_list_dir(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            let path = Path::new(path);

            if !path.is_dir() {
                return Err(RuntimeError::InvalidOperation(format!("'{}' is not a directory", path.display())));
            }

            let mut items: Vec<Object> = Vec::new();

            for entry in read_dir(path).map_err(|e| RuntimeError::InvalidOperation(e.to_string()))? {
                let entry = entry.map_err(|e| RuntimeError::InvalidOperation(e.to_string()))?;
                if let Some(name) = entry.file_name().to_str() {
                    items.push(Object::String(name.to_string()));
                }
            }

            Ok(Object::Array(items))
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}
