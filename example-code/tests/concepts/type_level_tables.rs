//! Code from `docs/concepts/type-level-tables.md` — *Look up a provider in a type-level table*.
//!
//! Module names follow the page's headings. The generated-code listings are assembled by hand in
//! [`the_wiring_table`] (the provider blanket impl with its table bound only) and
//! [`the_hidden_requirement`] (with the `IsProviderFor` bound as well); that checks their structure,
//! while `cargo cgp expand` establishes what the library generates. Rejected code lives in trybuild
//! fixtures.

/// ## A trait as a map
///
/// A trait with one associated type, implemented once per key, is a compile-time map. A second
/// entry for the same key is rejected:
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/type_level_tables_a_trait_as_a_map.rs`.
pub mod a_trait_as_a_map {
    use core::marker::PhantomData;

    pub trait HasEntry<Key> {
        type Value;
    }

    pub struct AreaTable;

    pub struct Rectangle;
    pub struct Circle;
    pub struct RectangleArea;
    pub struct CircleArea;

    impl HasEntry<Rectangle> for AreaTable {
        type Value = RectangleArea;
    }

    impl HasEntry<Circle> for AreaTable {
        type Value = CircleArea;
    }

    pub fn entry<Key>() -> PhantomData<<AreaTable as HasEntry<Key>>::Value>
    where
        AreaTable: HasEntry<Key>,
    {
        PhantomData
    }

    /// The bound reads an entry and the projection names its value.
    #[test]
    fn each_key_reads_its_entry() {
        let _: PhantomData<RectangleArea> = entry::<Rectangle>();
        let _: PhantomData<CircleArea> = entry::<Circle>();
    }
}

/// ## The wiring table
///
/// The same table written with `delegate_components!`, read through `DelegateComponent`; then the
/// call trace with the provider blanket impl shown with its table bound only, as the page shows it
/// before stating the `IsProviderFor` requirement. Its table entry is written by hand, as the page
/// spells it out.
pub mod the_wiring_table {
    use core::marker::PhantomData;

    use cgp::core::component::DelegateComponent;
    use cgp::prelude::*;

    pub struct Rectangle;
    pub struct Circle;
    pub struct RectangleArea;
    pub struct CircleArea;

    delegate_components! {
        new AreaTable {
            Rectangle: RectangleArea,
            Circle: CircleArea,
        }
    }

    pub fn entry<Key>() -> PhantomData<<AreaTable as DelegateComponent<Key>>::Delegate>
    where
        AreaTable: DelegateComponent<Key>,
    {
        PhantomData
    }

    #[test]
    fn delegate_components_writes_the_same_entries() {
        let _: PhantomData<RectangleArea> = entry::<Rectangle>();
        let _: PhantomData<CircleArea> = entry::<Circle>();
    }

    pub trait CanFetchStorageObject {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
    }

    pub trait StorageObjectFetcher<Context> {
        fn fetch_storage_object(context: &Context, object_id: &str) -> anyhow::Result<Vec<u8>>;
    }

    pub struct StorageObjectFetcherComponent;

    pub struct FetchS3Object;

    impl<Context> StorageObjectFetcher<Context> for FetchS3Object {
        fn fetch_storage_object(context: &Context, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Ok(b"s3".to_vec())
        }
    }

    pub struct App;

    impl DelegateComponent<StorageObjectFetcherComponent> for App {
        type Delegate = FetchS3Object;
    }

    impl<Context> CanFetchStorageObject for Context
    where
        Context: StorageObjectFetcher<Context>,
    {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Context::fetch_storage_object(self, object_id)
        }
    }

    impl<Context, Table> StorageObjectFetcher<Context> for Table
    where
        Table: DelegateComponent<StorageObjectFetcherComponent>,
        Table::Delegate: StorageObjectFetcher<Context>,
    {
        fn fetch_storage_object(context: &Context, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Table::Delegate::fetch_storage_object(context, object_id)
        }
    }

    #[test]
    fn a_call_resolves_through_the_entry() {
        assert_eq!(App.fetch_storage_object("alice.png").unwrap(), b"s3");
    }
}

/// ## The hidden requirement
///
/// The complete call trace, with the `IsProviderFor` bound the page states at the end of the lookup:
/// the wiring table, the consumer blanket impl, and the provider blanket impl, assembled into one
/// program. Because the last two are what `#[cgp_component]`
/// *generates*, they are written by hand here rather than derived, which is the point: the page
/// claims a component is these pieces, and a hand-rolled component built from exactly them resolves
/// a call and passes a check.
///
/// **This proves the listings are structurally sound, not that they are textually current.** The
/// authority on what the macro emits is `cargo cgp expand --lib --item …`, which is also what the
/// page's own note about reserved names and the spelled-out delegate refers to. Check the text
/// against `expand`; check the shape against this module.
pub mod the_hidden_requirement {
    use cgp::core::component::DelegateComponent;
    use cgp::core::component::IsProviderFor;
    use cgp::prelude::*;

    pub trait CanFetchStorageObject {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
    }

