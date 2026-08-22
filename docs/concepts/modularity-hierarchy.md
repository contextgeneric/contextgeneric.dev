---
sidebar_label: 'Modularity Hierarchy'
sidebar_position: 18
---

# Modularity Hierarchy

How much CGP does a capability need? The answer is a tier in a five-tier hierarchy, where each tier
allows more independent implementations of one interface than the tier below, at a matching cost in
syntax or coupling. The right tier is the lowest one that still expresses the problem.

This page explains each tier in turn on one running capability, encoding a value, so the only thing
that changes from tier to tier is the modularity, not the problem. It then turns to the decision: how
to pick a tier, when a plainer tool is the better choice, and what the machinery costs. If you are
still deciding whether CGP is for you at all, read to the end of the decision guide.

## The five tiers at a glance

The table is the summary; the sections below explain each tier in full.

| Tier | What it allows | The construct | The coherence limit it escapes |
|---|---|---|---|
| **1** | One implementation for every type | a blanket impl, or [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) | none: one implementation only |
| **2** | One implementation per type | a plain Rust trait | none: one implementation per type |
| **3** | Many implementations, one wired per type | [`#[cgp_component]`](/docs/reference/macros/cgp_component) and [`delegate_components!`](/docs/reference/macros/delegate_components) | the overlap rule |
| **4** | Many implementations, one per type per context | a type parameter and the [`open` statement](/docs/reference/macros/delegate_components) | the orphan rule as well |
| **5** | Many implementations, per type per provider | a [higher-order provider](./higher-order-providers.md) | none new: a local override within tier 4 |

Tiers 1 and 2 are ordinary Rust. Tiers 3 through 5 are CGP, and each one loosens a coherence
constraint that the tier below still obeys.

## Tier 1: one implementation for every type

The lowest tier is a single implementation that applies to every type meeting a bound. A blanket
trait implementation captures one piece of logic and offers no alternative to it.

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

Every type that is `AsRef<[u8]>` now encodes the same way, and no second strategy is possible.
[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) builds this same tier from a plain function, and hides
the bound behind a clean interface. Reach for tier 1 when a capability genuinely has one
implementation for all types.

## Tier 2: one implementation per type

A plain Rust trait raises the ceiling to one implementation per type. Each type supplies its own
body, and coherence still allows only one.

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

Each type varies, but the choice is global: once `u32` encodes one way, that is the only way for the
whole program. A blanket implementation that would share logic across several types is rejected,
because it could overlap. This is where Rust's coherence guarantee delivers its value, and also where
it starts to bind. Tier 2 is the right tier for a capability that varies by type but never by
application.

## Tier 3: many implementations, one wired per type

The first CGP tier splits the trait into a consumer trait you call and a provider trait you
implement. Many overlapping implementations become legal, each one a named provider, and every type
wires the one it uses.

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

`EncodeSelfAsText` and a bytes-based `EncodeSelfAsBytes` can both exist, even on a type that matches
both, because each provider is its own name rather than an implementation of the trait itself. A type
then wires one provider:

```rust
delegate_components! {
    u32 {
        SelfEncoderComponent: EncodeSelfAsText,
    }
}
```

Add [`check_components!`](/docs/reference/macros/check_components) when you wire. The wiring is checked
lazily, so an unchecked table is the main source of confusing errors.

### Tier 3 holds two shapes

This tier behaves differently depending on what the wired type *is*, and the difference is the one
readers most often miss, because nothing in a signature marks it.

In the **retrofit** shape the wired type is the data itself, as `u32` is above. That type is usually
one you do not own, so it gets one wiring for the whole program. This is the right shape when the
capability belongs to the data, or when an existing trait cannot change its signature.

In the **application** shape the wired type stands for the application rather than for data, and it
often has no fields at all. Here the "one wiring per type" limit stops mattering, because you decide
how many application types exist. The capability is about the application, such as sending an email or
querying a user, rather than about a value:

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

`App` and `TestApp` are both yours, so each wires its own provider with no type parameter anywhere.
Notice the crossing: `u32` in the first example is data in the `Self` position, while `App` here is a
type that stands for the application. **The application shape is where most CGP code lives**, so a
reader who takes tier 3 to be only the retrofit shape has undersold it.

## Tier 4: many implementations, one per type per context

Tier 4 moves the target type out of `Self` and into a parameter, so `Self` is always an application
context. This lifts the orphan rule and lets each context choose a provider per target type,
including for types it does not own.

```rust
#[cgp_component(Encoder)]
pub trait CanEncodeValue<Value> {
    fn encode(&self, value: &Value) -> Vec<u8>;
}
```

Two applications can now encode the same foreign type differently. With an `EncodeAsText` provider
for `Display` types and an `EncodeAsHex` provider for `AsRef<[u8]>` types, `ApiServer` writes a
`String` as text where `Firmware` writes it as hexadecimal:

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

Because the wiring keys on the context rather than on `String`, a crate that owns neither the trait
nor `String` can still wire it, as long as it owns the context. The cost is that the trait must carry
the extra parameter from the start, so it cannot be added to an existing trait such as
`serde::Serialize` without a breaking change, and every value type a context uses must be wired.
Reach for tier 4 when different applications must treat the same foreign type differently. Most code
does not need it.

