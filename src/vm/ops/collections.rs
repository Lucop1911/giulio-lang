//! Collection operations: arrays, hashes, indexing.

use crate::vm::runtime::runtime_errors::RuntimeError;
use crate::vm::obj::{HashMap, Object};
use ahash::HashMapExt;

pub fn execute_build_array(stack: &mut Vec<Object>, count: u16) {
    let count = count as usize;
    if stack.len() < count {
        stack.push(Object::Error(RuntimeError::InvalidOperation(
            "Stack underflow on BuildArray".to_string(),
        )));
        return;
    }
    let elements: Vec<Object> = stack.drain(stack.len() - count..).collect();
    stack.push(Object::Array(elements));
}

pub fn execute_build_hash(stack: &mut Vec<Object>, pair_count: u16) {
    let pair_count = pair_count as usize;
    if stack.len() < pair_count * 2 {
        stack.push(Object::Error(RuntimeError::InvalidOperation(
            "Stack underflow on BuildHash".to_string(),
        )));
        return;
    }

    // Safe: only Integer, Boolean, String (immutable types) are allowed as keys,
    // validated at runtime before insertion.
    #[allow(clippy::mutable_key_type)]
    let mut hashmap = HashMap::new();
    for _ in 0..pair_count {
        let value = stack.pop().unwrap();
        let key = stack.pop().unwrap();
        match &key {
            Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                hashmap.insert(key, value);
            }
            Object::Error(e) => {
                stack.push(Object::Error(e.clone()));
                return;
            }
            _ => {
                stack.push(Object::Error(RuntimeError::NotHashable(key.type_name())));
                return;
            }
        }
    }
    stack.push(Object::Hash(hashmap));
}

pub fn execute_index(stack: &mut Vec<Object>) {
    let index = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on Index".to_string(),
            )))
        }
    };
    let collection = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on Index".to_string(),
            )))
        }
    };

    let result = match collection {
        Object::Array(arr) => match index {
            Object::Integer(i) => {
                if i < 0 {
                    Object::Error(RuntimeError::IndexOutOfBounds {
                        index: i,
                        length: arr.len(),
                    })
                } else {
                    let idx = i as usize;
                    if idx >= arr.len() {
                        Object::Error(RuntimeError::IndexOutOfBounds {
                            index: i,
                            length: arr.len(),
                        })
                    } else {
                        arr[idx].clone()
                    }
                }
            }
            _ => Object::Error(RuntimeError::InvalidOperation(
                "Array index must be an integer".to_string(),
            )),
        },
        Object::Hash(mut hash) => match index {
            Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                hash.remove(&index).unwrap_or(Object::Null)
            }
            _ => Object::Error(RuntimeError::NotHashable(index.type_name())),
        },
        other => Object::Error(RuntimeError::NotIndexable(other.type_name())),
    };

    stack.push(result);
}

pub fn execute_set_index(stack: &mut Vec<Object>) {
    let value = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on SetIndex".to_string(),
            )))
        }
    };
    let index = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on SetIndex".to_string(),
            )))
        }
    };
    let collection = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on SetIndex".to_string(),
            )))
        }
    };

    let result = match collection {
        Object::Array(mut arr) => match index {
            Object::Integer(i) => {
                if i < 0 {
                    Object::Error(RuntimeError::IndexOutOfBounds {
                        index: i,
                        length: arr.len(),
                    })
                } else {
                    let idx = i as usize;
                    if idx >= arr.len() {
                        Object::Error(RuntimeError::IndexOutOfBounds {
                            index: i,
                            length: arr.len(),
                        })
                    } else {
                        arr[idx] = value;
                        Object::Array(arr)
                    }
                }
            }
            _ => Object::Error(RuntimeError::InvalidOperation(
                "Array index must be an integer".to_string(),
            )),
        },
        Object::Hash(mut hash) => match index {
            Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                hash.insert(index, value);
                Object::Hash(hash)
            }
            _ => Object::Error(RuntimeError::NotHashable(index.type_name())),
        },
        other => Object::Error(RuntimeError::NotIndexable(other.type_name())),
    };

    stack.push(result);
}
