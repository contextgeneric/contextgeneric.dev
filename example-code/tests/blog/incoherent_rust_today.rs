//! Compiled counterparts of `blog/2026-11-25-incoherent-rust-today.md`, the incoherent-Rust post.
//! Modules follow the post's headings. Snippets written in Incoherent Rust or TIR syntax are
//! pseudocode and have no counterpart; the struct-binding type error is a compile-fail fixture
//! under `tests/compile_fail/blog/`.
#![allow(dead_code, unused_variables)]

mod bypassing_overlapping_implementations {
    use cgp::prelude::*;
    use core::fmt::{Display, Error, Formatter};
    use core::hash::Hasher;
    #[cgp_component(HashImpl)]
    pub trait Hash {
        fn hash<H: Hasher>(&self, state: &mut H);
    }

    #[cgp_impl(new HashWithDisplay)]
    #[uses(Display)]
    impl HashImpl {
        fn hash<H: Hasher>(&self, state: &mut H) {
            core::hash::Hash::hash(&self.to_string(), state)
        }
    }

    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    impl Display for Person {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            write!(f, "{} {}", self.first_name, self.last_name)
        }
    }

    delegate_components! {
        Person {
            HashImplComponent: HashWithDisplay,
        }
    }

    check_components! {
        Person {
            HashImplComponent,
        }
    }

    #[test]
    fn person_hashes_like_its_display_string() {
        use std::hash::{DefaultHasher, Hash as _, Hasher as _};

        let person = Person {
            first_name: "John".to_owned(),
            last_name: "Smith".to_owned(),
        };

        let mut by_person = DefaultHasher::new();
        Hash::hash(&person, &mut by_person);

        let mut by_string = DefaultHasher::new();
        "John Smith".to_owned().hash(&mut by_string);

        assert_eq!(by_person.finish(), by_string.finish());
    }
}

mod bypassing_orphan_implementations {
    use cgp::prelude::*;
    use core::fmt::{Display, Error, Formatter};
    use serde::Serializer;

    #[cgp_component(SerializeImpl)]
    pub trait Serialize<T> {
        fn serialize<S: Serializer>(
            &self,
            value: &T,
            serializer: S,
        ) -> Result<S::Ok, S::Error>;
    }

    #[cgp_impl(new SerializeWithDisplay)]
    impl<T: Display> SerializeImpl<T> {
        fn serialize<S: Serializer>(
            &self,
            value: &T,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            serde::Serialize::serialize(&value.to_string(), serializer)
        }
    }

    // `Person` as the post defines it in the previous section.
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    impl Display for Person {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            write!(f, "{} {}", self.first_name, self.last_name)
        }
    }

    // inside a third party crate
    pub struct MyApp;

    delegate_components! {
        MyApp {
            open SerializeImplComponent;

            @SerializeImplComponent.Person: SerializeWithDisplay,
        }
    }

    check_components! {
        MyApp {
            SerializeImplComponent: Person,
        }
    }
}

mod incoherent_traits_and_named_impls {
    // One module covers "Incoherent traits" and "Named impls", which build one program.
    // The post places these items in four crates; modules stand in for them.
    pub mod a {
        use cgp::prelude::*;

        #[cgp_component(SerializeImpl)]
        pub trait Serialize<T> {
            fn serialize(&self, value: &T) -> String;
        }
    }

    pub mod b {
        pub struct Matrix(pub Vec<Vec<f64>>);
    }

    pub mod c {
        use super::{a, b};
        use cgp::prelude::*;

        #[cgp_impl(new CSerialize)]
        impl a::SerializeImpl<b::Matrix> {
            fn serialize(&self, value: &b::Matrix) -> String {
                // The post elides this body.
                format!("{:?}", value.0)
            }
        }
    }

    pub mod d {
        use super::{a, b};
        use cgp::prelude::*;