## Tier 5: many implementations, per type per provider

The top tier lets one provider fix the wiring of a nested type on its own, without routing that
choice back through the context. It takes the inner provider as a parameter, which makes it a
[higher-order provider](./higher-order-providers.md). An encoder for a `Vec` encodes each
element through an inner provider, and defaults that inner provider to the context when none is named:

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

A context can then pin the inner encoding for one collection while leaving others to its general
wiring:

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

The `Vec<u32>` entry omits the parameter, so each `u32` routes back through the context to
`EncodeAsText`. The `Vec<Vec<u8>>` entry pins its inner encoding to `EncodeAsHex`, whatever the
context does with `Vec<u8>` elsewhere. The `UseContext` default is what makes both entries read the
same. This tier adds coupling and higher-order machinery, so reach for it only when a local override
genuinely matters.

## Choosing a tier

The guiding rule is to settle at the lowest tier that expresses the use case, because each higher
tier trades simplicity for modularity that may not be needed. Most code has one implementation of
most things, and for that code a plain trait is the right answer rather than a compromise.

Two questions place a capability faster than working through the tiers one by one.

**Is the capability about the data, or about the application?** About the data means the wired type
*is* the thing being operated on, as `Rectangle: CanCalculateArea` is. About the application means the
wired type is one you define to carry choices, often with no fields at all.

**Does it concern a type you do not own, which different applications must treat differently?** If so,
that type moves out of `Self` and becomes a parameter. If not, targeting `Self` is enough.

The answers name one of three shapes, and none of them is more advanced than the others. Each is the
right answer to a different question.

| Answers | The shape | What it gives you |
|---|---|---|
| About the data | **Retrofit** (tier 3) | Alternative implementations for an existing type, without changing its trait. One choice per type, program-wide. |
| About the application | **Application** (tier 3) | One choice per context you define, and you can define as many as you like. |
| About a foreign type, differing per application | **Fully modular** (tiers 4–5) | One choice per context, per target type. |

**The application shape is where most CGP code lives.** A capability about the application, such as
sending an email or running the server, wired per application with no type parameter anywhere, is the
ordinary case rather than the elaborate one.

As wiring grows, [`cgp_namespace!`](/docs/reference/macros/cgp_namespace) and aggregate providers keep
it readable by grouping shared wiring into reusable tables. Both add a hop between a component and its
provider, so neither pays until the repetition is real.

## CGP, or the tool already in place?

Deciding a tier assumes CGP is the right tool at all, and often it is not. The useful question is
never "is CGP good" but "CGP, or the thing already in place?". Each row below concedes where the
alternative wins, because in most rows it does.

| The alternative | Prefer it when | Reach for CGP when |
|---|---|---|
| **A plain trait or generic** | A capability has one implementation, or one per type with a single global choice. **This is most code.** | The implementations multiply, the choice must differ per context, or threading a generic through every layer has started to hurt. |
| **A direct impl on the context** | A provider would have exactly one user. Two contexts that each have their own single implementation need no providers and no wiring at all. | A second context wants the *same* implementation, or the implementation should compose with a wrapper. |
| **An enum** | The variant set is small, closed, and known, with fixed operations. A `match` is clearer than any machinery. | The variant set is open, or independent modules must each contribute one. |
| **`dyn Trait`** | The set of implementations is not known until runtime. CGP gives up exactly that openness. | The set *is* known at build time, and you want the decoupling without the dispatch cost. |
| **A dependency-injection crate** | You specifically want a container's lifecycle and object-graph semantics. | You want compile-time-checked, reflection-free injection, which is the traits-and-generics approach, not a framework. |
| **A generic-programming library such as `frunk`** | A one-off manipulation of a heterogeneous list. The lighter, focused library is less to learn and enough. | The structural machinery is part of a larger component-and-wiring design. |
| **A hand-rolled macro** | The generation is narrow and local. | You notice you are reinventing marker types plus a helper trait, which is CGP's own mechanism. |
| **Waiting for a language feature** | A first-class facility would serve better and you can afford to wait. Rust's effects and reflection work is pursuing ground CGP covers. | You need the capability now, on stable Rust. CGP is complementary to what the language is building rather than a bet against it. |

That last row is worth taking seriously rather than reading as a formality. Some of what CGP does is
plausibly better as a language feature eventually, and a project that can wait should.

## Where CGP is the wrong tool

Four cases are not trade-offs. Naming them is more useful than any argument in CGP's favour.

**Runtime dynamism.** CGP cannot give you plugins loaded at startup, heterogeneous collections, or
redefinition while the program runs. Its wiring is fixed at build time. That lives on the runtime
side, where `dyn Trait` and the dynamic languages remain right.

**Exactly one implementation.** A capability with one definition belongs in a plain trait, which is
tier 1 or tier 2. If you want it to read like CGP anyway,
[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) builds it from a function with no component, no provider,
and no wiring, and it keeps working unchanged if a second implementation ever arrives.

**A small closed set of variants.** An enum and a `match` are clearer, faster to read, and free.

