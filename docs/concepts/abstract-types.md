---
sidebar_label: 'Abstract types'
sidebar_position: 5
---

# Abstract types

An **abstract type** lets generic code name a type while each context chooses its concrete form.
Several providers can share the same scalar or error type without taking a separate type parameter
for it. This page starts with Rust's associated types, explains how CGP wires and shares them, and
ends with the limits of that abstraction.

## Rust already has this

Rust's associated types let a trait name a type that each implementer supplies. For example,
`HasScalarType` can leave the scalar open while two contexts make different choices:

```rust
pub trait HasScalarType {
    type Scalar: Copy;
}

impl HasScalarType for HighPrecision {
    type Scalar = f64;
}

impl HasScalarType for Embedded {
    type Scalar = f32;
}
```

Generic code with a `Context: HasScalarType` bound can use `Context::Scalar` without choosing `f32`
or `f64`. The compiler resolves that associated type when the context is known. CGP uses this same
mechanism, and a context can implement an abstract-type trait directly as shown above.

A generic parameter lets a caller choose a type for each use; an associated type ties the choice to
an implementing type. With `HasScalarType`, every use of `HighPrecision::Scalar` means `f64`.
That shared choice becomes useful when several providers need to agree on a type.

## Choosing the type by wiring

CGP lets a context select its associated types in the same table as its behavior providers.
[`#[cgp_type]`](/docs/reference/macros/cgp_type) turns an associated-type trait into a component:

```rust
#[cgp_type]
pub trait HasScalarType {
    type Scalar: Copy;
}
```

The generated `ScalarTypeProviderComponent` is the wiring key, and `UseType<T>` supplies the concrete
type. These tables make the same choices as the direct implementations above:

```rust
delegate_components! {
    HighPrecision {
        ScalarTypeProviderComponent: UseType<f64>,
    }
}

delegate_components! {
    Embedded {
        ScalarTypeProviderComponent: UseType<f32>,
    }
}
```

`UseType<f64>` sets `Scalar` to `f64`. The macro generates the provider implementation that makes
this work, so the context only needs to name the type. Other providers can compute the choice from
further type information, but `UseType<T>` suffices for a fixed choice.

The selected type must satisfy the associated type's bounds. Here `Scalar: Copy` accepts `f32` and
`f64`, but rejects `String`. Wiring `UseType<String>` would produce a `String: Copy` error when the
component is checked or used. Like other component wiring, the table is
[checked lazily](./check-traits.md).

## One type, agreed on by everything that needs it

Providers that refer to the same context's `Scalar` share one type choice. An area-calculation
component can use this to return a common scalar for several shape types:

```rust
#[cgp_component(ShapeAreaCalculator)]
#[use_type(HasScalarType.Scalar)]
pub trait CanCalculateShapeArea<Shape> {
    fn shape_area(&self, shape: &Shape) -> Scalar;
}
```

Here the context represents an application, while `Shape` is the data being measured. The application
chooses the result type through `HasScalarType`; the shape types do not need to implement that trait.
A fieldless `struct App;` can serve as the context if its providers need only its type choices and wiring.

Changing the application's wiring from `UseType<f32>` to `UseType<f64>` changes the declared result
type for every shape. The selected providers must support that choice: a provider restricted to
`Scalar = f64` cannot also serve a context that selects `f32`. Abstracting the result type does not
supply conversions or arithmetic implementations automatically.

`#[use_type(HasScalarType.Scalar)]` lets the signature use `Scalar` as a local shorthand for
`<Self as HasScalarType>::Scalar`. It also adds the required trait bound. On this component, that
bound is part of the interface because its return type depends on it.

## Type dependencies without extra parameters

A context can collect type choices that would otherwise become separate generic parameters.
Code that needs a scalar refers to `Context::Scalar`; it does not need an additional `Scalar`
parameter alongside `Context`. Providers can use the same approach for an error type or a runtime.

An implementation can keep a type dependency out of its caller's interface when the type is used
only inside the implementation. If the type appears in a public argument or result, the interface
must expose enough information to name it, as `CanCalculateShapeArea` does above.
[Impl-side dependencies](./impl-side-dependencies.md) explains that distinction.

## A shared error type {#the-canonical-one-a-contexts-error-type}

`HasErrorType` gives fallible providers a common error type selected by their context. A provider
can report a concrete source error through `CanRaiseError` while returning the context's abstract
`Error`:

```rust
#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}

#[cgp_impl(new LoadOrFail)]
#[uses(CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self, path: &str) -> Result<String, Error> {
        if path.is_empty() {
            return Err(Self::raise_error("empty path".to_owned()));
        }
        Ok(format!("contents of {path}"))
    }
}
```

`LoadOrFail` reports an empty path without choosing the application's error representation.
The context supplies both the error type and a provider that converts the source `String` into it.
The resulting type could be `anyhow::Error`, a domain enum, or `String`, provided the selected
conversion supports it.

Providers that return this context's `Error` can pass errors between them without converting at
every boundary. Errors from external libraries still need conversion into that common type.
[Modular error handling](./modular-error-handling.md) develops this example further.

## The provider and the import attribute {#two-things-called-usetype}

`UseType<T>` and `#[use_type(...)]` perform complementary jobs despite their similar names:

- **`UseType<T>`:** A provider placed in a wiring table to select a concrete type.
- **`#[use_type(...)]`:** An attribute placed on a definition to import an associated type and its bound.

The wiring chooses the type; the import lets generic code refer to that choice by a short name.

## What it costs

An abstract type defers the concrete choice without hiding its representation. Code that knows
`HighPrecision` uses `f64` can use the operations available on `f64`. Use Rust's module privacy when
callers must not access a representation.

Generic code can rely only on the bounds available where it uses the type. `Scalar: Copy` permits
copying but does not imply arithmetic or comparison. Those operations need additional bounds on the
associated type or on the code that uses it. Adding a bound to the trait requires every context's
chosen type to satisfy it.

A non-generic abstract-type trait supplies one choice per context. If one context needs distinct
error types for separate purposes, represent those choices with separate components or a tagged
component, or use separate contexts.

An abstract-type component can remain unwired until a use exposes the missing choice. Add
[`check_components!`](/docs/reference/macros/check_components) beside the context's wiring to verify
that the required type is available and satisfies its bounds.

## Where to go next

These pages explain related dependencies and the constructs used here:

- [Impl-side dependencies](./impl-side-dependencies.md): Which requirements can stay inside a provider.
- [Implicit arguments](./implicit-arguments.md): How a provider obtains values from its context.
- [Modular error handling](./modular-error-handling.md): Sharing, constructing, and enriching errors.
- [`#[cgp_type]`](/docs/reference/macros/cgp_type) and
  [`UseType`](/docs/reference/providers/use_type): Declaring and selecting an abstract type.
- [`#[use_type]`](/docs/reference/attributes/use_type): Importing an associated type into a definition.
- [`HasType`](/docs/reference/components/has_type): The built-in component for tagged type choices.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