        #[cgp_impl(new DSerialize)]
        impl a::SerializeImpl<b::Matrix> {
            fn serialize(&self, value: &b::Matrix) -> String {
                // The post elides this body.
                format!("{} rows", value.0.len())
            }
        }
    }
}

mod using_incoherent_trait_bounds {
    use cgp::prelude::*;

    #[cgp_component(NameImpl)]
    pub trait Name<T> {
        fn name() -> &'static str;
    }

    #[cgp_impl(new DummyName)]
    impl<T> NameImpl<T> {
        fn name() -> &'static str {
            "dummy"
        }
    }

    #[cgp_impl(new RealName)]
    impl<T> NameImpl<T> {
        fn name() -> &'static str {
            core::any::type_name::<T>()
        }
    }

    pub struct Foo<T>(pub T);

    #[cgp_fn]
    #[uses(Name<T>)]
    pub fn do_stuff<T>(&self, foo: &Foo<T>) {
        println!("{}", Self::name());
    }

    #[test]
    fn each_context_names_the_type_its_own_way() {
        let foo = Foo(1);

        {
            struct Context;

            delegate_components! {
                Context {
                    NameImplComponent: DummyName,
                }
            }

            // prints "dummy"
            Context.do_stuff(&foo);
            assert_eq!(<Context as Name<i32>>::name(), "dummy");
        }

        {
            struct Context;

            delegate_components! {
                Context {
                    NameImplComponent: RealName,
                }
            }

            // prints "i32"
            Context.do_stuff(&foo);
            assert_eq!(<Context as Name<i32>>::name(), "i32");
        }
    }
}

mod use_of_incoherent_trait_bounds_within_a_struct {
    use cgp::prelude::*;
    use core::marker::PhantomData;

    // `Name`, `DummyName`, and `RealName` as the previous section defines them.
    #[cgp_component(NameImpl)]
    pub trait Name<T> {
        fn name() -> &'static str;
    }

    #[cgp_impl(new DummyName)]
    impl<T> NameImpl<T> {
        fn name() -> &'static str {
            "dummy"
        }
    }

    #[cgp_impl(new RealName)]
    impl<T> NameImpl<T> {
        fn name() -> &'static str {
            core::any::type_name::<T>()
        }
    }

    pub struct Foo<Context: ?Sized, T> {
        pub value: T,
        pub phantom: PhantomData<Context>,
    }

    #[cgp_fn]
    #[uses(Name<T>)]
    pub fn do_stuff<T>(&self, foo: &Foo<Self, T>) {
        println!("{}", Self::name());
    }

    pub struct ContextA;

    delegate_components! {
        ContextA {
            NameImplComponent: DummyName,
        }
    }

    pub struct ContextB;

    delegate_components! {
        ContextB {
            NameImplComponent: RealName,
        }
    }

    #[test]
    fn the_struct_keeps_its_context() {
        let foo: Foo<ContextA, i32> = Foo {
            value: 1,
            phantom: PhantomData,
        };

        ContextA.do_stuff(&foo);
        // `ContextB.do_stuff(&foo)` is the compile-fail fixture
        // `tests/compile_fail/blog/incoherent_rust_today_struct_context.rs`.
    }
}

mod impl_specific_bindings_within_a_struct {
    use cgp::prelude::*;
    use core::marker::PhantomData;

    // `Name` and `RealName` as the earlier sections define them.
    #[cgp_component(NameImpl)]
    pub trait Name<T> {
        fn name() -> &'static str;
    }

    #[cgp_impl(new RealName)]
    impl<T> NameImpl<T> {
        fn name() -> &'static str {
            core::any::type_name::<T>()
        }
    }

    pub struct Foo<NameProvider, T> {
        pub value: T,
        pub phantom: PhantomData<NameProvider>,
    }

    #[cgp_fn]
    #[use_provider(NameProvider: NameImpl<T>)]
    pub fn do_stuff<NameProvider, T>(&self, foo: &Foo<NameProvider, T>) {
        println!("{}", NameProvider::name());
    }

    #[test]
    fn any_context_uses_the_struct_s_provider() {
        pub struct ContextA;
        pub struct ContextB;

        let foo: Foo<RealName, i32> = Foo {
            value: 1,
            phantom: PhantomData,
        };

        // Both contexts can call do_stuff on the same foo value,
        // because the Name implementation is fixed by RealName, not by the context.
        ContextA.do_stuff(&foo);
        ContextB.do_stuff(&foo);
    }
}

