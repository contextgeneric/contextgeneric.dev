---
sidebar_label: '#[extend]'
sidebar_position: 5
---

# `#[extend]`

Add a capability as a supertrait of a generated trait, so callers can rely on it too.

## Overview

`#[extend]` declares that a capability is part of what a trait *guarantees*, rather than something its
implementation happens to need. Where [`#[uses]`](uses.md) adds a private requirement, `#[extend]` adds a
supertrait. Every **context** implementing the trait, meaning every type the capability runs against,
must also satisfy it, and every caller may rely on it.

```rust
#[extend(HasName)]
```

The clearest way to hold the pair apart is that **`#[extend]` is the `pub use` to `#[uses]`'s `use`**.
Both take the same syntax and both read as imports, but one imports a capability for the implementation's
own use and keeps it out of sight, while the other re-exports it as part of the contract.

That framing is also why `#[extend]` is preferred over Rust's native supertrait syntax on a
[`#[cgp_component]`](../macros/cgp_component.md). Writing `pub trait CanGreet: HasName` reads as
inheritance from a parent, an is-a relationship that a CGP supertrait is not. `#[extend(HasName)]` reads
as importing a capability the trait passes on, which is the accurate description. The two generate the
same trait, so this is a choice about how the definition reads.

On [`#[cgp_fn]`](../macros/cgp_fn.md) the attribute is not merely preferred but necessary. The `#[cgp_fn]`
macro treats a `where` clause you write as an implementation detail and never puts it on the generated
trait, so you cannot spell a supertrait by hand. `#[extend]` is the only mechanism for one.

## Usage

`#[extend(...)]` takes a comma-separated list of trait bounds, each becoming a supertrait of the generated
trait:

```rust
#[extend(HasName, HasTitle)]
```

Each entry names a trait, optionally with type arguments. You may list several in one attribute or spread
them across attributes, and they accumulate. Prefer one attribute carrying the whole list, as with
[`#[uses]`](uses.md).

The attribute is accepted on [`#[cgp_fn]`](../macros/cgp_fn.md) and on
[`#[cgp_component]`](../macros/cgp_component.md). It is **not** available on
[`#[cgp_impl]`](../macros/cgp_impl.md), which has no trait definition of its own to attach a supertrait to.
The supertraits belong to the component's trait, and a provider states its private needs with
[`#[uses]`](uses.md) instead.

### When the supertrait supplies a type

If the reason for the supertrait is that the trait's signatures name a type the other component provides
(an error type, a scalar, a runtime), reach for [`#[use_type]`](use_type.md) instead. It adds the
supertrait *and* rewrites bare mentions of the type into their fully qualified form, so the signature
reads `Result<String, Error>` rather than `Result<String, <Self as HasErrorType>::Error>`:

```rust
#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}
```

`#[extend]` is therefore for a **capability** supertrait, one whose methods matter and whose associated
types the signatures do not name.

## Examples

A `#[cgp_fn]` capability that guarantees two getters alongside its own method:

```rust
use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_auto_getter]
pub trait HasTitle {
    fn title(&self) -> &str;
}

#[cgp_fn]
#[extend(HasName, HasTitle)]
pub fn full_label(&self) -> String {
    format!("{} {}", self.title(), self.name())
}
```

The body can call both getters, and so can anyone holding a `T: FullLabel`, which is the difference from
`#[uses]`. A function that takes a labelled value can format its name without asking for `HasName`
separately.

On a component the effect is on the consumer trait, and the supertrait becomes part of the capability:

```rust
#[cgp_component(Greeter)]
#[extend(HasName)]
pub trait CanGreet {
    fn greet(&self);
}
```

`CanGreet` now means "can greet, and can tell you its name". Every provider may rely on the name being
available, and so may every caller.

## When to use it

**Reach for `#[extend]` when callers of the trait should be able to rely on the capability too**, and for
a capability supertrait on a `#[cgp_component]` in preference to native `:` syntax. Reach for something
else when the requirement belongs elsewhere, and the choice turns on where it should be visible.

- **The implementation needs it privately.** Use [`#[uses]`](uses.md). This is the common case by a wide
  margin: most dependencies are private to the implementation, and putting one on the trait forces it on
  every context and every caller whether they touch it or not.
