---
sidebar_label: '#[cgp_computer]'
sidebar_position: 13
---

# `#[cgp_computer]`

Define a `Computer` provider from a plain function.

## Overview

CGP models computation as a family of components varying along three axes: synchronous or async, fallible
or not, taking an input or not. A provider in that family is a struct with one or more impls threading a
**context** (the type the capability runs against, which supplies the values it needs as its fields), a
phantom `Code` tag, and an `Input`. Written by hand for a computation as small as "add two numbers", that is
disproportionate ceremony.

`#[cgp_computer]` lets you write just the computation:

```rust
#[cgp_computer]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
```

From that the macro produces the provider struct `Add`, an impl of the right base trait, and the wiring
that makes the *same function* answer the whole family. That wiring is the part worth understanding.
`Add` satisfies
`compute`, `try_compute`, `compute_async`, and `handle`, along with their by-reference forms, without you
implementing any of them.

That last property is the whole reason for the macro. The family exists so a provider can declare exactly
the capabilities it has, and the [promotion combinators](../providers/handler_combinators.md) exist so a
simpler provider can stand in where a more capable one is expected: an infallible computation is a
fallible one that never fails, a synchronous one is an async one that never awaits. `#[cgp_computer]` picks
the narrowest
base that fits your function and wires the promotions for the rest, so you write one body and get every
shape.

## Usage

Apply the attribute to a free function. It takes an optional provider name:

```rust
#[cgp_computer]
fn add(a: u64, b: u64) -> u64 {
    a + b
}

#[cgp_computer(MyAdder)]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
```

Omitted, the provider struct takes the function name in PascalCase: `add` becomes `Add`. Given, the
argument is used verbatim.

The function's shape decides everything else:

- **Its parameters become the input.** Several parameters are collected into one tuple, so
  `fn add(a: u64, b: u64)` has input `(u64, u64)`.
- **Its return type becomes the output.**
- **It must not take `self`.** A handler provider has no receiver; the context is supplied separately by
  the handler machinery.
- **It may be `async`**, and it may return a `Result`. Those two choices select the base trait.
- **Its generics and `where` clause carry over** to the generated impl, so the provider can itself be
  generic.

### What the two axes select

The macro reads `async`-ness and `Result`-ness independently, and each combination picks a base trait and a
promotion bundle:

| Your function | Base trait | Promotion bundle |
|---|---|---|
| `fn f(..) -> T` | `Computer` | `PromoteComputer<Self>` |
| `fn f(..) -> Result<T, E>` | `Computer` | `PromoteTryComputer<Self>` |
| `async fn f(..) -> T` | `AsyncComputer` | `PromoteAsyncComputer<Self>` |
| `async fn f(..) -> Result<T, E>` | `AsyncComputer` | `PromoteHandler<Self>` |

The `Result` row is worth reading twice. The base trait stays `Computer`, and its `Output` is simply the
`Result` type as written. Only the *bundle* changes, and it makes `try_compute` and `handle`
surface the `Ok`/`Err` outcome as success or failure rather than handing back a `Result` as a plain value.

## Examples

One function, every shape. The context here only has to supply the error type the fallible members need:

```rust
use cgp::prelude::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{Computer, TryComputer, AsyncComputer, Handler};

#[cgp_computer]
fn add(a: u64, b: u64) -> u64 {
    a + b
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
    }
}

// All four are answered by the single `add` definition:
// Add::compute(&App, PhantomData::<()>, (1, 2))        == 3
// Add::try_compute(&App, PhantomData::<()>, (1, 2))    == Ok(3)
// Add::compute_async(&App, PhantomData::<()>, (1, 2))  resolves to 3
// Add::handle(&App, PhantomData::<()>, (1, 2))         resolves to Ok(3)
```

Because the function returns a plain `u64`, the fallible forms always succeed. Switching it to return a
`Result` changes which bundle is wired and therefore what those forms mean, with no change at the call
sites:

```rust
#[cgp_computer]
fn add_with_error(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b).ok_or_else(|| "Overflow".to_string())
}
```

