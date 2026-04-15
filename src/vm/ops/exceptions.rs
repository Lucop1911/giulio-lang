//! Exception handling operations.

use crate::runtime::obj::Object;
use crate::runtime::runtime_errors::RuntimeError;

#[derive(Debug, Clone)]
pub struct ExceptionHandler {
    pub catch_addr: Option<u16>,
    pub finally_addr: Option<u16>,
}

pub fn execute_throw(stack: &mut Vec<Object>) -> ExecResult {
    if let Some(value) = stack.pop() {
        stack.push(Object::ThrownValue(Box::new(value)));
    } else {
        stack.push(Object::ThrownValue(Box::new(Object::Null)));
    }
    ExecResult::Throw
}

pub fn execute_push_catch(
    handlers: &mut Vec<ExceptionHandler>,
    catch_addr: u16,
    finally_addr: u16,
) {
    handlers.push(ExceptionHandler {
        catch_addr: if catch_addr == 0 {
            None
        } else {
            Some(catch_addr)
        },
        finally_addr: if finally_addr == 0 {
            None
        } else {
            Some(finally_addr)
        },
    });
}

pub fn execute_pop_catch(handlers: &mut Vec<ExceptionHandler>, _stack: &Vec<Object>) {
    if !handlers.is_empty() {
        handlers.pop();
    }
}

pub fn execute_push_finally(handlers: &mut Vec<ExceptionHandler>, addr: u16) {
    handlers.push(ExceptionHandler {
        catch_addr: None,
        finally_addr: Some(addr),
    });
}

pub fn execute_end_finally(
    handlers: &mut Vec<ExceptionHandler>,
    stack: &mut Vec<Object>,
    pending_return: &mut bool,
) -> ExecResult {
    // Check if there's a ThrownValue on the stack
    let should_rethrow = if let Some(obj) = stack.last() {
        matches!(obj, Object::ThrownValue(_))
    } else {
        false
    };

    handlers.pop();

    if should_rethrow {
        // Keep the ThrownValue on stack and return Throw
        return ExecResult::Throw;
    }

    // If a return was pending (from inside the finally block), do the return now
    if *pending_return {
        *pending_return = false;
        return ExecResult::Return;
    }

    ExecResult::Continue
}

pub fn handle_throw_result(
    stack: &mut Vec<Object>,
    handlers: &mut Vec<ExceptionHandler>,
    frames: &mut Vec<CallFrame>,
) -> Result<ExecResult, RuntimeError> {
    let thrown = match stack.pop() {
        Some(Object::ThrownValue(v)) => *v,
        Some(v) => Object::ThrownValue(Box::new(v)),
        None => Object::ThrownValue(Box::new(Object::Null)),
    };

    let handler = handlers.pop();
    match handler {
        Some(ExceptionHandler {
            catch_addr: Some(addr),
            ..
        }) => {
            if let Some(frame) = frames.last_mut() {
                frame.ip = addr as usize;
            }
            // Push the unwrapped value - catch block receives the actual thrown value
            stack.push(thrown);
            Ok(ExecResult::Continue)
        }
        Some(ExceptionHandler {
            finally_addr: Some(addr),
            ..
        }) => {
            if let Some(frame) = frames.last_mut() {
                frame.ip = addr as usize;
            }
            // Push the ThrownValue so finally can detect an exception occurred
            stack.push(Object::ThrownValue(Box::new(thrown)));
            Ok(ExecResult::Continue)
        }
        Some(ExceptionHandler { .. }) => {
            // Invalid handler - shouldn't happen, but treat as uncaught
            let msg = format!("{:?}", thrown);
            Err(RuntimeError::UncaughtException(msg))
        }
        None => {
            // Try to get a string representation of the thrown value
            let msg = match &thrown {
                Object::String(s) => s.clone(),
                Object::Integer(i) => i.to_string(),
                Object::Boolean(b) => b.to_string(),
                Object::Float(f) => f.to_string(),
                _ => format!("{:?}", thrown),
            };
            Err(RuntimeError::UncaughtException(msg))
        }
    }
}

use crate::vm::frame::CallFrame;
use crate::vm::vm::ExecResult;