mod context_aware_dictionaries {
    use cgp::prelude::*;

    #[cgp_component(CloneImpl)]
    trait Clone<T> {
        fn clone(&self, value: &T) -> T;
    }

    #[cgp_impl(new CloneU32)]
    impl CloneImpl<u32> {
        fn clone(&self, value: &u32) -> u32 {
            // The post elides this body.
            *value
        }
    }

    #[cgp_impl(new CloneVec)]
    #[uses(Clone<T>)]
    impl<T> CloneImpl<Vec<T>> {
        fn clone(&self, values: &Vec<T>) -> Vec<T> {
            values.iter().map(|value| self.clone(value)).collect()
        }
    }

    #[test]
    fn the_context_clones_a_vec() {
        struct Context;

        delegate_components! {
            Context {
                open CloneImplComponent;

                @CloneImplComponent.u32: CloneU32,
                @CloneImplComponent.Vec<u32>: CloneVec,
            }
        }

        check_components! {
            Context {
                CloneImplComponent: [u32, Vec<u32>],
            }
        }

        let my_vec: Vec<u32> = vec![0, 1, 2];
        let cloned_vec = Context.clone(&my_vec);

        assert_eq!(cloned_vec, my_vec);
    }
}

mod context_and_capabilities {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::prelude::*;
    use cgp_error_anyhow::{RaiseAnyhowError, UseAnyhowError};
    use cgp_serde::components::{CanDeserializeValue, ValueDeserializer, ValueDeserializerComponent};
    use cgp_serde::providers::{DeserializeExtend, DeserializeRecordFields, UseSerde};
    use cgp_serde_json::impls::CanDeserializeJsonString;
    use typed_arena::Arena;

    #[derive(Debug, PartialEq, CgpData)]
    pub struct Foo {
        pub id: u64,
    }

    #[cgp_impl(new DeserializeFooRef)]
    #[uses(CanDeserializeValue<'de, Foo>)]
    impl<'de, 'a> ValueDeserializer<'de, &'a Foo> {
        fn deserialize<D>(
            &self,
            #[implicit] arena: &&'a Arena<Foo>,
            deserializer: D,
        ) -> Result<&'a Foo, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let foo = self.deserialize(deserializer)?;
            Ok(arena.alloc(foo))
        }
    }

    #[derive(HasField)]
    pub struct Context<'a> {
        pub arena: &'a Arena<Foo>,
    }

    delegate_components! {
        <'a> Context<'a> {
            open ValueDeserializerComponent;

            ErrorTypeProviderComponent:
                UseAnyhowError,
            ErrorRaiserComponent:
                RaiseAnyhowError,
            @ValueDeserializerComponent.u64:
                UseSerde,
            @ValueDeserializerComponent.Foo:
                DeserializeRecordFields,
            @ValueDeserializerComponent.<'b> &'b Foo:
                DeserializeFooRef,
            @ValueDeserializerComponent.<'b> Vec<&'b Foo>:
                DeserializeExtend,
        }
    }

    #[test]
    fn the_context_allocates_into_its_arena() -> Result<(), anyhow::Error> {
        let arena = Arena::new();
        let context = Context { arena: &arena };

        let foos: Vec<&Foo> = context.deserialize_json_string(r#"[{"id": 1}, {"id": 2}]"#)?;

        assert_eq!(foos, vec![&Foo { id: 1 }, &Foo { id: 2 }]);
        Ok(())
    }
}

mod multiple_implementations_on_the_same_name {
    use cgp::prelude::*;

