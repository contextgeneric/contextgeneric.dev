+++

title = "Programming Extensible Data Types in Rust with CGP - Part 4: Implementing Extensible Variants"

description = ""

authors = ["Soares Chen"]

+++


# Implementation of Extensible Variants

Now that we've covered how extensible records work in CGP, we can turn our attention to **extensible variants**. At first glance, it might seem like a completely different mechanism — but surprisingly, the approach used to implement extensible variants is very similar to that of extensible records. In fact, many of the same principles apply, just in the “opposite” direction.

This close relationship between records and variants is rooted in **category theory**. In that context, records are known as **products**, while variants (or enums) are referred to as **sums** or **coproducts**. These terms highlight a deep **duality** between the two: just as products combine values, coproducts represent a choice among alternatives. CGP embraces this theoretical foundation and leverages it to create a unified design for both extensible records and extensible variants.

This duality is not just theoretical — it has practical implications for software architecture. Our design in CGP builds directly on prior research into **extensible data types**, particularly in the context of functional programming and type systems. For more on the background, see the paper on [Extensible Data Types](https://dl.acm.org/doi/10.1145/3290325), as well as this excellent [intro to category theory](https://bartoszmilewski.com/2015/01/07/products-and-coproducts/) by Bartosz Milewski.

With this in mind, we’ll now explore the CGP constructs that support extensible variants. As you go through the examples and implementation details, we encourage you to look for the parallels and contrasts with extensible records.

# Base Implementation

## `FromVariant` Trait

Just as extensible records use the `HasField` trait to extract values from a struct, extensible variants in CGP use the `FromVariant` trait to *construct* an enum from a single variant value. The trait is defined as follows:

```rust
pub trait FromVariant<Tag> {
    type Value;

    fn from_variant(_tag: PhantomData<Tag>, value: Self::Value) -> Self;
}
```

Like `HasField`, the `FromVariant` trait is parameterized by a `Tag`, which identifies the name of the variant. It also defines an associated `Value` type, representing the data associated with that variant. Unlike `HasField`, which extracts a value, `from_variant` takes in a `Value` and returns an instance of the enum.

## Example: Deriving `FromVariant`

To see how this works in practice, consider the following enum:

```rust
#[derive(FromVariant)]
pub enum FooBar {
    Foo(String),
    Bar(u64),
}
```

Using the `#[derive(FromVariant)]` macro, the following trait implementations will be automatically generated:

```rust
impl FromVariant<symbol!("Foo")> for FooBar {
    type Value = String;

    fn from_variant(_tag: PhantomData<symbol!("Foo")>, value: Self::Value) -> Self {
        FooBar::Foo(value)
    }
}

impl FromVariant<symbol!("Bar")> for FooBar {
    type Value = u64;

    fn from_variant(_tag: PhantomData<symbol!("Bar")>, value: Self::Value) -> Self {
        FooBar::Bar(value)
    }
}
```

This allows the `FooBar` enum to be constructed generically using just the tag and value.

## Limitations on Enum Shape

To ensure ergonomics and consistency, CGP restricts the kinds of enums that can derive `FromVariant`. Specifically, supported enums must follow the **sums of products** pattern—each variant must contain *exactly one unnamed field*.

The following forms, for example, are **not** supported:

```rust
pub enum FooBar {
    Foo(String, String),
    Bar(u64, u64),
}

pub enum FooBar {
    Foo {
        foo_a: String,
        foo_b: String,
    },
    Bar {
        bar_a: u64,
        bar_b: u64,
    },
}
```

These more complex variants are not supported because they would make it harder to represent variant fields as simple types, which would, in turn, lead to less ergonomic APIs. By limiting each variant to a single unnamed field, CGP ensures that types like `FromVariant::Value` remain straightforward and intuitive.

If you need to represent more complex data in a variant, we recommend wrapping that data in a dedicated struct. This way, you can still take advantage of CGP's extensible variant system while maintaining type clarity and composability.

# Conclusion
