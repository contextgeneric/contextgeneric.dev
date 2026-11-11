// The front page's rejected block: the encoder pair without CGP. The text between the markers is
// exactly what the page shows; the import and `fn main` are what it omits.
use core::fmt::Display;

// front-page:rejected-block:begin
// Rust allows only one of these. `String` is both `Display` and
// `AsRef<[u8]>`, so the compiler has no principled way to choose.
pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}

impl<T: Display> CanEncode for T {
    fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
}

impl<T: AsRef<[u8]>> CanEncode for T {          // error[E0119]
    fn encode(&self) -> Vec<u8> { self.as_ref().to_vec() }
}
// front-page:rejected-block:end

fn main() {}
