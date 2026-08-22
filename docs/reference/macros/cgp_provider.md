---
sidebar_label: '#[cgp_provider] & #[cgp_new_provider]'
sidebar_position: 20
---

# `#[cgp_provider]` & `#[cgp_new_provider]`

Implement a provider trait directly, in the inside-out shape the sugar desugars to, with or without
declaring the provider struct.

## Overview

A provider trait is not shaped like the trait it came from. [`#[cgp_component]`](./cgp_component.md)
moves the original `Self` into an explicit leading type parameter for the **context**. The context is
the type the capability runs against, and it supplies the values it needs as its own fields. An
implementation targets a small named struct instead. Written out, a provider looks like this:

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

That impl is correct on its own but incomplete. CGP requires a provider to *also* implement a marker
trait under exactly the same conditions, so that a context missing something the provider needs gets an
error naming the missing thing rather than a bare "trait not implemented". Written by hand, that means
duplicating the generics and the whole `where` clause, and keeping the two copies synchronized forever.

`#[cgp_provider]` writes the second impl for you. You write only the real one, and the macro derives
its companion from it: same generics, same bounds, body stripped. The dependencies cannot drift out
of sync, because they are read off the impl rather than restated.

`#[cgp_new_provider]` is the same macro with one addition: it also declares
`pub struct RectangleArea;`, so a provider that does not exist yet can be introduced in one block.

**Neither is the form to write for a new provider.** [`#[cgp_impl]`](./cgp_impl.md) expresses the same
implementation in the consumer trait's shape, keeping `self`, `Self`, and the original signatures, and
desugars to exactly these two macros. Read this page to understand generated code and the narrow cases
the sugar cannot reach; write `#[cgp_impl]` the rest of the time.

## Usage

Apply the attribute to an `impl` block that implements a provider trait for a provider struct. The impl
is ordinary Rust: the trait carries the context as its leading type argument, the `Self` type is the
provider, and methods take the context as a plain parameter rather than as a receiver.

```rust
#[cgp_provider]
impl<Context> AreaCalculator<Context> for RectangleArea
where
    Context: HasDimensions,
{
    fn area(context: &Context) -> f64 {
        context.width() * context.height()
    }
}
```

Which of the two you use depends only on whether the struct already exists.

| Macro | The provider struct |
|---|---|
| `#[cgp_provider]` | Must already be declared. |
| `#[cgp_new_provider]` | Declared by the macro. Must *not* be declared separately. |

Both accept the same single optional argument: the **component type** to use in the generated marker
impl. Omitted, it defaults to the provider trait's name plus `Component`, so implementing
`AreaCalculator` targets `AreaCalculatorComponent`. Pass it explicitly when the trait's name does not
follow that convention:

```rust
#[cgp_provider(RunnerComponent)]
impl<Context, Code> Runner<Context, Code> for RunWithFooBar
where
    Context: CanFetchFoo + CanFetchBar,
{
    fn run(context: &Context, _code: PhantomData<Code>) -> Result<(), Context::Error> {
        /* ... */
    }
}
```

### The struct `#[cgp_new_provider]` declares

