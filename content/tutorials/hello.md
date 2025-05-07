+++

title = "Hello World Tutorial"

+++

We will demonstrate various concepts of CGP with a simple hello world example.

## Greeter Component

To begin, we import the `cgp` crate and define a greeter component as follows:

```rust
use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}
```

The `cgp` crate provides common constructs through its `prelude` module, which should be imported in most cases. The first CGP construct we use here is the `#[cgp_component]` macro. This macro generates additional CGP constructs for the greeter component.

The target of this macro, `CanGreet`, is a _consumer trait_ used similarly to regular Rust traits. However, unlike traditional traits, we won't implement anything directly on this trait.

In its simplified form, the argument to the macro, `Greeter`, designates a _provider trait_ for the component. The `Greeter` provider is used to define the actual implementations for the greeter component. It has a similar structure to `CanGreet`, but with the implicit `Self` type replaced by a generic `Context` type.

The macro also generates an empty `GreeterComponent` struct, which is used as the _name_ of the greeter component which can be used for the component wiring later on.

## Abstract Name Type

Next, let's introduce dependencies needed to implement an example provider for `Greeter`. We'll start by declaring an abstract `Name` type:

```rust
#[cgp_type]
pub trait HasNameType {
    type Name;
}
```

Here, the `#[cgp_type]` macro is used for abstract type traits that contain only one associated type. This macro is an extension to `#[cgp_component]`, and generates additional CGP constructs to work with abstract types.

Similar to `#[cgp_component]`, `#[cgp_type]` also generates a provider trait called `NameTypeProvider`, and a component name type called `NameTypeProviderComponent`.

## Name Getter

Now, we will define an _getter trait_ to retrieve the name value from a context:

```rust
#[cgp_auto_getter]
pub trait HasName: HasNameType {
    fn name(&self) -> &Self::Name;
}
```

The `HasName` trait inherits from `HasNameType` to access the abstract type `Self::Name`. It includes the method `name`, which returns a reference to a value of type `Self::Name`.

The `#[cgp_auto_getter]` attribute macro applied to `HasName` automatically generates a blanket implementation. This enables any context containing a field named `name` of type `Self::Name` to automatically implement the `HasName` trait.

## Hello Greeter

The traits `CanGreet`, `HasNameType`, and `HasName` can be defined separately across different modules or crates. However, we can import them into a single location and then implement a `Greeter` provider that uses `HasName` in its implementation:

```rust
#[cgp_new_provider]
impl<Context> Greeter<Context> for GreetHello
where
    Context: HasName,
    Context::Name: Display,
{
    fn greet(context: &Context) {
        println!("Hello, {}!", context.name());
    }
}
```

We use `#[cgp_new_provider]` to define a new provider, called `GreetHello`, which implements the `Greeter` provider trait. Unlike the consumer trait, where `Self` refers to the implementing type, the `Greeter` provider trait uses an explicit generic type `Context`, which fulfills the role of `Self` from `CanGreet`.

Behind the scene, the macro generates an empty struct named `GreetHello`, which is used as the `Self` type for `Greeter`. However, the type is used as an identifier for wiring later, and we don't reference `self` in the provider trait implementation.

The implementation comes with two additional constraints:

- `Context: HasName` ensures the context implements `HasName`.
- `Context::Name: Display` allows the provider to work with any abstract `Name` type, as long as it implements `Display`.

Notice that these constraints are specified only in the `impl` block, not in the trait bounds for `CanGreet` or `Greeter`. This design allows us to use _dependency injection_ for both values and _types_ through Rust’s trait system.

In the `greet` method, we use `context: &Context` as a parameter instead of the `&self` argument used in `CanGreet::greet`. We then call `context.name()` to retrieve the name value from the context and print a greeting with `println!`.

The `GreetHello` provider implementation is _generic_ over any `Context` and `Context::Name` types, as long as they satisfy the corresponding constraints for `HasName` and `Display`. By separating the provider trait from the consumer trait, we can have multiple provider implementations like `GreetHello` that are all generic over any `Context`, without causing the overlapping implementation issues typically imposed by Rust's trait system.

## Person Context

Next, we define a concrete context, `Person`, and wire it up to use `GreetHello` for implementing CanGreet:

```rust
#[cgp_context]
#[derive(HasField)]
pub struct Person {
    pub name: String,
}
```

The `Person` context is defined as a struct containing a `name` field of type `String`. We apply `#[cgp_macro]` on the `Person` context, which would generate a `PersonComponents` provider and some wirings to allow CGP components to be used with the context.

We also use the `#[derive(HasField)]` macro to automatically derive `HasField` implementations for every field in `Person`. This works together with the blanket implementation generated by `#[cgp_auto_getter]` for `HasName`, allowing `HasName` to be automatically implemented for `Person` without requiring any additional code.

