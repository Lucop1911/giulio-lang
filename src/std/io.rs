/* 
list_dir(path)
*/

use std::fs::{read_to_string, write, OpenOptions, read_dir};
use std::io::Write;
use std::path::Path;

use crate::interpreter::obj::Object;

pub fn io_read_file(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(path)) => {
            match read_to_string(path) {
                Ok(text) => Ok(Object::String(text)),
                Err(e) => Err(format!("Could not read from file: {}", e))
            }
        }
        _ => {
            Err("read_file() expects a string (path)".to_string())
        }
    }
}

pub fn io_write_file(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();

    match (args.next(), args.next()) {
        (Some(Object::String(path)), Some(Object::String(content))) => {
            match write(path, content) {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(format!("Could not write to file: {}", e))
            }
        }
        _ => {
            Err("write_file() expects exactly two strings (path, content)".to_string())
        }
    }
}

pub fn io_append_file(args: Vec<Object>) -> Result<Object, String> {
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
                Err(e) => Err(format!("Could not append to file: {}", e))
            }
        }
        _ => Err("append_file() expects two strings (path, content)".to_string())
    }
}

pub fn io_exists(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(path)) => {
            let path = Path::new(path);
            Ok(Object::Boolean(path.exists()))
        }
        _ => Err("exists() expects a string (path)".to_string())
    }
}

pub fn io_is_file(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(path)) => {
            let path = Path::new(path);
            Ok(Object::Boolean(path.is_file()))
        }
        _ => Err("is_file() expects a string (path)".to_string())
    }
}

pub fn io_is_dir(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(path)) => {
            let path = Path::new(path);
            Ok(Object::Boolean(path.is_dir()))
        }
        _ => Err("is_dir() expects a string (path)".to_string())
    }
}

pub fn io_list_dir(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(path)) => {
            let path = Path::new(path);

            if !path.is_dir() {
                return Err("list_dir() expects a directory path".to_string());
            }

            let mut items: Vec<Object> = Vec::new();

            for entry in read_dir(path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if let Some(name) = entry.file_name().to_str() {
                    items.push(Object::String(name.to_string()));
                }
            }

            Ok(Object::Array(items))
        }
        _ => Err("list_dir() expects a string (path)".to_string()),
    }
}
