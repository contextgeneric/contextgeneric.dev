---
sidebar_label: 'Bypassing coherence'
sidebar_position: 1
---

# Bypassing coherence

Rust's coherence rules keep trait implementation selection unambiguous. CGP works within those rules
by giving alternative implementations separate provider types, then letting each context select one.
This page explains what coherence provides, why overlapping blanket implementations are rejected,
and how the provider arrangement allows reuse without ambiguous calls.

## The trait system is already a dependency-injection mechanism

Rust resolves trait dependencies for generic code at compile time. A function can require an
implementation through a bound without asking its caller to pass that implementation explicitly:

```rust
pub fn describe<T: Display>(value: &T) -> String {
    format!("{value}")
}
```

Calling `describe(&42)` makes the compiler resolve `i32: Display`. The caller supplies the value;
the type system determines which formatting implementation the function uses. This serves as a form
of dependency injection within ordinary Rust.

Trait resolution also follows an implementation's dependencies. A pair can implement `Display`
whenever both of its fields implement it:

```rust
struct Pair<A, B>(A, B);

impl<A: Display, B: Display> Display for Pair<A, B> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
```

Calling `describe(&Pair("foo", 42u32))` requires `Pair<&str, u32>: Display`, which in turn requires
`&str: Display` and `u32: Display`. The caller need not list those transitive requirements separately.
If an implementation acquires another dependency, generic callers can retain their existing bounds,
provided the concrete types still satisfy the full chain.

## That only works because every lookup finds the same answer

Coherence gives trait resolution a consistent answer across a program. For a given trait, including
its type arguments, and an implementing type, Rust permits at most one applicable implementation.
Generic code can therefore rely on the same implementation wherever it is called.

Rust enforces coherence through overlap and orphan rules. The **overlap rule** rejects implementations
that could apply to the same type. These blanket implementations each cover every type satisfying
their bound, so they conflict:

```rust
pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}

// Legal on its own.
impl<T: Display> CanEncode for T {
    fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
}

// error[E0119]: conflicting implementations of trait `CanEncode`
impl<T: AsRef<[u8]>> CanEncode for T {
    fn encode(&self) -> Vec<u8> { self.as_ref().to_vec() }
}
```

`String` satisfies both bounds. Both implementations would apply to `String: CanEncode`, and Rust
does not let a call site select one of them. Each blanket implementation is valid alone; defining
both for the same trait causes the conflict.

The **orphan rule** restricts implementations involving foreign traits and types. For a trait without
type parameters, your crate must define either the trait or the implementing type. Neither `Display`
nor `Vec<u8>` belongs to this crate, so Rust rejects this implementation:

```rust
// error[E0117]: only traits defined in the current crate can be implemented
//               for types defined outside of the crate
impl Display for Vec<u8> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { /* ... */ }
}
```

This restriction prevents unrelated crates from independently defining conflicting implementations
for the same foreign trait and type. Traits with type parameters have additional cases involving a
local type in those parameters; the
[Rust Reference](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules)
specifies the full rule.

Together, these rules let crates compose without making implementation selection depend on which
crate asks. CGP retains that guarantee while changing how alternative implementations are represented.

## What the guarantee costs

The overlap rule prevents the encoding trait from carrying both blanket implementations above.
Even if an application author knows which behavior each type should use, the trait system does not
provide a selection table for those conflicting implementations. Multiple blanket implementations
are allowed when Rust can establish that they do not overlap.

A newtype provides an ordinary Rust way to choose a distinct implementation for a foreign type.
The wrapper is local, so it can implement a foreign trait such as `Display`. It also makes the
semantic distinction explicit. Its cost is that callers must use the wrapper, and operations on the
underlying type may need forwarding methods or trait implementations.

Named helper types provide another way to represent implementation choices. Each helper implements
a trait parameterized by the data type, so the helpers can coexist. CGP uses this pattern and adds
wiring that connects a type's normal method calls to its chosen helper.

## The move: make `Self` a type you own

CGP separates the caller's interface from the provider's implementation interface. The
**consumer trait** keeps the method callers use:

```rust
#[cgp_component(Encoder)]
pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}
```

`#[cgp_component]` also generates a **provider trait**, `Encoder<Context>`. The value being encoded
becomes its `Context` parameter, while each provider supplies a distinct implementing type:

