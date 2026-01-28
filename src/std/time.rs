use std::time::{SystemTime, UNIX_EPOCH};
use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};
use std::time::Duration;
use std::thread::sleep;

use crate::interpreter::obj::Object;

pub fn time_now(_: Vec<Object>) -> Result<Object, String> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => {
            Ok(Object::BigInteger(BigInt::from_u128(dur.as_millis()).unwrap()))
        }
        Err(_) => Ok(Object::BigInteger(BigInt::from(0))),
    }
}

pub fn time_sleep(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::Integer(i)) => {
            sleep(Duration::from_millis(*i as u64));
            Ok(Object::Null)
        }
        Some(Object::BigInteger(bi)) => {
            sleep(Duration::from_millis(bi.to_u64().unwrap_or(std::u64::MAX)));
            Ok(Object::Null)
        }
        _ => Err("sleep() expects an Integer or BigInteger".to_string())
    }
}