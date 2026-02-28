use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};
use std::time::Duration;
use tokio::time::sleep;

use crate::interpreter::obj::Object;
use crate::errors::RuntimeError;

pub fn time_now(_: Vec<Object>) -> Result<Object, RuntimeError> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => {
            Ok(Object::BigInteger(BigInt::from_u128(dur.as_millis()).unwrap()))
        }
        Err(_) => Ok(Object::BigInteger(BigInt::from(0))),
    }
}

pub async fn async_time_sleep(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Integer(i)) => {
            sleep(Duration::from_millis(*i as u64)).await;
            Ok(Object::Null)
        }
        Some(Object::BigInteger(bi)) => {
            sleep(Duration::from_millis(bi.to_u64().unwrap_or(std::u64::MAX))).await;
            Ok(Object::Null)
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "integer".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}

pub fn time_sleep_wrapper(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let args = args;
    Ok(Object::Future(Arc::new(Mutex::new(Some(Box::pin(async_time_sleep(args)) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Object, RuntimeError>> + Send + 'static>>)))))
}