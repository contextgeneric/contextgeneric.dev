---
sidebar_label: 'UseInputDelegate'
sidebar_position: 13
---

# `UseInputDelegate`

Dispatch a handler to a different inner provider per input type, through a lookup table.

## Overview

`UseInputDelegate<Components>` chooses a handler provider by the type of the value being handled. An
ordinary handler component is answered by one provider. `UseInputDelegate` performs a second lookup: it
treats the handler's `Input` type as a key and reads the matching inner provider out of a table, so a
single wiring entry on a **context**, the type a method runs on, fans out to many
input-specific providers. Like every CGP provider, it holds no runtime value; the table rides in
`PhantomData`.

It is the sibling of [`UseDelegate`](../use_delegate.md). Where `UseDelegate` keys on the first generic
parameter of a provider trait, the `Code` for a handler, `UseInputDelegate` keys on the `Input`
parameter, so the provider that handles a value is chosen by the type of that value. The handler
components enable both dispatchers at once. Both are legacy forms: the `open` statement of
[`delegate_components!`](../../macros/delegate_components.md#choosing-a-provider-per-type-the-open-statement)
dispatches on the code, the input, or both with no table type, as the next section shows for the
input.

## Usage

Import it from `cgp::extra::handler`. It takes one type parameter, the lookup table, and is wired
through a nested table that maps each concrete input type to its provider:

```rust
use cgp::extra::handler::UseInputDelegate;

delegate_components! {
    App {
        ComputerComponent: UseInputDelegate<new AppComputers {
            Circle: ComputeCircleArea,
            Rectangle: ComputeRectangleArea,
        }>,
    }
}
```

The outer entry routes the handler component to `UseInputDelegate<AppComputers>`, and the inner table
maps each input type to the provider responsible for it. The current form stores the same entries on
the context with `open`. Each key has one path segment per type parameter of
`CanCompute<Code, Input>`, and a generic first segment matches any code, so the second selects by the
input type:

```rust
delegate_components! {
    App {
        open ComputerComponent;

        @ComputerComponent.<Code> Code.Circle: ComputeCircleArea,
        @ComputerComponent.<Code> Code.Rectangle: ComputeRectangleArea,
    }
}
```

## When to use it

**Read `UseInputDelegate` when you meet it in existing code, and write the `open` form instead.**
Dispatch by input type is common with the [dispatch combinators](../dispatch/index.md), where a matcher
routes each variant of an enum to a handler chosen by the payload type, and `open` expresses it
without a table type. The table form still works, since every handler component keeps the
`#[derive_delegate(UseInputDelegate<Input>)]` that generates it, so existing wiring does not need to
change.

## Under the hood

`UseInputDelegate<Components>` carries the table in `PhantomData`:

```rust
pub struct UseInputDelegate<Components>(pub PhantomData<Components>);
```

Each handler component trait is declared with two
[`#[derive_delegate]`](../../attributes/derive_delegate.md) directives, `UseDelegate<Code>` and
`UseInputDelegate<Input>`, as on the `Computer` component:

```rust
#[cgp_component(Computer)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
pub trait CanCompute<Code, Input> {
    type Output;

    fn compute(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
}
```

The second directive makes [`#[cgp_component]`](../../macros/cgp_component.md) generate a provider impl
that looks `Components` up by the `Input` type and forwards to the matching delegate:

```rust
impl<Context, Code, Input, Components, Delegate> Computer<Context, Code, Input>
    for UseInputDelegate<Components>
where
    Components: DelegateComponent<Input, Delegate = Delegate>,
    Delegate: Computer<Context, Code, Input>,
{
    type Output = Delegate::Output;

    fn compute(context: &Context, code: PhantomData<Code>, input: Input) -> Self::Output {
        Delegate::compute(context, code, input)
    }
}
```

The lookup key is the `Input` type, while `Code` and `Context` pass through unchanged. The same impl
shape is generated for every handler family member.

## Related constructs

- [`UseDelegate`](../use_delegate.md) — the sibling that keys on the `Code` selector.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates both dispatch impls on a
  handler component.
- [`delegate_components!`](../../macros/delegate_components.md) — wires it through a nested table.
- [Dispatch combinators](../dispatch/index.md) — the main users of input dispatch, selecting a per-variant
  handler by payload type.
- [`DelegateComponent`](../../traits/wiring/delegate_component.md) — the table the lookup reads.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family whose input this dispatches on.
- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data value to per-type handlers.

## Source

- [`types.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/types.rs),
  with the impls generated from the `#[derive_delegate(UseInputDelegate<Input>)]` directive on the
  handler components in
  [`components/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-handler/src/components).

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
