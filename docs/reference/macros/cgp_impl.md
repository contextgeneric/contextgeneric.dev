---
sidebar_label: '#[cgp_impl]'
sidebar_position: 2
---

# `#[cgp_impl]`

Write a provider using consumer-trait syntax, keeping `self`, `Self`, and the method signatures you
already know.

## Overview

A [provider trait](./cgp_component.md) is shaped inside-out compared with the trait it came from. The
original `Self` has moved into an explicit leading type parameter, the implementation targets a small
marker type rather than the type the capability is about, and the method receiver is a plain parameter.
Written by hand it looks like this:

```rust
impl<Context> AreaCalculator<Context> for RectangleArea
where
    Context: HasDimensions,
{
    fn area(context: &Context) -> f64 {
        context.width() * context.height()
    }
}
```

Every part of that is correct, and almost every part of it is unfamiliar. The trait carries a
parameter you did not write, `Self` is a type with no values, and there is no `self` anywhere. Written
this way, the code does not read as an implementation of `CanCalculateArea`, even though that is
exactly what it is. [Bypassing coherence](/docs/concepts/coherence) is why the trait needs a `Self` it
owns instead of the type the capability is really about.

`#[cgp_impl]` gives you back the familiar shape. Write the implementation as though you were
implementing the consumer trait directly, with `self` and `Self` and the signatures from the original
trait, and put the provider's name in the attribute:

```rust
#[cgp_impl(new RectangleArea)]
impl AreaCalculator
where
    Self: HasDimensions,
{
    fn area(&self) -> f64 {
        self.width() * self.height()
    }
}
```

The macro performs the rewrite for you. Write providers this way. The macro produces the inside-out
form shown earlier, and you will see that form in generated code.

This convenience has one exception worth stating plainly: **inside a `#[cgp_impl]` block, `self` and
`Self` mean the context, not the provider.** The context is the type the capability runs against, and
it supplies the values it needs as its own fields. The provider is a type-level name with no fields
and no value: it is never constructed, and there is nothing in it to read. The macro rewrites `self`
to the context value and `Self` to the context type because the context is the only thing that exists
when the method runs.

## Usage

Apply the attribute to an `impl` block. Its argument names the provider. The argument has three parts,
and only the name is required.

```rust
#[cgp_impl(new RectangleArea)]
impl AreaCalculator
where
    Self: HasDimensions,
{
    fn area(&self) -> f64 {
        self.width() * self.height()
    }
}
```

| Part | Meaning |
|---|---|
| `new` | Optional. Also emits `pub struct RectangleArea;`, so you need not declare the provider separately. |
| provider name | Required. The type that takes the `Self` position in the generated provider impl. |
| `: ComponentType` | Optional. Overrides which component this provider is registered as implementing. Defaults to the provider trait's name plus `Component`. |

