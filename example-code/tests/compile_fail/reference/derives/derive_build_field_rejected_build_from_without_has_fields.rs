use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

#[derive(BuildField)]
pub struct FooBar {
    pub foo: u64,
    pub bar: String,
}

#[derive(BuildField)]
pub struct FooBarBaz {
    pub foo: u64,
    pub bar: String,
    pub baz: bool,
}

fn main() {
    // `build_from` needs `#[derive(HasFields)]` on the source, which `FooBar` lacks.
    let source = FooBar {
        foo: 1,
        bar: "bar".to_owned(),
    };

    let _ = FooBarBaz::builder()
        .build_from(source)
        .build_field(PhantomData::<Symbol!("baz")>, true)
        .finalize_build();
}
