---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Concepts

This section explains the ideas behind CGP: why it is built the way it is, what each of its ideas
actually does, and where the boundaries of each one lie. It is written to be read away from a
keyboard. There is nothing here to follow along with, and nothing to install first.

It sits alongside the other two parts of the documentation. The [tutorials](/docs/tutorials/hello)
teach by building something, so they show a construct at the moment it is needed and move on. The
[reference](/docs/reference/) explains one construct completely, for a reader who already knows the
name of the thing they want. A concept page answers the question neither of those is shaped for:
*why does CGP work this way, and when is it the right thing to reach for?* Each page names the
constructs it involves and links to the reference for their exact syntax, rather than repeating it.

## Where to start

**Read [Bypassing coherence](./coherence.md) first if you want to know why CGP exists.** It explains
the rule in Rust's trait system that allows only one implementation per type, why that rule is
correct, what it costs in practice, and the move CGP makes to work around it without giving up what
the rule buys. Almost everything else here is downstream of it.

**Read [Consumer and provider traits](./consumer-and-provider-traits.md) first if you want to know
how CGP works.** It covers the trait split at the centre of the design, one trait you call and another
you implement, and how a call finds its way from one to the other.

**Read [How much CGP to use](./modularity-hierarchy.md) first if you are deciding whether to adopt
it.** It lays out the range from an ordinary Rust trait to fully wired components as a ladder, and
argues for climbing no higher than a problem requires.

**New to Rust's traits and generics?** These pages explain the reasoning behind CGP and assume you are
comfortable with traits and generic bounds. If you are not yet, the [tutorials](/docs/tutorials/hello)
build the ideas up from working code and are the gentler way in; come back here for the *why* behind
them.

## The ideas, grouped

The pages below are ordered in the sidebar roughly as a reader meets them, and grouped here by what
they are about.

**The foundation** is the coherence problem and CGP's answer to it.
[Bypassing coherence](./coherence.md) states the problem;
[Consumer and provider traits](./consumer-and-provider-traits.md) is the mechanism that solves it.

**Dependency injection** is how an implementation gets what it needs without the interface carrying
it. [Impl-side dependencies](./impl-side-dependencies.md) is the general idea;
[Implicit arguments](./implicit-arguments.md) is how it looks for a value read from a field, and
[Abstract types](./abstract-types.md) is how it looks for a type the context chooses.

**Composition and scale** cover what happens as a program grows.
[Checking your wiring](./check-traits.md) explains why a wiring mistake compiles and how to make it
fail loudly instead. [Higher-order providers](./higher-order-providers.md) build one implementation
out of another, [Aggregate providers](./aggregate-providers.md) bundle a group of choices into one,
and [Namespaces](./namespaces.md) keep a large wiring table readable.

**Applied ideas** are the places CGP's machinery is put to a specific use.
[Modular error handling](./modular-error-handling.md) separates the error type from how errors are
built. [Handlers](./handlers.md) and [Monadic handlers](./monadic-handlers.md) model computation as
components that compose. [Extensible records](./extensible-records.md),
[Extensible variants](./extensible-variants.md), and [Dispatching](./dispatching.md) let generic code
work over the shape of a struct or an enum. [Type-level DSLs](./type-level-dsls.md) push that as far
as it goes, turning a small language into types the compiler interprets.

**Two pages stand slightly apart.** [Recovering `Send` bounds](./send-bounds.md) is a workaround for a
gap in stable Rust rather than a CGP idea, and it matters as soon as an async CGP task is spawned.
[How much CGP to use](./modularity-hierarchy.md) is a decision guide rather than an explanation, and
it is the page to reach for when the question is how far to go.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
