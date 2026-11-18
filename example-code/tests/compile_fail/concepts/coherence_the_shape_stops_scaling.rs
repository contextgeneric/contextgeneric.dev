use core::fmt::Display;

pub trait CanEncodeValue<Value> {
    fn encode(&self, value: &Value) -> Vec<u8>;
}

pub struct ApiServer;

impl<V: Display> CanEncodeValue<V> for ApiServer {
    fn encode(&self, value: &V) -> Vec<u8> { value.to_string().into_bytes() }
}

// error[E0119]: conflicting implementations of trait `CanEncodeValue<_>`
//               for type `ApiServer`
impl<V: AsRef<[u8]>> CanEncodeValue<V> for ApiServer {
    fn encode(&self, value: &V) -> Vec<u8> { value.as_ref().to_vec() }
}

fn main() {}
