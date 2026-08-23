use core::fmt::Display;

pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}

// Legal on its own.
impl<T: Display> CanEncode for T {
    fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
}

// error[E0119]: conflicting implementations of trait `CanEncode`
impl<T: AsRef<[u8]>> CanEncode for T {
    fn encode(&self) -> Vec<u8> { self.as_ref().to_vec() }
}

fn main() {}
