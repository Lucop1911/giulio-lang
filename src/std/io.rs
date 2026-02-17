use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::path::Path;

use crate::interpreter::obj::Object;
use crate::errors::RuntimeError;
use std::sync::{Arc, Mutex};

pub fn io_read_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match std::fs::read_to_string(path) {
                Ok(text) => Ok(Object::String(text)),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not read from file: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub async fn async_io_read_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match fs::read_to_string(path).await {
                Ok(text) => Ok(Object::String(text)),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not read from file: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_read_file_async(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let args = args;
    Ok(Object::Future(Arc::new(Mutex::new(Some(Box::pin(async_io_read_file(args)) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Object, RuntimeError>> + Send + 'static>>)))))
}

pub fn io_create_dir(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match std::fs::create_dir_all(path) {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not create directory: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub async fn async_io_create_dir(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match fs::create_dir_all(path).await {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not create directory: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_create_dir_async(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let args = args;
    Ok(Object::Future(Arc::new(Mutex::new(Some(Box::pin(async_io_create_dir(args)) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Object, RuntimeError>> + Send + 'static>>)))))
}

pub fn io_delete_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match std::fs::remove_file(path) {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not delete file: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub async fn async_io_delete_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match fs::remove_file(path).await {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not delete file: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_delete_file_async(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let args = args;
    Ok(Object::Future(Arc::new(Mutex::new(Some(Box::pin(async_io_delete_file(args)) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Object, RuntimeError>> + Send + 'static>>)))))
}

pub fn io_delete_dir(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match std::fs::remove_dir_all(path) {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not delete directory: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub async fn async_io_delete_dir(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            match fs::remove_dir_all(path).await {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not delete directory: {}", e)))
            }
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn io_delete_dir_async(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let args = args;
    Ok(Object::Future(Arc::new(Mutex::new(Some(Box::pin(async_io_delete_dir(args)) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Object, RuntimeError>> + Send + 'static>>)))))
}

pub fn io_write_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let mut args = args.into_iter();

    match (args.next(), args.next()) {
        (Some(Object::String(path)), Some(Object::String(content))) => {
            match std::fs::write(path, content) {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not write to file: {}", e)))
            }
        }
        (Some(Object::String(_)), Some(o)) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        (Some(o), _) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        _ => Err(RuntimeError::WrongNumberOfArguments { min: 2, max: 2, got: 0 }),
    }
}

pub async fn async_io_write_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let mut args = args.into_iter();

    match (args.next(), args.next()) {
        (Some(Object::String(path)), Some(Object::String(content))) => {
            match fs::write(path, content).await {
                Ok(_) => Ok(Object::Null),
                Err(e) => Err(RuntimeError::InvalidOperation(format!("Could not write to file: {}", e)))
            }
        }
        (Some(Object::String(_)), Some(o)) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        (Some(o), _) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        _ => Err(RuntimeError::WrongNumberOfArguments { min: 2, max: 2, got: 0 }),
    }
}

pub fn io_write_file_async(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let args = args;
    Ok(Object::Future(Arc::new(Mutex::new(Some(Box::pin(async_io_write_file(args)) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Object, RuntimeError>> + Send + 'static>>)))))
}

pub fn io_append_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let mut args = args.into_iter();
    
    match (args.next(), args.next()) {
        (Some(Object::String(path)), Some(Object::String(content))) => {
             let result = std::fs::OpenOptions::new()
             .create(true)
             .append(true)
             .open(path)
             .and_then(|mut file| std::io::Write::write_all(&mut file, content.as_bytes()));

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

pub async fn async_io_append_file(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let mut args = args.into_iter();
    
    match (args.next(), args.next()) {
        (Some(Object::String(path)), Some(Object::String(content))) => {
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await
                .map_err(|e| RuntimeError::InvalidOperation(format!("Could not open file: {}", e)))?;
            
            file.write_all(content.as_bytes()).await
                .map_err(|e| RuntimeError::InvalidOperation(format!("Could not append to file: {}", e)))?;

            Ok(Object::Null)
        }
        (Some(Object::String(_)), Some(o)) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        (Some(o), _) => Err(RuntimeError::TypeMismatch { expected: "string".to_string(), got: o.type_name() }),
        _ => Err(RuntimeError::WrongNumberOfArguments { min: 2, max: 2, got: 0 }),
    }
}

pub fn io_append_file_async(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let args = args;
    Ok(Object::Future(Arc::new(Mutex::new(Some(Box::pin(async_io_append_file(args)) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Object, RuntimeError>> + Send + 'static>>)))))
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

            for entry in std::fs::read_dir(path).map_err(|e| RuntimeError::InvalidOperation(e.to_string()))? {
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

pub async fn async_io_list_dir(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(path)) => {
            let path = Path::new(path);

            if !path.is_dir() {
                return Err(RuntimeError::InvalidOperation(format!("'{}' is not a directory", path.display())));
            }

            let mut items: Vec<Object> = Vec::new();

            let mut dir = fs::read_dir(path).await
                .map_err(|e| RuntimeError::InvalidOperation(e.to_string()))?;
            
            while let Some(entry) = dir.next_entry().await
                .map_err(|e| RuntimeError::InvalidOperation(e.to_string()))? {
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

pub fn io_list_dir_async(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let args = args;
    Ok(Object::Future(Arc::new(Mutex::new(Some(Box::pin(async_io_list_dir(args)) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Object, RuntimeError>> + Send + 'static>>)))))
}
