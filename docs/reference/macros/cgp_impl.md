---
sidebar_label: '#[cgp_impl]'
---

# `#[cgp_impl]`

Write a provider using consumer-trait syntax, keeping `self`, `Self`, and the method signatures you
already know.

## What it's for

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
parameter you did not write, `Self` is a type with no values, and there is no `self` anywhere — which
obscures the simple thing that is actually going on: this is an implementation of `CanCalculateArea`.

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

The macro performs the rewrite for you. **This is the form to write**; the inside-out one is what it
desugars to and what you will meet in generated code.

One fact the convenience must not hide: **inside a `#[cgp_impl]` block, `self` and `Self` mean the
context — the type the capability runs against, which supplies the values it needs as its fields —
and not the provider.** The provider is a
type-level name with no fields and no value: it is never constructed, and there is nothing in it to
read. The macro rewrites `self` to the context value and `Self` to the context type precisely because
the context is the only thing that exists when the method runs.

## Using it

Apply the attribute to an `impl` block. Its argument names the provider, and has three parts of which
only the name is required.

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

Without `new`, the provider struct must already exist — a wiring entry naming a struct nothing
declares is a common and confusing first error.

A provider may be generic. Parameters go in the attribute and in the impl generics together, which is
how a higher-order provider takes an inner provider:

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

### Companion attributes

Several attributes are processed on a `#[cgp_impl]` block before the rewrite happens, and together
they are how an idiomatic provider states what it needs.

- [`#[implicit]`](../attributes/implicit.md) on a parameter removes it from the signature and fills it
  from a same-named field on the context.
- [`#[uses(...)]`](../attributes/uses.md) adds the capabilities the provider depends on, reading like
  a `use` statement instead of a hand-written `where Self: Trait` clause.
- [`#[use_type(Trait.Type)]`](../attributes/use_type.md) imports an abstract type and rewrites its
  occurrences to fully qualified form.
- [`#[use_provider(...)]`](../attributes/use_provider.md) completes an inner provider's bound in a
  higher-order provider.
- [`#[default_impl(...)]`](../traits/default_namespace.md) registers the provider as a namespace's
  per-type default, for use with [`cgp_namespace!`](./cgp_namespace.md).

Each may be repeated. All except `#[use_provider]` also take a comma-separated list inside one
attribute, which is the form to prefer — `#[uses(HasName, CanRaiseError<String>)]` reads as one
dependency list. `#[use_provider]` is the exception because its own argument already ends in a bound
list, so a second pair after a comma has nowhere to go; write one attribute per inner provider.