The struct's shape is taken from the `Self` type of the impl. A plain name yields a unit struct. A
generic provider yields a tuple struct holding a
[`PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) over its parameters, so
that the parameters are bound:

```rust
// from `... for RectangleArea`
pub struct RectangleArea;

// from `... for SpawnAndRun<InCode>`
pub struct SpawnAndRun<InCode>(pub ::core::marker::PhantomData<InCode>);
```

This is also the limit of what the attribute form can express, and the reason `#[cgp_provider]` still
has a job. A struct that needs a **default** generic parameter has to be written by hand, and so does
one shared by several impls. The common shape is
`pub struct IterSum<Inner = UseContext>(PhantomData<Inner>);`, which a higher-order provider uses so it
can fall back to the context's own wiring.

## Examples

A complete provider with its struct declared separately, the case `#[cgp_provider]` handles:

```rust
use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasDimensions {
    fn width(&self) -> &f64;
    fn height(&self) -> &f64;
}

pub struct RectangleArea;

#[cgp_provider]
impl<Context> AreaCalculator<Context> for RectangleArea
where
    Context: HasDimensions,
{
    fn area(context: &Context) -> f64 {
        context.width() * context.height()
    }
}
```

A context wires it exactly as it would any provider. The derived marker impl makes a missing `width`
field report itself as a missing field:

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
```

The same provider with `#[cgp_new_provider]` drops the separate declaration and nothing else:

```rust
#[cgp_new_provider]
impl<Context> AreaCalculator<Context> for RectangleArea
where
    Context: HasDimensions,
{
    fn area(context: &Context) -> f64 {
        context.width() * context.height()
    }
}
```

And the same provider once more, in the form to actually write, where the context parameter is
inserted for you and the body keeps `self`:

```rust
#[cgp_impl(new RectangleArea)]
#[uses(HasDimensions)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        self.width() * self.height()
    }
}
```

All three produce the same three items. The difference is only how much of the machinery is visible in
the source.

## When to reach for it, and when not

**Write [`#[cgp_impl]`](./cgp_impl.md) instead, by default.** It produces the identical output from
source that reads like an ordinary trait impl, so it is the recommended form. You mostly *read* these
two instead: in generated code, in a desugaring, and in CGP's own crates.

Two cases call for writing the raw form yourself.

- **You need the inside-out provider-trait shape itself**, not merely a separately-declared struct: a
  bound `#[cgp_impl]`'s sugar cannot express, or a rare construct its rewrite does not support, whether
  from a limitation or a bug. This is uncommon. Most bounds that look like they need the raw form are
  expressible in `#[cgp_impl]`'s explicit-context form instead (`impl<Context> AreaCalculator for
  Context`), which reaches a higher-ranked bound such as `for<'a> &'a Context: IntoIterator` and pins
  the provider to a single concrete context with `impl AreaCalculator for Rectangle` alike. A struct
  that `new` cannot declare, such as one carrying a default generic parameter
  (`pub struct IterSum<Inner = UseContext>(PhantomData<Inner>);`) or one shared by several impls, is
  *not* by itself a reason to write the raw form: declare the struct yourself and keep writing the body
  with `#[cgp_impl(ProviderName)]`, just without `new`.
- **Reading, rather than writing.** A confusing provider error names types from this shape, and
  `cargo cgp expand` prints it. Recognizing the form is the main reason to read this page.

Once you are writing the raw form, the choice between the two macros here is mechanical:
`#[cgp_new_provider]` when the provider struct is new, `#[cgp_provider]` when it already exists. There
is no other difference, and using the wrong one is a duplicate-definition error rather than anything
subtle.

## Under the hood

`#[cgp_provider]` emits two items: your impl, passed through unchanged, and a marker impl derived from
it. From this input:

```rust
#[cgp_provider]
impl<Context, Code, Input> ComputerRef<Context, Code, Input> for FirstNameToString
where
    Context: HasField<Symbol!("first_name"), Value: Display>,
{
    type Output = String;

    fn compute_ref(context: &Context, _code: PhantomData<Code>, _input: &Input) -> String {
        context.get_field(PhantomData).to_string()
    }
}
```

the macro adds:

```rust
impl<Context, Code, Input> IsProviderFor<ComputerRefComponent, Context, (Code, Input)>
    for FirstNameToString
where
    Context: HasField<Symbol!("first_name"), Value: Display>,
{}
```

The derived impl is your impl with the body and associated types removed and the trait swapped. It
keeps the same generic parameters and the same bounds, so it holds under precisely the conditions the
real impl holds. This is the whole point: a check evaluates
[`IsProviderFor`](../traits/is_provider_for.md) to find out *why* a provider does not apply.

The macro assembles its three trait arguments from the provider trait's own. The first is the
**component** (`ComputerRefComponent`, the default derived from the trait name, or whatever the
attribute argument said). The second is the **context**. The third is a **tuple of everything left
over**: `(Code, Input)` here, and the empty `()` for a provider trait that takes nothing but a
context.

Two rules decide that split, and they only become visible on a provider trait that carries a
[lifetime](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html). The context is the first
*type* argument rather than the first argument, because Rust puts lifetime arguments first and a
lifetime cannot be a context. And the macro lifts a lifetime into [`Life<'a>`](../types/life.md) to
occupy its slot in the tuple, which holds types. A provider for `ReferenceGetter<'a, Context, T>` therefore
derives `IsProviderFor<ReferenceGetterComponent, Context, (Life<'a>, T)>`, keeping the order the
arguments were written in.

**The macro rewrites one bound instead of copying it**, and this is the mechanism that makes a nested
provider stack diagnosable. A bound naming *this component's provider trait*, the inner-provider bound
of a [higher-order provider](../attributes/use_provider.md), gains its marker counterpart alongside it:

```rust
#[cgp_new_provider]
impl<Context, Inner> AreaCalculator<Context> for Scaled<Inner>
where
    Inner: AreaCalculator<Context>,
{
    fn area(context: &Context) -> f64 { Inner::area(context) * 2.0 }
}
```

derives a marker impl whose clause carries both:

```rust
impl<Context, Inner> IsProviderFor<AreaCalculatorComponent, Context, ()> for Scaled<Inner>
where
    Inner: IsProviderFor<AreaCalculatorComponent, Context, ()> + AreaCalculator<Context>,
{}
```

That added bound carries a requirement from the *inner* provider outward through the wrapper, so a
field missing three layers down still reaches the context where the component is checked. The macro
copies bounds of any other kind, such as an ordinary `Context: Clone` or a consumer-trait bound like
`Context: CanPerimeter`, verbatim, without a counterpart.

`#[cgp_new_provider]` emits the same two items plus the struct, whose shape follows the `Self` type as
described [above](#the-struct-cgp_new_provider-declares).

### Input the macros refuse

Both macros reject four shapes at expansion, rather than lowering them into code that fails later. An
**inherent impl**, with no trait, has no provider trait to read the component and context from. A
**provider trait with no type argument** leaves nothing to be the context. A **const argument** in the
provider trait's argument list has nowhere to live in the type-only params tuple. This is about the
*trait's* arguments, not about a const generic on the provider struct, which passes through untouched.
And an item that is not an `impl` is refused outright. Each message names what was missing.

<details>
<summary>Formal grammar</summary>

The attribute argument of either macro is a single optional component type, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
CgpProviderArgs -> ComponentType?

ComponentType   -> Type
```

Omitted, the component defaults to the provider trait's name plus `Component`; given, that `Type` is
substituted into the first position of the generated marker impl. `Type` is the Rust type production.
The struct declaration that distinguishes `#[cgp_new_provider]` is implied by the macro name and is not
written in the argument.

</details>

## Gotchas

**`#[cgp_new_provider]` fails if the struct already exists**, which is the usual result of converting
one macro to the other and forgetting to delete the declaration:

```text
error[E0428]: the name `RectangleArea` is defined multiple times
```

**`new` is not part of this attribute's grammar.** `#[cgp_provider(new RectangleArea)]` does not
declare the struct. It fails to parse, because the argument holds a component type and nothing else.
Which macro you invoke decides whether the struct is declared, so use
`#[cgp_new_provider]`. The `new` keyword you may have seen belongs to
[`#[cgp_impl]`](./cgp_impl.md#usage).

**A higher-order provider over a lifetime-carrying component loses the inner marker bound.** The
rewrite above finds the context by reading the inner bound's first argument. On a component with a
lifetime, that argument is the lifetime, so the macro builds no counterpart and copies the bound as-is.
The stack compiles and runs correctly, but it loses the propagation: a dependency unmet inside the
inner provider no longer surfaces at the outer one, and `#[check_providers(...)]` cannot say which
layer of such a stack is at fault. Components without lifetime parameters are unaffected.

**The macro does not check the component argument against the trait.** Passing a component that does
not belong to the provider trait you are implementing produces a marker impl for the wrong key, so the
provider silently fails to satisfy the wiring that names it. The error appears at the wiring site,
saying the provider is not a provider for that component. Omit the argument unless the trait's name
departs from the `{Trait}Component` convention.

**A provider's own associated const or type still needs qualifying**, though for a different reason than
in [`#[cgp_impl]`](./cgp_impl.md#gotchas). Here `Self` really is the provider, and nothing rewrites it.
But the provider struct is also a perfectly good context, so the consumer blanket impl gives it the
*consumer* trait as well, and both traits declare the item. `Self::LIMIT` is therefore ambiguous rather
than missing:

```text
error[E0034]: multiple applicable items in scope
   |
   |     fn allowed(_context: &Context) -> bool { Self::LIMIT > 0 }
   |                                                    ^^^^^ multiple `LIMIT` found
   |
note: candidate #1 is defined in an impl of the trait `RateLimiter` for the type `AllowUnderLimit`
note: candidate #2 is defined in an impl of the trait `CanRateLimit` for the type `__Context__`
```

Name the provider trait to disambiguate: `<Self as RateLimiter<Context>>::LIMIT` works here
because `Self` is the provider. The `#[cgp_impl]` form needs
`<AllowUnderLimit as RateLimiter<Self>>::LIMIT` instead, since there `Self` is the context. Either way
the consumer side is unaffected: a wired context reads `<App as CanRateLimit>::LIMIT`.

## Related constructs

- [`#[cgp_impl]`](./cgp_impl.md) — the form to write, which desugars to these two.
- [`#[cgp_component]`](./cgp_component.md) — defines the provider trait being implemented.
- [`IsProviderFor`](../traits/is_provider_for.md) — the marker trait the macro derives, and why.
- [`delegate_components!`](./delegate_components.md) — wires a provider onto a context.
- [`check_components!`](./check_components.md) — evaluates the derived marker to report a missing
  dependency by name.
- [`#[use_provider]`](../attributes/use_provider.md) — the higher-order-provider bound whose marker
  counterpart the macro adds.
- [`UseContext`](../providers/use_context.md) — the usual default for a hand-written provider struct's
  inner parameter.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the shape these two
  macros write out longhand.
- [Higher-order providers](/docs/concepts/higher-order-providers) — the stacks whose inner bound the
  marker derivation augments.

## Source

- Entry points: [`cgp_provider.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_provider.rs)
  and [`cgp_new_provider.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_new_provider.rs)
- Implementation: [`types/cgp_provider/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_provider/)
- The marker derivation: [`types/provider_impl.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/provider_impl.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