    pub struct Matrix;

    // The two components as the post's earlier sections define them, reduced to strings.
    #[cgp_component(SerializeImpl)]
    pub trait Serialize<T> {
        fn serialize(&self, value: &T) -> String;
    }

    #[cgp_component(DeserializeImpl)]
    pub trait Deserialize<T> {
        fn deserialize(&self, serialized: &str) -> T;
    }

    // A provider is just a struct
    pub struct MyMatrixSerde;

    #[cgp_impl(MyMatrixSerde)]
    impl SerializeImpl<Matrix> {
        fn serialize(&self, value: &Matrix) -> String {
            // The post elides this body.
            "matrix".to_owned()
        }
    }

    #[cgp_impl(MyMatrixSerde)]
    impl DeserializeImpl<Matrix> {
        fn deserialize(&self, serialized: &str) -> Matrix {
            // The post elides this body.
            Matrix
        }
    }
}

mod grouping_named_impls {
    use cgp::prelude::*;

    pub struct Matrix;
    pub struct Scalar;

    #[cgp_component(SerializeImpl)]
    pub trait Serialize<T> {
        fn serialize(&self, value: &T) -> String;
    }

    #[cgp_component(DeserializeImpl)]
    pub trait Deserialize<T> {
        fn deserialize(&self, serialized: &str) -> T;
    }

    // The four providers the post names without defining.
    #[cgp_impl(new MyMatrixSerialize)]
    impl SerializeImpl<Matrix> {
        fn serialize(&self, value: &Matrix) -> String {
            "matrix".to_owned()
        }
    }

    #[cgp_impl(new MyMatrixDeserialize)]
    impl DeserializeImpl<Matrix> {
        fn deserialize(&self, serialized: &str) -> Matrix {
            Matrix
        }
    }

    #[cgp_impl(new MyScalarSerialize)]
    impl SerializeImpl<Scalar> {
        fn serialize(&self, value: &Scalar) -> String {
            "scalar".to_owned()
        }
    }

    #[cgp_impl(new MyScalarDeserialize)]
    impl DeserializeImpl<Scalar> {
        fn deserialize(&self, serialized: &str) -> Scalar {
            Scalar
        }
    }

    delegate_components! {
        new MyAppSerde {
            open { SerializeImplComponent, DeserializeImplComponent };

            @SerializeImplComponent.Matrix:
                MyMatrixSerialize,
            @DeserializeImplComponent.Matrix:
                MyMatrixDeserialize,
            @SerializeImplComponent.Scalar:
                MyScalarSerialize,
            @DeserializeImplComponent.Scalar:
                MyScalarDeserialize,
        }
    }

    // A context that delegates to the aggregate provider checks it.
    pub struct App;

    delegate_components! {
        App {
            [SerializeImplComponent, DeserializeImplComponent]: MyAppSerde,
        }
    }

    check_components! {
        App {
            SerializeImplComponent: [Matrix, Scalar],
            DeserializeImplComponent: [Matrix, Scalar],
        }
    }
}

mod higher_order_providers {
    use cgp::prelude::*;
    use core::marker::PhantomData;

    #[cgp_component(SerializeImpl)]
    pub trait Serialize<T> {
        fn serialize(&self, value: &T) -> String;
    }

    // A higher-order provider is just a struct with phantom generic parameters
    pub struct SerializeIterator<ItemSerializer>(pub PhantomData<ItemSerializer>);

    #[cgp_impl(SerializeIterator<ItemSerializer>)]
    // Inner provider import
    #[use_provider(ItemSerializer: SerializeImpl<T::Item>)]
    impl<T, ItemSerializer> SerializeImpl<T>
    where
        T: IntoIterator,
    {
        fn serialize(&self, value: &T) -> String {
            // The post elides this body.
            String::new()
        }
    }
}

mod type_level_dsls {
    use hypershell::prelude::*;

