---
sidebar_label: 'DelegateComponent'
---

# `DelegateComponent`

The per-context type-level table mapping a component key to its provider.

## What it's for

Wiring a **context** — the type the capability runs against, which supplies the values it needs as its
fields — means recording which implementation it uses for each capability. `DelegateComponent` is the trait
that record is made of. One impl is one entry:

```rust
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetHello;
}
```

Read that as *"in `App`'s table, the entry at `GreeterComponent` is `GreetHello`."* The key is a type, the
value is a type, and the whole lookup happens while the compiler is type-checking — which is why the
"table" costs nothing at runtime and compiles down to a direct call.

**You will almost never write this by hand.** [`delegate_components!`](../macros/delegate_components.md)
generates one impl per line of its body, and that macro is what you write. What makes the trait worth a
page is that it appears by name in compiler errors, and that reading it is how you understand what wiring
*is*: not a registry, not a lookup at startup, but a set of trait impls the compiler resolves.

One thing to notice early, because it explains why the trait is so general: **the key does not have to be a
component name.** When it is, the entry makes the context inherit the component's provider trait. When it
is any other type — a shape, a tag, a path segment — the same trait is just a lookup table that a provider
walks. One trait covers both, which is why it underlies both ordinary wiring and the inner dispatch tables.

## Using it

The trait is in the prelude, so `use cgp::prelude::*;` is enough to name it. It carries one associated type
and no methods:

```rust
pub trait DelegateComponent<Key: ?Sized> {
    type Delegate;
}
```

`Self` is the table — the context or the provider bundle that owns the entry. `Key` is what is being looked
up, and it is `?Sized` so that an unsized marker can serve as one. `Delegate` is the value stored there.
There is nothing else: no method, no data, no runtime representation.

### Setting an entry, and reading one

There are only two operations, and both are ordinary Rust rather than anything CGP-specific.

**Setting** an entry is writing the impl, which in practice means writing a
[`delegate_components!`](../macros/delegate_components.md) line. **Reading** one is bounding on the trait
and projecting the associated type:

```rust
fn provider_for<Table, Key>() -> PhantomData<<Table as DelegateComponent<Key>>::Delegate>
where
    Table: DelegateComponent<Key>,
{
    PhantomData
}
```

The bound is the "get"; `<Table as DelegateComponent<Key>>::Delegate` is the value it returns. Because Rust
forbids two impls of one trait for the same `Self` and `Key`, each key maps to exactly one value — which is
what makes this a map rather than a relation, and what makes a duplicate wiring entry a coherence error
rather than a silent overwrite.

### What can be a key

The key's type decides what the entry *means*, and there are three shapes in practice.

A **component name** — `GreeterComponent`, `AreaCalculatorComponent` — is the ordinary case. An entry keyed
this way is read by the blanket impl that [`#[cgp_component]`](../macros/cgp_component.md) generates, which
concludes that the context has the component's provider trait, and from there its consumer trait.

An **arbitrary type** — a shape, a tag — makes the table a plain dispatch map with no provider trait
attached. This is what the nested tables inside [`UseDelegate`](../providers/use_delegate.md) are, and what
the `open` statement's per-key entries resolve through.

A **type-level path** — a [`PathCons`](../types/type_level_spines.md) list built by
[`Path!`](../macros/path.md) — is how namespaces key their entries, so a lookup walks one segment at a time.
[`RedirectLookup`](../providers/redirect_lookup.md) is the provider that performs that walk.

### What owns a table

`Self` is not always a context. A [`delegate_components!`](../macros/delegate_components.md) block with a
leading `new` declares a provider bundle that owns its own table, so several contexts can delegate a whole
group of components to it as one unit. Resolution is shallow — one read yields the immediate `Delegate` —
and chaining happens when that delegate is itself a table, which is exactly what a bundle is.

## Examples

The single wired component is the whole idea. Given a component and a provider:

```rust
use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {
        println!("Hello!");
    }
}
```

a context wires it in one line:

```rust
pub struct App;

delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}
```

and that line is exactly one `DelegateComponent` impl:

```rust
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetHello;
}
```

From there the generated blanket impls take over: because `App` delegates `GreeterComponent`, it gets the
`Greeter` provider trait, and because it has that, it gets `CanGreet` — so `app.greet()` compiles. Swap
`GreetHello` for another provider and only that one line changes.

An arbitrary-key table looks identical and means something different. The nested-table syntax builds a
dispatch map keyed on shapes:

```rust
delegate_components! {
    new AreaComponents {
        Rectangle: RectangleArea,
        Circle: CircleArea,
    }
}
```

`AreaComponents` now has `DelegateComponent<Rectangle>` and `DelegateComponent<Circle>` entries. No provider
trait attaches to either; they are data that a [`UseDelegate`](../providers/use_delegate.md) provider reads
at dispatch time to pick `RectangleArea` or `CircleArea` for the shape it was asked about.

## When to reach for it, and when not

**Write [`delegate_components!`](../macros/delegate_components.md), not `DelegateComponent`.** That is the
honest summary of this page: the trait is what you read, and the macro is what you write. The macro also
emits the [`IsProviderFor`](./is_provider_for.md) forwarding impl beside each entry, which is what keeps a
missing dependency diagnosable — so a hand-written `DelegateComponent` impl gets you the wiring and loses
the error message.

