//! Code from `docs/reference/attributes/use_type.md` — *`#[use_type]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/attributes/`. Type
//! equalities are checked by assigning one `PhantomData` to a binding typed with the other.

/// ## Overview
///
/// The error-type import, on a component whose context picks `String` as its error type.
pub mod overview {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    #[use_type(HasErrorType.Error)]
    pub trait CanGreet {
        fn greet(&self) -> Result<String, Error>;
    }

    #[cgp_impl(new GreetOrFail)]
    #[use_type(HasErrorType.Error)]
    impl Greeter {
        fn greet(&self) -> Result<String, Error> {
            Ok("hello".to_owned())
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            GreeterComponent: GreetOrFail,
        }
    }

    check_components! {
        App {
            GreeterComponent,
        }
    }

    #[test]
    fn the_method_names_only_the_bare_error() {
        assert_eq!(App.greet(), Ok("hello".to_owned()));
    }
}

/// ## Usage, Importing several types, Pinning a type to a concrete one, and Importing from another
/// type
///
/// One function per form the sections name. The page names the traits without declaring them.
pub mod usage {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar;
    }

    pub mod errors {
        pub use cgp::prelude::HasErrorType;
    }

    pub trait HasFooType<X> {
        type Foo;
        type Bar;
    }

    #[cgp_type]
    pub trait HasUserIdType {
        type UserId;
    }

    #[cgp_type]
    pub trait HasCurrencyType {
        type Currency;
    }

    #[cgp_type]
    pub trait HasPasswordType {
        type Password;
    }

    #[cgp_type]
    pub trait HasHashedPasswordType {
        type HashedPassword;
    }

    #[cgp_type]
    pub trait HasDbType {
        type Db;
    }

    pub trait HasPoolType<Db> {
        type Pool;
    }

    #[cgp_type]
    pub trait HasTransactionType {
        type Transaction;
    }

    pub struct Tx<Db>(pub Db);

    pub struct X;
    pub struct Y;

    #[cgp_fn]
    #[use_type(HasScalarType.Scalar)]
    pub fn scalar(&self, value: Scalar) -> Scalar {
        value
    }

    #[cgp_fn]
    #[use_type(errors::HasErrorType.Error)]
    pub fn error_by_path(&self, error: Error) -> Error {
        error
    }

    #[cgp_fn]
    #[use_type(HasFooType<X>.Foo)]
    pub fn foo_of_x(&self, foo: Foo) -> Foo {
        foo
    }

    #[cgp_fn]
    #[use_type(HasUserIdType.UserId, HasCurrencyType.Currency, HasErrorType.Error)]
    pub fn charge(&self, _user: UserId, _amount: Currency) -> Result<(), Error> {
        Ok(())
    }

    #[cgp_fn]
    #[use_type(HasFooType<X>.{Foo, Bar as Baz})]
    pub fn renamed(&self, foo: Foo, baz: Baz) -> (Foo, Baz) {
        (foo, baz)
    }

    #[cgp_fn]
    #[use_type(HasFooType<X>.{Foo as FooX}, HasFooType<Y>.{Foo as FooY})]
    pub fn two_instantiations(&self, x: FooX, y: FooY) -> (FooX, FooY) {
        (x, y)
    }

    #[cgp_fn]
    #[use_type(HasScalarType.{})]
    pub fn bound_only(&self) -> <Self as HasScalarType>::Scalar
    where
        <Self as HasScalarType>::Scalar: Default,
    {
        Default::default()
    }

    #[cgp_fn]
    #[use_type(HasUserIdType.UserId)]
    #[use_type(HasCurrencyType.Currency)]
    pub fn stacked(&self, user: UserId, amount: Currency) -> (UserId, Currency) {
        (user, amount)
    }

    #[cgp_fn]
    #[use_type(HasErrorType.{Error = String})]
    pub fn pinned(&self) -> Error {
        "pinned".to_owned()
    }

    #[cgp_fn]
    #[use_type(HasFooType<u8>.{Foo = u32})]
    pub fn pinned_generic(&self) -> Foo {
        7
    }

    #[cgp_fn]
    #[use_type(HasPasswordType.Password, HasHashedPasswordType.{HashedPassword = Password})]
    pub fn hash(&self, password: Password) -> HashedPassword {
        password
    }

    #[cgp_fn]
    #[use_type(HasDbType.Db, HasTransactionType.{Transaction = Tx<Db>})]
    pub fn begin(&self, db: Db) -> Transaction {
        Tx(db)
    }

    #[cgp_fn]
    #[use_type(HasScalarType.Scalar in Types)]
    pub fn scalar_of<Types>(&self, value: Scalar) -> Scalar {
        value
    }

    #[cgp_fn]
    #[use_type(HasDbType.Db, HasPoolType<Db>.Pool)]
    pub fn pool(&self, pool: Pool) -> Pool {
        pool
    }

    pub struct App;

    delegate_components! {
        App {
            ScalarTypeProviderComponent: UseType<f64>,
            cgp::core::error::ErrorTypeProviderComponent: UseType<String>,
            UserIdTypeProviderComponent: UseType<u64>,
            CurrencyTypeProviderComponent: UseType<u32>,
            PasswordTypeProviderComponent: UseType<String>,
            HashedPasswordTypeProviderComponent: UseType<String>,
            DbTypeProviderComponent: UseType<u8>,
            TransactionTypeProviderComponent: UseType<Tx<u8>>,
        }
    }

    impl<X> HasFooType<X> for App {
        type Foo = u32;
        type Bar = bool;
    }

    impl HasPoolType<u8> for App {
        type Pool = &'static str;
    }

    #[test]
    fn every_form_resolves_against_the_wiring() {
        assert_eq!(App.scalar(1.5), 1.5);
        assert_eq!(App.error_by_path("e".to_owned()), "e");
        assert_eq!(App.foo_of_x(3), 3);
        assert_eq!(App.charge(1, 2), Ok(()));
        assert_eq!(App.renamed(1, true), (1, true));
        assert_eq!(App.two_instantiations(1, 2), (1, 2));
        assert_eq!(App.bound_only(), 0.0);
        assert_eq!(App.stacked(1, 2), (1, 2));
        assert_eq!(App.pinned(), "pinned");
        assert_eq!(App.pinned_generic(), 7);
        assert_eq!(App.hash("pw".to_owned()), "pw");
        assert_eq!(App.begin(4).0, 4);
        assert_eq!(ScalarOf::<App>::scalar_of(&App, 2.5), 2.5);
        assert_eq!(App.pool("main"), "main");
    }
}

