---
title: 'OptionAsRef — borrow the value inside an Option'
sidebar_label: 'OptionAsRef'
sidebar_position: 7
description: 'Borrow the value inside an Option as a &T, the bound an Option<&str> implicit argument generates so the field may be a String, a Cow<str>, or a Box<str>.'
---

# `OptionAsRef`

`OptionAsRef<T>` borrows the value inside an
[`Option`](https://doc.rust-lang.org/core/option/enum.Option.html) as a `&T`, for any `Option<U>`
whose `U` implements `AsRef<T>`.

## Overview

The trait lets an [`#[implicit]`](../../attributes/implicit.md) argument of type `Option<&str>` read
any `Option<U>` whose `U` converts to `&str`. The two obvious bounds do not work: requiring the field
to be an `Option<String>` ties it to one type and pulls in `alloc`, and requiring `Value: AsRef<str>`
fails because `Option<String>` does not implement `AsRef<str>`. `OptionAsRef<T>` names the conversion
instead of the field's concrete type.

The plain `&str` case does not need such a helper, because `AsRef<str>` already covers a `String`,
a `Cow<str>`, and a `Box<str>` field.

## Definition

```rust
pub trait OptionAsRef<T: ?Sized> {
    fn option_as_ref(&self) -> Option<&T>;
}

impl<T: ?Sized, U> OptionAsRef<T> for Option<U>
where
    U: AsRef<T>,
{
    fn option_as_ref(&self) -> Option<&T> {
        self.as_ref().map(|value| value.as_ref())
    }
}
```

The `?Sized` bound is what lets `T` be `str`, which is unsized.

## Usage

An `#[implicit] nickname: Option<&str>` argument generates a bound and a binding:

```rust
Self: HasField<Symbol!("nickname"), Value: OptionAsRef<str> + 'static>,
// ...
let nickname: Option<&str> = OptionAsRef::option_as_ref(
    HasField::get_field(self, PhantomData::<Symbol!("nickname")>),
);
```

The call is fully qualified, so a local trait with the same method name cannot capture it.

A getter keeps the `Option<String>` field instead, because its `HasField` bound is also used by the
[`WithProvider`](../../providers/with_provider.md) adapter, which expects the field type itself.

## Examples

A context with an `Option<String>` field answers the argument, and so does one with an
`Option<Cow<str>>` or an `Option<Box<str>>` field:

```rust
#[cgp_fn]
pub fn describe(&self, #[implicit] nickname: Option<&str>) -> String {
    match nickname {
        Some(name) => format!("aka {name}"),
        None => "anonymous".to_owned(),
    }
}

#[derive(HasField)]
pub struct App {
    pub nickname: Option<String>,
}
```

## When to use it

Reach for it only through an `#[implicit]` argument of type `Option<&str>`. A plain `&str` argument
does not need a helper, because `AsRef<str>` already covers the same field types, and a getter keeps
the
`Option<String>` field.

## Related constructs

- [`#[implicit]`](../../attributes/implicit.md): the whole table of argument forms.
- [`HasField`](has_field.md): the trait the bound is stated on.

## Source

- [`traits/option_as_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/option_as_ref.rs):
  the trait and its blanket impl

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
