---
sidebar_label: 'How much CGP to use'
sidebar_position: 18
---

# How much CGP to use

The ladder from a plain trait to fully wired components, and how to settle at the lowest rung that solves
the problem in front of you.

This page is a decision guide rather than an explanation, so it is written to be acted on rather than read
straight through: a rule of thumb first, then the alternatives you are actually choosing between, the cases
where CGP is simply the wrong tool, which of its shapes to reach for, and how far to go once you are inside
it. If you are still deciding whether CGP is for you at all, this is the page to read.

## The rule of thumb

**Use CGP when a capability needs more than one implementation and the choice belongs to the context, not
before.**

Almost everything follows from that. Most code has one implementation of most things, and for that code a
plain trait is not a compromise, it is the right answer. CGP is a rung you climb to deliberately, and the
useful question is never "is CGP good" but "CGP, or the thing already in place?".

## What you are choosing between

Each row concedes where the alternative wins, because in most rows it does.

| The alternative | Prefer it when | Reach for CGP when |
|---|---|---|
| **A plain trait or generic** | A capability has one implementation, or one per type with a single global choice. **This is most code.** | The implementations multiply, the choice must differ per context, or threading a generic through every layer has started to hurt. |
| **A direct impl on the context** | A provider would have exactly one user. Two contexts that each have their own single implementation need no providers and no wiring at all. | A second context wants the *same* implementation, or the implementation should compose with a wrapper. |
| **An enum** | The variant set is small, closed, and known, with fixed operations. A `match` is clearer than any machinery. | The variant set is open, or independent modules must each contribute one. |
| **`dyn Trait`** | The set of implementations is not known until runtime. That openness is exactly what CGP gives up. | The set *is* known at build time, and you want the decoupling without the dispatch cost. |
| **A dependency-injection crate** | You specifically want a container's lifecycle and object-graph semantics. | You want compile-time-checked, reflection-free injection, which is the traits-and-generics approach, not a framework. |
| **A generic-programming library such as `frunk`** | A one-off manipulation of a heterogeneous list. The lighter, focused library is less to learn and enough. | The structural machinery is part of a larger component-and-wiring design. |
| **A hand-rolled macro** | The generation is narrow and local. | You notice you are reinventing marker types plus a helper trait, which is CGP's own mechanism. |
| **Waiting for a language feature** | A first-class facility would serve better and you can afford to wait. Rust's effects and reflection work is pursuing ground CGP covers. | You need the capability now, on stable Rust. CGP is complementary to what the language is building rather than a bet against it. |

That last row is worth taking seriously rather than reading as a formality. Some of what CGP does is
plausibly better as a language feature eventually, and a project that can wait should.

## Where CGP is the wrong tool

Four cases are not trade-offs. Naming them is more useful than any argument in CGP's favour.

**Runtime dynamism.** CGP cannot give you plugins loaded at startup, heterogeneous collections, or
redefinition while the program runs. Its wiring is fixed at build time. That lives on the runtime side,
where `dyn Trait` and the dynamic languages remain right.

**Exactly one implementation.** A capability with one definition belongs in a plain trait. If you want it
to read like CGP anyway, [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) builds it from a function with no
component, no provider, and no wiring, and it keeps working unchanged if a second implementation ever
arrives.

**A small closed set of variants.** An enum and a `match` are clearer, faster to read, and free.

**One instance program-wide.** When a program genuinely wants a single globally consistent answer, one
`Ord` for a map key where two orderings in one program would corrupt the map, per-context choice is the
wrong shape. Coherent traits are safer, and this is the limit of CGP's central bargain.

## "I only have one application — what does this buy me?"

This is the question most people reach in their first week, looking at the single context their codebase
contains, and it deserves a straight answer rather than a prediction about the future.

**Concede the obvious part first: with one context, the swappability payoff is not available.** Every
wiring line has one plausible value and every provider one user. Nothing about that is going to feel
worthwhile, and being told you will want a second context later asks you to spend now for a benefit you
cannot check.