/// ## Examples
pub mod examples {
    use core::ops::Mul;

    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar: Clone + Mul<Output = Self::Scalar>;
    }

    #[cgp_component(AreaCalculator)]
    #[use_type(HasScalarType.Scalar)]
    pub trait CanCalculateArea {
        fn area(&self) -> Scalar;
    }

    #[cgp_impl(new RectangleArea)]
    #[use_type(HasScalarType.Scalar)]
    impl AreaCalculator {
        fn area(&self, #[implicit] width: Scalar, #[implicit] height: Scalar) -> Scalar {
            width * height
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! {
        Rectangle {
            ScalarTypeProviderComponent: UseType<f64>,
            AreaCalculatorComponent: RectangleArea,
        }
    }

    check_components! {
        Rectangle {
            AreaCalculatorComponent,
        }
    }

    #[cgp_component(Loader)]
    #[use_type(HasErrorType.Error)]
    pub trait CanLoad {
        fn load(&self, path: &str) -> Result<String, Error>;
    }

    #[test]
    fn the_context_picks_the_scalar() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(rect.area(), 6.0);
    }
}

/// ## When to use it
///
/// A construct's own associated type stays qualified beside a bare imported one.
pub mod when_to_use_it {
    use cgp::prelude::*;

    #[cgp_component(Parser)]
    #[use_type(HasErrorType.Error)]
    pub trait CanParse {
        type Output;

        fn parse(&self, input: &str) -> Result<Self::Output, Error>;
    }
}

