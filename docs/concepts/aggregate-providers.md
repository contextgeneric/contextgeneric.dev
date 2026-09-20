---
sidebar_label: 'Aggregate providers'
sidebar_position: 8
---

# Aggregate providers

An **aggregate provider** bundles wiring choices so several contexts can reuse them. Each context
names the bundle for the components it needs, and the bundled providers still operate on that context.
This page explains how delegation preserves the context, where to check the result, and when a
namespace is a better fit.

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

`GeometryComponents` groups the choices of `RectangleArea` and `RectanglePerimeter` under one name.
It is a zero-sized marker used for delegation; the program does not construct it or store data in it.
The context remains the type whose data those providers use.

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

`Rectangle` gets both traits if it meets the bundled providers' requirements. The provider
choices live in `GeometryComponents`, so changing that table changes the choices for every context
that delegates those components to it.

## The context stays the context

Delegation through a bundle preserves the original context. The compiler resolves
`rectangle.area()` through this chain:

1. `Rectangle` delegates `AreaCalculatorComponent` to `GeometryComponents`.
2. `GeometryComponents` delegates the same component to `RectangleArea`.
3. `RectangleArea` calculates the area using `Rectangle` as its context.

When `RectangleArea` reads `width` and `height`, it reads them from `Rectangle`. The bundle selects
the implementation but does not supply the field values. Another context can reuse the bundle if it
satisfies the same provider requirements.

Bundles can delegate to other bundles using the same mechanism. The compiler follows each table
while keeping the context argument fixed. It resolves the entire chain at compile time, without a
runtime lookup for each table.

## Checking a bundle asks the wrong question

Checking a bundle as a context does not verify that an application can use it. In particular,
[`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components) checks the
type receiving the table. Applied to `GeometryComponents`, it asks whether the bundle itself can
supply everything its providers require.

The result depends on those requirements:

- A provider without context dependencies accepts `GeometryComponents` as its context, so the check
  passes without checking any application that will use the bundle.
- A provider that needs `width` fails because `GeometryComponents` lacks that field, even if
  `Rectangle` supplies it correctly.

Define bundles with plain [`delegate_components!`](/docs/reference/macros/delegate_components).
Verify their requirements against a context that will actually use them.

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

This verifies the full chain, including nested bundles. A missing requirement fails at the check
site even if the provider requiring it is several tables away.

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

Each named provider gets its own assertion for `Rectangle`. The bundle is checked in the role it
serves: supplying an implementation for that context.

## When a namespace is the better tool

A bundle suits a small group whose components you want to name explicitly. The entry
`[A, B]: TheBundle` shows which choices the context adopts. As the group grows, each context must
still list every component it takes from the bundle.

A [namespace](./namespaces.md) lets a context inherit shared wiring by joining a named table.
It also organizes lookups under paths, which helps when a library supplies wiring for many components.
Customization requires the namespace to leave the relevant paths unbound; a direct context entry
cannot override a key the inherited namespace already binds.

Use a bundle for explicit delegation of a group of components. Consider a namespace when many
contexts need a shared table and a path structure that separates fixed wiring from application choices.

## What it costs

A bundle adds a table to inspect when tracing a method call. Nested bundles add further tables, even
though the compiler resolves them statically. Keep a bundle when centralizing shared choices is worth
that extra reading.

A bundle used by only one context may add little value. Repeated wiring across contexts gives a
clear reason to introduce one; anticipated reuse alone may not justify it.

The type system does not distinguish a bundle from a context by declaration. Both can receive a
delegation table, so the author must check the bundle against its intended context. A name such as
`GeometryComponents` helps identify its role, but does not enforce it.

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
