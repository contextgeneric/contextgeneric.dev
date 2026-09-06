---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Wiring and checking

The three traits every context's wiring rests on, and the ones a wiring error names.

## Overview

Wiring a **context**, the type a capability runs against, records which provider it uses for each
component and checks that the provider's own requirements are met. These traits carry that, and you
read them far more often than you write them, because a
[`delegate_components!`](../../macros/delegate_components.md) table generates the first two and a
[`check_components!`](../../macros/check_components.md) block asserts the third.

They build on one another. A context becomes a type-level table through
[`DelegateComponent`](delegate_component.md), one entry per component. Each entry also carries an
[`IsProviderFor`](is_provider_for.md) impl that re-exposes the chosen provider's `where` bounds, so a
missing dependency is named rather than hidden. [`CanUseComponent`](can_use_component.md) combines the
two into a single question that a check asserts: does the context delegate this component, and do its
provider's requirements hold?

- [`DelegateComponent`](delegate_component.md): the per-context table mapping a component to its
  provider; what a wiring line becomes.
- [`IsProviderFor`](is_provider_for.md): the marker that surfaces a provider's missing dependency by
  name, and the trait a `#[check_providers]` assertion names.
- [`CanUseComponent`](can_use_component.md): the context-side check a `check_components!` block
  reduces to, holding only when a component is both delegated and satisfiable.

## The ideas behind them

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): the split this wiring
  connects.
- [Bypassing coherence](/docs/concepts/coherence): why the choice is recorded per context.
- [Check traits](/docs/concepts/check-traits): why wiring is lazy, and how an assertion makes its
  failures readable.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
