---
sidebar_position: 2
---

# Hello World Tutorial

In this tutorial, we will build a small working program that greets people by name, using CGP's core features. Along the way we will encounter CGP functions, implicit arguments, struct-based contexts, and `HasField` derivation. By the end you will have seen a complete working example and will understand at a high level how CGP leverages Rust's trait system to let you write highly reusable code without any runtime overhead.

## Using the `cgp` crate

To get started, first include the latest version of the [`cgp` crate](https://crates.io/crates/cgp) as your dependency in `Cargo.toml`:

```toml title="Cargo.toml"
cgp = "0.7.0"
```

## The CGP Prelude

To use CGP features, we first need to import the CGP prelude in every Rust module that uses CGP constructs:

```rust
use cgp::prelude::*;
```

With the setup done, we are ready to write context-generic code.

## CGP Functions

The simplest CGP feature you can use is to write a context-generic method. Let's define a `greet` function as follows:

```rust title="greet.rs"
#[cgp_fn]
pub fn greet(&self, #[implicit] name: &str) {
    println!("Hello, {name}!");
}
```

The `greet` function looks almost the same as how we would write it in plain Rust, but there are a few differences worth noting. First, we annotate the function with `#[cgp_fn]` to turn it into a context-generic *method* that will work with multiple context types. Second, we include `&self` so that it can be used to access other context-generic methods in more complex examples. Third, the `name` argument is annotated with the `#[implicit]` attribute, which means it is an **implicit argument** that is automatically retrieved from the `Self` context rather than being passed by the caller.

With the CGP function defined, let's create a concrete context and call `greet` on it.

## The `Person` Context

The simplest way to call a CGP function is to define a context struct that contains all the required implicit arguments. Here is the `Person` struct:

```rust title="person.rs"
#[derive(HasField)]
pub struct Person {
    pub name: String,
}
```

To enable CGP functions to access the fields of a context, we use `#[derive(HasField)]` to derive the necessary CGP traits that power the generic field access machinery. In practice, this means the `greet` function will be able to find the `name` field automatically, without any further wiring on our part.

With the `Person` struct defined, we can call the `greet` method on it with no additional work:

```rust title="main.rs"
let person = Person {
    name: "Alice".to_owned(),
};

person.greet();
```

Running this program will print:

```
Hello, Alice!
```

That's it! There is no need to manually pass the `name` field to `greet`. CGP automatically extracts the corresponding field from the `Person` struct and passes it to `greet`.

## The `PersonWithAge` Context

With an example as simple as hello world, it might not be obvious why we would want to define `greet` as a context-generic method instead of as a concrete method on `Person`. Let's explore that question by introducing a second context.

Consider that the `greet` method only needs access to the `name` field. But a real-world `Person` struct may contain many other fields, and what fields a struct should have will vary depending on the application being built. Since `greet` is defined as a context-generic method, it can work *generically* across any context type that satisfies its requirements. This effectively *decouples* the implementation of `greet` from any specific struct, allowing it to be reused across different contexts, such as the `PersonWithAge` struct below:

```rust
#[derive(HasField)]
pub struct PersonWithAge {
    pub name: String,
    pub age: u8,
}
```

Both the original `Person` struct and the new `PersonWithAge` struct can coexist and both can call `greet` without any changes to the function itself:

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

Running this program will print:

```
Hello, Alice!
Hello, Bob!
```

Notice that `greet` works identically for both contexts, even though `PersonWithAge` has an extra field that `greet` never needs to know about. The benefits of decoupling methods from contexts will become clearer as we explore more complex examples in further tutorials and documentation.

## Behind the Scenes

The hello world example demonstrates how CGP unlocks new capabilities for writing context-generic constructs in Rust. You might wonder how the underlying machinery works, and whether CGP employs some magic that requires unsafe code or runtime overhead. This section offers a brief look at the mechanics to dispel those concerns.

A full explanation of how CGP works is beyond this tutorial, but you can think of the `greet` function as being roughly equivalent to the following plain Rust definition:

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

The plain-Rust version is considerably more verbose, but it can be understood with a straightforward explanation. `HasName` is a *getter trait* that a context implements to expose its `name` value. `Greet` is defined as a trait with a [**blanket implementation**](https://blog.implrust.com/posts/2025/09/blanket-implementation-in-rust/) that works for any context type `T` that implements `HasName`.

When we use `#[derive(HasField)]` on a context like `Person`, we are effectively automatically implementing the `HasName` trait for it:

```rust
pub struct Person {
    pub name: String,
}

impl HasName for Person {
    fn name(&self) -> &str {
        &self.name
    }
}
```

There is more advanced machinery involved in the actual desugared CGP code, but the generated code is *semantically* roughly equivalent to the manually implemented plain Rust constructs shown above.

### Zero Cost Abstractions

The plain Rust expansion above illustrates a few key properties of CGP. Firstly, CGP makes heavy use of the existing machinery provided by Rust's trait system to implement context-generic abstractions. It is also worth understanding that CGP macros like `#[cgp_fn]` and `#[derive(HasField)]` act primarily as **syntactic sugar** that performs a straightforward desugaring of CGP code into plain Rust constructs, just as shown above.

This means there is **no hidden logic at either compile time or runtime** used by CGP to resolve dependencies like `name`. The main contribution of CGP is that it introduces new language syntax and leverages Rust's trait system to enable new capabilities. You do not need to understand any new machinery beyond the trait system to understand how CGP works.

Furthermore, implicit arguments like `#[implicit] name: &str` are automatically desugared by CGP to use getter traits similar to `HasName`. Contexts like `Person` implement those getter traits by simply returning a *reference* to the field value. This means that implicit argument access is **zero cost** and is as cheap as direct field access from a concrete context.

The important takeaway is that CGP follows the same **zero cost abstraction** philosophy of Rust, enabling us to write highly modular Rust programs without any runtime overhead.

### Generalized Getter Fields

When walking through the desugared Rust code, you might wonder: since `Greet` requires the context to implement `HasName`, does this mean a context type like `Person` must be explicitly aware of `Greet` and implement `HasName` before it can use `Greet`?

The answer is yes for the simplified desugared code shown above. But CGP actually employs a more generalized trait called `HasField` that works universally across all possible structs. This means there is **no need** to specifically generate a `HasName` trait to be used by `Greet`, or to implement it manually for `Person`.

The full explanation of how `HasField` works is beyond the scope of this tutorial. The general idea, however, is that a `HasField` instance is implemented for every field inside a struct that uses `#[derive(HasField)]`. Traits like `Greet` then use this to access a specific field by its field name. In practice, this means that `Greet` and `Person` can be defined in entirely different crates without knowing anything about each other. When they are imported together in a third crate, `Greet` will still be automatically implemented for `Person`.

## Conclusion

In this tutorial, we have written a `greet` function using `#[cgp_fn]` and seen it work seamlessly with two different context types — `Person` and `PersonWithAge` — without any changes to the function itself. We have used `#[derive(HasField)]` to allow our structs to participate in CGP's generic field access machinery, and we have seen how `#[implicit]` arguments are automatically resolved from the calling context.

The core insight to take away is that CGP allows you to write code that is decoupled from the specific shape of any context, while still relying entirely on Rust's standard trait system. There are no hidden costs, no runtime magic, and no unsafe code involved.

This hello world example only scratches the surface of what CGP makes possible. In the next tutorials, we will explore more advanced features such as CGP components and providers, which allow multiple alternative implementations of the same interface to coexist and be swapped at the type level.