Now `try_compute` and `handle` propagate the `Err` when the addition overflows.

A generic function carries its parameters and bounds onto the provider, so one definition covers every type
that satisfies them:

```rust
#[cgp_computer]
pub fn add_generic<T: core::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}
```

## When to reach for it, and when not

**Reach for `#[cgp_computer]` when a step in a pipeline is naturally a function.** That is the case it
exists for, and it is the shortest route into the handler family.

- **Use [`#[cgp_producer]`](./cgp_producer.md) when the computation takes no input.** A constant, or a value
  drawn from the context alone. It is the input-free sibling and produces a `Producer`.
- **Write the provider by hand with [`#[cgp_impl]`](./cgp_impl.md) when the body needs the context.** This
  is the real boundary. A `#[cgp_computer]` function has no receiver and no access to the context, so it can
  only transform its inputs. The moment the computation needs a field, an abstract type, or another
  capability, it wants a provider impl of `Computer` or `Handler` written with `#[cgp_impl]`, where `self` is
  the context and [`#[implicit]`](../attributes/implicit.md) and [`#[uses]`](../attributes/uses.md) work
  normally.
- **Use [`#[cgp_fn]`](./cgp_fn.md) when what you want is a capability on the context, not a pipeline step.**
  The two look similar and differ in what they produce: `#[cgp_fn]` gives a trait a context implements,
  called as `self.thing()`; `#[cgp_computer]` gives a *provider* that gets wired into a handler component and
  composed with combinators. If you are not building a pipeline, reach for `#[cgp_fn]` instead.
- **Wire the [handler combinators](../providers/handler_combinators.md) directly for composition.** The macro
  produces one step; `PipeHandlers` and friends chain them.

One thing not to do is reach for the handler family because a capability happens to transform a value. The
family earns its keep when computations are *composed*: piped, dispatched on a `Code` tag, promoted between
variants. A single transform with one caller is a method.

## Under the hood

:::note

### Advanced

This section shows the three items the macro emits, and how the promotion wiring makes one function answer
the whole family. You do not need it to use `#[cgp_computer]`, but the delegation block is unusual enough
that seeing it once explains a lot of handler errors. `cargo cgp expand` prints the same thing for your own
code.

:::

The macro emits the function unchanged, a provider impl of the base trait, and a
[`delegate_components!`](./delegate_components.md) block wiring the rest of the family. From this input:

```rust
#[cgp_computer]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
```

it produces the base impl, with the parameters collected into a tuple that the method destructures back
apart:

```rust
#[cgp_new_provider]
impl<__Context__, __Code__> Computer<__Context__, __Code__, (u64, u64)> for Add {
    type Output = u64;

    fn compute(
        _context: &__Context__,
        _code: PhantomData<__Code__>,
        (arg_0, arg_1): (u64, u64),
    ) -> Self::Output {
        add(arg_0, arg_1)
    }
}
```

[`#[cgp_new_provider]`](./cgp_provider.md) declares `pub struct Add;` and derives the
[`IsProviderFor`](../traits/is_provider_for.md) impl, whose parameter tuple here is
`(__Code__, (u64, u64))`, the code tag and the input. The context and code parameters are introduced under
the reserved names `__Context__` and `__Code__`, and the body ignores both.

Then the promotion wiring, which is the interesting half:

```rust
delegate_components! {
    Add {
        [
            ComputerRefComponent,
            TryComputerComponent,
            TryComputerRefComponent,
            AsyncComputerComponent,
            AsyncComputerRefComponent,
            HandlerComponent,
            HandlerRefComponent,
        ] ->
            PromoteComputer<Self>,
    }
}
```

Note the **`->` operator** rather than `:`. It delegates each key to *the value's own entry for that key*
rather than to the value itself, so `Add` inherits whatever `PromoteComputer<Self>` resolves each component
to. [`PromoteComputer`](../providers/handler_combinators.md) is itself a table of single-step promoters.
`ComputerComponent` is absent from the list because `Add` implements it directly.