/// ## Under the hood
///
/// The listing's input and its hand-written expansion, the foreign-context trait, the
/// expression-path rewrite and its unit-struct boundary, the header rewrite, and a chain written
/// back to front.
pub mod under_the_hood {
    use core::ops::Mul;

    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar: Clone + Mul<Output = Self::Scalar>;
    }

    #[cgp_fn]
    #[use_type(HasScalarType.Scalar)]
    pub fn area(&self, #[implicit] width: Scalar, #[implicit] height: Scalar) -> Scalar {
        width * height
    }

    pub trait AreaExpanded: HasScalarType {
        fn area_expanded(&self) -> <Self as HasScalarType>::Scalar;
    }

    impl<__Context__> AreaExpanded for __Context__
    where
        Self: HasField<Symbol!("width"), Value = <Self as HasScalarType>::Scalar>
            + HasField<Symbol!("height"), Value = <Self as HasScalarType>::Scalar>,
        Self: HasScalarType,
    {
        fn area_expanded(&self) -> <Self as HasScalarType>::Scalar {
            let width: <Self as HasScalarType>::Scalar =
                self.get_field(PhantomData::<Symbol!("width")>).clone();
            let height: <Self as HasScalarType>::Scalar =
                self.get_field(PhantomData::<Symbol!("height")>).clone();
            width * height
        }
    }

    #[cgp_fn]
    #[use_type(HasScalarType.Scalar in Types)]
    pub fn area_of<Types>(&self, scalar: Scalar) -> Scalar {
        scalar
    }

    pub trait Begin {
        fn begin() -> Self;
    }

    #[cgp_type]
    pub trait HasTransactionType {
        type Transaction: Begin;
    }

    #[cgp_fn]
    #[use_type(HasTransactionType.Transaction)]
    pub fn start(&self) -> Transaction {
        Transaction::begin()
    }

    pub struct Marker;

    #[cgp_type]
    pub trait HasMarkerType {
        type Marker;
    }

    #[cgp_fn]
    #[use_type(HasMarkerType.Marker)]
    pub fn markers(&self, typed: Marker) -> Marker {
        // The page's `todo!()` is replaced by the argument.
        let _value = Marker; // stays the unit struct `Marker`
        typed
    }

    #[cgp_component(ErrorParser)]
    pub trait CanParseWith<Error> {
        fn parse_with(&self, input: &str) -> Error;
    }

    #[cgp_impl(new ParseToError)]
    #[use_type(HasErrorType.Error)]
    impl ErrorParser<Error> {
        fn parse_with(&self, input: &str) -> Error {
            let _ = input;
            todo!()
        }
    }

    pub trait HasA {
        type A;
    }

    pub trait HasB<A> {
        type B;
    }

    pub trait HasC<B> {
        type C;
    }

    #[cgp_fn]
    #[use_type(HasC<B>.C in B, HasB<A>.B in A, HasA.A)]
    pub fn chain(&self, c: C) -> C {
        c
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    pub struct Tx;

    impl Begin for Tx {
        fn begin() -> Self {
            Tx
        }
    }

    delegate_components! {
        Rectangle {
            ScalarTypeProviderComponent: UseType<f64>,
            TransactionTypeProviderComponent: UseType<Tx>,
            MarkerTypeProviderComponent: UseType<u8>,
            cgp::core::error::ErrorTypeProviderComponent: UseType<String>,
            ErrorParserComponent: ParseToError,
        }
    }

    impl HasA for Rectangle {
        type A = Rectangle;
    }

    impl HasB<Rectangle> for Rectangle {
        type B = Rectangle;
    }

    impl HasC<Rectangle> for Rectangle {
        type C = u16;
    }

    check_components! {
        Rectangle {
            ErrorParserComponent: String,
        }
    }

    #[test]
    fn the_rewrites_resolve() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(rect.area(), 6.0);
        assert_eq!(rect.area_expanded(), rect.area());
        assert_eq!(AreaOf::<Rectangle>::area_of(&rect, 1.0), 1.0);
        let Tx = rect.start();
        assert_eq!(rect.markers(9), 9);
        assert_eq!(rect.chain(5), 5);
    }
}
