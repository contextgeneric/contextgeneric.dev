use core::fmt::{self, Display, Formatter};

// error[E0117]: only traits defined in the current crate can be implemented
//               for types defined outside of the crate
impl Display for Vec<u8> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { Ok(()) }
}

fn main() {}
