---
sidebar_label: 'Implicit arguments'
sidebar_position: 4
---

# Implicit arguments

Implicit arguments let a provider declare the context values it needs as function parameters.
CGP reads those values from the context, so callers pass only the method's explicit arguments.
This page explains the field access behind that shorthand, how argument types control borrowing
and cloning, and when a getter trait is a better fit.

## Reading a field explicitly

A generic provider reads a context field through `HasField`, a trait keyed by the field's name.
The following fragment assumes a `Greeter` component with a `greet(&self) -> String` method and
imports from `cgp::prelude::*`:

```rust
#[cgp_impl(new GreetByName)]
impl Greeter
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn greet(&self) -> String {
        let name = self.get_field(PhantomData::<Symbol!("name")>);
        format!("Hello, {name}!")
    }
}
```

`HasField<Symbol!("name"), Value = String>` requires the context to expose a `String` field named
`name`. `Symbol!("name")` represents the name as a type, and `PhantomData` supplies that type to
`get_field` so Rust can select the field. The bound is an
[impl-side dependency](./impl-side-dependencies.md): it constrains this provider without adding a
field requirement to the consumer trait.

The explicit form separates the requirement from the read. For a provider that only needs a local
value, an implicit argument can express both together.

## The same provider with an implicit argument

An **implicit argument** names a field and declares the type the method body needs:

```rust
#[cgp_impl(new GreetByName)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}
```

`#[implicit]` removes `name` from the method's public signature and generates the field bound and
read. The body receives a local `name`, while a caller still writes `app.greet()`. Here, `&str`
borrows the contents of a `String` field.

Implicit arguments are the usual choice for reading fields from a provider's own context. They
keep each value's name and type beside the method that uses it, while the macro supplies the
`HasField` bound and tagged access.

## A trait from a function alone

[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) combines implicit arguments with a single blanket
implementation, so the operation needs neither a named provider nor wiring:

```rust
#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}
```

The macro generates a `RectangleArea` trait and implements it for contexts exposing `width` and
`height` through `HasField`, both with type `f64`. A struct can supply those implementations by
carrying the matching fields and deriving [`HasField`](/docs/reference/derives/derive_has_field).
Its values can then call `rectangle_area()`.

This form suits an operation with one shared implementation. A component becomes useful when
contexts need to select among alternative implementations of the same operation.

## The declared type decides how the field is read

The argument type determines whether the body receives an owned value or a borrow. These common
forms follow the same access rules as CGP's automatic getters:

```rust
#[cgp_fn]
pub fn describe(
    &self,
    // Cloned from the context's String field.
    #[implicit] name: String,
    // Borrows the contents of the context's String field.
    #[implicit] title: &str,
    // Borrows the Vec field directly.
    #[implicit] tags: &Vec<String>,
) -> String {
    format!("{title} {name} {tags:?}")
}
```

An owned argument calls `.clone()` on the field, leaving the stored value in the context. A `&str`
argument reads a `String` field with `.as_str()`, and `&Vec<String>` borrows the vector directly.
Prefer a borrow when the body only needs to read the value, especially when cloning would allocate
or copy substantial data.

The macro supports specific access forms rather than arbitrary conversions between types. The
[`#[implicit]` reference](/docs/reference/attributes/implicit) covers options, slices, mutable
access, and the other supported forms.

## When a getter trait is useful

A getter trait is useful when field access must be expressed as a named interface. Implicit
arguments read from the provider's own `self`, so they cannot directly read a field from a separate
argument. For example, this provider requires a getter on `Request`:

```rust
#[cgp_impl(new AuthenticateByHeader)]
impl<Request> RequestAuthenticator<Request>
where
    Request: HasAuthHeader,
{
    fn authenticate(&self, request: &Request) -> bool {
        request.auth_header().starts_with("Bearer ")
    }
}
```

This fragment assumes `RequestAuthenticator` and `HasAuthHeader` are defined elsewhere.
`HasAuthHeader` describes access on the request, while `self` is the [application context](/docs/reference/glossary#application-context). An
implicit argument on `authenticate` would read the application instead.

A named getter also lets other code require the accessor through a trait bound or supertrait.
A getter with an associated type can keep the field's type abstract for callers. These are reasons
to use [`#[cgp_auto_getter]`](/docs/reference/macros/cgp_auto_getter); sharing a context field
between several providers alone is not. Each provider can declare the same implicit argument.

A wireable getter supports a further choice: which field supplies the value in each context.
Use [`#[cgp_getter]`](/docs/reference/macros/cgp_getter) when that mapping must vary, rather than
fixing the field name in the implicit argument.

## What it costs

Field names become dependencies of providers. Renaming `name` breaks providers whose implicit
argument requires that name, and the error appears where Rust checks the provider's requirements.
The struct definition does not list the providers that depend on its fields.

The call site omits those field requirements. `app.greet()` does not show that `GreetByName` needs
`name`; you find that requirement in the selected provider. Rust checks the name and type, but it
cannot tell whether two fields with the same name and type have the same intended meaning.

Owned arguments incur the cost of cloning on each call. For a small scalar this is a copy; for a
`String` or a large collection it may allocate and copy data. Declare a borrow when ownership is
unnecessary.

## Where to go next

These pages connect implicit arguments to the rest of CGP:

- [Impl-side dependencies](./impl-side-dependencies.md): Why a provider's field requirements stay
  out of the consumer interface.
- [Abstract types](./abstract-types.md): Context-selected types shared by generic code.
- [Hello World](/docs/tutorials/hello) and [Area calculation](/docs/tutorials/area-calculation/):
  Working examples built from functions with implicit arguments.
- [`#[implicit]`](/docs/reference/attributes/implicit) and
  [`#[cgp_fn]`](/docs/reference/macros/cgp_fn): Accepted forms and generated code.
- [`HasField`](/docs/reference/traits/field-access/has_field): The field-access trait underlying
  implicit arguments and automatic getters.
- [Comparison: Implicit parameters](/docs/comparisons/implicit-parameters): How the design compares
  with Scala's `using` and Haskell's `ImplicitParams`.
- [Comparison: Algebraic effects](/docs/comparisons/algebraic-effects): The relationship between
  context field access and dynamic binding.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
