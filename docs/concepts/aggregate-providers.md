---
title: 'Reuse a group of wiring choices as one provider'
sidebar_label: 'Aggregate providers'
sidebar_position: 8
description: 'How a provider bundle records a group of wiring choices once, why a context delegates to it, and why a bundle must be checked through a real context.'
---

# Aggregate providers

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations whose
choices can be bundled. An **aggregate provider** collects provider choices into a wiring table that
several contexts can reuse. Each context delegates a group of components to that table, so changing
the shared choices can update them together. The chosen providers still receive the original context
and use its data; the bundle only connects them.

## A table whose target is not a context

`delegate_components!` can give a wiring table to a provider type. The `new` keyword declares that
type alongside its table:

```rust
delegate_components! {
    new GeometryComponents {
        AreaCalculatorComponent: RectangleArea,
        PerimeterCalculatorComponent: RectanglePerimeter,
    }
}
```

The `GeometryComponents` name now stands for the choice of `RectangleArea` and `RectanglePerimeter`.
Its type is a zero-sized marker for delegation, so the program needs no bundle value to construct or
store. The data those providers operate on still belongs to whichever context uses the bundle.

A context adopts the group by delegating the relevant components to the bundle:

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

With these entries, `Rectangle` uses the bundle's choices for both components, provided it meets
their requirements. Another context can delegate the same keys to `GeometryComponents` and share
those choices. Editing the bundle then changes both contexts together.

## The context stays the context

Delegation through a bundle preserves the original context. The compiler resolves
`rectangle.area()` through this chain:

1. `Rectangle` delegates `AreaCalculatorComponent` to `GeometryComponents`.
2. `GeometryComponents` delegates the same component to `RectangleArea`.
3. `RectangleArea` calculates the area using `Rectangle` as its context.

At the end of that chain, `RectangleArea` still reads `width` and `height` from `Rectangle`.
Passing through `GeometryComponents` has changed where the provider choice is recorded, while
leaving the source of its data intact. Another context can use the same chain if it satisfies
the provider's requirements.

The same delegation can continue through another bundle. At each step, the compiler follows the
next table while keeping the original context argument, until it reaches the provider that performs
the work. It resolves that chain at compile time, so adding a bundle does not add a runtime lookup.

## Checking a bundle asks the wrong question

Checking a bundle as a context does not verify that an application can use it. In particular,
[`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components) checks the
type receiving the table. Applied to `GeometryComponents`, it asks whether the bundle itself can
supply everything its providers require.

If those providers need nothing from their context, this check can pass without saying anything
about the intended application. If they need a field such as `width`, it can fail because the bundle
lacks that field, even though `Rectangle` supplies it. Either way, the check has treated
`GeometryComponents` as the context whose data the providers will use.

Because neither result answers whether an application can use the bundle, define bundles with
plain [`delegate_components!`](/docs/reference/macros/delegate_components) and check their
requirements against a context that will actually use them.

## Verifying one properly

A check on `Rectangle` follows the bundle's delegation and checks the providers against
`Rectangle`'s fields and traits:

```rust
check_components! {
    Rectangle {
        AreaCalculatorComponent,
        PerimeterCalculatorComponent,
    }
}
```

Because the check starts from `Rectangle`, it follows the delegation chain with the same context
that a method call would use. It can therefore catch a missing requirement even when the provider
that needs it sits several nested bundles away.

`#[check_providers]` lets you check the bundle and an individual provider separately against the
same context. This helps locate a failure within a chain:

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

The separate assertions ask whether `RectangleArea` and `GeometryComponents` can each supply this
component for `Rectangle`. Comparing their failures helps locate the broken layer, while keeping
the bundle in its intended role as a provider for that context.

## When a namespace is the better tool

With a small group, `[A, B]: TheBundle` makes the adopted choices easy to see at the context's
wiring site. As the group grows, though, every context still has to list the components it takes
from the bundle. Maintaining those lists can become another source of repetition.

When those repeated lists become difficult to maintain, a [namespace](./namespaces.md) can let
contexts inherit a shared table instead. Its paths organize the lookups and leave chosen
destinations for applications to fill. To allow that customization, the shared namespace must
leave those paths unbound, since a context cannot override an inherited binding.

A bundle therefore keeps adoption explicit at the context's table, while a namespace can share
the routes themselves. Which form is easier to maintain depends on whether naming each adopted
component is useful or has become repetition.

## What it costs

Centralizing choices gives readers another table to inspect when tracing a call. Nesting bundles
adds further tables to that trace, even though the compiler resolves the chain statically. The
shared choices need to justify this extra reading.

Repeated wiring across contexts provides a concrete reason to maintain a bundle. With only one
context, however, the bundle may add a name and a delegation step without reducing repetition.
Anticipated reuse alone may not be enough to justify it.

The bundle's role is a convention rather than a separate kind of declaration. Both a context and
a bundle can receive a delegation table, which is why the wrong check can treat the bundle as a
context. Naming it `GeometryComponents` helps readers recognize the distinction, but the checks
still need to use the intended context.

## Where to go next

These pages explain related wiring choices and checks:

- [Namespaces](./namespaces.md): Sharing tables through paths and inheritance.
- [Consumer and provider traits](./consumer-and-provider-traits.md): How delegation preserves the context.
- [Checking your wiring](./check-traits.md): Context checks and checks on individual providers.
- [`delegate_components!`](/docs/reference/macros/delegate_components): The `new` and grouped-key forms.
- [`check_components!`](/docs/reference/macros/check_components): The `#[check_providers]` form.
- [Comparison: Dynamic dispatch](/docs/comparisons/dynamic-dispatch): delegation through a bundle with `self` still bound to the context.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
