---
sidebar_label: '#[cgp_component]'
sidebar_position: 1
---

# `#[cgp_component]`

Turn a trait into a component: the consumer trait callers use, the provider trait implementations
target, and the key that wires them together.

## Overview

You can apply `#[cgp_component]` to any Rust trait definition to give that trait the full set of
CGP's capabilities. Applying it takes nothing away. The trait you wrote keeps working exactly as it
did, now under the name CGP calls the **consumer trait**, and every call site that already uses it
keeps compiling unchanged. The macro adds a matching **provider trait** for implementations to target,
a small marker type called the **component** that names the capability for wiring, and a pair of
[**blanket implementations**](https://blog.implrust.com/posts/2025/09/blanket-implementation-in-rust/)
that connect them without requiring you to write anything.

`#[cgp_component]` makes it possible to define several overlapping implementations of one capability
at once. An ordinary Rust trait does not allow this. A
[trait](https://doc.rust-lang.org/book/ch10-02-traits.html) can have only one implementation per
type, a rule called [coherence](/docs/concepts/coherence). That rule is usually the right one, because
it lets the compiler resolve a `where T: Display` bound without anyone naming which implementation
applies. But it also means a capability with several plausible implementations cannot hold the
alternatives. A type can send real email or record it for a test, but never both at once. Switching
between the two means editing the impl itself rather than choosing between them at the point of use.

`#[cgp_component]` solves that by splitting the one trait into two, the
[consumer and provider traits](/docs/concepts/consumer-and-provider-traits), so that a caller and an
implementation no longer target the same trait.

- The **consumer trait** is the trait callers use, as in `rect.area()`. It is your original trait,
  kept exactly as you wrote it.
- An implementation targets the **provider trait** instead. It repeats the same methods with `Self`
  moved into an explicit type parameter. So you write an implementation for a small named type of its
  own, rather than for the type the capability is about.

Moving `Self` into a parameter lets more than one implementation coexist, because each one now targets
its own small type instead of the single `Self` slot every plain trait has. A **provider** is one of
those targets: a zero-sized type such as `RectangleArea` without data of its own that exists only
to name one implementation. The type the capability runs against is called the **context**. It
supplies whatever values an implementation needs as its own fields, and it picks the provider it wants
through [`delegate_components!`](./delegate_components.md). The generated blanket implementations then
route a call on the consumer trait through to that choice automatically. This costs nothing at
runtime. Wiring fixes the provider once, at compile time, and the compiler turns the call into a
direct, statically-dispatched call, exactly as if you had written the implementation yourself.

The macro's remaining output is the **component** itself, a marker type such as
`AreaCalculatorComponent` that names the capability and is the key `delegate_components!` wires
against. You see it in every wiring entry and in most compiler errors that involve this capability, so
learn to recognize it, even though you never write its definition yourself.

## Usage

Apply the attribute to a trait definition and give it the provider trait's name. The simplest form is
a bare identifier:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

`CanCalculateArea` is the consumer trait and `AreaCalculator` is the provider trait. Follow the naming
convention: a consumer trait reads as a verb (`CanDoSomething`) and a provider trait as a noun
(`SomethingDoer`, or `…Provider` when a noun does not fit).

When you need more control, the macro accepts a key/value form instead:

```rust
#[cgp_component {
    name: AreaCalculatorComponent,
    provider: AreaCalculator,
    context: Context,
}]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

Only `provider` is required, and the other keys have defaults. Passing a bare identifier is shorthand
for setting `provider` alone.

| Key | What it sets | Default |
|---|---|---|
| `provider` | The provider trait's name | *required* |
| `name` | The component marker type | The provider name plus `Component` |
| `context` | The identifier used for the generated context type parameter | `__Context__` |

The `context` default is deliberately unusual so that it cannot collide with a type parameter of your
own.

### Companion attributes

The attributes below may be written above the trait, beside `#[cgp_component]`, and each changes what
the macro generates. Any of them may be repeated, and all except `#[prefix]` also accept a
comma-separated list inside one attribute.

| Attribute | What it adds |
|---|---|
| [`#[use_type(Trait.Type)]`](../attributes/use_type.md) | Imports an abstract type: adds the supertrait *and* rewrites the bare name in your signatures |
| [`#[extend(Trait)]`](../attributes/extend.md) | Adds a supertrait with no type to import |
| [`#[derive_delegate(...)]`](../attributes/derive_delegate.md) | Generates dispatch impls for a component generic over a parameter |
| [`#[prefix(@path in Namespace)]`](./cgp_namespace.md) | Registers the component into a namespace under a type-level path |

When a component depends on a type another component supplies, most often the error type from
[`HasErrorType`](../components/has_error_type.md), import it with `#[use_type]` rather than writing
the supertrait and the qualified path by hand:

```rust
#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}
```

The macro refuses one form of `#[use_type]` here. The equality form
`#[use_type(HasScalarType.{Scalar = f64})]` pins an imported type to a concrete one. That decision
belongs to a provider that has chosen the type, not to the definition every provider has to satisfy,
so the macro rejects it with *Type equality constraints cannot be used in component trait definition*.
Every other form of the attribute works on a component exactly as it does elsewhere.

For a supertrait without an associated type to import, use `#[extend(...)]` in preference to native
`: Supertrait` syntax. `#[extend(HasName)]` reads as importing a capability, whereas
`pub trait CanGreet: HasName` reads as inheritance, which a CGP supertrait is not. An associated type
the trait declares *itself* is not imported and stays written as `Self::Output`.

The `open` statement of [`delegate_components!`](./delegate_components.md) supersedes
`#[derive_delegate(...)]`, so you mostly read it in existing code rather than write it.

`#[uses(...)]`, `#[extend_where(...)]`, and `#[use_provider(...)]` look like they belong here, but
only [`#[cgp_impl]`](./cgp_impl.md) and [`#[cgp_fn]`](./cgp_fn.md) read them. None of them is an
attribute in its own right, so writing one above a component leaves a name nothing can resolve, and
the error is confusing; see [Common Mistakes](#common-mistakes). The equivalent of `#[uses]` on a
component is a supertrait, written with `#[extend]`.

## Examples

A component, a provider for it, and a type that wires the two together:

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

`rect.area()` is an ordinary method call on the consumer trait. It resolves through `Rectangle`'s
table, which maps `AreaCalculatorComponent` to `RectangleArea`, and `RectangleArea::area` reads the two
fields to compute the result. Swapping the provider in that one wiring line changes what `rect.area()`
does, and nothing else in the program changes.

## When to use it

Use `#[cgp_component]` when a capability needs **more than one implementation, and the
choice belongs to the type using it.** The macro exists for that case, and it has a cost: a component
is a trait, a second trait, a marker type, and a line of wiring per type.

Prefer something simpler when you can.

- **One implementation, ever.** Use [`#[cgp_fn]`](./cgp_fn.md) instead. It builds the capability
  straight from a function and does not need wiring. If a second implementation ever arrives, you
  promote it to a component and its call sites keep working unchanged. That makes it the right
  starting point rather than a lesser one.
- **One implementation per type, chosen globally.** A plain Rust trait already does this well.
  Use a component when two *different* applications must make different choices for the same
  type, or when the implementations must overlap in a way the compiler rejects.
- **A closed set of variants with fixed operations.** An `enum` and a `match` are clearer than any
  machinery.

One more distinction matters. A capability may need several implementations while each individual
implementation serves exactly one type. In that case you can implement the consumer trait directly on
each concrete type, as you would any Rust trait, and skip providers entirely. Named providers become
worth their cost once a second type wants the *same* implementation, or once an implementation should
compose with a wrapper.

### How many items should a component have?

A component trait is an ordinary trait. It takes as many methods, associated types, and associated
consts as any other, and the macro reproduces every one of them on the provider trait. CGP's own
[`CanCompute`](../components/handler/computer.md) declares an associated `Output` beside its method,
and the macro does not limit the count.

How much to group is a judgement rather than a rule, and the useful test is this: **one provider
choice should answer everything in one component.** Items a single choice settles belong together: a
method and the associated type it returns, or several field reads one getter provider answers by name.
Items that separate choices settle are better apart, because grouping them costs reuse. Every provider
then carries the union of the dependencies of all the methods, a wrapper must forward the methods it
does not touch, and a type that needs only part of the surface must still supply the rest.

The usual sign of a component that has grown past one decision is a consumer trait named after a noun
rather than a verb, such as `Shape` carrying `area`, `perimeter`, `scale`, and `rotate`. It compiles,
but very little of it is reusable. So check before committing: could a second type plausibly reuse
one of this trait's providers *whole*? If not, implement the trait directly on the concrete type and
skip the machinery.

## Under the hood

From this input:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

the macro emits the items below. The **consumer trait** comes out unchanged:

```rust
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

The **provider trait** repeats the same interface with `Self` replaced by a leading `Context` type
parameter and every `self`/`Self` rewritten to `context`/`Context`. Its
[`IsProviderFor`](../traits/wiring/is_provider_for.md) supertrait makes an unmet dependency report
itself by name rather than as a bare "trait not implemented". The supertrait's third argument is a
tuple of the component's extra type parameters, empty here:

```rust
pub trait AreaCalculator<Context>:
    IsProviderFor<AreaCalculatorComponent, Context, ()>
{
    fn area(context: &Context) -> f64;
}
```

`IsProviderFor` *replaces* the supertrait list rather than joining it. Any supertrait the consumer
trait had, whether written natively or added by `#[extend]` or `#[use_type]`, becomes a `where`
predicate on the context instead. This follows from the `Self`-to-`Context` move: a supertrait
constrains the type the capability is about, and on the provider side that type is the context
parameter. So `#[extend(HasName)]` on a `CanGreet` component produces:

```rust
pub trait Greeter<Context>: IsProviderFor<GreeterComponent, Context, ()>
where
    Context: HasName,
{
    fn greet(context: &Context);
}
```

The macro adds that predicate to every emitted item that mentions the context. The two blanket impls
below and the `UseContext` and `RedirectLookup` impls further down each gain their own
`Context: HasName`, because none of them can apply where the supertrait does not hold.

**A method may carry a default body, and the body moves to the provider trait.** The
`Self`-to-`Context` move reaches the body as well, and that move lets a provider inherit a default.
Given `fn greet(&self) -> String { format!("Hello, {}!", self.name()) }` on the consumer trait, the
provider trait gets `fn greet(context: &Context) -> String { format!("Hello, {}!", context.name()) }`,
rewritten but still a default. The consumer trait keeps its copy too, so the body appears twice in an
expansion.

This is why an **empty provider impl** is meaningful, and it is how
[`UseDefault`](../providers/use_default.md) works:

```rust
#[cgp_impl(UseDefault)]
impl<Context: HasName> Greeter for Context {}
```

The **consumer blanket impl** lets callers write `context.area()`: any context implementing the
provider trait *for itself* gets the consumer trait.

```rust
impl<Context> CanCalculateArea for Context
where
    Context: AreaCalculator<Context>,
{
    fn area(&self) -> f64 {
        Context::area(self)
    }
}
```

The **provider blanket impl** makes wiring work: anything that delegates this component through
[`DelegateComponent`](../traits/wiring/delegate_component.md) inherits the provider trait from
whatever it delegates to.

```rust
impl<Context, Provider> AreaCalculator<Context> for Provider
where
    Provider: DelegateComponent<AreaCalculatorComponent>
        + IsProviderFor<AreaCalculatorComponent, Context, ()>,
    Provider::Delegate: AreaCalculator<Context>,
{
    fn area(context: &Context) -> f64 {
        Provider::Delegate::area(context)
    }
}
```

The `IsProviderFor` bound sits on `Provider` itself rather than on `Provider::Delegate`. That
placement carries a provider's dependencies along the delegation chain, so they still appear in an
error message.

The **component marker** is the key wiring uses:

```rust
pub struct AreaCalculatorComponent;
```

The macro emits these items in the order consumer trait, consumer impl, provider trait, provider impl,
marker. The listings above instead pair each trait with the impl that routes to it, which reads better
than the order they actually appear in.

The macro also emits the provider impls that let the component take part in CGP's usual patterns. It
always emits the `UseContext` and `RedirectLookup` impls, and it generates the `UseDelegate` and
namespace impls one per attribute:

- A [`UseContext`](../providers/use_context.md) impl, so the provider trait can be satisfied by
  routing back through the context's own implementation. Its only bound is
  `Context: CanCalculateArea`, and each method forwards to the consumer method.
- A [`RedirectLookup`](../providers/redirect_lookup.md) impl, which the `open` statement and
  [namespaces](./cgp_namespace.md) resolve through.
- One [`UseDelegate`](../providers/use_delegate.md) impl per
  [`#[derive_delegate(...)]`](../attributes/derive_delegate.md) attribute.
- One namespace impl per [`#[prefix(@path in Namespace)]`](./cgp_namespace.md) attribute, binding the
  component's key inside that namespace to a redirect down the given path.

The `RedirectLookup` impl puts a component's own type parameters into a path, and it makes
`@AreaCalculatorComponent.Rectangle` resolve. For a component with type parameters the impl does not
look the incoming path up directly. It appends the parameters to the path first, so a lookup that
arrives at `AreaCalculatorComponent` without a path ends up looking for `Rectangle`. Only *type*
parameters take part. A lifetime or a const parameter cannot key a path, so the impl leaves it out.

Some details of the real output differ from the listings above, and each is easy to misread in an
error. The generated parameters carry **reserved names**: the context is literally `__Context__`
unless you override it, and the provider parameter is `__Provider__`. The readable `Context` and
`Provider` here are for legibility only. Also, a component with parameters of its own appends them
*after* the context in the provider trait. Lifetimes are the exception, because Rust requires them to
lead. The macro also groups those parameters into the `IsProviderFor` parameter tuple. That tuple
holds types, so it lifts a lifetime into [`Life<'a>`](../types/life.md):

| Component | Provider trait | `IsProviderFor` params |
|---|---|---|
| `CanCalculateArea` | `AreaCalculator<__Context__>` | `()` |
| `CanCalculateArea<Shape>` | `AreaCalculator<__Context__, Shape>` | `(Shape)` |
| `CanEncode<Value, Format>` | `Encoder<__Context__, Value, Format>` | `(Value, Format)` |
| `HasReference<'a, T>` | `ReferenceGetter<'a, __Context__, T>` | `(Life<'a>, T)` |

The tuple drops bounds and defaults, and it names the parameters positionally and nothing more. A
const parameter cannot appear in it at all, which is why the macro rejects one; see
[Common Mistakes](#common-mistakes).

## Formal grammar

The attribute argument is either a bare provider name or a comma-separated set of keyed values, in the
Rust Reference's [notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
CgpComponentArgs -> ProviderName
                  | KeyValueArg ( `,` KeyValueArg )* `,`?

ProviderName     -> IDENTIFIER

KeyValueArg      -> `name` `:` ComponentName
                  | `provider` `:` IDENTIFIER
                  | `context` `:` IDENTIFIER

ComponentName    -> IDENTIFIER GenericArgs?
```

`ProviderName` is shorthand for setting `provider` alone. In the key/value form each key may appear at
most once, in any order, and `provider` is required. `IDENTIFIER` is a Rust identifier token, and
`GenericArgs` is the Rust grammar's `< … >` argument list, so the component name may carry generic
parameters while the provider name may not. The attribute delimiter, `(...)` for the bare form and
`{...}` for the key/value form, is ordinary Rust attribute syntax and does not change how the macro
parses the arguments inside.

## Common Mistakes

**The macro, not the compiler, rejects a const generic parameter on the trait:**

```text
error: const generic parameters are not supported on CGP component traits
```

The `IsProviderFor` supertrait records a component's extra parameters as a tuple of *types*, and
CGP's wiring dispatches on types rather than values, so that machinery cannot hold a const value.

The rule does not affect an associated `const` *item*. `const LIMIT: u64;` as a trait member is an
[associated const](https://doc.rust-lang.org/reference/items/associated-items.html), not a generic
parameter, and a provider supplies it in the ordinary way. Naming your own associated const from
*inside* a [`#[cgp_impl]`](./cgp_impl.md) body has one complication of its own, covered in that page's
[Common Mistakes](./cgp_impl.md#common-mistakes).

**The attribute must be applied to a trait.** The macro refuses a struct, an enum, or a free function
at parse time, with an error that names the trait it expected, instead of lowering it into code that
fails to compile later.

**The compiler, not the macro, reports a misplaced companion attribute, and it reports it several
times.** The macro carries an attribute it does not recognize onto *every* item it generates: the
consumer trait, the provider trait, and the impls built from each. For `#[allow(...)]` or a doc
comment, that repetition is welcome. For `#[uses(HasName)]` written above a component trait, it means
one *cannot find attribute* resolution error per generated item, and none of them mentions
`#[cgp_component]`. Read the repetition as the signal. It is the same error a typo in an attribute
name gives, so the fix is to move the attribute rather than to add an import.

## Related constructs

- [`#[cgp_impl]`](./cgp_impl.md) — the idiomatic way to write a provider for a component.
- [`#[cgp_provider]`](./cgp_provider.md) — the lower-level provider form it desugars to.
- [`#[cgp_fn]`](./cgp_fn.md) — the lighter alternative when one implementation is enough.
- [`delegate_components!`](./delegate_components.md) — wires a component to a provider on a type.
- [`check_components!`](./check_components.md) — verifies that wiring at compile time.
- [`#[cgp_type]`](./cgp_type.md) — the specialized form for a component that supplies a type.
- [`#[cgp_getter]`](./cgp_getter.md) — the specialized form for a component that reads a field.
- [`#[use_type]`](../attributes/use_type.md), [`#[extend]`](../attributes/extend.md), and
  [`#[derive_delegate]`](../attributes/derive_delegate.md) — attributes that change what it generates.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the trait split this
  macro creates, developed at length.
- [Bypassing coherence](/docs/concepts/coherence) — why the split exists at all.
- [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) — when a component is the right tier,
  and when it is more than the problem needs.

## Source

- Entry point: [`cgp_component.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_component.rs)
- Implementation: [`types/cgp_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_component/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