    // A Hypershell program is just a type containing higher order providers
    pub type Program = Pipe<Product![
        SimpleExec<
            StaticArg<Symbol!("echo")>,
            WithStaticArgs<Product![
                Symbol!("hello"),
                Symbol!("world!"),
            ]>,
        >,
        StreamToStdout,
    ]>;
}

mod incoherent_functions {
    use cgp::prelude::*;

    // `Name` and `Foo` as the earlier sections define them.
    #[cgp_component(NameImpl)]
    pub trait Name<T> {
        fn name() -> &'static str;
    }

    #[cgp_impl(new DummyName)]
    impl<T> NameImpl<T> {
        fn name() -> &'static str {
            "dummy"
        }
    }

    pub struct Foo<T>(pub T);

    #[cgp_fn]
    #[uses(Name<T>)]
    pub fn do_stuff<T>(&self, foo: &Foo<T>) {
        println!("{}", Self::name());
    }

    #[cgp_fn]
    #[uses(DoStuff<T>)]
    pub fn do_more_stuff<T>(&self, foo: &Foo<T>) {
        self.do_stuff(foo);
    }

    /// The post's hand-written rendering of what the two functions expand to.
    mod expansion {
        use super::{Foo, Name};

        pub trait DoStuff<T> {
            fn do_stuff(&self, foo: &Foo<T>);
        }

        impl<Context, T> DoStuff<T> for Context
        where
            Context: Name<T>,
        {
            fn do_stuff(&self, foo: &Foo<T>) {
                println!("{}", Self::name());
            }
        }

        pub trait DoMoreStuff<T> {
            fn do_more_stuff(&self, foo: &Foo<T>);
        }

        impl<Context, T> DoMoreStuff<T> for Context
        where
            Context: DoStuff<T>,
        {
            fn do_more_stuff(&self, foo: &Foo<T>) {
                self.do_stuff(foo);
            }
        }
    }

    #[test]
    fn the_caller_never_names_the_inner_bound() {
        struct Context;

        delegate_components! {
            Context {
                NameImplComponent: DummyName,
            }
        }

        Context.do_more_stuff(&Foo(1));
    }
}

mod zero_arity_traits {
    use cgp::prelude::*;

    #[cgp_component(GreetImpl)]
    pub trait Greet {
        fn greet(&self);
    }

    #[cgp_impl(new GreetHello)]
    impl GreetImpl {
        fn greet(&self, #[implicit] name: &String) {
            println!("Hello, {}!", name);
        }
    }

    // A context for the provider, which the post leaves out.
    #[derive(HasField)]
    pub struct Context {
        pub name: String,
    }

    delegate_components! {
        Context {
            GreetImplComponent: GreetHello,
        }
    }

    check_components! {
        Context {
            GreetImplComponent,
        }
    }
}

mod abstract_types {
    use cgp::prelude::*;
    use core::fmt::Display;

    // `Greet` as the previous section defines it.
    #[cgp_component(GreetImpl)]
    pub trait Greet {
        fn greet(&self);
    }

    #[cgp_type]
    pub trait HasNameType {
        type Name: Display;
    }

    #[cgp_impl(new GreetHello)]
    #[use_type(HasNameType.Name)]
    impl GreetImpl {
        fn greet(&self, #[implicit] name: &Name) {
            println!("Hello, {}!", name);
        }
    }

    pub struct FullName {
        pub first_name: String,
        pub last_name: String,
    }

    impl Display for FullName {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{} {}", self.first_name, self.last_name)
        }
    }

    #[derive(HasField)]
    pub struct Context {
        pub name: FullName,
    }

    delegate_components! {
        Context {
            NameTypeProviderComponent:
                UseType<FullName>,
            GreetImplComponent:
                GreetHello,
        }
    }

    check_components! {
        Context {
            GreetImplComponent,
        }
    }

    #[test]
    fn the_context_greets_with_its_full_name() {
        let context = Context {
            name: FullName {
                first_name: "John".to_owned(),
                last_name: "Smith".to_owned(),
            },
        };

        // Prints "Hello, John Smith!"
        context.greet();
    }
}

