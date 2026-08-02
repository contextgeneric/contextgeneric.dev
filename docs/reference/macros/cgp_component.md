---
sidebar_label: '#[cgp_component]'
---

# `#[cgp_component]`

Turn a trait into a component: the consumer trait callers use, the provider trait implementations
target, and the key that wires them together.

## What it's for

An ordinary Rust [trait](https://doc.rust-lang.org/book/ch10-02-traits.html) can have only one
implementation per type. That is usually what you want, and it is what lets the compiler resolve a
`where T: Display` bound without anyone naming an implementation — but it means a capability with
several plausible implementations has nowhere to put them. A type either sends real email or records
it for a test; it cannot do both, and choosing between them means editing the impl rather than
choosing at the point of use.

`#[cgp_component]` splits that one trait into two, so that *using* a capability and *implementing* it
stop being the same act:

- The **consumer trait** is what callers write — `rect.area()`. It keeps the name and shape you gave
  it.
- The **provider trait** is what implementations target. It is the same interface with `Self` moved
  into an explicit type parameter, so an implementation is written for a small named type of its own
  rather than for the type the capability is about.

Because each implementation now targets its own name, any number of them can coexist. A **provider**
is one of those names — a zero-sized type such as `RectangleArea` that exists only to identify an
implementation. A **context** — the type the capability runs against, which supplies the values it
needs as its fields — then picks the provider it wants through
[`delegate_components!`](./delegate_components.md), and generated glue routes calls on the consumer
trait to the provider that was picked. The whole choice is resolved during compilation and compiles
down to a direct call.

The macro also emits a **component name** — a marker type such as `AreaCalculatorComponent` — which is
the key wiring uses. You will see it in every `delegate_components!` entry and in most compiler
errors, so it is worth recognizing even though you never write its definition.

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

### Supertraits, and importing an abstract type

When a component depends on a type another component supplies — most often the error type from
[`HasErrorType`](../components/has_error_type.md) — import it with
[`#[use_type]`](../attributes/use_type.md) rather than writing the supertrait and the qualified path by
hand. The attribute adds the supertrait *and* rewrites a bare `Error` in your signatures to its fully
qualified form:

```rust
#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}
```

For a supertrait with no associated type to import, use [`#[extend(...)]`](../attributes/extend.md) in
preference to native `: Supertrait` syntax — `#[extend(HasName)]` reads as importing a capability,
where `pub trait CanGreet: HasName` reads as inheritance, which is not what a CGP supertrait is. An
associated type the trait declares *itself* is not imported and stays written as `Self::Output`.

One further companion attribute exists: [`#[derive_delegate(...)]`](../attributes/derive_delegate.md)
generates dispatch impls for a component generic over a parameter. It is superseded by the `open`
statement of [`delegate_components!`](./delegate_components.md) and is mostly something you read in
existing code rather than write.

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

Beyond those five, the macro emits the standard provider impls that let the component participate in
CGP's usual patterns: a [`UseContext`](../providers/use_context.md) impl, so the provider trait can be
satisfied by routing back through the context's own implementation; a
[`RedirectLookup`](../providers/redirect_lookup.md) impl, which is what the `open` statement and
[namespaces](./cgp_namespace.md) resolve through; and a
[`UseDelegate`](../providers/use_delegate.md) impl for each
[`#[derive_delegate(...)]`](../attributes/derive_delegate.md) attribute present.

Two details of the real output differ from the listings above, and both trip people up when reading an
error. The generated parameters carry **reserved names** — the context is literally `__Context__`
unless you override it, and the provider parameter is `__Provider__`; the readable `Context` and
`Provider` here are for legibility only. And a component with its own type parameters, such as
`CanCalculateArea<Shape>`, appends them *after* the context in the provider trait and groups them into
the `IsProviderFor` parameter tuple: `IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)>`.

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

## Source

- Entry point: [`cgp_component.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_component.rs)
- Implementation: [`types/cgp_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_component/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