## Delegate Components

Next, we want to define some wirings to link up the `GreetHello` that we defined earlier, so that we can use it on the `Person` context. This is done by using the `delegate_components!` macro as follows:

```rust
delegate_components! {
    PersonComponents {
        NameTypeProviderComponent:
            UseType<String>,
        GreeterComponent:
            GreetHello,
    }
}
```

We use the `delegate_components!` macro on the target provider `PersonComponents`, which was generated eariler by `#[cgp_context]`. For each entry in `delegate_components!`, we use the component name type as the key, and the chosen provider as the value.

The first mapping, `NameTypeProviderComponent: UseType<String>`, makes use of the generic `UseType` provider to implement the provider trait `NameTypeProvider`. The `String` argument to `UseType` is used to implement the associated type `Name`.

The second mapping, `GreeterComponent: GreetHello`, indicates that we want to use `GreetHello` as the implementation of the `CanGreet` consumer trait.

## Check Components

We have now declared the wirings for `PersonComponents` using `delegate_components!`. However, the implementation of the wiring is done _lazily_ in CGP. This means that invalid wiring will only raise compile errors the first time we try to use the concrete implementation.

However, we can make use of `check_components!` to perform compile-time checks that our previous wiring is correct. This can be done as follows:

```rust
check_components! {
    CanUsePerson for Person {
        NameTypeProviderComponent,
        GreeterComponent,
    }
}
```

The `check_components!` macro generates a `CanUsePerson` _check trait_, which is used for implementing checks that the `Person` context has correctly implemented the consumer traits for `NameTypeProviderComponent` and `GreeterComponent`.

If there is any unsatisfied dependency, such as if `Person` does not contain the necessary `name: String` field, then such errors will be raised here.

We can think of the use of `check_components!` as writing CGP tests that run at compile time. The reason this check is done separately from `delegate_components!`, is that we can use `check_components!` to define more advanced tests, such as when the CGP traits contain additional generic parameters.

## Calling Greet

Now that the wiring is set up, we can construct a `Person` instance and call `greet` on it:

```rust
fn main() {
    let person = Person {
        name: "Alice".into(),
    };

    // prints "Hello, Alice!"
    person.greet();
}
```

This is made possible by a series of blanket implementations generated by CGP. Here's how the magic works:

- We can call `greet` because `CanGreet` is implemented for `Person`.
- `PersonComponents` contains the wiring that uses `GreetHello` as the provider for `GreeterComponent`.
- `GreetHello` implements `Greeter<Person>`.
- `Person` implements `HasName` via the `HasField` implementation.
- `Person::Name` is `String`, which implements `Display`.

There’s quite a bit of indirection happening behind the scenes!

## Conclusion

By the end of this tutorial, you should have a high-level understanding of how programming in CGP works. There's much more to explore regarding how CGP handles the wiring behind the scenes, as well as the many features and capabilities CGP offers. To dive deeper, check out our book [Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/).

## Full Example Code

Below, we show the full hello world example code, so that you can walk through them again without the text.


```rust
use core::fmt::Display;

use cgp::prelude::*; // Import all CGP constructs

// Derive CGP provider traits and blanket implementations
#[cgp_component(Greeter)]
pub trait CanGreet // Name of the consumer trait
{
    fn greet(&self);
}

// Declare a CGP abstract type `Name`
#[cgp_type]
pub trait HasNameType {
    type Name;
}

// A getter trait representing a dependency for `name` value
#[cgp_auto_getter] // Derive blanket implementation
pub trait HasName: HasNameType {
    fn name(&self) -> &Self::Name;
}

// Implement `Greeter` that is generic over `Context`
#[cgp_new_provider]
impl<Context> Greeter<Context> for GreetHello
where
    Context: HasName, // Inject the `name` dependency from `Context`
    Context::Name: Display,
{
    fn greet(context: &Context) {
        // `self` is replaced by `context` inside providers
        println!("Hello, {}!", context.name());
    }
}

// A concrete context that uses CGP components
#[cgp_context]
#[derive(HasField)] // Deriving `HasField` automatically implements `HasName`
pub struct Person {
    pub name: String,
}

// Compile-time wiring of CGP components
delegate_components! {
    PersonComponents {
        NameTypeProviderComponent:
            UseType<String>, // Instantiate the `Name` type to `String`
        GreeterComponent:
            GreetHello, // Use `GreetHello` to provide `Greeter`
    }
}

// Compile-time checks that all dependencies are wired correctly
check_components! {
    CanUsePerson for Person {
        NameTypeProviderComponent,
        GreeterComponent,
    }
}

fn main() {
    let person = Person {
        name: "Alice".into(),
    };

    // `CanGreet` is automatically implemented for `Person`
    person.greet();
}
```