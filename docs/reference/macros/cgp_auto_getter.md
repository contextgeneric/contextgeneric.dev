---
sidebar_label: '#[cgp_auto_getter]'
sidebar_position: 8
---

# `#[cgp_auto_getter]`

Define a getter as a blanket impl over `HasField`, keyed by the method name.

## Overview

`#[cgp_auto_getter]` publishes a context field as a named, reusable accessor. You write a trait of getter
methods, and the macro implements it for every **context** that happens to carry fields of the matching
names. The context is the type the capability runs against, and it supplies the values it needs as its
own fields. For example:

```rust
#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}
```

Any struct with a `name: String` field now implements `HasName`, and `self.name()` works. Nothing is
wired and no impl is written: the macro emits one
[blanket implementation](https://blog.implrust.com/posts/2025/09/blanket-implementation-in-rust/)
covering every qualifying context, keyed on each method's own name.

It hides the field-access trait CGP reads fields through, which is precise and unpleasant to write by
hand: a bound naming the field as a type-level string, and a `PhantomData` tag at every read. Being
able to state the same thing as `fn name(&self) -> &str;` is the point.

**Reach for it sparingly.** For the ordinary case, a provider reading a field of its own context, an
[`#[implicit]`](../attributes/implicit.md) argument does the same job with no trait to declare, using the
same field access and the same conversion rules, so a getter trait declared only to read a field adds a
name and buys nothing. What a getter trait *does* buy is a capability other code can depend on by name,
and the three cases where that matters are in
[When to use it](#when-to-use-it).

## Usage

Apply the attribute to a trait definition. It takes no arguments. The body is getter methods, each taking
`&self` or `&mut self` and returning a reference:

```rust
#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}
```

A trait may declare several methods, and each maps independently to its own field:

```rust
#[cgp_auto_getter]
pub trait HasDimensions {
    fn width(&self) -> &f64;
    fn height(&self) -> &f64;
}
```

**The method name is the field name.** That is the whole convention, and also the construct's one real
limitation: a context must expose a field of exactly that name.

### How the return type decides the read

The return type controls how the field is read, and several shorthands exist so the signature stays
natural. The field's type is inferred from what you return:

| The method returns | The field's type | How it is read |
|---|---|---|
| `&T` | `T` | by reference, no conversion |
| `&str` | `String` | `.as_str()` |
| `&[T]` | anything `AsRef<[T]>` | `.as_ref()` |
| `Option<&T>` | `Option<T>` | `.as_ref()` |
| `Option<&str>` | `Option<String>` | `.as_deref()` |
| [`MRef<'_, T>`](../types/mref.md) | `T` | by reference, wrapped as `MRef::Ref(…)` |
| An owned type (`f64`, `String`, a tuple, an array) | the same type | by reference, then `.clone()` |

A `&mut self` receiver reads mutably, and each reference form has a mutable mirror: `&mut T`, `&mut [T]`
through `AsMut<[T]>`, `Option<&mut T>` via `.as_mut()`, and `Option<&mut str>` via `.as_deref_mut()`.

The `&str` row is the one most often wanted and least obvious: the context stores a `String` and the
getter hands out a borrow of it, so no context ever has to hold a `&str`. **These are the same rules an
[`#[implicit]`](../attributes/implicit.md) argument follows**, so learning them once covers everywhere CGP
reads a field. There is one difference worth holding onto. A getter takes its mutability from the
**receiver**, while an implicit argument takes it from the *argument's type*. So
`fn name(&mut self) -> &mut String` reads mutably because of the `&mut self`, whereas
`#[implicit] name: &String` on a `&mut self` method still reads through a shared borrow.

### Reading a field of another type

The first argument need not be `self`. A typed reference stands in for it, which is how a getter reaches
a field on something the context only *names*:

```rust
#[cgp_auto_getter]
pub trait HasFooBar: HasFooType + HasBarType {
    fn foo_bar(foo: &Self::Foo) -> &Self::Bar;
}
```

The generated bound now falls on `Self::Foo` rather than on the context, and the method is called as an
associated function: `App::foo_bar(&foo)`. The macro rewrites `Self` inside the argument and return
types to the context, and `&` versus `&mut` decides the access mode exactly as a receiver would.

**This is the one getter shape an [`#[implicit]`](../attributes/implicit.md) argument cannot reach**,
because there is no `self` field to read. This makes it the clearest case for declaring a getter at
all.

### An optional `PhantomData` argument

A getter method may take one further argument, and it must be a `PhantomData`:

```rust
#[cgp_auto_getter]
pub trait HasFoo {
    fn foo(&self, _tag: PhantomData<Foo>) -> &Foo;
}
```

It is forwarded to the generated method untouched and plays no part in the field lookup; it is there so a
getter can carry a type-level argument in its signature. Anything else in that position is rejected with
*only PhantomData is allowed as second argument*, and a third argument is rejected outright.

### A type inferred from the field

A getter may declare a single associated type and return it, which keeps the type abstract for callers
while letting the context's field decide it:

```rust
#[cgp_auto_getter]
pub trait HasName {
    type Name: Display;

    fn name(&self) -> &Self::Name;
}
```

The bound is enforced on whatever the field holds. When an associated type is present the trait must
contain **exactly one** getter method, whose return type is `&Self::AssocType`. There is only one field
for the type to be inferred from.

## Examples

A getter trait and a context that satisfies it by having the right field:

```rust
use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

pub fn greet(person: &Person) {
    println!("Hello, {}!", person.name());
}
```

`Person` derives [`HasField`](../derives/derive_has_field.md), which is its entire qualification. The
blanket impl applies automatically, with no wiring anywhere in the program.

The reason to declare the trait at all is that other code can now *require* it. A provider states the
dependency by name and never mentions a field:

```rust
#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}
```

Any context with a `name` field satisfies `HasName` and therefore satisfies `GreetHello`. A context that
stores the name elsewhere can implement the trait by hand instead, which the blanket impl does not
prevent:

```rust
pub struct Employee {
    pub full_name: String,
}

impl HasName for Employee {
    fn name(&self) -> &str {
        &self.full_name
    }
}
```

That hand-written impl is worth seeing, because it shows the trait is an ordinary Rust trait. The macro's
only job was to save you writing the body.

## When to use it

**Prefer an [`#[implicit]`](../attributes/implicit.md) argument, and reach for a getter trait only when one
cannot do the job.** An implicit argument reads a field of the provider's own context as an ordinary
parameter, with no trait, no declaration, and the same access rules, so it covers the common read
directly, including a field several providers each consume, declared as the same implicit argument in
each.

A getter trait earns its keep in three cases an implicit argument cannot reach.

- **The field lives on another type.** An implicit argument reads only from `self`, so a value held by a
  request, a payload, or any other type needs a getter that can be demanded as a bound on *that* type,
  as in `Request: HasBasicAuthHeader<Self>`. There is no `self` field to read.
- **The accessor must be a named capability.** When other code depends on "this context can tell you its
  name" rather than on a field, that dependency needs a trait to point at, importable with
  [`#[uses]`](../attributes/uses.md) or usable as a supertrait via [`#[extend]`](../attributes/extend.md).
- **The getter carries a type inferred from the field.** The associated-type form above keeps the type
  abstract for callers in a way a concrete implicit parameter cannot.

Between the two getter macros the line is narrow and worth stating plainly.

- **`#[cgp_auto_getter]` is the default getter.** One blanket impl, no wiring, field name fixed to the
  method name.
- **[`#[cgp_getter]`](./cgp_getter.md) is the advanced one**, and not a general upgrade. Reach for it only
  when a context needs to choose *which field* the getter reads, as a different name per context, or an
  implementation selected by wiring. That control costs a line of wiring per context, and most getters do
  not want it.

So the ordering is: implicit argument by default, `#[cgp_auto_getter]` for the three cases above, and
`#[cgp_getter]` only for per-context field choice.

## Under the hood

The macro re-emits the trait unchanged and adds one blanket impl over a generic context. From this input:

```rust
#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}
```

it produces the trait verbatim plus:

```rust
impl<__Context__> HasName for __Context__
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(&self) -> &str {
        self.get_field(PhantomData::<Symbol!("name")>).as_str()
    }
}
```

Three things to recognize. The context parameter is literally `__Context__`, a reserved name chosen so it
cannot collide with one of yours. [`Symbol!("name")`](./symbol.md) is a type-level string standing for the
field name. The compiler prints its expanded `Symbol<4, Chars<'n', …>>` form in errors, and
`cargo cgp expand` resugars it back to this. And because the return type is `&str`, the bound asks for a
`String` field and the body appends `.as_str()`, which is the conversion table in action.

A trait with several methods produces one predicate and one body per field, all in the same impl:

```rust
impl<__Context__> HasDimensions for __Context__
where
    __Context__: HasField<Symbol!("width"), Value = f64>,
    __Context__: HasField<Symbol!("height"), Value = f64>,
{
    fn width(&self) -> &f64 {
        self.get_field(PhantomData::<Symbol!("width")>)
    }

    fn height(&self) -> &f64 {
        self.get_field(PhantomData::<Symbol!("height")>)
    }
}
```

The associated-type form lifts the type into an extra generic parameter on the impl and binds it through
the field's value, which lets the field decide it:

```rust
impl<__Context__, Name> HasName for __Context__
where
    Name: Display,
    __Context__: HasField<Symbol!("name"), Value = Name>,
{
    type Name = Name;

    fn name(&self) -> &Self::Name {
        self.get_field(PhantomData::<Symbol!("name")>)
    }
}
```

The declared bound moves onto that parameter, so `Display` is required of whatever the field holds. Note
that the field tag still comes from the **method** name rather than from the associated type's.

## Common Mistakes

**The field name is fixed to the method name.** A context storing the value under any other name does not
satisfy the trait, and because the failure is an unmet bound on a blanket impl the method reads as missing
rather than the field:

```text
error[E0599]: the method `name` exists for reference `&Person`, but its trait bounds were not satisfied
```

Either rename the field, implement the trait by hand, or use [`#[cgp_getter]`](./cgp_getter.md), which
exists precisely to decouple the two.

**An associated type forces a single method**, since the type is inferred from one field and two getters
give it nothing to choose between:

```text
error: if associated type is defined, exactly one getter method must be defined
```

**The macro takes no arguments.** Unlike its sibling [`#[cgp_getter]`](./cgp_getter.md) there is no
provider trait to name, because no component is generated, and an argument is rejected rather than
ignored:

```text
error: #[cgp_auto_getter] does not accept any attribute argument
```

**The trait still has to be in scope to call its method.** That is ordinary Rust, but it surprises readers
here, because nothing else about the getter had to be declared. A missing `use` reports the method as not
found rather than the trait as not imported.

## Related constructs

- [`#[implicit]`](../attributes/implicit.md) — the default way to read a field, and the form to prefer.
- [`#[cgp_getter]`](./cgp_getter.md) — the wireable counterpart, for per-context field choice.
- [`#[derive(HasField)]`](../derives/derive_has_field.md) — what a context derives to qualify.
- [`HasField`](../traits/has_field.md) — the trait the generated bound is written against.
- [`Symbol!`](./symbol.md) — the type-level field name the bound is keyed on.
- [`#[uses]`](../attributes/uses.md) — how a provider depends on a getter by name.
- [`#[cgp_type]`](./cgp_type.md) — for an abstract type standing on its own rather than inferred from a
  field.

The ideas behind it:

- [Implicit arguments](/docs/concepts/implicit-arguments) — the lighter way to read a field, and why
  a getter is the exception.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — why the field requirement stays
  off the trait interface.

## Source

- Entry point: [`cgp_auto_getter.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_auto_getter.rs)
- Implementation: [`types/cgp_auto_getter/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_auto_getter/)
- Getter parsing and the return-type shorthands: [`functions/getter/parse.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/functions/getter/parse.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
