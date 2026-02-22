---
sidebar_position: 2
---

# Hello World CGP Tutorial

We will demonstrate various concepts of CGP with a simple hello world example.

## Using the `cgp` crate

To get started, first include the latest version of [`cgp` crate](https://crates.io/crates/cgp) as your dependency in `Cargo.toml`:

```toml title="Cargo.toml"
cgp = "0.6.2"
```

## The CGP Prelude

To use CGP features, we would first need to import the CGP prelude in the Rust module that uses CGP features:

```rust
use cgp::prelude::*;
```

With the setup done, we are now ready to write context-generic code.

## CGP Functions

The simplest CGP feature that you can use is to write a context-generic method, such as the `greet` function as follows:

```rust title="greet.rs"
#[cgp_fn]
pub fn greet(&self, #[implicit] name: &str) {
    println!("Hello, {name}!");
}
```

The `greet` function looks almost the same as how we would write it in plain Rust, except the following differences:

- We annotate the function with `#[cgp_fn]` to turn it into a context-generic *method* that would work with multiple context types.
- We include `&self` so that it can be used to access other context-generic methods for more complex examples.
- The `name` argument is annotated with an `#[implicit]` attribute. Meaning that it is an **implicit argument** that is automatically retrieved from the `Self` context.

With the CGP function defined, let's define a concrete context and call `greet` on it.

## `Person` Context

The simplest way we can call a CGP function is to define a context that contains all the required implicit arguments, such as the `Person` struct below:

```rust title="person.rs"
#[derive(HasField)]
pub struct Person {
    pub name: String,
}
```

To enable CGP functions to access the fields in a context, we use `#[derive(HasField)]` to derive the necessary CGP traits that empower generic field access machinery.

With the `Person` struct defined, we can simply call the `greet` method on it with no further action required:

```rust title="main.rs"
let person = Person {
    name: "Alice".to_owned(),
};

person.greet();
```

And that's it! There is no need for us to manually pass the `name` field to `greet`. CGP can automatically extract the corresponding field from the `Person` struct and pass it `greet`.

## `PersonWithAge` Context

With an example as simple as hello world, it might not be clear why we would want to define `greet` as a context-generic method, instead of a concrete method on `Person`.

One way to think of it is that the `greet` method only needs to access the `name` field in `Person`. But an actual `Person` struct for real world applications may contain many other fields. Furthermore, what fields should a `Person` struct has depends on the kind of applications being built.

Since `greet` is defined as a context-generic method, it means that the method can work *generically* across any *context* type that satisfies the requirements. With this, we effectively *decouples* the implementation of `greet` from the `Person` struct. This allows the function to be reused across different person contexts, such as the `PersonWithAge` struct below:

```rust
#[derive(HasField)]
pub struct PersonWithAge {
    pub name: String,
    pub age: u8,
}
```

Both the original `Person` struct and the new `PersonWithAge` struct can co-exist. And both structs can call `greet` easily:

```rust title="main.rs"
let alice = Person {
    name: "Alice".to_owned(),
};

alice.greet();


let bob = PersonWithAge {
    name: "Bob".to_owned(),
    age: 32,
};

bob.greet();
```

The benefits of decoupling methods from contexts will become clearer as we explore more complex examples in further tutorials and documentation.

## Behind the scenes

The hello world example here demonstrates how CGP unlocks new capabilities for us to easily write new forms of context-generic constructs in Rust. But you might wonder how the underlying machinery works, and whether CGP employs some magic that requires unsafe code or runtime overhead.

A full explanation of how CGP works is beyond this tutorial, but you can think of the `greet` function being roughly equivalent to the following plain Rust definition:

```rust
pub trait HasName {
    fn name(&self) -> &str;
}

pub trait Greet {
    fn greet(&self);
}

impl<T> Greet for T
where
    T: HasName,
{
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}
```

The plain-Rust version of the code look a lot more verbose, but it can be understood with some straightforward explanation: `HasName` is a *getter trait* that would be implemented by a context to get the `name` value. `Greet` is defined as a trait with a [**blanket implementation**](https://blog.implrust.com/posts/2025/09/blanket-implementation-in-rust/) that works with any context type `T` that implements `HasName`.

When we use `#[derive(HasField)]` on a context like `Person`, we are effectively automatically implementing the `HasName` trait:

```rust
pub struct Person {
    pub name: String;
}

impl HasName for Person {
    fn name(&self) -> &str {
        &self.name
    }
}
```

There are more advanced machinery that are involved with the desugared CGP code. But the generated code are *semantically* roughly equals to the manually implemented plain Rust constructs above.

### Zero Cost Abstractions

The plain Rust expansion demonstrates a few key properties of CGP. Firstly, CGP makes heavy use of the existing machinery provided by Rust's trait system to implement context-generic abstractions. It is also worth understanding that CGP macros like `#[cgp_fn]` and `#[derive(HasField)]` mainly act as **syntactic sugar** that perform simple desugaring of CGP code into plain Rust constructs like we shown above.

This means that there is **no hidden logic at both compile time and runtime** used by CGP to resolve dependencies like `name`. The main complexity of CGP lies in how it introduces new language syntax and leverages Rust's trait system to enable new language features. But you don't need to understand new machinery beyond the trait system to understand how CGP works.

Furthermore, implicit arguments like `#[implicit] name: &str` are automatically desugared by CGP to use getter traits similar to `HasName`. And contexts like `Person` implement `HasName` by simply returning a *reference* to the field value. This means that implicit argument access are **zero cost** and are as cheap as direct field access from a concrete context.

The important takeaway from this is that CGP follows the same **zero cost abstraction** philosophy of Rust, and enables us to write highly modular Rust programs without any runtime overhead.

### Generalized Getter Fields

When we walk through the desugared Rust code, you might wonder: since `Greet` requires the context to implement `HasName`, does this means that a context type like `Person` must know about it beforehand and explicitly implement `HasName` before it can use `Greet`?

The answer is yes for the simplified desugared code that we have shown earlier. But CGP actually employs a more generalized trait called `HasField` that can work generally for all possible structs. This means that there is **no need** to specifically generate a `HasName` trait to be used by `Greet`, or implemented by `Person`.

The full explanation of how `HasField` works is beyond the scope of this tutorial. But the general idea is that an instance of `HasField` is implemented for every field inside a struct that uses `#[derive(HasField)]`. This is then used by traits like `Greet` to access a specific field by its field name.

In practice, this means that both `Greet` and `Person` can be defined in totally different crate without knowing each other. They can then be imported inside a third crate, and `Greet` would still be automatically implemented for `Person`.

## Conclusion