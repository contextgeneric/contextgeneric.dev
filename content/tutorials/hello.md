+++

title = "Hello World Tutorial"

insert_anchor_links = "heading"

+++

# Hello World Tutorial

We will demonstrate various concepts of CGP with a simple hello world example.

## Greeter Component

To begin, we import the `cgp` crate and define a greeter component as follows:

```rust
use cgp::prelude::*;

#[cgp_component {
    name: GreeterComponent,
    provider: Greeter,
}]
pub trait CanGreet {
    fn greet(&self);
}
```

The `cgp` crate provides common constructs through its `prelude` module, which should be imported in most cases. The first CGP construct we use here is the `#[cgp_component]` macro. This macro generates additional CGP constructs for the greeter component.

The target of this macro, `CanGreet`, is a _consumer trait_ used similarly to regular Rust traits. However, unlike traditional traits, we won't implement anything directly on this trait.

The `name` argument, `GreeterComponent`, specifies the name of the greeter component. The `provider` argument, `Greeter`, designates a _provider trait_ for the component. The `Greeter` provider is used to define the actual implementations for the greeter component. It has a similar structure to `CanGreet`, but with the implicit `Self` type replaced by a generic `Context` type.

## A Name Dependency

Next, let's introduce dependencies needed to implement an example provider for `Greeter`. We'll start by declaring an abstract `Name` type:

```rust
cgp_type!( Name )
```

Here, the `cgp_type!` macro defines a CGP component for the abstract type `Name`. This macro is a concise alternative to using `#[cgp_component]`. It also derives additional implementations useful later. For now, it is enough to know that `cgp_type!` generates a `HasNameType` consumer trait, which includes an _associated type_ called `Name`.

Now, we'll define an _accessor trait_ to retrieve the name value from a context:

```rust
#[cgp_auto_getter]
pub trait HasName: HasNameType {
    fn name(&self) -> &Self::Name;
}
```

The `HasName` trait inherits from `HasNameType` to access the abstract type `Self::Name`. It includes the method `name`, which returns a reference to a value of type `Self::Name`.

The `#[cgp_auto_getter]` attribute macro applied to `HasName` automatically generates a blanket implementation. This enables any context containing a field named `name` of type `Self::Name` to automatically implement the `HasName` trait.

## Hello Greeter

The traits `CanGreet`, `HasNameType`, and `HasName` can be implemented independently across different modules or crates. However, we can import them into a single location and then implement a `Greeter` provider that uses `HasName` in its implementation:

```rust
pub struct GreetHello;

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

Here, we define `GreetHello` as a struct, which will be used to implement the `Greeter` provider trait. Unlike the consumer trait, where `Self` refers to the implementing type, the `Greeter` provider trait uses an explicit generic type `Context`, which fulfills the role of `Self` from `CanGreet`. The `GreetHello` type will serve as the `Self` type for `Greeter`, but we don't reference `self` in the provider trait implementation.

The implementation comes with two additional constraints:

- `Context: HasName` ensures the context implements `HasName`.
- `Context::Name: Display` allows the provider to work with any abstract `Name` type, as long as it implements `Display`.

Notice that these constraints are specified only in the `impl` block, not in the trait bounds for `CanGreet` or `Greeter`. This design allows us to use _dependency injection_ for both values and _types_ through Rust’s trait system.

In the `greet` method, we use `context: &Context` as a parameter instead of the `&self` argument used in `CanGreet::greet`. We then call `context.name()` to retrieve the name value from the context and print a greeting with `println!`.

The `GreetHello` provider implementation is _generic_ over any `Context` and `Context::Name` types, as long as they satisfy the corresponding constraints for `HasName` and `Display`. By separating the provider trait from the consumer trait, we can have multiple provider implementations like `GreetHello` that are all generic over any `Context`, without causing the overlapping implementation issues typically imposed by Rust's trait system.

## Person Context

Next, we define a concrete context, `Person`, and wire it up to use `GreetHello` for implementing CanGreet:

```rust
#[derive(HasField)]
pub struct Person {
    pub name: String,
}

pub struct PersonComponents;

impl HasComponents for Person {
    type Components = PersonComponents;
}

delegate_components! {
    PersonComponents {
        NameTypeComponent: UseType<String>,
        GreeterComponent: GreetHello,
    }
}
```

The `Person` context is defined as a struct containing a `name` field of type `String`. We use the `#[derive(HasField)]` macro to automatically derive `HasField` implementations for every field in `Person`. This works together with the blanket implementation generated by `#[cgp_auto_getter]` for `HasName`, allowing `HasName` to be automatically implemented for `Person` without requiring any additional code.

Next, we declare a struct, `PersonComponents`, which is used to wire up the provider components for `Person`. We then implement `HasComponents` for `Person`, using `PersonComponents` to indicate that `Person` will utilize the providers specified in `PersonComponents`.

We use the `delegate_components!` macro to wire up `PersonComponents` with the necessary components. The first mapping, `NameTypeComponent: UseType<String>`, is a shorthand to associate the abstract `Name` type with `String`. The second mapping, `GreeterComponent: GreetHello`, indicates that we want to use `GreetHello` as the implementation of the `CanGreet` consumer trait.

The expressive use of `delegate_components!` makes it easy to rewire the components for `Person`. For instance, if we want to use a custom `FullName` struct for the name type, we can rewire `NameTypeComponent` to `UseType<FullName>`. Similarly, if there’s an alternative `Greeter` provider, `GreetLastName`, that implements `Greeter` with additional constraints, we can simply rewire `GreeterComponent` to use `GreetLastName` and add any necessary additional wiring to meet those constraints.

It’s important to note that CGP allows component wiring to be done _lazily_, meaning any errors (such as unsatisfied constraints) will only be detected when a consumer trait is actually used.

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

By the end of this tutorial, you should have a high-level understanding of how programming in CGP works. There's much more to explore regarding how CGP handles the wiring behind the scenes, as well as the many features and capabilities CGP offers. To dive deeper, check out our book [Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/).