The other three combinations differ only in which base trait is implemented and which bundle is named. An
`async` function implements `AsyncComputer` instead, its method is `compute_async` and awaits the call, and
it delegates a smaller set (`AsyncComputerRefComponent`, `HandlerComponent`, `HandlerRefComponent`), since
the synchronous members are not derivable from an async base. A `Result`-returning function keeps its base
trait and swaps the bundle to `PromoteTryComputer<Self>` or, when also async, `PromoteHandler<Self>`.

Generics flow through to the impl, appended *ahead* of the introduced `__Context__` and `__Code__`
parameters, so `add_generic<T>` yields a provider generic over `T` carrying the `T: Add<Output = T>` bound. A
reference parameter is preserved too: `fn to_string_ref<V: Display>(value: &V) -> String` becomes a
`Computer` whose input tuple is `(&V)`, and the bundle's by-reference entries make it serve the `…Ref`
components.

<details>
<summary>Formal grammar</summary>

The attribute argument is a single optional provider name, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
CgpComputerArgs -> ProviderName?

ProviderName    -> IDENTIFIER
```

Omitted, the provider struct takes the function name converted to PascalCase; a given `IDENTIFIER` is used
verbatim. The annotated function is plain Rust, and the macro reads its parameters, return type,
`async`-ness, generics, and `where` clause to choose the base trait and promotion bundle as described above.

</details>

## Gotchas

**The function cannot reach the context.** There is no receiver and no context parameter in scope, so
`#[implicit]` arguments and `#[uses]` have nothing to attach to. A computation that needs anything from its
context is a provider impl written with [`#[cgp_impl]`](./cgp_impl.md), not a `#[cgp_computer]`.

**Several parameters become one tuple input.** That is invisible when the provider is wired into a pipeline
that supplies the tuple, and very visible when calling the provider directly:
`Add::compute(&App, PhantomData::<()>, (1, 2))` takes one argument, not two. A single-parameter function's
input is `(T)`, which Rust treats as plain `T`.

**The fallible forms need an error type on the context.** `try_compute` and `handle` name the context's
abstract error, so a context wiring the provider without an
[`ErrorTypeProviderComponent`](../components/has_error_type.md) fails on those members while `compute` works
fine. This is an error about the error type, arriving only for part of the family. Note also that the wiring key is
**not in the prelude**: it has to be imported from `cgp::core::error`, and forgetting that reports the
component as an unresolved type rather than as a missing import.

**A `Result` return changes the bundle, not the base trait.** The `Computer` impl's `Output` *is* the
`Result`, so `compute` hands back a `Result` as an ordinary value while `try_compute` treats its `Err` as
failure. Both are available on the same provider, and which one a pipeline uses decides whether an error
short-circuits.

## Related constructs

- [`#[cgp_producer]`](./cgp_producer.md) — the input-free sibling, producing a `Producer`.
- [`Computer`](../components/computer.md) — the base component, with its by-reference and async variants.
- [`Handler`](../components/handler.md) and [`TryComputer`](../components/try_computer.md) — the more capable
  members the promotions reach.
- [Handler combinators](../providers/handler_combinators.md) — the `Promote*` bundles this wires, plus
  `PipeHandlers` and `ComposeHandlers` for composing steps.
- [`#[cgp_impl]`](./cgp_impl.md) — for a handler provider that needs its context.
- [`#[cgp_fn]`](./cgp_fn.md) — for a capability on the context rather than a pipeline step.
- [`#[cgp_new_provider]`](./cgp_provider.md) — what the generated impl is emitted through.
- [`#[cgp_auto_dispatch]`](./cgp_auto_dispatch.md) — generates per-variant computers from a trait.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family this provider joins, across its sync,
  fallible, and input-free axes.

## Source

- Entry point: [`entrypoints/cgp_computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-extra-macro-lib/src/entrypoints/cgp_computer.rs)
- `Result`-versus-value detection: [`parse/maybe_result.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-extra-macro-lib/src/parse/maybe_result.rs)
- The base traits: [`cgp-handler/src/components/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-handler/src/components/)
- The promotion bundles: [`providers/promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
