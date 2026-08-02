---
sidebar_label: '#[blanket_trait]'
---

# `#[blanket_trait]`

Generate a blanket impl from a trait with default methods and supertrait dependencies.

## What it's for

The pattern CGP is built on is an **extension trait**: a trait with a clean interface whose
[blanket implementation](https://blog.implrust.com/posts/2025/09/blanket-implementation-in-rust/) carries the
real requirements in its `where` clause. A trait declared `FooBar: Foo + Bar` with a default `foo_bar` method
exposes only `foo_bar` to callers, while the `Foo + Bar` requirements live on the impl — so any type
satisfying them gains the capability, and nobody calling `foo_bar` has to know or repeat what it needed.

That is worth doing rather than writing a generic function, because a function's `where` clause propagates:
every generic caller repeats it, and so does every caller of *those*. Hiding the requirement on an impl stops
the cascade at one level.

Written by hand the pattern is repetitive. You state the supertraits twice — once on the trait, once on the
impl — and copy each default body across. `#[blanket_trait]` lets you write the trait once and generates the
impl:

```rust
#[blanket_trait]
pub trait FooBar: Foo + Bar {
    fn foo_bar(&self) {
        self.foo();
        self.bar();
    }
}
```

**This is not a CGP component.** There is no consumer/provider split, no component marker, and no wiring —
just an ordinary Rust trait and an ordinary blanket impl. It is the tool for a capability with exactly one
definition, where you want extension-trait ergonomics without committing to the component machinery. When a
second implementation becomes necessary, the trait can be promoted to a
[`#[cgp_component]`](./cgp_component.md).

## Using it

`#[blanket_trait]` is **not exported by the prelude**, so it needs an explicit import:

```rust
use cgp::prelude::*;
use cgp::core::macros::blanket_trait;
```

Apply it to a trait definition. Every method and constant must have a default, since those defaults are what
the generated impl forwards, and the trait's supertraits become the dependencies the impl requires:

```rust
#[blanket_trait]
pub trait FooBar: Foo + Bar {
    fn foo_bar(&self) {
        self.foo();
        self.bar();
    }
}
```

The attribute takes an optional identifier naming the generic **context** type — the type the capability runs
against — in the generated impl. Omitted, it is the reserved `__Context__`, chosen so it cannot collide with
one of your own type parameters:

```rust
#[blanket_trait(Ctx)]
pub trait FooBar: Foo + Bar { /* ... */ }
```

The trait may carry generic parameters, which are copied onto the impl, and associated types, which get the
special treatment described next.

### Lifting an associated type out of a supertrait

An associated type on the trait becomes a fresh generic parameter on the impl, bound through the supertrait's
associated-type equality. This is one of the pattern's main uses — re-exporting a supertrait's type under a
local name:

```rust
#[blanket_trait]
pub trait HasFooTypeAtBar: HasFooTypeAt<BarTag, Foo = Self::FooBar> {
    type FooBar: Clone;
}
```

A context implementing `HasFooTypeAt<BarTag>` gets `HasFooTypeAtBar` automatically, with `FooBar` resolved to
whatever its `Foo` was. Bounds on the associated type move onto the lifted parameter, so `Clone` above becomes
a requirement on that underlying type. Associated types need no default — the macro supplies the assignment
itself.

## Examples

An extension trait hiding two dependencies behind one method:

```rust
use cgp::prelude::*;
use cgp::core::macros::blanket_trait;

pub trait Foo {
    fn foo(&self);
}

pub trait Bar {
    fn bar(&self);
}

#[blanket_trait]
pub trait FooBar: Foo + Bar {
    fn foo_bar(&self) {
        self.foo();
        self.bar();
    }
}

pub struct Ctx;

impl Foo for Ctx {
    fn foo(&self) {}
}

impl Bar for Ctx {
    fn bar(&self) {}
}

pub fn run(ctx: &Ctx) {
    ctx.foo_bar(); // available because Ctx: Foo + Bar
}
```

`Ctx` never mentions `FooBar`. It implements `Foo` and `Bar`, and the generated blanket impl supplies
`foo_bar` — while a caller of `foo_bar` sees a one-method interface and is shielded from the requirement
entirely.

The degenerate case is useful too. A trait with no methods becomes a trait alias in all but name, which is a
tidy way to bundle a set of bounds under one shorter name:

```rust
#[blanket_trait]
pub trait FooBar: Foo + Bar {}
```

Any type that is `Foo + Bar` is now `FooBar`, and a signature can say so in one word.

## When to reach for it, and when not

**Reach for `#[blanket_trait]` when a capability has one definition and its dependencies are other traits.**
That last clause is the real discriminator, and it is what separates this macro from its closest neighbour.

- **[`#[cgp_fn]`](./cgp_fn.md) when the dependencies are context *fields*.** Both macros produce a
  single-implementation capability with a blanket impl and no wiring, so the choice is about the input.
  `#[cgp_fn]` derives the trait and body from a function, reading values through
  [`#[implicit]`](../attributes/implicit.md) arguments; `#[blanket_trait]` takes a trait with supertraits and
  default bodies. Values point to `#[cgp_fn]`, capabilities point here.
- **[`#[cgp_component]`](./cgp_component.md) when a second implementation is needed.** A blanket impl covers
  every type satisfying its bounds, which leaves nowhere for an alternative to live. Promoting later is
  cheap — the trait keeps its name and method — so starting here costs nothing if that changes.
- **A trait alias, when one exists.** The empty-body form above is a workaround for a language feature Rust
  does not have on stable; it is the right workaround, but do not reach for the macro if a plain supertrait
  bound reads fine at the use site.
- **Nothing at all, for a plain generic function.** If the requirement genuinely belongs in the signature
  and no caller is generic over the type, a function with a `where` clause is simpler and the propagation
  problem never arises.

One caution specific to this construct: a blanket impl is **coherence-visible**, so it competes with any
other impl of the same trait. That is why the pattern admits exactly one definition, and why two
`#[blanket_trait]` traits whose bounds can both hold for one type cannot both cover that type for the same
trait. Needing that is the signal to move to a component.

## Under the hood

:::note

### Advanced

This section shows the two items the macro emits. You do not need them to use `#[blanket_trait]`, but the
associated-type lifting is genuinely surprising the first time, and knowing what the impl requires makes an
"unsatisfied bound" error much easier to read. `cargo cgp expand` prints the same thing for your own code.

:::

The macro emits the trait and a blanket impl over a generic context. From the method example:

```rust
#[blanket_trait]
pub trait FooBar: Foo + Bar {
    fn foo_bar(&self) {
        self.foo();
        self.bar();
    }
}
```

it produces the trait **unchanged, default body included**, plus the impl whose `where` clause carries the
supertraits as the hidden dependency:

```rust
pub trait FooBar: Foo + Bar {
    fn foo_bar(&self) {
        self.foo();
        self.bar();
    }
}

impl<__Context__> FooBar for __Context__
where
    __Context__: Foo + Bar,
{
    fn foo_bar(&self) {
        self.foo();
        self.bar();
    }
}
```

The body appears twice — once as the trait's default and once in the impl — and that is what the macro
actually emits rather than a simplification. The impl is what supplies the method for every qualifying type;
the retained default is harmless, and a type implementing the trait by hand can still rely on it.

A trait with no methods generates an empty impl, which is the trait-alias form:

```rust
impl<__Context__> FooBar for __Context__
where
    __Context__: Foo + Bar,
{}
```

Associated types are where the expansion earns its keep. Each becomes a new generic parameter on the impl,
the supertrait's equality bound is rewritten to name that parameter, and the impl assigns it back:

```rust
pub trait HasFooTypeAtBar: HasFooTypeAt<BarTag, Foo = Self::FooBar> {
    type FooBar: Clone;
}

impl<__Context__, FooBar> HasFooTypeAtBar for __Context__
where
    __Context__: HasFooTypeAt<BarTag, Foo = FooBar>,
    FooBar: Clone,
{
    type FooBar = FooBar;
}
```

Reading it back: the impl applies to any context whose `HasFooTypeAt<BarTag>::Foo` is some `FooBar` that is
`Clone`, and re-exposes that type under the local name. The declared bound moved from the associated type onto
the lifted parameter, which is what makes it a requirement on the *underlying* type rather than an assertion
about the alias.

Associated constants are forwarded like methods — their default expressions become the impl's definitions. A
method or constant with no usable default is an error, since the macro has nothing to forward.

<details>
<summary>Formal grammar</summary>

The attribute argument is a single optional context name, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
BlanketTraitArgs -> ContextName?

ContextName      -> IDENTIFIER
```

Omitted, the generic context type in the generated impl is the reserved identifier `__Context__`; a given
`IDENTIFIER` overrides that name.

</details>

## Gotchas

**The macro is not in the prelude.** `use cgp::prelude::*;` does not bring it into scope, and the failure is
reported as an unknown attribute rather than as a missing import:

```text
error: cannot find attribute `blanket_trait` in this scope
```

Add `use cgp::core::macros::blanket_trait;`. This is the one macro on this page's family that needs it —
`#[cgp_fn]`, `#[cgp_component]`, and the rest are all re-exported by the prelude.

**A method without a default body is an error.** The macro forwards defaults into the impl, so a bare
declaration leaves it nothing to emit. This is the opposite of an ordinary trait, where a bodiless method is
the normal case.

**The trait keeps its default bodies.** The expansion does not strip them, so each body appears both on the
trait and in the impl. Harmless, but worth knowing when reading an expansion and wondering whether the macro
duplicated something by mistake.

**The blanket impl blocks any hand-written impl of the same trait.** Because it covers every type satisfying
its bounds, implementing the trait manually for a type that also satisfies them is a coherence conflict.
Where a type needs different behaviour, the trait wants to be a [component](./cgp_component.md) instead.

## Related constructs

- [`#[cgp_fn]`](./cgp_fn.md) — the same single-implementation, no-wiring shape, built from a function whose
  dependencies are fields rather than traits.
- [`#[cgp_component]`](./cgp_component.md) — the step up, when one implementation is not enough.
- [`#[cgp_auto_getter]`](./cgp_auto_getter.md) — the same constraint-hiding applied to reading a field.
- [`HasField`](../traits/has_field.md) — what value-level dependency injection is built on.
- [`#[uses]`](../attributes/uses.md) — how a CGP provider declares the trait dependencies a supertrait
  declares here.

## Source

- Entry point: [`blanket_trait.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/blanket_trait.rs)
- Implementation: [`types/blanket_trait.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/blanket_trait.rs)
- The `Self`-to-parameter rewriting for associated types: [`visitors/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/visitors/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