There are two narrow reasons to name the trait yourself.

- **Reading an entry in generic code.** A provider that must resolve a key itself — a dispatcher, a
  higher-order provider walking a table — bounds on `DelegateComponent<Key>` and projects `Delegate`. This
  is the one legitimate hand-written use, and it is a *read*.
- **Recognizing it in an error.** `the trait bound App: DelegateComponent<GreeterComponent> is not
  satisfied` means the context never wired that component. That is the most common wiring error there is,
  and knowing the trait is how you read it.

And the alternative to reach for instead: **implement the consumer trait directly on the context** when a
capability has exactly one implementation for it. A CGP consumer trait is an ordinary trait, so
`impl CanGreet for App { … }` is legal and needs no table entry at all. Wiring earns its keep when the
implementation is one of several, or is shared, or should compose with a wrapper.

## Under the hood

:::note

### Advanced

This section shows how the generated code reads the table. You do not need it to wire a context, but the
trait's name appears in wiring errors and the routing explains why a call resolves the way it does.
`cargo cgp expand` prints the impls for your own code.

:::

The trait is the plainest thing in CGP — one associated type, no method — so the interesting part is what
reads it. The provider blanket impl generated by [`#[cgp_component]`](../macros/cgp_component.md) is the
reader, and its body is itself a table lookup: it bounds the provider on `DelegateComponent<FooComponent>`
and forwards each method to `<Provider as DelegateComponent<FooComponent>>::Delegate`. So the "routing" is
literally the projection, and the call it forwards to is resolved statically and inlined.

The trait carries a diagnostic attribute that shapes the error when a lookup misses:

```rust
#[diagnostic::on_unimplemented(
    message = "{Self} does not contain any DelegateComponent entry for {Key}",
    note = "You might want to implement the provider trait for {Key} on {Self}"
)]
pub trait DelegateComponent<Key: ?Sized> {
    type Delegate;
}
```

That message is why an unwired component reports as a missing *entry* rather than as an opaque unimplemented
trait — one of the few places CGP can improve a diagnostic without a tool.

Two further facts about the generated impls are worth carrying away. Each entry is emitted **alongside** an
[`IsProviderFor`](./is_provider_for.md) impl that forwards the chosen provider's requirements, so the entry
is both resolvable and checkable; the pair is what
[`CanUseComponent`](./can_use_component.md) then combines into a single assertion. And a namespace's
`namespace N;` header emits not one entry but a *blanket* `DelegateComponent` impl that forwards every key
through the namespace, which is how a context inherits a whole table while still letting a direct entry
shadow one key.

## Gotchas

**A missing entry and an unsatisfied provider are different errors.** `DelegateComponent` not implemented
means the component was never wired. A wired component whose provider's dependencies are unmet fails
elsewhere, through [`IsProviderFor`](./is_provider_for.md). Reading which one you have tells you whether to
add a wiring line or supply a dependency.

**Two entries for the same key is a coherence error, not an override.** Wiring one component twice on one
context reports `E0119` conflicting implementations. A namespace *can* be shadowed by a direct entry,
because the namespace side is a blanket impl rather than a second concrete one — but two concrete entries
always conflict.

**`Self` is not always a context.** A provider bundle declared with `new` owns a table too, and reading an
error that names one as the table's `Self` does not mean something is being used as a context.

**A hand-written impl loses the diagnostics.** It wires correctly and silently gives up the `IsProviderFor`
forwarding that makes a missing transitive dependency readable. Use the macro.

**The table is resolved at compile time, in full.** There is no runtime map to inspect, no way to add an
entry dynamically, and no cost per call. If you are looking for a place to register something at startup,
this is not it — and that is the point.

## Related constructs

- [`delegate_components!`](../macros/delegate_components.md) — the macro that generates these impls; what
  you write instead of the trait.
- [`#[cgp_component]`](../macros/cgp_component.md) — generates the blanket impl that reads the table.
- [`IsProviderFor`](./is_provider_for.md) — emitted beside each entry so a missing dependency stays
  readable.
- [`CanUseComponent`](./can_use_component.md) — combines a delegation and a valid provider into one
  assertion.
- [`check_components!`](../macros/check_components.md) — asserts that combination.
- [`UseDelegate`](../providers/use_delegate.md) — reads an arbitrary-key table for per-type dispatch.
- [`RedirectLookup`](../providers/redirect_lookup.md) — walks path-keyed entries; the namespace mechanism.
- [`DefaultNamespace`](./default_namespace.md) — the lookup traits a `namespace` header forwards through.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the split this table
  connects.
- [Bypassing coherence](/docs/concepts/coherence) — why the choice is recorded per context at all.
- [Aggregate providers](/docs/concepts/aggregate-providers) — a table owned by a provider rather than a
  context.

## Source

- Trait: [`delegate_component.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/traits/delegate_component.rs)
- Entry codegen: [`delegate_component/mapping/eval.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/delegate_component/mapping/eval.rs)
- The blanket impl that reads it: [`cgp_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_component)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