The `for Context` clause is optional. Omitting it, as every example on this page does, lets the macro
insert a generic context parameter for you: write `impl AreaCalculator`, not
`impl<Context> AreaCalculator for Context`. Name the context explicitly only to bound it in a way the
omitted form cannot express, such as a lifetime or a higher-ranked bound, or to serve one *concrete*
context alone, as in `impl AreaCalculator for Rectangle`.
[When to use it](#when-to-use-it) says more about choosing between these
forms.

Without `new`, the provider struct must already exist. A wiring entry naming a struct nothing
declares is a common and confusing first error.

A provider may be generic. Parameters go in the attribute and in the impl generics together, which is
how a [higher-order provider](/docs/concepts/higher-order-providers) takes an inner provider:

```rust
#[cgp_impl(new ScaledAreaCalculator<InnerCalculator>)]
#[use_provider(InnerCalculator: AreaCalculator)]
impl<InnerCalculator> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        let base_area = InnerCalculator::area(self);
        base_area * scale_factor * scale_factor
    }
}
```

### Declaring the provider struct separately

`new` is convenient for a provider used once, but a struct implementing several provider traits, or
one whose shape `new` cannot express, is usually declared on its own instead.

**A provider often implements more than one component.** Each provider trait still needs its own
`#[cgp_impl]` block, but only one of those blocks should declare the struct. Write `new` on exactly
one of them and omit it from the rest:

```rust
#[cgp_impl(new RectangleGeometry)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[cgp_impl(RectangleGeometry)]
impl PerimeterCalculator {
    fn perimeter(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        2.0 * (width + height)
    }
}
```

The first block's `new` declares `pub struct RectangleGeometry;`; the second block names the same
struct without `new` and adds a second impl to it. Writing `new` on more than one block for the same
struct is a duplicate-definition error.

Declaring the struct explicitly is an equally valid style for this same case: write
`pub struct RectangleGeometry;` once, above both blocks, and omit `new` from all of them rather than
from just the second. Either style works; declaring the struct explicitly reads more consistently once
there are more than two or three blocks, since a reader does not have to find the one block carrying
`new` to know where the struct comes from. Neither style requires
[`#[cgp_provider]`](./cgp_provider.md): the struct is only being declared once and reused, and the body
of every block still reads as an ordinary trait impl.

**A struct whose shape `new` cannot express must be declared explicitly, because `new`'s own grammar is
narrow.** `#[cgp_impl(new ...)]` desugars to
[`#[cgp_new_provider]`](./cgp_provider.md#the-struct-cgp_new_provider-declares), and that macro can
only emit one of two shapes: a plain provider name becomes a unit struct with no fields, and
a generic provider becomes a tuple struct with one public
[`PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) field wrapping all of its
generic parameters together, always `pub` and never with a default. Anything the struct needs outside
that shape has to be written by hand.

The case you see most often is a default on a higher-order provider's inner parameter, so an
application can omit the inner provider and fall back to
[`UseContext`](../providers/use_context.md), which routes back through the context's own
implementation of the component. If `ScaledAreaCalculator` from [Usage](#usage) should default this
way, `new` has no syntax for it, and the struct has to be declared by hand instead:

```rust
pub struct ScaledAreaCalculator<InnerCalculator = UseContext>(PhantomData<InnerCalculator>);
```

Every `#[cgp_impl(ScaledAreaCalculator<InnerCalculator>)]` block implementing a component for it then
looks exactly like the block shown in [Usage](#usage), minus `new`. The same reasoning covers any other
shape `new` cannot spell: an extra derive, a visibility narrower than `pub`, or fields beyond that one
blanket `PhantomData`. Declare the struct with whatever shape it needs, and keep every implementing
block written with `#[cgp_impl(ProviderName)]`.

### Companion attributes

Several attributes are processed on a `#[cgp_impl]` block before the rewrite happens. Together they
let an idiomatic provider state what it needs.

- [`#[implicit]`](../attributes/implicit.md) on a parameter removes it from the signature and fills it
  from a same-named field on the context.
- [`#[uses(...)]`](../attributes/uses.md) adds the capabilities the provider depends on, reading like
  a `use` statement instead of a hand-written `where Self: Trait` clause.
- [`#[use_type(Trait.Type)]`](../attributes/use_type.md) imports an abstract type and rewrites its
  occurrences to fully qualified form.
- [`#[use_provider(...)]`](../attributes/use_provider.md) completes an inner provider's bound in a
  higher-order provider.
- [`#[default_impl(...)]`](../traits/namespace/default_namespace.md) registers the provider as a namespace's
  per-type default, emitting a delegation impl alongside it, for use with
  [`cgp_namespace!`](./cgp_namespace.md).

Each may be repeated. All except `#[use_provider]` also take a comma-separated list inside one
attribute, which is the form to prefer: `#[uses(HasName, CanRaiseError<String>)]` reads as one
dependency list. `#[use_provider]` is the exception because its own argument already ends in a bound
list, so a second pair after a comma has nowhere to go; write one attribute per inner provider.

`#[cgp_impl]` does not read three attributes you may see on other CGP macros: `#[extend]`,
`#[extend_where]`, and `#[impl_generics]`. All three act on a *generated trait definition*, which a
provider impl does not have, so they belong to [`#[cgp_fn]`](./cgp_fn.md) and, for `#[extend]`, to
[`#[cgp_component]`](./cgp_component.md). Writing one here leaves a name nothing resolves; see
[Common Mistakes](#common-mistakes). An impl-side bound that really is impl-side goes in the block's own `where`
clause, which passes through untouched.

### Implementing the consumer trait directly

Naming `Self` as the provider bypasses the rewrite entirely and emits the block unchanged as an
ordinary consumer-trait impl on a concrete type. This form requires the `for Context` clause. Omitting
it is rejected with an `Expected context type to be specified` error, since there is no context left to
find. The form is useful when you want a hand-written impl while still applying the companion
attributes. Because the macro does not generate a provider struct here, `new` and the component
override have no effect; see [Common Mistakes](#common-mistakes) for what happens if you write them anyway.

```rust
#[cgp_impl(Self)]
#[use_provider(RectangleArea: AreaCalculator)]
impl CanCalculateArea for Rectangle {
    fn area(&self) -> f64 {
        RectangleArea::area(self)
    }
}
```

## Examples

The most idiomatic form of a provider reads almost like a plain function. Given the `AreaCalculator`
component:

```rust
use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}
```

The macro removes the two `#[implicit]` parameters from the signature and replaces them with reads of
the `width` and `height` fields, so the provider requires a context that has them. The `new` keyword
defines `RectangleArea`. A concrete type then wires the component and calls through it:

```rust
#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

fn print_area(rect: &Rectangle) {
    println!("area = {}", rect.area());
}
```

## When to use it

**Write providers with `#[cgp_impl]`.** It is the recommended form for every provider, and the cases
that call for something else are narrower than they look.

Prefer the unqualified header from [Usage](#usage), `impl AreaCalculator` with no `for Context`,
letting the macro insert the context parameter, which keeps a provider reading like an ordinary trait
impl. Name the context explicitly only when the sugar falls short, such as a lifetime or a
higher-ranked bound.

Naming a *concrete* type as that context, as in `impl AreaCalculator for Rectangle`, is a different
choice worth keeping distinct from the
[`#[cgp_impl(Self)]` form](#implementing-the-consumer-trait-directly), which looks similar but does
something else. The concrete-context form still produces a named provider that a context wires like
any other. The `Self` form produces no provider at all.

Use something else in these cases:

- **The capability has only one implementation.** [`#[cgp_fn]`](./cgp_fn.md) builds it from a plain
  function with no component, no provider, and no wiring, which is the bottom tier of
  [Modularity Hierarchy](/docs/concepts/modularity-hierarchy).
- **You want to implement the consumer trait directly on one concrete type.** Use the
  [`#[cgp_impl(Self)]` form](#implementing-the-consumer-trait-directly), which keeps the companion
  attributes while emitting an ordinary impl.

Needing to [declare the provider struct separately](#declaring-the-provider-struct-separately) is not
by itself a reason to drop to [`#[cgp_provider]`](./cgp_provider.md). Use the raw form only when
you need the inside-out provider-trait shape itself: a bound the sugar cannot express, or a rare
construct `#[cgp_impl]`'s rewrite does not support, whether from a limitation or a bug.

## Under the hood

`#[cgp_impl]` desugars to [`#[cgp_provider]`](./cgp_provider.md). It moves the context type to the
leading position of the provider trait, swaps the provider name into the `Self` position, and rewrites
every `self`/`Self`. From this input, with the context named explicitly for clarity:

```rust
#[cgp_impl(new ValueToString)]
impl<Context> FooProvider for Context {
    fn foo(&self, value: u32) -> String {
        value.to_string()
    }
}
```

Because `new` was given, the macro also produces the provider struct alongside the provider impl and
its `IsProviderFor` impl:

```rust
impl<Context> FooProvider<Context> for ValueToString {
    fn foo(__context__: &Context, value: u32) -> String {
        value.to_string()
    }
}

impl<Context> IsProviderFor<FooProviderComponent, Context, ()> for ValueToString {}

pub struct ValueToString;
```

A few things changed. The trait gained `Context` as its leading argument. The `Self` type became the
provider. `&self` became the explicit parameter `__context__: &Context`. The receiver identifier is
the snake-cased context type wrapped in double underscores, so both `Context` and the default
`__Context__` become `__context__`. The macro rewrites every `self` in a body to that identifier, and
every `Self` to the context type.

When `for Context` is omitted the only difference is that the inserted parameter is `__Context__`. The
earlier `RectangleArea` example is exactly equivalent to writing it out:

```rust
#[cgp_new_provider]
impl<__Context__> AreaCalculator<__Context__> for RectangleArea
where
    __Context__: HasDimensions,
{
    fn area(__context__: &__Context__) -> f64 {
        __context__.width() * __context__.height()
    }
}
```

The generated `IsProviderFor` impl copies the provider impl's signature, drops the body, and keeps the
`where` clause, capturing the provider's dependencies for error reporting. Its arguments are the
component name, the context, and a tuple of any remaining provider-trait parameters,
so a provider for `ComputerRef<Context, Code, Input>` gets
`IsProviderFor<ComputerRefComponent, Context, (Code, Input)>`.

**An associated type the block declares is exempt.** The macro leaves `Self::Output` alone in a
provider that supplies `type Output`, because it gathers the block's own associated-type names first
and skips any `Self::` path starting with one. In the emitted impl, `Self` is the provider struct,
which is the type that declares `Output`, so the path still resolves. Every other `Self` in the block is
still rewritten, including one naming an abstract type the *context* supplies. Associated consts are
not covered by the exemption; see [Common Mistakes](#common-mistakes).

**The rewrite is scoped to the block's own method bodies.** An item nested *inside* a body, such as a
local `struct` with its own impl, a helper `fn`, or an inline `trait`, introduces a fresh `self`/`Self`
that belongs to that item, exactly as in ordinary Rust, and the macro leaves it alone. Closures, which
capture the enclosing `self`, are rewritten like any other expression.

## Formal grammar

The attribute argument names the provider, optionally preceded by `new` and followed by a
component-type override, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
CgpImplArgs   -> `new`? ProviderType ( `:` ComponentType )?

ProviderType  -> Type
ComponentType -> Type
```

`ProviderType` is the type taking the `Self` position of the generated impl: a plain provider name, a
generic provider such as `ScaledAreaCalculator<InnerCalculator>`, or the literal `Self` for the
passthrough form. `ComponentType` overrides the component in the generated `IsProviderFor` impl,
defaulting to the provider trait's name plus `Component`. Both are Rust `Type` productions.

## Common Mistakes

**A provider's own associated const is awkward to name inside its body**, and this follows directly
from the rewrite. The exemption described above covers associated *types* only, so a provider that
declares `const LIMIT: u64 = 100;` cannot then write `Self::LIMIT` in a method body: that `Self` is
rewritten to the context, which has no such const. The error reads:

```text
error[E0599]: no associated function or constant named `LIMIT` found for type parameter `__Context__` in the current scope
```

Name it through the provider trait instead, which the rewrite turns into the correct qualified path:

```rust
<AllowUnderLimit as RateLimiter<Self>>::LIMIT
```

The consumer side is unaffected: a wired context reads the same const as `<App as CanRateLimit>::LIMIT`.

**One nesting case escapes the scoping rule.** An item written inside a `macro!( … )` invocation has
its `self`/`Self` rewritten too, because a token-level rewrite cannot see the scope the macro will
eventually create. A `self::` *module* path inside a macro invocation is safe: the trailing `::`
distinguishes the two meanings of `self`, and only the value form is rewritten.

**A misplaced companion attribute is reported by the compiler, not by the macro.** An attribute
`#[cgp_impl]` does not recognize is re-attached to the generated provider impl rather than dropped, so
`#[allow(...)]` and similar attributes carry over unaffected. A stray `#[extend(HasName)]` therefore
produces a *cannot find attribute* resolution error pointing at the impl, with no mention of
`#[cgp_impl]`.

**`new` and a component override are accepted, and ignored, on the `Self` form.** Writing
`#[cgp_impl(new Self)]` or `#[cgp_impl(Self: SomeComponent)]` parses and compiles, but has exactly the
same effect as a plain `#[cgp_impl(Self)]`: no struct is declared, and the component override is never
consulted, because this form builds neither a provider nor an `IsProviderFor` impl for either one to
apply to. Leave both out.

## Related constructs

- [`#[cgp_component]`](./cgp_component.md) — defines the component this implements.
- [`#[cgp_provider]` and `#[cgp_new_provider]`](./cgp_provider.md) — the lower-level forms `#[cgp_impl]`
  desugars to, the latter equivalent to `#[cgp_impl(new ...)]`.
- [`#[cgp_fn]`](./cgp_fn.md) — the lighter alternative when one implementation is enough.
- [`delegate_components!`](./delegate_components.md) — wires the provider onto a type.
- [`check_components!`](./check_components.md) — verifies the wiring resolves.
- [`#[implicit]`](../attributes/implicit.md), [`#[uses]`](../attributes/uses.md),
  [`#[use_type]`](../attributes/use_type.md), [`#[use_provider]`](../attributes/use_provider.md),
  [`#[default_impl]`](../traits/namespace/default_namespace.md) — the companion attributes.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the inside-out shape
  this macro hides.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — what the block's `where` clause
  is really doing.

To see it taught as a sequence of small steps rather than as a reference, the
[Static Dispatch tutorial](/docs/tutorials/area-calculation/static-dispatch) builds `#[cgp_component]`
and `#[cgp_impl]` up from the same coherence failure that
[Bypassing coherence](/docs/concepts/coherence) explains from the other side.

## Source

- Entry point: [`cgp_impl.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_impl.rs)
- Implementation: [`types/cgp_impl/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_impl/)
- The `self`/`Self` rewrite: [`visitors/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/visitors/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
