use cgp::core::field::traits::{TransformMap, TransformMapFields};
use cgp::prelude::*;

// Only the `IsNothing` case, so a field that is already present has no conversion.
pub struct OnlyAbsent;

impl<T: Default> TransformMap<IsNothing, IsPresent, T> for OnlyAbsent {
    fn transform_mapped(_value: ()) -> T {
        T::default()
    }
}

#[derive(CgpData)]
pub struct Config {
    pub port: u16,
    pub verbose: bool,
}

fn main() {
    let partial = Config::builder().build_field(PhantomData::<Symbol!("port")>, 8080);
    let _ = TransformMapFields::<OnlyAbsent, IsPresent>::transform_map_fields(partial);
}
