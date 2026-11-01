---
sidebar_label: 'Bypassing coherence'
sidebar_position: 1
---

# Bypassing coherence

Rust's coherence rules keep trait lookup consistent across a program. For a given trait, its
type arguments, and an implementing type, at most one implementation applies. Generic code can
therefore rely on trait bounds without asking callers to choose an implementation. The same
guarantee rules out overlapping implementations, even when each would be useful on its own.

CGP lets alternatives coexist by giving each implementation a separate provider type. A context
then selects its provider through wiring, and the generated code continues to obey Rust's
coherence rules. This page explains why that guarantee matters, how the provider split works
within it, and where ownership and wiring limit the available choices.

## The trait system is already a dependency-injection mechanism

Consider a function that formats a value. To request the formatting behavior it needs, the
function only has to name `Display` in a bound:

```rust
use core::fmt::{self, Display, Formatter};

pub fn describe<T: Display>(value: &T) -> String {
    format!("{value}")
}
```

Calling `describe(&42)` supplies the value, while the compiler finds the `Display` implementation
for `i32`. The caller never names or passes that implementation. This is a form of dependency
injection already available in ordinary Rust.

The selected implementation can have dependencies of its own. A pair, for example, can implement
`Display` by asking each of its fields to format itself:

```rust
struct Pair<A, B>(A, B);

impl<A: Display, B: Display> Display for Pair<A, B> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
```

When `describe(&Pair("foo", 42u32))` requires `Pair<&str, u32>: Display`, Rust finds the pair
implementation and follows its bounds to `&str: Display` and `u32: Display`. The `describe`
function still needs only `T: Display`, because the field requirements belong to the pair's
implementation.

That separation keeps implementation details out of generic callers. An implementation can gain
a dependency while callers retain their existing bounds, provided the concrete types still
satisfy the full chain.

## That only works because every lookup finds the same answer

As Rust follows these dependencies, each lookup must have a consistent answer wherever it occurs
in the program. Coherence provides that guarantee: for a given trait, its type arguments, and an
implementing type, there can be at most one applicable implementation.