Three things do pay with one context.

- **Overlapping implementations Rust rejects outright.** These multiply along the *target types* one
  application touches, not along its contexts, so a single application with several types to encode,
  serialize, or validate already has more than one implementation to place. That is
  [bypassing coherence](./coherence.md), and it needs no second context at all.
- **The orphan-rule escape.** Adding a capability to a type from another crate needs one context and a
  foreign type. No newtype wrapper, and no second application.
- **Dependencies declared where the implementation is.** A `&self` method on a concrete struct may read any
  field and call any other method, so nothing short of reading the body tells you what it depends on. A
  provider's declared requirements *are* that answer, and the compiler enforces them. This is worth
  something in one context and more in each one added.

And then the second context you probably already have: **the test harness.** It is the cheapest one to
justify and the one nobody argues about.

**If none of that applies, the honest recommendation is to stay put.** A codebase with one application, no
capability needing more than one implementation, and no foreign type to extend is one where CGP's central
bargain does not pay. Reach for `#[cgp_fn]` alone, or nothing.

## "Won't I end up with a context per configuration?"

Partly justified, and the answer is composition rather than reassurance. Separating every axis really would
multiply: four independent binary choices would mean sixteen types.

CGP does not ask you to separate an axis you do not need separated, and it composes with the patterns that
collapse one. A generic parameter on the context struct keeps a single type across several database
engines. An enum keeps a single type for a choice made from configuration at runtime. A context holds
either of those and wires its components normally.

A real application lands on separation for the axis where a wrong combination must be
**impossible**, such as a mock client never reaching production, and collapse for everything else. CGP
decides which variations are worth their own type; the tools you already use handle the ones that are
not.

There is a payoff here worth naming if you are writing a library. An enum you expose for supported backends
is normally a ceiling on what downstream users can have: a new variant means a pull request or a fork. When
the context-generic code is written against capabilities rather than against the enum, a downstream crate
defines its own wider enum and its own context and reuses everything, with nothing to petition for.

## Which shape to reach for

CGP's code comes in three shapes, and readers sometimes resist being taught three where they expected one.
They are not three levels of sophistication. Each answers a different question, and two questions settle
which you are in.

**Is the capability about the data, or about the application?** About the data means the wired type *is* the
thing being operated on, as in `Rectangle: CanCalculateArea`. About the application means the wired type is
one you define to carry choices, often with no fields at all.

**Does it concern a type you do not own, which different applications must treat differently?** If so, that
type moves out of `Self` and becomes a parameter. If not, targeting `Self` is enough.

| Answers | The shape | What it gives you |
|---|---|---|
| About the data | **Retrofit** | Alternative implementations for an existing type, without changing its trait. One choice per type, program-wide. |
| About the application | **Application** | One choice per context you define, and you can define as many as you like. |
| About a foreign type, differing per application | **Fully modular** | One choice per context *per target type*. |

**The application shape is where most CGP code lives**, which is worth stating plainly so that the most
elaborate shape does not read as the intended destination. A capability about the application, such as
sending an email, querying a user, or running the server, wired per application with no type parameter
anywhere, is the ordinary case.

## How far to go inside CGP

Deciding to use CGP is not the last decision. Its constructs form a progression, and the same rule applies
inside as outside: **stop at the lowest step that solves the problem in front of you.** Each step below is a
response to something you have already hit, not a level to reach.

1. **A capability from a function.** [`#[cgp_fn]`](/docs/reference/macros/cgp_fn): one implementation, no
   component, no wiring. Most capabilities never need more, and starting here costs nothing later: the trait
   keeps its name and method, so promoting it leaves every call site untouched.
2. **A component.** [`#[cgp_component]`](/docs/reference/macros/cgp_component) with a provider and a
   [wiring table](/docs/reference/macros/delegate_components), when a second implementation is real, not
   anticipated. Add [`check_components!`](/docs/reference/macros/check_components) at the same time; the
   wiring is checked lazily and an unchecked table is the main source of confusing errors.
