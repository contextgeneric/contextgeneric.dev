---
sidebar_label: 'Modularity Hierarchy'
sidebar_position: 18
---

# Modularity Hierarchy

The modularity hierarchy describes how implementation choices grow from a single shared body to
providers selected independently by context, target type, and enclosing provider. Each tier adds a
place where a program can make a choice, along with the structure needed to express it. The useful
tier is the least complex one that supports the variation your program actually needs.

Encoding illustrates how those choices differ. The examples below omit imports from
`cgp::prelude::*`, context structs, and some provider bodies so the implementation choices remain
visible. The decision guide that follows compares those choices with ordinary Rust alternatives.

## The tiers at a glance

The tiers differ in where a program can select an implementation:

| Tier | Implementation choice | Construct | Main constraint |
| --- | --- | --- | --- |
| **1** | One shared implementation for types satisfying its bounds | A blanket impl or [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) | Matching types share the same logic |
| **2** | One implementation per target type | A plain Rust trait | Selection follows the trait and target type |
| **3** | One provider per wired type | A CGP component and wiring | Each wired type still has one choice for that component |
| **4** | One provider per target type in each context | A parameter-targeted component and `open` | The interface must separate context from target |
| **5** | An inner provider chosen by an enclosing provider | A [higher-order provider](/docs/reference/glossary#higher-order-provider) | The enclosing provider depends on the inner provider interface |

These choices all work within Rust's [coherence](/docs/reference/glossary#coherence) rules.
CGP places alternative implementations on distinct provider types, allowing wiring to select one
unambiguously. The final tier extends that selection into a provider composition, where a wrapper
can choose an inner provider independently of the context's general choice.

## Tier 1: one implementation for matching types

A [blanket implementation](/docs/reference/glossary#blanket-implementation) shares one body across every type satisfying its bounds. Here, all
byte-like values encode by copying their bytes:

```rust
pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}

impl<Value: AsRef<[u8]>> CanEncode for Value {
    fn encode(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }
}
```

Every type satisfying the bound receives this body, so another implementation for a covered type
would overlap it and be rejected. That is the constraint of this tier: matching types share the
logic. [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) can generate this arrangement from a function,
keeping its implementation bounds off the consumer interface.

When that shared body is the desired behavior, this tier needs nothing more. Provider names and
wiring become useful only when the program needs a further choice.

## Tier 2: one implementation per type

A plain trait allows different types to supply different bodies. With this non-generic interface,
each type still has one implementation:

```rust
pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}

impl CanEncode for u32 {
    fn encode(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl CanEncode for Vec<u8> {
    fn encode(&self) -> Vec<u8> {
        self.clone()
    }
}
```

Now `u32` uses decimal text and `Vec<u8>` copies its bytes. Each choice follows its type wherever
this trait is used. A blanket implementation could still cover other types, provided it does not
overlap these implementations or any others.

This tier fits behavior that varies by type but does not need an independent choice per application.
Rust's [orphan rules](/docs/reference/glossary#orphan-rule) also determine which crate may supply an implementation.

## Tier 3: reusable providers, one selected per wired type

A CGP component separates the consumer trait callers use from the provider trait implementations
supply. Distinct provider types can support the same target without their implementations
overlapping:

```rust
#[cgp_component(SelfEncoder)]
pub trait CanEncodeSelf {
    fn encode(&self) -> Vec<u8>;
}

#[cgp_impl(new EncodeSelfAsText)]
#[uses(core::fmt::Display)]
impl SelfEncoder {
    fn encode(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}
```

The `EncodeSelfAsText` provider can serve any context implementing `Display`. A separate
`EncodeSelfAsBytes` can support `AsRef<[u8]>` contexts, even when some contexts satisfy both bounds,
because the two implementations belong to distinct provider types. Wiring then chooses which one
a concrete type will use:

```rust
delegate_components! {
    u32 {
        SelfEncoderComponent: EncodeSelfAsText,
    }
}
```

Here, `u32` uses `EncodeSelfAsText`. The example defines the component locally, which permits this
wiring on a foreign value type. A downstream crate cannot independently rewire both a foreign
component and a foreign value type.

Check concrete wiring with [`check_components!`](/docs/reference/macros/check_components).
A table entry alone does not force Rust to verify all of the selected provider's dependencies.

### Value contexts and application contexts

In this example the wired type is the value being encoded. But a type can also represent an
application environment, carrying dependencies and implementation choices for its operations.
Both arrangements fit tier 3, even though “one choice per type” serves a different purpose in each.

In the **value-context** arrangement, `Self` is the data, as `u32` is in the encoding example.
The provider operates on that value, and wiring selects one provider for it. This arrangement suits
adding reusable behavior to a value while retaining a self-targeted interface.

In the **environmental-context** arrangement, `Self` carries an application's dependencies and
implementation choices. An operation such as sending email belongs to that context:

```rust
#[cgp_component(EmailSender)]
pub trait CanSendEmail {
    fn send_email(&self, to: &str, body: &str);
}

#[cgp_impl(new SendViaSmtp)]
impl EmailSender { /* connect and send over SMTP */ }

#[cgp_impl(new RecordEmails)]
impl EmailSender { /* record to a Vec a test can read */ }

delegate_components! { App     { EmailSenderComponent: SendViaSmtp } }
delegate_components! { TestApp { EmailSenderComponent: RecordEmails } }
```

The omitted SMTP body would need connection settings or a client, while the recording body would
need storage for captured messages. `App` and `TestApp` supply the appropriate dependencies. Their
separate wiring lets callers use the same consumer trait in either environment.

The application owns both context types, so it can wire them independently even though the
consumer trait has no target parameter. This makes tier 3 sufficient for many service operations
and test environments: the selected behavior belongs to the application context itself.

Components may group methods, associated types, and consts, just as ordinary traits do. The examples
use one method to isolate provider selection; the appropriate grouping depends on which items a
provider should implement and reuse together.

## Tier 4: one provider per target type per context

A parameter-targeted component separates the application context from the value it operates on.
`Self` selects the implementation, while `Value` identifies the target:

```rust
#[cgp_component(Encoder)]
pub trait CanEncodeValue<Value> {
    fn encode(&self, value: &Value) -> Vec<u8>;
}
```

Each application can now choose how the same foreign type is encoded. Assume `EncodeAsText`
supports `Display` values and `EncodeAsHex` supports `AsRef<[u8]>` values:

```rust
delegate_components! {
    ApiServer {
        open EncoderComponent;
        @EncoderComponent.String: EncodeAsText,
    }
}

delegate_components! {
    Firmware {
        open EncoderComponent;
        @EncoderComponent.String: EncodeAsHex,
    }
}
```

`ApiServer` encodes a `String` as text, while `Firmware` encodes its bytes as hexadecimal. A crate
owning either context can supply that wiring even when it owns neither the component nor `String`.
Ownership of the context satisfies Rust's rules for the wiring implementation.

Making the value a parameter is what enables these independent application choices. A self-targeted
trait cannot gain that separation through wiring alone, because its interface still treats the
context as the value. With the parameter-targeted interface, each context must cover the target
types it uses, although generic entries can supply providers for whole families of targets.

## Tier 5: an inner provider chosen locally

A higher-order provider can select an inner implementation independently of the context's default
choice. This vector encoder accepts an element encoder as a type parameter:

```rust
pub struct EncodeVecWith<Inner = UseContext>(pub PhantomData<Inner>);

#[cgp_impl(EncodeVecWith<Inner>)]
#[use_provider(Inner: Encoder<Item>)]
impl<Item, Inner> Encoder<Vec<Item>> {
    fn encode(&self, value: &Vec<Item>) -> Vec<u8> {
        value.iter().flat_map(|item| Inner::encode(self, item)).collect()
    }
}
```

`Inner = UseContext` makes element encoding use the context's ordinary selection unless wiring
supplies another inner provider. A context can choose both forms:

```rust
delegate_components! {
    App {
        open EncoderComponent;
        @EncoderComponent.u32: EncodeAsText,
        @EncoderComponent.Vec<u32>: EncodeVecWith,
        @EncoderComponent.Vec<Vec<u8>>: EncodeVecWith<EncodeAsHex>,
    }
}
```

`Vec<u32>` uses the default, so each element goes through the context's `u32` route to
`EncodeAsText`. `Vec<Vec<u8>>` explicitly selects `EncodeAsHex` for each inner byte vector,
regardless of how the context encodes `Vec<u8>` elsewhere.

Here the collection specifies how its elements should be interpreted while other operations keep
the context's general choice. Expressing that local decision requires another provider parameter
and an interface dependency, so it earns its place when the wrapper needs that independent choice.

## Choosing a tier

Start with a plain trait or blanket implementation and add provider selection where there is a
concrete need for reuse or alternatives. A consumer trait can also be implemented directly on a
context; using CGP does not require turning every implementation into a provider.

When providers are useful, identify what the context represents and whether the operation targets
another type. These arrangements answer different needs:

| Need | Arrangement | Selection scope |
| --- | --- | --- |
| Reusable behavior on the value itself | Value context, self-targeted component (tier 3) | One choice per wired value type |
| Application operations with selectable dependencies | Environmental context, self-targeted component (tier 3) | One choice per application context |
| Behavior for a target type that differs by application | Environmental context, parameter-targeted component (tier 4) | One choice per context and target type |
| A wrapper needs a particular inner implementation | Higher-order provider (tier 5) | A choice within that provider composition |

Testing alone does not require moving to tier 4. At tier 3, `App` and `TestApp` can already select
different providers for an operation such as sending email. A target parameter earns its place
when the operation acts on a separate value whose treatment must vary between applications,
independently of that value's own trait implementations.

Whichever tier supplies the needed choices, several contexts may repeat part of their wiring.
[Aggregate providers](./aggregate-providers.md) and [namespaces](./namespaces.md) can collect those
selections once the shared table is useful to maintain as a unit. Sharing a table does not itself
require moving to another tier.

## Comparing ordinary alternatives

Choose the tool that supplies the variation you actually need. These alternatives remain useful
alongside CGP:

| Alternative | Prefer it when | Consider CGP when |
| --- | --- | --- |
| **Plain traits and generics** | One shared implementation or one implementation per type is sufficient. | Reusable alternatives must coexist, or implementation dependencies should stay out of intermediate interfaces. |
| **A direct impl on the context** | The body is specific to that context. | Another context can reuse the body, or a provider wrapper needs to compose with it. |
| **An enum and `match`** | The alternatives are a small, closed set with fixed operations. | Independent modules need to contribute handlers reused across different variant sets. |
| **`dyn Trait`** | Implementations must be selected at runtime or stored behind a common runtime interface. | Provider selection can be fixed at compilation. |
| **A dependency-injection library** | Its object construction, lifecycle, or configuration model matches the application. | Rust traits and compile-time wiring express the needed dependency choices. |
| **A focused generic-programming library** | Structural operations on heterogeneous data are the main requirement. | Those operations need to compose with a broader provider-and-wiring design. |
| **A local macro** | The generated pattern is narrow and specific to the crate. | Repeated helper traits and marker types amount to a reusable component system. |

Runtime choices can live inside a CGP context. For example, a provider can call through a trait
object stored in a field. Static wiring selects that provider; the [trait object](/docs/reference/glossary#trait-object) still performs
its own runtime dispatch.

## Where provider wiring does not help

Static wiring cannot load previously unknown implementations at runtime. A plugin system or
runtime-open collection needs an appropriate dynamic interface. CGP can participate in such a
program, but its wiring does not supply that runtime extensibility.

An operation with one suitable implementation does not need provider selection.
A plain impl, blanket impl, or [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) can express it directly.
Likewise, a small closed enum usually needs only a `match` rather than generic variant dispatch.

For some APIs, variation would weaken a contract the program relies on. A map, for example,
needs a consistent ordering across its operations. Independent provider choices help only when
the surrounding design preserves that invariant, so a coherent trait that already expresses it
may remain the better fit.

## Evaluating a single-context application

One context does not demonstrate the benefit of selecting different providers across contexts.
It may still have needs that provider-based code addresses:

- **Alternative implementations for target types:** Distinct providers can coexist where ordinary
  blanket impls would overlap.
- **Behavior for foreign targets:** A locally owned context can select a provider for a foreign
  component and target type.
- **Implementation-specific dependencies:** A provider's bounds state which operations and values
  its body may use without adding those requirements to the consumer interface.

A test environment may provide another concrete use for separate choices, but it should justify
its own context. The possibility of future variation alone does not justify present wiring.
If these needs are absent, use a simpler implementation form.

## Representing configuration choices

Configuration does not require a handwritten context for every combination of settings.
A generic context can share its structure across backend types, although each concrete instantiation
is still a distinct Rust type. If a choice comes from runtime configuration instead, an enum or
trait object can represent it inside the context.

A distinct context type is useful when Rust should enforce the distinction, such as preventing
a test client from being substituted for a production client. Other settings can stay in fields
or generic parameters, according to how the application needs to choose and share them.

Context-generic libraries can also leave backend representation to downstream applications.
When providers depend on context traits rather than a fixed backend enum, a downstream crate can
define its own context and backend representation. It can reuse the providers whose requirements
that context satisfies without changing the upstream enum.

## What it costs

Reading a wired method call requires finding the provider in the context's table as well as the
consumer trait. The component generates that trait, its provider counterpart, and a wiring key;
wrappers and namespaces can add more tables or types to follow. Those steps disappear from runtime
selection, but remain part of understanding the code.

Wiring failures can produce verbose diagnostics involving generated types and transitive bounds.
[`check_components!`](/docs/reference/macros/check_components) forces selected requirements to be
checked near the wiring. `cargo cgp check` leads with the root cause for the classes it recognizes,
and the tool does not yet reshape every class. The
[cargo-cgp documentation](/docs/cargo-cgp/) explains the checking workflow.

The compiler does additional work through macros, trait resolution, and
[monomorphization](/docs/reference/glossary#monomorphization). Its effect needs measuring in the
program being evaluated. Although static dispatch removes runtime provider lookup, compilation
costs do not necessarily correspond to runtime work that the program would otherwise have done.

The vocabulary and generated code take time to learn. CGP's
[agent skill](https://github.com/contextgeneric/cgp-skills) can help a coding assistant explain
providers, trace wiring, and diagnose missing requirements. Reviewers still need to understand the
result, and assistance does not make an unnecessary abstraction worthwhile.

## Where to go next

These pages explain the mechanisms and develop working examples:

- [Bypassing coherence](./coherence.md): Why Rust requires unambiguous implementations and how
  distinct providers allow alternatives.
- [Consumer and provider traits](./consumer-and-provider-traits.md): How calls reach selected
  implementations.
- [Higher-order providers](./higher-order-providers.md): Inner provider choices and composition.
- [`#[cgp_fn]`](/docs/reference/macros/cgp_fn): A single implementation generated from a function.
- [Area calculation](/docs/tutorials/area-calculation/): A tutorial progressing from functions to
  components and composition.
- [`#[cgp_component]`](/docs/reference/macros/cgp_component): Component definitions and generated traits.
- [Comparison: Type classes](/docs/comparisons/type-classes): How context-based selection changes
  the scope of implementation choices.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