Three attributes that appear on other CGP macros are not read here. `#[extend]`, `#[extend_where]`,
and `#[impl_generics]` all act on a *generated trait definition*, which a provider impl does not have,
so they belong to [`#[cgp_fn]`](./cgp_fn.md) and — for `#[extend]` —
[`#[cgp_component]`](./cgp_component.md). Writing one here leaves a name nothing resolves; see
[Gotchas](#gotchas). An impl-side bound that really is impl-side goes in the block's own `where`
clause, which passes through untouched.

### Implementing the consumer trait directly

Naming `Self` as the provider bypasses the rewrite entirely and emits the block unchanged as an
ordinary consumer-trait impl on a concrete type. This form requires the `for Context` clause, and it is
useful when you want a hand-written impl while still applying the companion attributes. Because no
provider struct is generated, `new` and the component override have no effect.

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

The two `#[implicit]` parameters are removed from the signature and replaced by reads of the `width`
and `height` fields, so the provider requires a context that has them. The `new` keyword defines
`RectangleArea`. A concrete type then wires the component and calls through it:

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

## When to reach for it, and when not

**Write providers with `#[cgp_impl]`.** It is the recommended form for every provider, and the cases
that call for something else are narrower than they look.

Prefer the unqualified header `impl AreaCalculator`, with no `for Context`, and let the macro insert
the context parameter. That is what makes a provider read like an ordinary trait impl. Name the context
explicitly — `impl<Context> AreaCalculator for Context` — only when you actually need to say something
about it that the sugar cannot express, such as a lifetime or a higher-ranked bound.

That named context may be a *concrete* type rather than a parameter — `impl AreaCalculator for Rectangle`
— which writes a provider serving only that one context. Keep it distinct from the
[`#[cgp_impl(Self)]` form](#implementing-the-consumer-trait-directly), which looks similar and does
something else: this one still produces a named provider that a context wires like any other, and that
one produces no provider at all.

Reach for something else in three cases.

- **The provider struct has to be declared separately.** [`#[cgp_provider]`](./cgp_provider.md) is the
  raw form for that case, and `new` is what you drop: a struct carrying a default generic parameter,
  such as `pub struct IterSum<Inner = UseContext>(PhantomData<Inner>);`, cannot be declared by the
  attribute, and neither can one shared by several impls. Write the struct, then annotate each impl.
- **The capability has only one implementation.** [`#[cgp_fn]`](./cgp_fn.md) builds it from a plain
  function with no component, no provider, and no wiring.
- **You want to implement the consumer trait directly on one concrete type.** Use the
  [`#[cgp_impl(Self)]` form](#implementing-the-consumer-trait-directly), which keeps the companion
  attributes while emitting an ordinary impl.

## Under the hood

:::note

### Advanced

This section shows what the macro generates. You do not need it to write a provider, but the rewrite is
worth seeing once — most confusing errors in a provider body are explained by it. `cargo cgp expand`
prints the same thing for your own code.

:::

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

the macro produces the provider impl, an `IsProviderFor` impl, and — because `new` was given — the
provider struct:

```rust
impl<Context> FooProvider<Context> for ValueToString {
    fn foo(__context__: &Context, value: u32) -> String {
        value.to_string()
    }
}

impl<Context> IsProviderFor<FooProviderComponent, Context, ()> for ValueToString {}

pub struct ValueToString;
```

Three things changed. The trait gained `Context` as its leading argument; the `Self` type became the
provider; and `&self` became the explicit parameter `__context__: &Context`. The receiver identifier is
the snake-cased context type wrapped in double underscores, so both `Context` and the default
`__Context__` become `__context__`. Every `self` in a body is rewritten to that identifier, and every
`Self` to the context type.

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
`where` clause — which is how the provider's dependencies are captured for error reporting. Its
arguments are the component name, the context, and a tuple of any remaining provider-trait parameters,
so a provider for `ComputerRef<Context, Code, Input>` gets
`IsProviderFor<ComputerRefComponent, Context, (Code, Input)>`.

**An associated type the block declares is exempt.** `Self::Output` in a provider that supplies
`type Output` is left alone, because the macro gathers the block's own associated-type names first and
skips any `Self::` path starting with one. That is what it has to do: in the emitted impl `Self` is the
provider struct, which is the type that declares `Output`, so the path resolves as written. Every
other `Self` in the block is still rewritten — including one naming an abstract type the *context*
supplies. Associated consts are not covered by the exemption; see [Gotchas](#gotchas).

**The rewrite is scoped to the block's own method bodies.** An item nested *inside* a body — a local
`struct` with its own impl, a helper `fn`, an inline `trait` — introduces a fresh `self`/`Self` that
belongs to that item, exactly as in ordinary Rust, and the macro leaves it alone. Closures, which
capture the enclosing `self`, are rewritten like any other expression.

<details>
<summary>Formal grammar</summary>

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

</details>

## Gotchas

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

The consumer side is unaffected — a wired context reads the same const as `<App as CanRateLimit>::LIMIT`.

**One nesting case escapes the scoping rule.** An item written inside a `macro!( … )` invocation has
its `self`/`Self` rewritten too, because a token-level rewrite cannot see the scope the macro will
eventually create. A `self::` *module* path inside a macro invocation is safe: the trailing `::` is
what tells the two meanings of `self` apart, and only the value form is rewritten.

**A misplaced companion attribute is reported by the compiler, not by the macro.** An attribute
`#[cgp_impl]` does not recognize is re-attached to the generated provider impl — which is what lets
`#[allow(...)]` ride through — so a stray `#[extend(HasName)]` produces a *cannot find attribute*
resolution error pointing at the impl, with no mention of `#[cgp_impl]`.

## Related constructs

- [`#[cgp_component]`](./cgp_component.md) — defines the component this implements.
- [`#[cgp_provider]`](./cgp_provider.md) — the lower-level form `#[cgp_impl]` desugars to.
- [`#[cgp_fn]`](./cgp_fn.md) — the lighter alternative when one implementation is enough.
- [`delegate_components!`](./delegate_components.md) — wires the provider onto a type.
- [`check_components!`](./check_components.md) — verifies the wiring resolves.
- [`#[implicit]`](../attributes/implicit.md), [`#[uses]`](../attributes/uses.md),
  [`#[use_type]`](../attributes/use_type.md), [`#[use_provider]`](../attributes/use_provider.md),
  [`#[default_impl]`](../traits/default_namespace.md) — the companion attributes.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the inside-out shape
  this macro hides.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — what the block's `where` clause
  is really doing.

## Source

- Entry point: [`cgp_impl.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_impl.rs)
- Implementation: [`types/cgp_impl/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_impl/)
- The `self`/`Self` rewrite: [`visitors/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/visitors/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