- **The signatures name a type it supplies.** Use [`#[use_type]`](use_type.md), which adds the supertrait
  and removes the qualified paths.
- **The requirement is a predicate rather than a supertrait**, a bound on a generic parameter rather than
  on `Self`. Use [`#[extend_where]`](extend_where.md), which puts it on the generated trait's own `where`
  clause.

Weigh one cost before promoting anything. A supertrait widens the contract permanently: you cannot later
narrow it without breaking every implementor, and it demands the capability from contexts that only ever
call the one method the trait actually declares. Prefer `#[uses]` unless callers genuinely need the
guarantee.

## Under the hood

On a `#[cgp_fn]`, each entry lands in **two** places: as a supertrait of the generated trait, and as a
`Self:` predicate on the generated implementation. From this input:

```rust
#[cgp_fn]
#[extend(HasName, HasTitle)]
pub fn full_label(&self) -> String {
    format!("{} {}", self.title(), self.name())
}
```

the macro emits:

```rust
pub trait FullLabel: HasName + HasTitle {
    fn full_label(&self) -> String;
}

impl<__Context__> FullLabel for __Context__
where
    Self: HasName + HasTitle,
{
    fn full_label(&self) -> String { /* ... */ }
}
```

Callers see the supertrait, and the predicate on the implementation lets the body call the methods.
Compare [`#[uses]`](uses.md), which emits only the second of those two and leaves the trait declaration
bare. That single difference is the whole of the distinction.

On a `#[cgp_component]` the supertrait goes on the consumer trait, and it also joins the `where` clause of
the generated consumer blanket implementation, since that implementation can only apply where the
supertrait holds:

```rust
pub trait CanGreet: HasName {
    fn greet(&self);
}

impl<__Context__> CanGreet for __Context__
where
    __Context__: HasName,
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) {
        __Context__::greet(self)
    }
}
```

Here the result is identical to `pub trait CanGreet: HasName`, so on a component
`#[extend]` generates nothing the language cannot already spell. It remains the preferred form for the
reason given above: it presents the bound as an import rather than as inheritance, and it keeps the
`use`/`pub use` pairing with `#[uses]` reading consistently across both macros.

## Formal grammar

The attribute argument is a comma-separated list of bounds, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
ExtendArgs -> TypeParamBound ( `,` TypeParamBound )* `,`?
```

This is the same production [`#[uses]`](uses.md) accepts, the Rust grammar's own bound, so a lifetime, a
`?Sized`, or an associated-type equality parses as readily as a plain trait name. The list may be empty,
and the attribute may be repeated, with every occurrence's entries collected together. The two attributes
differ in where the bounds land, not in the grammar.

## Common Mistakes

**On a `#[cgp_impl]` the attribute is not recognized at all**, rather than being accepted and ignored.
Nothing consumes it, so it falls through to the compiler as an unknown attribute:

```text
error: cannot find attribute `extend` in this scope
```

Expect a second error alongside it, because the bound was never added. The body's calls to the capability
then fail with `E0599`, reporting that the method exists but its trait bounds were not satisfied. Both
errors have the same cause: move the requirement to [`#[uses]`](uses.md), or onto the component's trait
where supertraits belong.

**A supertrait cannot be narrowed later.** Because every implementor and every caller may now rely on it,
removing an entry from `#[extend]` is a breaking change in a way removing one from `#[uses]` is not. This
is the practical reason to default to `#[uses]`.

## Related constructs

- [`#[uses]`](uses.md) — the same syntax as a private bound on the implementation; the usual choice.
- [`#[extend_where]`](extend_where.md) — a predicate on the generated trait rather than a supertrait.
- [`#[use_type]`](use_type.md) — preferred when the supertrait exists to supply a type.
- [`#[cgp_fn]`](../macros/cgp_fn.md) — where `#[extend]` is the only way to declare a supertrait.
- [`#[cgp_component]`](../macros/cgp_component.md) — where it is preferred over native `:` syntax.
- [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) — defines the getter capabilities most often extended.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — the distinction between a
  requirement callers see and one they do not.

## Source

- Parsing: [`types/attributes/function.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/function.rs)
- On `#[cgp_fn]`: [`types/cgp_fn/preprocessed.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_fn/preprocessed.rs)
- On `#[cgp_component]`: [`types/attributes/cgp_component_attributes.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/cgp_component_attributes.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
