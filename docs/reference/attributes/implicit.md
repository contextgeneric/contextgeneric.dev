---
sidebar_label: '#[implicit]'
sidebar_position: 1
---

# `#[implicit]`

Source a function argument from a same-named field on the context.

## Overview

`#[implicit]` lets a CGP construct read a field value from a generic **context** by naming the field
like an ordinary function argument. The context is the type the construct runs against, and the
implicit argument is hidden from the public signature.

```rust
#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}
```

A call like `rect.rectangle_area()` does not require the arguments to be passed explicitly. Instead, the
`width` and `height` fields are automatically extracted from the `rect` value. Any type carrying a
`width` and a `height` field can call it.

`#[implicit]` gives better ergonomics than the alternatives, such as
[`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) or the direct use of
[`HasField`](../traits/field-access/has_field.md). With those, you declare that the context has each field and then
fetch it by hand, so you have to understand the mechanism behind field access, such as how a type-level
[`Symbol!`](../macros/symbol.md) and a `PhantomData` tag work. `#[implicit]` keeps all of that behind an
argument that reads as `width: f64`, and the argument's *name* names the field.

That is why it is the recommended way to read a context's own field, and why it is usually the first
piece of CGP anyone writes. A reader who understands functions and arguments can write a working
capability and meet the machinery later, when they have a reason to care.

## Usage

`#[implicit]` is a bare marker on a typed argument. It takes no arguments in any form, and a list or
name-value spelling is rejected rather than ignored:

```rust
fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}
```

`#[implicit]` works only inside the two macros that rewrite a function body into an implementation:
[`#[cgp_fn]`](../macros/cgp_fn.md) and the methods of a [`#[cgp_impl]`](../macros/cgp_impl.md) block. It
is not a macro of its own, so it does nothing on an ordinary function.

Three rules constrain where it may appear. The function must take `self` first, since the field is read
from `self`. The argument must be a bare identifier, not a destructuring or `mut` pattern. And a
**mutable** implicit argument carries two further requirements of its own, covered in
[Mutable arguments](#mutable-arguments) below.

### How the argument's type decides the read

The body works with the argument's type, and the macro inserts whatever conversion bridges it to the
stored field. The table below is the whole mapping:

| The argument's type | The field's type | How it is read |
|---|---|---|
| An owned type (`f64`, `String`, a tuple, an array) | the same type | by reference, then `.clone()` |
| `&T` | `T` | by reference, no conversion |
| `&str` | `String` | `.as_str()` |
| `&[T]` | anything `AsRef<[T]>` | `.as_ref()` |
| `Option<&T>` | `Option<T>` | `.as_ref()` |
| `Option<&str>` | `Option<String>` | `.as_deref()` |
| `&mut T` | `T` | mutably, no conversion |
| `&mut [T]` | anything `AsMut<[T]>` | `.as_mut()` |
| `Option<&mut T>` | `Option<T>` | `.as_mut()` |
| `Option<&mut str>` | `Option<String>` | `.as_deref_mut()` |
| [`MRef<'a, T>`](../types/mref.md) | `T` | by reference, wrapped as `MRef::Ref(…)` |

Three of those rows are easy to misread. `&str` is the case most often wanted and least obvious: the
field is a `String`, and the argument borrows from it, so a context never has to store a `&str`. An
owned argument *clones* the value. That is cheap for an `f64` and less so for a large `String`, which
is a reason to take `&str` or `&T` wherever the body only needs to read. And `MRef` is the
owned-or-borrowed form: it reads a plain `T` field, hands the body a value that may be either, and is
the one reference-shaped row with no mutable counterpart.

The macro matches the `MRef` row by *shape* rather than by resolving the name: a single-segment `MRef`
with one lifetime argument and one type argument. Anything else spelled `MRef` falls into the owned row
and is cloned, which is the one place a small change to the type silently changes the read.

The same rules govern the getter traits, so learning them once covers everywhere CGP reads a field.

### Mutable arguments

An implicit argument is mutable when its type carries a `&mut`: the outer reference of a `&mut T` or a
`&mut [T]`, or the inner reference of an `Option<&mut T>` or an `Option<&mut str>`. A mutable argument
reads through [`HasFieldMut`](../traits/field-access/has_field_mut.md) and `get_field_mut` rather than through
[`HasField`](../traits/field-access/has_field.md) and `get_field`, so it borrows the field for writing.

Two rules follow. A mutable argument requires a `&mut self` receiver, since a function cannot borrow a
field mutably through a shared `&self`. And it must be the only implicit argument on its function,
because reading one field mutably borrows the whole context exclusively and cannot coexist with any
other field read.

Mutability follows the *argument's* type rather than the receiver's. An argument carrying a `&mut` reads
mutably, and every other argument, an `MRef` included, reads through a shared borrow. A `&mut self`
function may therefore still take any number of immutable implicit arguments, as long as none of them is
mutable.

To get a mutable local from a field the body only reads, take the argument immutably and clone it inside
the body. `#[implicit]` rejects a `mut` binding on the argument itself, because a mutable argument means
a mutable field borrow rather than a mutable local copy.

## Examples

A complete capability, and a context that qualifies for it by having the right fields:

```rust
use cgp::prelude::*;

#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

pub fn print_area(rect: &Rectangle) {
    println!("area = {}", rect.rectangle_area());
}
```

`Rectangle` derives [`HasField`](../derives/derive_has_field.md), which is its entire qualification.
There is no wiring anywhere in this program.

Inside a [`#[cgp_impl]`](../macros/cgp_impl.md) provider the attribute behaves identically and mixes
freely with the method's real arguments. The implicit ones vanish from the signature while `to` and
`body` remain:

```rust
#[cgp_impl(new SendViaSmtp)]
impl EmailSender {
    fn send_email(&self, #[implicit] smtp_server: &str, to: &str, body: &str) {
        // connect to `smtp_server` and send the message
    }
}
```

Callers still write `app.send_email(to, body)`. The provider's requirement from its context, an
`smtp_server` field borrowed here as a `&str` from a `String`, never appears in the trait everyone else
calls.

## When to use it

**Reach for `#[implicit]` by default whenever a provider needs a value from its own context.** It is the
shortest form, it introduces no new declaration, and it uses the same field access a getter would, so a
getter trait declared only to read a field adds a name for no benefit.

That reach is wider than it first appears. An implicit argument reads a plain `&T` by reference with no
clone, so it suits a large value as well as a small one. A field that several providers each need is
declared as the same implicit argument in each of them, which costs nothing and keeps every provider's
requirements visible where the provider is written.

A **getter trait** is worth using in the three cases an implicit argument cannot reach.

- **The field is on a different type.** An implicit argument reads only from `self`, so a value living on
  a request, a payload, or any other type needs a getter that can be demanded as a bound on *that* type.
  There is no `self` field to read.
- **The accessor must be a named capability.** When other code depends on "this context can tell you its
  name" rather than on a field, that dependency needs a trait to point at, importable with
  [`#[uses]`](uses.md) or usable as a supertrait.
- **The getter carries a type inferred from the field.** A getter may declare an associated type and
  return it, which keeps the type abstract for callers in a way a concrete argument cannot.

For those, use [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md), which generates the getter from the
method name. [`#[cgp_getter]`](../macros/cgp_getter.md) goes further still and is genuinely advanced:
reach for it only when the *field a getter reads* should be chosen per context at wiring time.

## Under the hood

Each marked argument becomes two things: a field bound on the generated implementation, and a `let`
binding at the top of the body. From this input:

```rust
#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}
```

the macro strips the arguments from the trait's method and turns them into
[`HasField`](../traits/field-access/has_field.md) bounds and reads:

```rust
pub trait RectangleArea {
    fn rectangle_area(&self) -> f64;
}

impl<__Context__> RectangleArea for __Context__
where
    Self: HasField<Symbol!("width"), Value = f64>
        + HasField<Symbol!("height"), Value = f64>,
{
    fn rectangle_area(&self) -> f64 {
        let width: f64 = self
            .get_field(PhantomData::<Symbol!("width")>)
            .clone();
        let height: f64 = self
            .get_field(PhantomData::<Symbol!("height")>)
            .clone();

        width * height
    }
}
```

The macro inserts the bindings in argument order, ahead of every original statement, so the names are in
scope for the whole body. [`Symbol!("width")`](../macros/symbol.md) is a type-level string standing for the
field name. The compiler prints its expanded `Symbol<5, Chars<'w', …>>` form in errors, and
`cargo cgp expand` resugars it back. The context parameter is literally `__Context__`, a reserved name
chosen so it cannot collide with one of yours.

Two shapes in the table above generate something other than a `Value = T` equality, which is worth
recognizing because it reads oddly at first. A slice argument bounds the *associated type* instead, so
`&[u8]` emits `HasField<Symbol!("slice"), Value: AsRef<[u8]> + 'static>`, so any field whose value can be
viewed as a `[u8]` qualifies rather than one exact type. And every mutable form swaps the trait as well
as the accessor: `&mut u64` emits `HasFieldMut<Symbol!("counter"), Value = u64>` and reads through
`get_field_mut`.

Inside a [`#[cgp_impl]`](../macros/cgp_impl.md) block the rewrite is the same, except the bounds join
that provider's `where` clause. One addition shows only in a multi-method block: the macro collects the
bounds across *every* method and de-duplicates them, so two methods each taking `#[implicit] name: &str`
add one `HasField<Symbol!("name"), Value = String>` bound rather than two. The macro still emits the
`let` bindings per method, since each body needs its own.

## Common Mistakes

**The attribute takes no arguments**, and says so rather than ignoring what it was given:

```text
error: `#[implicit]` does not take any arguments; write it as a bare `#[implicit]`
```

**The argument must be a plain identifier.** A destructuring pattern reports `Expected an identifier`,
and a `mut` binding is rejected with the fix in the message:

```text
error: Mutable variables are not allowed in implicit arguments. (Explicitly clone a `&` reference if you
       want a mutable local copy of the value)
```

**A function with implicit arguments must take `self` first**, since there is otherwise no context to
read from:

```text
error: The first argument of a function with implicit arguments must be `self`
```

**A mutable argument needs a `&mut self` receiver, and must be the only implicit argument.** The two
mistakes report separately:

```text
error: &mut self is required for mutable field reference `& mut u64`

error: a `&mut` implicit argument must be the only implicit argument, since its mutable
       borrow of the context conflicts with reading any other field
```

## Related constructs

- [`#[cgp_fn]`](../macros/cgp_fn.md) — the usual host, turning a function into a capability.
- [`#[cgp_impl]`](../macros/cgp_impl.md) — the other host, for a component's provider.
- [`#[derive(HasField)]`](../derives/derive_has_field.md) — what a context derives to qualify.
- [`HasField`](../traits/field-access/has_field.md) — the trait the generated bounds are written against.
- [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) — the getter form, for the cases above.
- [`#[cgp_getter]`](../macros/cgp_getter.md) — a getter whose source field is chosen by wiring.
- [`#[uses]`](uses.md) — imports a capability rather than a value.
- [`#[use_type]`](use_type.md) — imports a type rather than a value; the third of the three.
- [`Symbol!`](../macros/symbol.md) — the type-level field name the bounds are keyed on.

The ideas behind it:

- [Implicit arguments](/docs/concepts/implicit-arguments) — the idea in full, and why it is the
  default way to reach a context's field.

## Source

- Parsing: [`functions/implicits/parse.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/functions/implicits/parse.rs)
- Bounds and bindings: [`types/implicits/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/implicits/)
- Access-mode mapping: [`functions/field/parse.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/functions/field/parse.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