mod using_abstract_types_across_incoherent_traits {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasTimeType {
        type Time;
    }

    #[cgp_component(CurrentTimeImpl)]
    #[use_type(HasTimeType.Time)]
    pub trait CurrentTime {
        fn current_time(&self) -> Time;
    }

    #[cgp_component(UpdateLastModifiedImpl)]
    #[use_type(HasTimeType.Time)]
    pub trait UpdateLastModified<T> {
        fn update_last_modified(value: &mut T, time: Time);
    }

    #[cgp_fn]
    #[uses(CurrentTime, UpdateLastModified<T>)]
    pub fn touch<T>(&self, value: &mut T) {
        let time = self.current_time();
        Self::update_last_modified(value, time);
    }

    // A context the post leaves out: a fixed clock and a file whose time it updates.
    pub struct File {
        pub last_modified: u64,
    }

    #[cgp_impl(new FixedClock)]
    #[use_type(HasTimeType.{Time = u64})]
    impl CurrentTimeImpl {
        fn current_time(&self) -> u64 {
            42
        }
    }

    #[cgp_impl(new UpdateFile)]
    #[use_type(HasTimeType.{Time = u64})]
    impl UpdateLastModifiedImpl<File> {
        fn update_last_modified(value: &mut File, time: u64) {
            value.last_modified = time;
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open UpdateLastModifiedImplComponent;

            TimeTypeProviderComponent: UseType<u64>,
            CurrentTimeImplComponent: FixedClock,
            @UpdateLastModifiedImplComponent.File: UpdateFile,
        }
    }

    check_components! {
        App {
            CurrentTimeImplComponent,
            UpdateLastModifiedImplComponent: File,
        }
    }

    #[test]
    fn touch_sets_the_context_s_time() {
        let mut file = File { last_modified: 0 };
        App.touch(&mut file);
        assert_eq!(file.last_modified, 42);
    }
}

mod contexts_as_explicit_types {
    use cgp::prelude::*;

    // Stand-ins for the clients, domain types, and operation the post names without defining.
    pub struct Database;
    pub struct S3Client;
    pub struct RedisClient;
    pub struct UserId(pub u64);
    pub struct User;
    #[derive(Debug)]
    pub struct Error;
    pub struct Config;

    #[cgp_component(UserQuerier)]
    pub trait CanQueryUser {
        fn query_user(&self, user_id: &UserId) -> Result<User, Error>;
    }

    #[cgp_impl(new QueryUserWithDatabase)]
    impl UserQuerier {
        fn query_user(&self, #[implicit] database: &Database, user_id: &UserId) -> Result<User, Error> {
            // The post elides this body.
            Ok(User)
        }
    }

    #[cgp_impl(new CachedQueryUser<InnerQuerier>)]
    #[use_provider(InnerQuerier: UserQuerier)]
    impl<InnerQuerier> UserQuerier {
        fn query_user(&self, #[implicit] redis_cache: &RedisClient, user_id: &UserId) -> Result<User, Error> {
            // The post elides this body: a cache lookup before the inner query.
            InnerQuerier::query_user(self, user_id)
        }
    }

    #[derive(HasField)]
    pub struct App {
        // Runtime values accessible by providers
        pub database: Database,
        pub s3_client: S3Client,
        pub redis_cache: RedisClient,
    }

    delegate_components! {
        App {
            // Use higher-order provider to provide caching middlewares
            UserQuerierComponent:
                CachedQueryUser<QueryUserWithDatabase>,
        }
    }

    check_components! {
        App {
            UserQuerierComponent,
        }
    }

    // App is used in the return type
    pub async fn build_app_from_config(config: &Config) -> Result<App, Error> {
        // The post elides this body.
        Ok(App {
            database: Database,
            s3_client: S3Client,
            redis_cache: RedisClient,
        })
    }
}
