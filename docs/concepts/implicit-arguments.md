---
sidebar_label: 'Implicit arguments'
sidebar_position: 4
---

# Implicit arguments

Writing a provider as an ordinary function whose arguments are filled from fields of the context.

This page answers *how does an implementation get a value out of its context, without that becoming a
thing to learn?* It shows what reading a field costs when spelled out, the argument that replaces it,
and the rules deciding how the value arrives. It closes on where an implicit argument stops working and
a getter is the right tool instead.

## What reading a field costs, spelled out

A provider almost always needs data from its context, and the underlying mechanism is a trait keyed by
the field's name lifted into a type:

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

This works, and everything in it is doing something. `Symbol!("name")` is the field's name as a type, so
one trait can describe every field rather than needing one trait per field. `PhantomData` carries that
type to the call so inference knows which field is meant. The bound is an
[impl-side dependency](./impl-side-dependencies.md), which is why the capability's own trait says
nothing about names.

It is also four unfamiliar things stacked in front of a one-line function, and a reader meeting them
before they have a reason to care about type-level tags will conclude that CGP *is* that. The value the
provider wants is a `String` called `name`; nothing in the paragraph above is about that.

## The same provider, written as a function

An **implicit argument** says the same thing as a parameter:

```rust
#[cgp_impl(new GreetByName)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}
```

The argument is not passed by anyone. `#[implicit]` removes it from the method's public signature, adds
the field requirement to the implementation, and binds the value at the top of the body — so a caller
still writes `app.greet()` with no arguments, and the body reads a plain local.

Nothing was hidden that was not already hidden. The bound, the tag, and the read are the same three
things; what changed is that the author writes the name of the value they want and the macro derives
the rest from it. Someone who understands functions and arguments can write a complete provider without
meeting a type-level anything, which is why this is the recommended way to read a context field and the
on-ramp most introductions to CGP should use.

## A capability from a function alone

Combined with [`#[cgp_fn]`](/docs/reference/macros/cgp_fn), an implicit argument gets you a working CGP
capability with no trait, no provider, and no wiring line:

```rust
#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}
```

Any context with a `width` and a `height` can now call `rectangle_area()`. That is the whole program —
there is no component to define and nothing to choose, because this capability has one implementation
and needs none. It is the smallest useful thing CGP does, and the right place to start a codebase that
may never need more.

## The declared type decides how the field is read

The type on the argument says what the body wants, and the macro inserts whatever bridges it to the
field the context stores. Three cases cover nearly everything:

```rust
#[cgp_fn]
pub fn describe(
    &self,
    // Owned: read by reference and cloned, so the context keeps its field.
    #[implicit] name: String,
    // `&str`: backed by a `String` field, borrowed rather than cloned.
    #[implicit] title: &str,
    // Any other borrow: taken as it stands.
    #[implicit] tags: &Vec<String>,
) -> String {
    format!("{title} {name} {tags:?}")
}
```

The rule of thumb is that the declared type is what the body works with and the conversion is the
macro's problem. Prefer a borrow where the body only reads — `&str` over `String` — since that is free
while an owned argument clones. Further forms exist for options and slices, and they are enumerated on
the [`#[implicit]`](/docs/reference/attributes/implicit) reference page rather than here.

These are the same rules a getter follows, so learning them once covers both.

## When a getter trait is still the right thing

An implicit argument reads from the provider's own `self`. That covers every value a provider wants
from its own context — including one that several providers each read, declared as the same argument in
each — so it is the default, and a getter trait is the exception.

Three cases fall outside it. The value may live on a **type other than the context**, where there is no
`self` field to read and the requirement is a bound on that other type:

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

`HasAuthHeader` there is a getter on `Request`, not on the application, and no implicit argument can
express that. The accessor may also need to exist as a **named capability** that other code depends on
through `#[uses(...)]` or a supertrait — a name is something you can require, and an argument is not. Or
the getter may carry an **associated type inferred from the field**, so callers stay generic over what
the value actually is.

Outside those three, prefer the argument. A getter trait declared only so a provider can read a field of
its own context is a trait, an impl, and an import bought for nothing.

## What it costs

**The field name is part of the interface, silently.** Renaming a struct field breaks every provider
whose argument was named after it, and the failure surfaces as a missing-field error at whatever site
forces the check rather than at the rename. Nothing in the struct definition marks the coupling.

**The requirement is invisible at the call site.** `app.greet()` gives no hint that the context must
carry a `name`. That is the point — it is what keeps the requirement off the interface — and it is also
why the answer to "what does this context need?" lives in the providers it wired rather than in the
traits it implements.

**Matching by name is looser than matching by type.** Two unrelated values with the same name and type
are the same implicit argument as far as the machinery is concerned, so a context that happens to have a
field called `name` satisfies a provider written for something else. In practice that is what makes the
mechanism so cheap; it is still worth knowing that nobody is checking your intent.

**The clone is real.** An owned argument copies the field on every call. Usually that is nothing, and on
a large value in a hot path it is not — declare a borrow.

## Where to go next

[Impl-side dependencies](./impl-side-dependencies.md) is the general idea an implicit argument is one leg
of: a requirement stated on the implementation rather than on the interface.
[Abstract types](./abstract-types.md) is the third leg, where what the implementation needs is a type
rather than a value.

To write this rather than read about it, the [Hello World tutorial](/docs/tutorials/hello) reaches an
implicit argument within its first few minutes, and the
[Area calculation series](/docs/tutorials/area-calculation/) builds up from plain functions of exactly
this shape.

For the constructs, [`#[implicit]`](/docs/reference/attributes/implicit) carries the full list of
accepted forms and access rules, [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) is the no-wiring
capability, [`#[cgp_auto_getter]`](/docs/reference/macros/cgp_auto_getter) is the getter to reach for in
the three cases above, and [`HasField`](/docs/reference/traits/has_field) is the trait underneath all of
them.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
