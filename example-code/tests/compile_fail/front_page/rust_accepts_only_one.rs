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

// The front page's rejected block: the encoder pair without CGP. Everything above the end marker is
// exactly what the page shows, starting at line 1, so the line numbers in the compiler's error match
// the numbered block on the page. The import and `fn main` it omits come after it, which Rust allows.
use core::fmt::Display;

fn main() {}