    pub trait StorageObjectFetcher<Context>:
        IsProviderFor<StorageObjectFetcherComponent, Context, ()>
    {
        fn fetch_storage_object(
            context: &Context,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>>;
    }

    // The key. The page calls it "a marker generated alongside the two traits".
    pub struct StorageObjectFetcherComponent;

    pub struct FetchS3Object;
    pub struct FetchLocalObject;

    impl<Context> StorageObjectFetcher<Context> for FetchS3Object {
        fn fetch_storage_object(context: &Context, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Ok(b"s3".to_vec())
        }
    }

    impl<Context> StorageObjectFetcher<Context> for FetchLocalObject {
        fn fetch_storage_object(context: &Context, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Ok(b"local".to_vec())
        }
    }

    // Provider macros generate these checking implementations with the provider's bounds.
    impl<Context> IsProviderFor<StorageObjectFetcherComponent, Context, ()> for FetchS3Object {}
    impl<Context> IsProviderFor<StorageObjectFetcherComponent, Context, ()> for FetchLocalObject {}

    pub struct App;
    pub struct TestApp;

    delegate_components! { App     { StorageObjectFetcherComponent: FetchS3Object } }
    delegate_components! { TestApp { StorageObjectFetcherComponent: FetchLocalObject } }

    // The first generated implementation: a context that implements the provider trait for itself
    // gets the consumer trait.
    impl<Context> CanFetchStorageObject for Context
    where
        Context: StorageObjectFetcher<Context>,
    {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Context::fetch_storage_object(self, object_id)
        }
    }

    // The second: anything with a table entry inherits the provider trait from what the entry names.
    impl<Context, Provider> StorageObjectFetcher<Context> for Provider
    where
        Provider: DelegateComponent<StorageObjectFetcherComponent>
            + IsProviderFor<StorageObjectFetcherComponent, Context, ()>,
        Provider::Delegate: StorageObjectFetcher<Context>,
    {
        fn fetch_storage_object(
            context: &Context,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            Provider::Delegate::fetch_storage_object(context, object_id)
        }
    }

    /// The three-step resolution the page traces, asserted as a bound rather than described.
    #[test]
    fn a_call_resolves_through_the_table() {
        fn resolves<Context: CanFetchStorageObject>() {}

        resolves::<App>();
        resolves::<TestApp>();

        assert_eq!(App.fetch_storage_object("alice.png").unwrap(), b"s3");
        assert_eq!(TestApp.fetch_storage_object("alice.png").unwrap(), b"local");
    }
}

/// ## Tables keyed by other types
///
/// The legacy nested table read by `UseDelegate`, then the same per-type choice with `open`.
pub mod tables_keyed_by_other_types {
    use cgp::prelude::*;

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    pub struct Circle {
        pub radius: f64,
    }

    pub mod legacy {
        use cgp::prelude::*;

        use super::{Circle, Rectangle};

        #[cgp_component(AreaCalculator)]
        #[derive_delegate(UseDelegate<Shape>)]
        pub trait CanCalculateArea<Shape> {
            fn area(&self, shape: &Shape) -> f64;
        }

        #[cgp_impl(new RectangleArea)]
        impl AreaCalculator<Rectangle> {
            fn area(&self, shape: &Rectangle) -> f64 {
                shape.width * shape.height
            }
        }

        #[cgp_impl(new CircleArea)]
        impl AreaCalculator<Circle> {
            fn area(&self, shape: &Circle) -> f64 {
                core::f64::consts::PI * shape.radius * shape.radius
            }
        }

        pub struct MyApp;

        delegate_components! {
            MyApp {
                AreaCalculatorComponent:
                    UseDelegate<new AreaCalculatorComponents {
                        Rectangle: RectangleArea,
                        Circle: CircleArea,
                    }>,
            }
        }

        check_components! {
            MyApp {
                AreaCalculatorComponent: [Rectangle, Circle],
            }
        }
    }

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator<Rectangle> {
        fn area(&self, shape: &Rectangle) -> f64 {
            shape.width * shape.height
        }
    }

    #[cgp_impl(new CircleArea)]
    impl AreaCalculator<Circle> {
        fn area(&self, shape: &Circle) -> f64 {
            core::f64::consts::PI * shape.radius * shape.radius
        }
    }

    pub struct MyApp;

    delegate_components! {
        MyApp {
            open AreaCalculatorComponent;

            @AreaCalculatorComponent.Rectangle: RectangleArea,
            @AreaCalculatorComponent.Circle: CircleArea,
        }
    }

    check_components! {
        MyApp {
            AreaCalculatorComponent: [Rectangle, Circle],
        }
    }

    #[test]
    fn each_shape_reaches_its_provider() {
        let rectangle = Rectangle { width: 3.0, height: 4.0 };
        assert_eq!(MyApp.area(&rectangle), 12.0);
        assert_eq!(legacy::CanCalculateArea::area(&legacy::MyApp, &rectangle), 12.0);
    }
}

/// ## Paths as keys
///
/// A shorter path key and a longer key beneath it overlap:
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/type_level_tables_paths_as_keys.rs`.
pub mod paths_as_keys {}

/// ## Nested tables and namespaces
///
/// A component registers its route in a namespace from the key's side with `#[prefix]`, and a
/// context that joins the namespace binds the provider at that path. The page's listing of the
/// registration impl is generated code; this module checks that the arrangement it describes
/// resolves.
pub mod nested_tables_and_namespaces {
    use cgp::prelude::*;

    cgp_namespace! { new AppNamespace {} }

    #[cgp_component(Greeter)]
    #[prefix(@app in AppNamespace)]
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
            namespace AppNamespace;

            @app.GreeterComponent: GreetHello,
        }
    }

    check_components! {
        App {
            GreeterComponent,
        }
    }

    #[test]
    fn the_route_reaches_the_bound_provider() {
        assert_eq!(App.greet(), "Hello!");
    }
}
