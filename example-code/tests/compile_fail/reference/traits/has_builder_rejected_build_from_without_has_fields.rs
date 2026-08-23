use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

#[derive(BuildField)]
pub struct FooBar {
    pub foo: u64,
}

#[derive(BuildField)]
pub struct FooBaz {
    pub foo: u64,
    pub baz: bool,
}

fn main() {
    // `build_from` needs `#[derive(HasFields)]` on the source, which `FooBar` lacks.
    let _ = FooBaz::builder()
        .build_from(FooBar { foo: 1 })
        .build_field(PhantomData::<Symbol!("baz")>, true)
        .finalize_build();
}