```rust
#[cgp_impl(new EncodeAsText)]
#[uses(Display)]
impl Encoder {
    fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
}

#[cgp_impl(new EncodeAsHex)]
#[uses(AsRef<[u8]>)]
impl Encoder {
    fn encode(&self) -> Vec<u8> { /* hex-encode self.as_ref() */ }
}
```

`EncodeAsText` and `EncodeAsHex` can both implement `Encoder<Context>` because their implementing
types differ. Their context requirements may overlap without the Rust implementations overlapping.
A crate can also implement a foreign provider trait for its own provider type, even when the context
comes from another crate. The generated implementations still satisfy Rust's coherence rules.

`#[cgp_impl]` lets provider code retain a familiar `self` receiver. Within that macro's input,
`self` refers to the value being encoded, and `#[uses(Display)]` requires that value's type to
implement `Display`. The macro translates this into a provider implementation with an explicit
context parameter.

## Coherence comes back, one type at a time

Each type selects one provider for the component through a wiring table:

```rust
delegate_components! { String  { EncoderComponent: EncodeAsText } }
delegate_components! { Vec<u8> { EncoderComponent: EncodeAsHex  } }
```

`String` implements `CanEncode` through `EncodeAsText`, while `Vec<u8>` uses `EncodeAsHex`.
A call to `value.encode()` resolves to the provider selected for that value's type. Both providers
remain available, but each wired type has an unambiguous choice.

The component key's ownership still matters when wiring foreign types. Here `EncoderComponent` is
generated in the crate defining `CanEncode`, so that crate can wire it onto `String` and `Vec<u8>`.
A downstream crate cannot independently wire that foreign key onto those foreign types. This example
therefore makes one choice per value type across the program.

CGP preserves coherence while making provider selection explicit. Alternative providers can apply
to the same context, and the wiring chooses which one supplies the consumer trait. Defining a second
conflicting entry for the same type and component remains an error.

Separate application contexts allow independent choices for the same data type. That arrangement
requires moving the data into a component parameter; [Modularity Hierarchy](./modularity-hierarchy.md)
explains when to use it and how it differs from the value-type wiring shown here.

## What it costs

A component introduces more declarations than a plain trait: a provider trait, a component marker,
and wiring for the types that use it. For a trait with one implementation, a plain trait or
[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) may provide the required reuse without wiring.

Provider selection belongs to CGP's component interface. It does not give an existing trait such as
`Ord` multiple implementations, nor change how standard collections select that trait's behavior.
Use ordinary traits when one consistent implementation per type expresses the intended contract.

Explicit wiring adds configuration to read and maintain. In the tables above, each type needs an
entry naming its provider. Shared bundles and namespaces can reduce repetition, but they introduce
further places to inspect when tracing a choice.

Wiring checks are deferred until the trait is checked or used. A table can name a provider
whose requirements the context does not satisfy, and a later call can produce a verbose error.
[`check_components!`](/docs/reference/macros/check_components) verifies requirements beside the
wiring. [`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) makes recognized causes
easier to read; its `v0.1.0-alpha` release covers core wiring errors rather than every class.
[Checking your wiring](./check-traits.md) shows the diagnostics and their limits.

## Where to go next

Choose the next page according to whether you want to evaluate, understand, or use the arrangement:

- [Modularity Hierarchy](./modularity-hierarchy.md): When a plain trait is enough and when further
  provider choices are useful.
- [Consumer and provider traits](./consumer-and-provider-traits.md): How a method call reaches its provider.
- [Hello World tutorial](/docs/tutorials/hello) and
  [Area calculation series](/docs/tutorials/area-calculation/): Working examples of the component model.
- [`#[cgp_component]`](/docs/reference/macros/cgp_component),
  [`#[cgp_impl]`](/docs/reference/macros/cgp_impl), and
  [`delegate_components!`](/docs/reference/macros/delegate_components): The constructs used here.
- [RustLab 2025 talk](/blog/rustlab-2025-coherence): A longer account of coherence and CGP's approach.
- [Comparison: Type classes](/docs/comparisons/type-classes): the same coherence trade seen from Haskell, Agda, and Lean.
- [Comparison: Rust's own proposals](/docs/comparisons/rust-language-proposals): what Rust itself has considered doing about its coherence rules.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
