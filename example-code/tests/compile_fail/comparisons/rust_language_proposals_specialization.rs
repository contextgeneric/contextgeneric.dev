// RFC 1210's own example. Specialization has never stabilized, so the `default` item is rejected
// on a stable toolchain.
use core::ops::{Add, AddAssign};

pub trait MyAddAssign<Rhs> {
    fn my_add_assign(&mut self, rhs: Rhs);
}

impl<R, T: Add<R, Output = T> + Clone> MyAddAssign<R> for T {
    default fn my_add_assign(&mut self, rhs: R) {
        let tmp = self.clone() + rhs;
        *self = tmp;
    }
}

fn main() {}
