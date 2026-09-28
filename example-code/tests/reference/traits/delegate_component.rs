//! Code from `docs/reference/traits/wiring/delegate_component.md` — `DelegateComponent`.
//!
//! Pins the reading function from Usage and the Examples program: one component wired on a context,
//! and a table keyed on shapes whose entries a bound reads back. A duplicate entry and an unwired
//! component are trybuild fixtures.

/// ## Usage
///
/// Reading an entry is a bound plus a projection of `Delegate`.
pub mod usage {
    use cgp::prelude::*;

    pub fn provider_for<Table, Key>() -> PhantomData<<Table as DelegateComponent<Key>>::Delegate>
    where
        Table: DelegateComponent<Key>,
    {
        PhantomData
    }
}

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    use super::usage::provider_for;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "Hello!".to_owned()
        }
    }

    pub struct App;

    delegate_components! {
        App {
            GreeterComponent: GreetHello,
        }
    }

    check_components! {
        App {
            GreeterComponent,
        }
    }

    pub struct Rectangle;
    pub struct Circle;
    pub struct RectangleArea;
    pub struct CircleArea;

    delegate_components! {
        new AreaComponents {
            Rectangle: RectangleArea,
            Circle: CircleArea,
        }
    }

    pub fn demo() {
        assert_eq!(App.greet(), "Hello!");

        // Each read resolves to the entry's value at compile time.
        let _: PhantomData<GreetHello> = provider_for::<App, GreeterComponent>();
        let _: PhantomData<RectangleArea> = provider_for::<AreaComponents, Rectangle>();
        let _: PhantomData<CircleArea> = provider_for::<AreaComponents, Circle>();
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