Rust enforces this partly through the **overlap rule**, which rejects implementations that could
apply to the same type. For example, an encoding trait might offer one
[blanket implementation](/docs/reference/glossary#blanket-implementation) for printable values
and another for values that expose bytes:

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

Both bounds hold for `String`, so Rust would have two answers for `String: CanEncode`. Each
implementation is valid on its own, but a call site has no way to choose between them. The compiler
therefore rejects the pair.

Overlap can also arise when unrelated crates define implementations independently. The
**orphan rule** prevents those crates from supplying conflicting implementations for the same
foreign trait and type. For a trait without type parameters, either the trait or the implementing
type must be local to the crate defining the implementation. Since both `Display` and `Vec<u8>`
come from the standard library, the following implementation is rejected:

```rust
// error[E0117]: only traits defined in the current crate can be implemented
//               for types defined outside of the crate
impl Display for Vec<u8> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { /* ... */ }
}
```

A local type in a trait's parameters can allow further implementations under the orphan rule.
The [Rust Reference](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules)
specifies those cases. Together, the orphan and overlap rules keep trait resolution consistent
as crates are combined into a program.

## What the guarantee costs

For `CanEncode`, keeping one answer means choosing which of the overlapping blanket
implementations to offer. Several blanket implementations can coexist when Rust can establish
that they do not overlap, but the bounds above do overlap. Knowing which behavior an application
wants for each type does not give the trait a way to record those choices.

In ordinary Rust, a newtype can express a distinct choice by wrapping the value in a locally
defined type. That wrapper satisfies the orphan requirement for a foreign trait such as
`Display`, though its implementations must still avoid overlap. Its name can also make the
different behavior explicit. The cost is that callers must use the wrapper, and operations on
the underlying value may need forwarding methods or trait implementations.

Another way to express the choice is to name each implementation with a helper type. Each helper
implements a trait parameterized by the data type, so several helpers can accept the same data.
CGP builds on this pattern and adds wiring that connects ordinary method calls to the chosen
helper.

## The move: make `Self` a type you own

For encoding, the interface callers use can remain `CanEncode`. In CGP, this is the
**consumer trait**:

```rust
use cgp::prelude::*;

#[cgp_component(Encoder)]
pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}
```

The `#[cgp_component]` macro also generates a **provider trait**, `Encoder<Context>`. Moving the
value's type into the `Context` parameter leaves the implementing type available for a provider
struct. With a separate struct for each implementation, the original `Display` and `AsRef<[u8]>`
bounds can describe two providers. Here, the byte-based provider uses hexadecimal encoding, which
makes the difference between their outputs visible:

```rust
#[cgp_impl(new EncodeAsText)]
#[uses(Display)]
impl Encoder {
    fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
}

#[cgp_impl(new EncodeAsHex)]
#[uses(AsRef<[u8]>)]
impl Encoder {
    fn encode(&self) -> Vec<u8> {
        self.as_ref()
            .iter()
            .flat_map(|byte| format!("{byte:02x}").into_bytes())
            .collect()
    }
}
```

The `new` argument declares the `EncodeAsText` and `EncodeAsHex` structs. Inside `#[cgp_impl]`,
`&self` still refers to the value being encoded, and `#[uses(Display)]` requires that value's type
to implement `Display`. The macro turns the receiver into an explicit context argument in the
generated provider implementation.

The resulting Rust implementations have different `Self` types: `EncodeAsText` implements
`Encoder<Context>` where `Context: Display`, while `EncodeAsHex` implements it where
`Context: AsRef<[u8]>`. A context can satisfy both bounds without making the implementations
overlap, because each is implemented for a different provider struct.

Since `String` satisfies both bounds, both providers can encode the same value. Calling them
explicitly makes the two choices visible before wiring selects either one:

```rust
let value = String::from("hi");

assert_eq!(<EncodeAsText as Encoder<String>>::encode(&value), b"hi");
assert_eq!(<EncodeAsHex as Encoder<String>>::encode(&value), b"6869");
```

The provider struct also gives its crate a local implementing type. Because of that, the crate
can supply a provider even when both the trait and context type are defined elsewhere. Distinct
provider types avoid overlap, and local ownership satisfies the orphan rule, so the generated
code follows both coherence rules.

## Selecting one provider for each type {#coherence-comes-back-one-type-at-a-time}

Naming a provider at the call site makes the choice explicit, but ordinary `value.encode()` calls
still need a single implementation. A wiring table records that choice for each value type:

```rust
delegate_components! { String  { EncoderComponent: EncodeAsText } }
delegate_components! { Vec<u8> { EncoderComponent: EncodeAsHex  } }
```

With these entries, `value.encode()` uses `EncodeAsText` for a `String` and `EncodeAsHex` for a
`Vec<u8>`. The string `"hi"` therefore encodes as `b"hi"`, while the byte vector `b"hi".to_vec()`
encodes as `b"6869"`. Callers can use the `CanEncode` consumer trait and leave provider selection
to the wiring.

Ownership determines which crate can record that choice. Here, `EncoderComponent` is generated
in the crate defining `CanEncode`, so that crate can wire it onto foreign types such as `String`
and `Vec<u8>`. A downstream crate cannot independently wire that foreign key onto those foreign
types, and a second conflicting entry for a type and component remains an error. The result is
one choice per value type across the program.

Applications that need independent choices for the same data type can wire their own
[application contexts](/docs/reference/glossary#application-context), with the data passed as a
component parameter. [Modularity Hierarchy](./modularity-hierarchy.md) develops that arrangement
and explains when the additional choice is useful.

## What it costs

Separating implementations adds declarations: a provider trait, a
[component marker](/docs/reference/glossary#component-marker), and wiring for the types that use
it. When a trait only needs one implementation, a plain trait or
[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) may provide enough reuse on its own.

The wiring itself also needs to be read and maintained. In the example, every wired type needs
an entry naming its provider. Shared bundles and namespaces can reduce repetition, but
tracing a choice then means following it through those additional definitions.

Recording the choice leaves another question open: whether the provider's requirements hold for
the context. Rust checks those requirements when the trait is checked or used, so a table can
compile even though a later call will fail. To check beside the wiring, use
[`check_components!`](/docs/reference/macros/check_components). If the check fails,
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
though it does not yet reshape every class. [Checking your wiring](./check-traits.md) explains
those diagnostics and their limits.

The choices also apply specifically to the component interface you define. They do not give an
existing trait such as `Ord` multiple implementations or change which ordering standard collections
use. Ordinary traits remain a good fit when one consistent implementation per type expresses the
intended contract. The provider split becomes useful when you need the combination illustrated
here: several reusable implementations and an explicit way to select one for normal method calls.

## Where to go next

To decide how much implementation choice your code needs, start with the modularity hierarchy.
From there, these pages explain the trait split in more detail, show it in use, and compare it
with related approaches:

- [Modularity Hierarchy](./modularity-hierarchy.md): When a plain trait is enough and when further
  provider choices are useful.
- [Consumer and provider traits](./consumer-and-provider-traits.md): How a method call reaches its provider.
- [Hello World tutorial](/docs/tutorials/hello) and
  [Area calculation series](/docs/tutorials/area-calculation/): Working examples of the component model.
- [`#[cgp_component]`](/docs/reference/macros/cgp_component),
  [`#[cgp_impl]`](/docs/reference/macros/cgp_impl), and
  [`delegate_components!`](/docs/reference/macros/delegate_components): The constructs used here.
- [RustLab 2025 talk](/blog/rustlab-2025-coherence): A longer account of coherence and CGP's approach.
- [Comparison: Type classes](/docs/comparisons/type-classes): How Haskell, Agda, and Lean handle coherence.
- [Comparison: Rust's own proposals](/docs/comparisons/rust-language-proposals): Changes Rust has considered for its coherence rules.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