3. **Per-type dispatch.** The [`open` statement](/docs/reference/macros/delegate_components), when one
   component needs a different provider per value of a type parameter. A type has moved out of `Self` into
   a parameter by this point, so it usually arrives with the fully-modular shape rather than before it.
4. **Providers composed from providers.** A provider that takes another provider as a parameter and builds
   on it, declared with [`#[use_provider]`](/docs/reference/attributes/use_provider), when one
   implementation should be expressed in terms of another rather than chosen alongside it. Genuinely
   useful, and the step most often taken too early: if nothing is being wrapped, there is nothing for it to
   do.
5. **Reusable wiring.** An [aggregate provider](/docs/reference/macros/delegate_components) for a small
   bundle several contexts delegate to, or [`cgp_namespace!`](/docs/reference/macros/cgp_namespace) when
   wiring repeats across many contexts or a table has outgrown reading. Both add a hop between a component
   and its provider, so neither pays until the repetition is real.

**Steps 2 and 3 are where most CGP code sits.** Steps 4 and 5 answer scale and composition specifically, and
reaching for either before the problem exists is how a CGP codebase ends up harder to read than the one it
replaced. The machinery is real either way, and only pays when something is actually being shared or
wrapped.

## What it costs

**Every rung above a plain trait costs syntax and indirection.** Two traits, a marker type, and a wiring
line per context, plus a hop between a call and the code that runs. The wiring table is a single greppable
place naming exactly one provider per component, and it is still a hop a reader has to follow.

**Diagnostics are the sharpest cost, and honesty here matters more than elsewhere.** A mis-wired context can
produce a wall of generated types. [`check_components!`](/docs/reference/macros/check_components) forces the
failure to the wiring line and names the actual missing requirement, and
[`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) un-hides the root cause the default trait
solver suppresses and leads with it. Both help substantially. Neither makes the raw output pleasant, and the
tool is a `v0.1.0-alpha` that reshapes the classes it recognizes rather than all of them. **Dramatically
better and actively improving, not solved.**

**Compile-time cost goes up.** CGP does more work at compile time, and no number is quoted here that could
not be cited. The honest reframing is that resolution costing at compile time is resolution that would
otherwise cost at runtime or not be checked at all.

**There is a learning curve.** The first useful thing, a capability written as a function used with no
wiring, needs only ordinary Rust knowledge, so the first step is small. The curve past it is real.

Three of those costs, the learning curve, decoding the diagnostics, and the volume of wiring to write and
read, are mechanical work over a vocabulary that is written down, which is the kind of thing a coding
assistant handles well. CGP publishes an [agent skill](https://github.com/contextgeneric/cgp-skills) that
teaches one that vocabulary, and a reader who already works with an assistant can attach it and judge for
themselves within the hour. It makes those three costs **smaller, not absent**: the diagnostics are still
verbose, the vocabulary still has to be learned by whoever reviews the code, and none of it moves any
boundary on this page. A codebase that will only ever have one application still belongs on `#[cgp_fn]`
alone, regardless of who writes the wiring.

## Where to go next

[Bypassing coherence](./coherence.md) is the argument behind the rule of thumb: why Rust allows one
implementation per type, and what CGP does about it. Read it if the question is *why* rather than *whether*.

[Consumer and provider traits](./consumer-and-provider-traits.md) covers the mechanism the upper rungs are
built on, and closes with the same costs stated for that construct specifically.

If the answer here was "not yet", [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) is the whole of what you
need: a capability from a function, no wiring, nothing to reverse later. If it was "yes",
the [Area calculation series](/docs/tutorials/area-calculation/) climbs the rungs in order, and
[`#[cgp_component]`](/docs/reference/macros/cgp_component) is where the component machinery starts.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
