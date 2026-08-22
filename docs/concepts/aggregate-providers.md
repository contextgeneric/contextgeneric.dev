---
sidebar_label: 'Aggregate providers'
sidebar_position: 8
---

# Aggregate providers

Bundling a group of wiring choices into one reusable provider that other contexts adopt as a unit.

This page answers *how do several contexts share a set of choices without repeating them?* It shows the
bundle, traces a call through it to establish the one fact everything else follows from, and ends on the
rule about checking, which is the only place a bundle behaves unlike anything else. It closes on when to
reach for the heavier alternative instead.

## A table whose target is not a context

A wiring table usually belongs to a context. It does not have to.

Add `new` and the table gets a fresh type of its own:

```rust
delegate_components! {
    new GeometryComponents {
        AreaCalculatorComponent: RectangleArea,
        PerimeterCalculatorComponent: RectanglePerimeter,
    }
}
```

`GeometryComponents` is an **aggregate provider**: a zero-sized type whose entire content is that table.
Nothing constructs it, it holds no data, and it does not stand for an application. It is a name for a
group of decisions, so that the decisions can be adopted together.

A context then takes the whole group in one line:

```rust
delegate_components! {
    Rectangle {
        [
            AreaCalculatorComponent,
            PerimeterCalculatorComponent,
        ]: GeometryComponents,
    }
}
```

`Rectangle` now has both capabilities, and the choices behind them live in one place. Change the bundle
and every context using it changes with it.

## The context stays the context

Everything else on this page follows from one fact, and tracing a call is the quickest way to establish
it. When `rectangle.area()` resolves:

- `Rectangle` gets the capability because it implements the provider trait *for itself*;
- it implements that because its table sends the component to `GeometryComponents`;
- and `GeometryComponents` implements it because *its* table sends the component to `RectangleArea`.

At every step the context is `Rectangle`. `GeometryComponents` only ever appears as the thing being
delegated to. So when `RectangleArea` reads a `width`, it reads it from `Rectangle`. The bundle has no
`width`, is never asked for one, and would not be consulted if it had one.

This is why a bundle is a *provider* rather than a context: it is something delegated **to**, never
something used **as**. Bundles nest for the same reason: a table entry may name another table, and
resolution walks each in turn while the context argument stays fixed on the real context at the end of
the chain.

## Checking a bundle asks the wrong question

There is one place a bundle behaves unlike anything else, and it is worth knowing before you meet it.

**Never wire a bundle with the fused
[`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components).** That macro
derives a check asking whether the target can *use* each component as a context. For a bundle that is a
question about a role it never plays, so the answer carries no information, and which answer comes back
depends on what is inside:

- If the bundled providers need nothing from their context, they implement the capability for *every*
  context, the bundle included. The check passes. It has proved nothing, and it looks like it has.
- If a bundled provider needs a field or a type, the check fails, reporting that `GeometryComponents`
  does not have a `width`, blaming the bundle for something the real context would have supplied.

The silent case is the dangerous one. Wire a bundle with plain
[`delegate_components!`](/docs/reference/macros/delegate_components) and let it be verified where the
question means something.

## Verifying one properly

A bundle is checked through a context that uses it. Checking `Rectangle` walks the whole chain, through
the bundle's table, down to `RectangleArea`'s requirements, against `Rectangle`'s fields, so a gap
several bundles deep still surfaces:

```rust
check_components! {
    Rectangle {
        AreaCalculatorComponent,
        PerimeterCalculatorComponent,
    }
}
```

When the bundle itself needs pinning down, usually to find which layer of a nested stack is broken,
the [`#[check_providers]`](/docs/reference/macros/check_components) form asserts the provider-side
question instead, naming the bundle *for a concrete context*:

```rust
check_components! {
    #[check_providers(
        RectangleArea,
        GeometryComponents,
    )]
    Rectangle {
        AreaCalculatorComponent,
    }
}
```

Both routes go through a real context. Neither treats the bundle as one.

## When a namespace is the better tool

A bundle and a [namespace](./namespaces.md) both package reusable wiring, and they differ in how a
context takes it on.

A context adopts a bundle by **delegating named components to it**, `[A, B]: TheBundle`, so the
context spells out which components come from where. That is direct, obvious to read, and it scales
linearly: twenty components from a bundle means twenty names in the brackets.

A context adopts a namespace by **joining it**, after which everything it does not wire itself falls
through, and a direct entry overrides just that key. That is what you want once the count is large or
the defaults should be inherited and selectively replaced.

Reach for a bundle when the group is small and you want the delegation visible. Reach for a namespace
when the table has outgrown reading, or when a library is publishing defaults for applications to
customize.

## What it costs

**It adds a hop.** Finding what answers a capability now means reading two tables instead of one, and
nested bundles mean more. The wiring is still explicit and still greppable, and it is one more step
between the call and the code.

**It pays only when the repetition is real.** A bundle used by one context is a table split in two for
nothing. The threshold is a second context wanting the same group, not the anticipation of one.

**It is the construct whose checking rule is a genuine trap**, per the section above, and the failure
mode is a check that passes. Nothing in the code marks a bundle as different from a context, since both
are `delegate_components!` on a type, so the distinction has to be held by the person writing it.

**And the type name carries no clue.** `GeometryComponents` is a struct like any other; only its usage
says it is a bundle. Naming the group rather than a thing, with a `…Components` suffix, is the convention
that keeps it legible.

## Where to go next

[Namespaces](./namespaces.md) is the heavier sibling, and the one to read if the reason you are here is
a table that has grown too long. [Consumer and provider traits](./consumer-and-provider-traits.md) is
where the provider-versus-context distinction this page rests on comes from, and
[Checking your wiring](./check-traits.md) covers both check forms in full.

For the constructs, [`delegate_components!`](/docs/reference/macros/delegate_components) defines and
consumes a bundle through its `new` keyword and its array-key form, and
[`check_components!`](/docs/reference/macros/check_components) carries `#[check_providers]`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
