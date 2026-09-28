---
title: 'HasField — read a field by a type-level name'
description: 'The trait for reading one field of a context by its name as a type, so generic code can bound on a field of a type it cannot name.'
sidebar_label: 'HasField'
sidebar_position: 1
---

# `HasField`

Reading a field by a type-level name, from a context you cannot name.

## Overview

An implementation written against a [**context**](/docs/reference/glossary#context), the type the
method runs on, which supplies the values it needs as its fields, is generic over that context and
cannot name its concrete type, so it cannot write `self.name`. `HasField<Tag>` closes the gap: the
field's name becomes a type, and the implementation asks for the field as a trait bound:

```rust
Self: HasField<Symbol!("name"), Value = String>
```

Any context with a matching field satisfies that bound. Nothing is compared by string at run time:
[`Symbol!("name")`](../../macros/symbol.md) is a type, so the compiler resolves which field is meant
and the read compiles to a direct field access.

The ergonomic constructs all stand on this trait. An [`#[implicit]`](../../attributes/implicit.md)
argument, a [`#[cgp_auto_getter]`](../../macros/cgp_auto_getter.md) method, and a
[`UseField`](../../providers/use_field.md) wiring entry each generate this bound from a name you
already wrote, and the impls come from [`#[derive(HasField)]`](../../derives/derive_has_field.md).
You read `HasField` far more often than you write it.

## Definition

`HasField<Tag>` carries the field's type as an associated `Value` and returns a reference to it:

```rust
pub trait HasField<Tag> {
    type Value;

    fn get_field(&self, _tag: PhantomData<Tag>) -> &Self::Value;
}
```

`Tag` is a type-level name: [`Symbol!("field_name")`](../../macros/symbol.md) for a named field, and
[`Index<N>`](../../types/index_type.md) for a tuple field. `Value` is the field's type, exposed as
an associated type so a bound can pin it, as `HasField<Symbol!("name"), Value = String>`, or leave
it open. `get_field` borrows the field; its `PhantomData<Tag>` argument carries no data and tells a
call site which field it means when several `HasField` impls are in scope, so a read is written
`self.get_field(PhantomData::<Symbol!("name")>)`.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. An implementation states the field it
needs in its `where` clause and reads it with `get_field`; a context gets the impls by deriving
them:

```rust
#[derive(HasField)]
pub struct Person {
    pub name: String,
}
```

Four neighbours complete the picture, each on its own page: [`HasFieldMut`](./has_field_mut.md) for
mutable access, [`FieldGetter`](./field_getter.md) for the provider-side form that gets wired, and
[`MapField`](./map_field.md) with [`FieldMapper`](./field_mapper.md) for reading through a field in
generic code.

## Examples

A provider that bounds on a field and reads it, and a generic function that reads the same field
through a `Box`:

```rust
use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn greet(&self) -> String {
        format!("Hello, {}!", self.get_field(PhantomData::<Symbol!("name")>))
    }
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    Person {
        GreeterComponent,
    }
}

pub fn read_name<Context>(context: &Context) -> &String
where
    Context: HasField<Symbol!("name"), Value = String>,
{
    context.get_field(PhantomData)
}

pub fn demo() {
    let person = Person {
        name: "Ada".to_owned(),
    };
    assert_eq!(person.greet(), "Hello, Ada!");

    // `Box<Person>` has the field through the `Deref` forwarding impl.
    let boxed = Box::new(Person {
        name: "Alice".to_owned(),
    });
    assert_eq!(read_name(&boxed), "Alice");
}
```

`Person` derives the access, so it satisfies exactly the bound `GreetHello` requires. `Person` is a
[value context](/docs/reference/glossary#value-context) and the component is
[self-targeted](/docs/reference/glossary#self-targeted-component): the wired type is the data the
method reads.

Written idiomatically, the bound is not visible. The same read is an
[`#[implicit]`](../../attributes/implicit.md) argument, which generates the identical bound and
read:

```rust
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}
```

## When to use it

**Bound on `HasField` by hand only when the ergonomic constructs cannot do the job**, which is
rarely. The order to try them in:

- **An [`#[implicit]`](../../attributes/implicit.md) argument** reads a field of the
  implementation's own context as a plain parameter, and covers the common case, including a field
  several implementations each read.
- **[`#[cgp_auto_getter]`](../../macros/cgp_auto_getter.md)** makes the read a named trait, reaches
  a field on a type other than the context, or infers a type from the field, which an implicit
  argument cannot.
- **[`#[cgp_getter]`](../../macros/cgp_getter.md) with [`UseField`](../../providers/use_field.md)**
  lets each context choose which field the getter reads, at the cost of a wiring line per context.
- **A hand-written bound** fits the rest, such as a generic function over any context with the
  field, as `read_name` is above.

Three neighbours are easy to confuse with it. [`HasFields`](../shape/has_fields.md), the plural, is
the whole-shape view for code that processes every field. [`HasFieldMut`](./has_field_mut.md) is the
same access with mutation. And [`FieldGetter`](./field_getter.md) is the provider-side form: you
bound against `HasField` and wire `FieldGetter`.

## Under the hood

`#[derive(HasField)]` emits one `HasField` impl per field, keyed by the field's `Symbol!`, and the
trait module adds two impls that make access compose. The first forwards through
[`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html), so a smart pointer or newtype has the
fields of its target:

```rust
#[diagnostic::do_not_recommend]
impl<Context, Tag, Target, Value> HasField<Tag> for Context
where
    Context: DerefMap<Target = Target>,
    Target: HasField<Tag, Value = Value>,
{
    type Value = Value;

    fn get_field(&self, tag: PhantomData<Tag>) -> &Self::Value {
        self.map_deref(|context| context.get_field(tag))
    }
}
```

`DerefMap` is a private helper over `Deref` that passes the borrow through a higher-ranked closure,
so the target needs no `'static` bound, and `#[diagnostic::do_not_recommend]` keeps the compiler
from suggesting this impl in errors, so a missing-field error names the struct that lacks the field.
The second connects field access to wiring: [`UseContext`](../../providers/use_context.md)
implements [`FieldGetter`](./field_getter.md) for any context that has the field, delegating
straight through, as that page shows.

## Common Mistakes

**Two spellings of a name are unrelated types.** A context whose field is `firstName` does not meet
a bound on `Symbol!("first_name")`, and the mismatch reports as a missing `HasField` bound, with
each name spelled out character by character:

```text
error[E0277]: the trait bound `Person: cgp::prelude::HasField<cgp::prelude::Symbol<10, cgp::prelude::Chars<'f', cgp::prelude::Chars<'i', cgp::prelude::Chars<'r', cgp::prelude::Chars<'s', cgp::prelude::Chars<'t', cgp::prelude::Chars<'_', cgp::prelude::Chars<'n', cgp::prelude::Chars<'a', cgp::prelude::Chars<'m', cgp::prelude::Chars<'e', Nil>>>>>>>>>>>>` is not satisfied
...
      but trait `HasField<cgp::prelude::Symbol<9, cgp::prelude::Chars<'f', cgp::prelude::Chars<'i', cgp::prelude::Chars<'r', cgp::prelude::Chars<'s', cgp::prelude::Chars<'t', cgp::prelude::Chars<'N', cgp::prelude::Chars<'a', cgp::prelude::Chars<'m', cgp::prelude::Chars<'e', Nil>>>>>>>>>>>` is implemented for it
```

The leading number is the name's length, and the `Chars` list spells it. A tuple field is keyed by
[`Index<N>`](../../types/index_type.md), never by `Symbol!("0")`.

**A type that derefs to a target cannot also derive a field its target has.** The `Deref` forwarding
impl already covers every tag of the target, so a second impl for the same tag overlaps. A `Wrapper`
that derefs to a `Person` with `name`, and also derives its own `name` field, fails:

```rust
#[derive(HasField)]
pub struct Wrapper {
    pub name: String,
    pub person: Person,
}

impl Deref for Wrapper {
    type Target = Person;

    fn deref(&self) -> &Person {
        &self.person
    }
}
```

```text
error[E0119]: conflicting implementations of trait `cgp::prelude::HasField<Symbol<4, cgp::prelude::Chars<'n', cgp::prelude::Chars<'a', cgp::prelude::Chars<'m', cgp::prelude::Chars<'e', Nil>>>>>>` for type `Wrapper`
```

Fields the target lacks are unaffected, so rename the wrapper's field or drop the `Deref` impl.

**The `PhantomData` argument carries the tag.** `self.get_field(PhantomData)` works only where
inference can determine the tag, as in a function whose only `HasField` bound fixes it; elsewhere
write `PhantomData::<Symbol!("name")>`.

**It returns a reference, always.** There is no owning read; an
[`#[implicit]`](../../attributes/implicit.md) argument inserts a `.clone()` when the parameter is
owned.

## Related constructs

- [`#[derive(HasField)]`](../../derives/derive_has_field.md): generates the per-field impls.
- [`HasFieldMut`](./has_field_mut.md): the mutable extension.
- [`FieldGetter`](./field_getter.md): the provider-side form that gets wired.
- [`MapField`](./map_field.md): reading through a field in generic code.
- [`HasFields`](../shape/has_fields.md): the whole-shape counterpart.
- [`Symbol!`](../../macros/symbol.md) and [`Index`](../../types/index_type.md): the tags that key a
  field.
- [`#[implicit]`](../../attributes/implicit.md): the idiomatic way to read a field.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): why a field requirement belongs
  on the implementation rather than the interface.
- [Implicit arguments](/docs/concepts/implicit-arguments): the ergonomic surface built on this
  trait.

## Source

- [`traits/has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs):
  `HasField`, `FieldGetter`, the `Deref` forwarding, and the `UseContext` impl

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