**One instance program-wide.** When a program genuinely wants a single globally consistent answer,
such as one `Ord` for a map key where two orderings in one program would corrupt the map, per-context
choice is the wrong shape. Coherent traits are safer, and this is the limit of CGP's central bargain.

## "I only have one application — what does this buy me?"

This is the question most people reach in their first week, looking at the single context their
codebase contains, and it deserves a straight answer rather than a prediction about the future.

**Concede the obvious part first: with one context, the swappability payoff is not available.** Every
wiring line has one plausible value and every provider one user. Nothing about that is going to feel
worthwhile, and being told you will want a second context later asks you to spend now for a benefit
you cannot check.

Three things do pay with one context.

- **Overlapping implementations Rust rejects outright.** These multiply along the *target types* one
  application touches, not along its contexts, so a single application with several types to encode,
  serialize, or validate already has more than one implementation to place. That is
  [bypassing coherence](./coherence.md), and it needs no second context at all.
- **The orphan-rule escape.** Adding a capability to a type from another crate needs one context and
  a foreign type. No newtype wrapper, and no second application.
- **Dependencies declared where the implementation is.** A `&self` method on a concrete struct may
  read any field and call any other method, so nothing short of reading the body tells you what it
  depends on. A provider's declared requirements *are* that answer, and the compiler enforces them.
  This is worth something in one context and more in each one added.

And then the second context you probably already have: **the test harness.** It is the cheapest one
to justify and the one nobody argues about.

**If none of that applies, the honest recommendation is to stay put.** A codebase with one
application, no capability needing more than one implementation, and no foreign type to extend is one
where CGP's central bargain does not pay. Reach for `#[cgp_fn]` alone, or nothing.

## "Won't I end up with a context per configuration?"

Partly justified, and the answer is composition rather than reassurance. Separating every axis really
would multiply: four independent binary choices would mean sixteen types.

CGP does not ask you to separate an axis you do not need separated, and it composes with the patterns
that collapse one. A generic parameter on the context struct keeps a single type across several
database engines. An enum keeps a single type for a choice made from configuration at runtime. A
context holds either of those and wires its components normally.

A real application lands on separation for the axis where a wrong combination must be **impossible**,
such as a mock client never reaching production, and collapse for everything else. CGP decides which
variations are worth their own type; the tools you already use handle the ones that are not.

There is a payoff here worth naming if you are writing a library. An enum you expose for supported
backends is normally a ceiling on what downstream users can have: a new variant means a pull request
or a fork. When the context-generic code is written against capabilities rather than against the
enum, a downstream crate defines its own wider enum and its own context and reuses everything, with
nothing to petition for.

## What it costs

**Every tier above a plain trait costs syntax and indirection.** Two traits, a marker type, and a
wiring line per context, plus a hop between a call and the code that runs. The wiring table is a
single greppable place naming exactly one provider per component, and it is still a hop a reader has
to follow.

**Diagnostics are the sharpest cost, and honesty here matters more than elsewhere.** A mis-wired
context can produce a wall of generated types.
[`check_components!`](/docs/reference/macros/check_components) forces the failure to the wiring line
and names the actual missing requirement, and
[`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) un-hides the root cause the default
trait solver suppresses and leads with it. Both help substantially. Neither makes the raw output
pleasant, and the tool is a `v0.1.0-alpha` that reshapes the classes it recognizes rather than all of
them. **Dramatically better and actively improving, not solved.**

**Compile-time cost goes up.** CGP does more work at compile time, and no number is quoted here that
could not be cited. The honest reframing is that resolution costing at compile time is resolution
that would otherwise cost at runtime or not be checked at all.

**There is a learning curve.** The first useful thing, a capability written as a function used with
no wiring, needs only ordinary Rust knowledge, so the first step is small. The curve past it is real.

Three of those costs, the learning curve, decoding the diagnostics, and the volume of wiring to write
and read, are mechanical work over a vocabulary that is written down, which is the kind of thing a
coding assistant handles well. CGP publishes an
[agent skill](https://github.com/contextgeneric/cgp-skills) that teaches one that vocabulary, and a
reader who already works with an assistant can attach it and judge for themselves within the hour. It
makes those three costs **smaller, not absent**: the diagnostics are still verbose, the vocabulary
still has to be learned by whoever reviews the code, and none of it moves any boundary on this page. A
codebase that will only ever have one application still belongs on `#[cgp_fn]` alone, regardless of
who writes the wiring.

## Where to go next

[Bypassing coherence](./coherence.md) is the argument behind the hierarchy: why Rust allows one
implementation per type, and what CGP does about it. Read it if the question is *why* rather than *how
far*.

[Consumer and provider traits](./consumer-and-provider-traits.md) covers the mechanism tiers 3
through 5 are built on, and closes with the same costs stated for that construct specifically.

If the answer here was "not yet", [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) is the whole of what
you need: a capability from a function, no wiring, nothing to reverse later. If it was "yes", the
[Area calculation series](/docs/tutorials/area-calculation/) works up the tiers in order, and
[`#[cgp_component]`](/docs/reference/macros/cgp_component) is where the component machinery starts.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
