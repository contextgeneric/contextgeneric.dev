---
sidebar_label: '#[cgp_component]'
---

# `#[cgp_component]`

Turn a trait into a component: the consumer trait callers use, the provider trait implementations
target, and the key that wires them together.

## Overview

`#[cgp_component]` can be applied to any Rust trait definition to give that trait the full set of
CGP's capabilities. Applying it takes nothing away: the trait you wrote keeps working exactly as it
did, now under the name CGP calls the **consumer trait**, and every call site that already uses it
keeps compiling unchanged. The macro adds a matching **provider trait** for implementations to target,
a small marker type called the **component** that names the capability for wiring, and a pair of
[**blanket implementations**](https://blog.implrust.com/posts/2025/09/blanket-implementation-in-rust/)
that connect the three without requiring you to write them.

`#[cgp_component]` makes it possible to define several overlapping implementations of one capability
at once. An ordinary Rust trait does not allow this. A
[trait](https://doc.rust-lang.org/book/ch10-02-traits.html) can have only one implementation per
type, a rule called [coherence](/docs/concepts/coherence). That rule is usually the right one: it lets
the compiler resolve a `where T: Display` bound without anyone naming which implementation applies.
But it also means a capability with several plausible implementations has nowhere to put the
alternatives. A type can send real email or record it for a test, never both at once, and switching
between the two means editing the impl itself rather than choosing between them at the point of use.

`#[cgp_component]` solves that by splitting the one trait into two: the
[consumer and provider traits](/docs/concepts/consumer-and-provider-traits), so that *using* a
capability and *implementing* it stop being the same act.

- The **consumer trait** is the trait callers write, such as `rect.area()`. It is your original
  trait, kept exactly as you wrote it.
- An implementation targets the **provider trait** instead. It repeats the same methods with `Self`
  moved into an explicit type parameter, so you write an implementation for a small named type of its
  own rather than for the type the capability is actually about.

Moving `Self` out of the way lets more than one implementation coexist: each one now targets its own
small type instead of competing for the single `Self` slot every plain trait has. A **provider** is
one of those targets: a zero-sized type such as `RectangleArea` that carries no data of its own and
exists only to name one implementation. The type the capability actually runs against is called the
**context**. It supplies whatever values an implementation needs as its own fields, then picks the
provider it wants through [`delegate_components!`](./delegate_components.md), and the generated
blanket implementations route a call on the consumer trait through to that choice automatically. None
of this costs anything at runtime. Wiring fixes the provider once, at compile time, and the compiler
turns the call into a direct, statically-dispatched call, exactly as if you had written the
implementation yourself.

The macro's last piece is the **component** itself, a marker type such as `AreaCalculatorComponent`
that names the capability and is the key `delegate_components!` wires against. You will meet it in
every wiring entry and in most compiler errors that involve this capability, so you should be able to
recognize it immediately, even though you never write its definition yourself.

## Using it

Apply the attribute to a trait definition and give it the provider trait's name. The simplest form is
a bare identifier:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

`CanCalculateArea` is the consumer trait and `AreaCalculator` is the provider trait. The naming
convention is worth following: a consumer trait reads as a verb (`CanDoSomething`) and a provider trait
as a noun (`SomethingDoer`, or `…Provider` when no noun fits).

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

Each key has a default and only `provider` is required — passing a bare identifier is shorthand for
setting it alone.

| Key | What it sets | Default |
|---|---|---|
| `provider` | The provider trait's name | *required* |
| `name` | The component marker type | The provider name plus `Component` |
| `context` | The identifier used for the generated context type parameter | `__Context__` |

The `context` default is deliberately unusual so that it cannot collide with a type parameter of your
own.

### Companion attributes

Four attributes may be written above the trait, beside `#[cgp_component]`, and each changes what the
macro generates. Any of them may be repeated, and the first three also accept a comma-separated list
inside one attribute.

| Attribute | What it adds |
|---|---|
| [`#[use_type(Trait.Type)]`](../attributes/use_type.md) | Imports an abstract type: adds the supertrait *and* rewrites the bare name in your signatures |
| [`#[extend(Trait)]`](../attributes/extend.md) | Adds a supertrait with no type to import |
| [`#[derive_delegate(...)]`](../attributes/derive_delegate.md) | Generates dispatch impls for a component generic over a parameter |
| [`#[prefix(@path in Namespace)]`](./cgp_namespace.md) | Registers the component into a namespace under a type-level path |

When a component depends on a type another component supplies — most often the error type from
[`HasErrorType`](../components/has_error_type.md) — import it with `#[use_type]` rather than writing
the supertrait and the qualified path by hand:

```rust
#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}
```

One form of `#[use_type]` is refused here. The equality form
`#[use_type(HasScalarType.{Scalar = f64})]`, which pins an imported type to a concrete one, belongs to
a provider that has decided the type rather than to the definition every provider has to satisfy, so
the macro rejects it with *Type equality constraints cannot be used in component trait definition*.
Every other form of the attribute works on a component exactly as it does elsewhere.

For a supertrait with no associated type to import, use `#[extend(...)]` in preference to native
`: Supertrait` syntax — `#[extend(HasName)]` reads as importing a capability, where
`pub trait CanGreet: HasName` reads as inheritance, which is not what a CGP supertrait is. An
associated type the trait declares *itself* is not imported and stays written as `Self::Output`.

`#[derive_delegate(...)]` is superseded by the `open` statement of
[`delegate_components!`](./delegate_components.md) and is mostly something you read in existing code
rather than write.

Three attributes that look like they belong here do not, and the error is confusing enough to be worth
naming: `#[uses(...)]`, `#[extend_where(...)]`, and `#[use_provider(...)]` are read by
[`#[cgp_impl]`](./cgp_impl.md) and [`#[cgp_fn]`](./cgp_fn.md) but not by `#[cgp_component]`. None of
them is an attribute in its own right, so writing one here leaves a name nothing can resolve — see
[Gotchas](#gotchas). The equivalent of `#[uses]` on a component is a supertrait, written with
`#[extend]`.

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

## When to reach for it, and when not

Reach for `#[cgp_component]` when a capability genuinely needs **more than one implementation, and the
choice belongs to the type using it.** That is the case it exists for, and its machinery is not free:
a component is a trait, a second trait, a marker type, and a line of wiring per type.

Prefer something simpler when you can.

- **One implementation, ever.** Use [`#[cgp_fn]`](./cgp_fn.md) instead. It builds the capability
  straight from a function, needs no wiring at all, and keeps working unchanged if a second
  implementation ever arrives — so it is the right starting point rather than a lesser one.
- **One implementation per type, chosen globally.** That is what a plain Rust trait already does well.
  Reach for a component when two *different* applications must make different choices for the same
  type, or when the implementations must overlap in a way the compiler rejects.
- **A closed set of variants with fixed operations.** An `enum` and a `match` are clearer than any
  machinery.

There is also a finer line worth knowing: a capability may genuinely need several implementations
while each individual implementation serves exactly one type. In that case you can implement the
consumer trait directly on each concrete type, as you would any Rust trait, and skip providers
entirely. Named providers earn their place once a second type wants the *same* implementation, or once
an implementation should compose with a wrapper.

### How many items should a component have?

A component trait is an ordinary trait. It takes as many methods, associated types, and associated
consts as any other, and every one of them is reproduced on the provider trait — CGP's own
[`CanCompute`](../components/computer.md) declares an associated `Output` beside its method. There is
no cap.

What to group is a judgement rather than a rule, and the useful question is: **everything in one
component is answered by one provider choice.** Items a single choice settles belong together — a
method and the associated type it returns, or several field reads one getter provider answers by name.
Items that separate choices settle are better apart, because grouping them costs reuse: every provider
then carries the union of the dependencies of all the methods, a wrapper must forward the methods it
has no opinion about, and a type that needs only part of the surface must still supply the rest.

The usual sign of a component that has grown past one decision is a consumer trait named after a noun
rather than a verb — `Shape` carrying `area`, `perimeter`, `scale`, and `rotate`. It compiles, but very
little of it is reusable. A good check before committing: could a second type plausibly reuse one of
this trait's providers *whole*? If not, implement the trait directly on the concrete type and skip the
machinery.

## Under the hood

:::note

### Advanced

This section shows the code the macro generates. You do not need it to use `#[cgp_component]`, but
CGP's constructs are macros, and reading what one produces is the fastest way to understand a compiler
error that names a type you did not write. Running `cargo cgp expand` prints the same thing for your
own code.

:::

From this input:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

the macro emits five items. First, the **consumer trait**, unchanged:

```rust
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

Second, the **provider trait**: the same interface with `Self` replaced by a leading `Context` type
parameter and every `self`/`Self` rewritten to `context`/`Context`. Its
[`IsProviderFor`](../traits/is_provider_for.md) supertrait is what makes an unmet dependency report
itself by name rather than as a bare "trait not implemented"; its third argument is a tuple of the
component's extra type parameters, empty here:

```rust
pub trait AreaCalculator<Context>:
    IsProviderFor<AreaCalculatorComponent, Context, ()>
{
    fn area(context: &Context) -> f64;
}
```

`IsProviderFor` *replaces* the supertrait list rather than joining it. Any supertrait the consumer
trait had — written natively, or added by `#[extend]` or `#[use_type]` — becomes a `where` predicate
on the context instead, which follows from the `Self`-to-`Context` move: a supertrait constrains the
type the capability is about, and on the provider side that type is the context parameter. So
`#[extend(HasName)]` on a `CanGreet` component produces:

```rust
pub trait Greeter<Context>: IsProviderFor<GreeterComponent, Context, ()>
where
    Context: HasName,
{
    fn greet(context: &Context);
}
```

That predicate is added to every emitted item that mentions the context — the two blanket impls below and
the `UseContext` and `RedirectLookup` impls further down each gain their own `Context: HasName` — since
none of them can apply where the supertrait does not hold.

**A method may carry a default body, and the body moves to the provider trait.** This is the other thing
the `Self`-to-`Context` move reaches, and it is worth knowing because it is what lets a provider inherit a
default at all. Given `fn greet(&self) -> String { format!("Hello, {}!", self.name()) }` on the consumer
trait, the provider trait gets `fn greet(context: &Context) -> String { format!("Hello, {}!", context.name()) }`
— rewritten, and still a default. The consumer trait keeps its copy too, so the body appears twice in an
expansion.

That is what makes an **empty provider impl** meaningful, and it is how
[`UseDefault`](../providers/use_default.md) works:

```rust
#[cgp_impl(UseDefault)]
impl<Context: HasName> Greeter for Context {}
```

Third, the **consumer blanket impl**, the bridge that lets callers write `context.area()`: any context
implementing the provider trait *for itself* gets the consumer trait.

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

Fourth, the **provider blanket impl**, which is what makes wiring work: anything that delegates this
component through [`DelegateComponent`](../traits/delegate_component.md) inherits the provider trait
from whatever it delegates to.

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

The `IsProviderFor` bound sits on `Provider` itself rather than on `Provider::Delegate`. That is what
threads a provider's dependencies down the delegation chain so they still appear in an error message.

Fifth, the **component marker**, the key wiring uses:

```rust
pub struct AreaCalculatorComponent;
```

Those five are emitted in the order consumer trait, consumer impl, provider trait, provider impl,
marker — the listings above pair each trait with the impl that routes to it, which reads better than
the order they actually appear in.

Beyond the five, the macro emits the provider impls that let the component participate in CGP's usual
patterns. Two are always emitted, and two are one per attribute:

- A [`UseContext`](../providers/use_context.md) impl, so the provider trait can be satisfied by
  routing back through the context's own implementation. Its only bound is
  `Context: CanCalculateArea`, and each method forwards to the consumer method.
- A [`RedirectLookup`](../providers/redirect_lookup.md) impl, which is what the `open` statement and
  [namespaces](./cgp_namespace.md) resolve through.
- One [`UseDelegate`](../providers/use_delegate.md) impl per
  [`#[derive_delegate(...)]`](../attributes/derive_delegate.md) attribute.
- One namespace impl per [`#[prefix(@path in Namespace)]`](./cgp_namespace.md) attribute, binding the
  component's key inside that namespace to a redirect down the given path.

The `RedirectLookup` impl is where a component's own type parameters earn a place in a path, and it is
what makes `@AreaCalculatorComponent.Rectangle` resolve. For a component with type parameters the
impl does not look the incoming path up directly: it appends the parameters to it first, so a lookup
that arrives at `AreaCalculatorComponent` carrying no path ends up looking for `Rectangle`. Only
*type* parameters take part — a lifetime or a const parameter cannot key a path and is left out.

Two details of the real output differ from the listings above, and both trip people up when reading an
error. The generated parameters carry **reserved names** — the context is literally `__Context__`
unless you override it, and the provider parameter is `__Provider__`; the readable `Context` and
`Provider` here are for legibility only. And a component with parameters of its own appends them
*after* the context in the provider trait — except lifetimes, which Rust requires to lead — and groups
them into the `IsProviderFor` parameter tuple. That tuple holds types, so a lifetime is lifted into
[`Life<'a>`](../types/life.md):

| Component | Provider trait | `IsProviderFor` params |
|---|---|---|
| `CanCalculateArea` | `AreaCalculator<__Context__>` | `()` |
| `CanCalculateArea<Shape>` | `AreaCalculator<__Context__, Shape>` | `(Shape)` |
| `CanEncode<Value, Format>` | `Encoder<__Context__, Value, Format>` | `(Value, Format)` |
| `HasReference<'a, T>` | `ReferenceGetter<'a, __Context__, T>` | `(Life<'a>, T)` |

Bounds and defaults are dropped from the tuple, which names the parameters positionally and nothing
more. A const parameter has no place in it at all, which is why the macro rejects one — see
[Gotchas](#gotchas).

<details>
<summary>Formal grammar</summary>

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
`GenericArgs` is the Rust grammar's `< … >` argument list — so the component name may carry generic
parameters while the provider name may not. The attribute delimiter — `(...)` for the bare form,
`{...}` for the key/value form — is ordinary Rust attribute syntax and does not change how the
arguments inside are parsed.

</details>

## Gotchas

**A const generic parameter on the trait is rejected**, by the macro rather than by the compiler:

```text
error: const generic parameters are not supported on CGP component traits
```

A component's extra parameters are recorded as a tuple of *types* in the `IsProviderFor` supertrait, and
CGP's wiring dispatches on types rather than values, so a const value has nowhere to live in that
machinery.

An associated `const` *item* is unaffected — `const LIMIT: u64;` as a trait member is an
[associated const](https://doc.rust-lang.org/reference/items/associated-items.html), not a generic
parameter, and a provider supplies it in the ordinary way. Naming your own associated const from
*inside* a [`#[cgp_impl]`](./cgp_impl.md) body has a wrinkle of its own, covered in that page's
Gotchas.

**The attribute must be applied to a trait.** A struct, an enum, or a free function is refused at
parse time with an error naming the trait the macro expected, rather than being lowered into code that
fails to compile later.

**A misplaced companion attribute is reported by the compiler, not by the macro — and reported several
times.** An attribute `#[cgp_component]` does not recognize rides through onto *every* item the macro
generates: the consumer trait, the provider trait, and the impls built from each. For `#[allow(...)]` or
a doc comment that is exactly what you want. For `#[uses(HasName)]` written above a component trait it
means one *cannot find attribute* resolution error per generated item, none of which mentions
`#[cgp_component]`. Read the repetition as the signal: it is the same error a typo in an attribute name
gives, so the fix is to move the attribute rather than to add an import.

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
- [How much CGP to use](/docs/concepts/modularity-hierarchy) — when a component is the right rung,
  and when it is more than the problem needs.

## Source

- Entry point: [`cgp_component.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_component.rs)
- Implementation: [`types/cgp_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_component/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
