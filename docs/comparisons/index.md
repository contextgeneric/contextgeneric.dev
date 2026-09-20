---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'Compare CGP with familiar programming models through vocabulary, examples, trade-offs, and limits.'
---

# Comparisons

These pages explain CGP through related ideas you may already know: dependency injection,
type classes, effect handlers, dynamic dispatch, and other approaches to reusable implementations.
Each comparison maps the vocabulary, illustrates the mechanisms, and explains the costs and cases
where the other approach fits better.

CGP is a language extension for Rust, with pluggable trait implementations at compile-time.
It is a library on stable Rust, and its consumer traits are ordinary Rust traits. The
[Introduction](/docs/) explains those foundations; the comparisons focus on how they relate to
other programming models.

## Which page to read

Choose the page that matches your background. Rust's proposals and C++ policy-based design come
first, followed by type-system concepts, dependency injection, and runtime or security mechanisms.

| If you know | Read |
| --- | --- |
| Rust's coherence debates: specialization, named impls, contexts and capabilities | [Rust's own proposals](./rust-language-proposals.md) |
| C++ templates, policy classes, CRTP, and concepts | [Policy-based design](./policy-based-design.md) |
| Haskell, Agda, or Lean type classes | [Type classes](./type-classes.md) |
| OCaml or Standard ML signatures, structures, and functors | [ML modules](./ml-modules.md) |
| Scala `given`/`using`, Haskell `ImplicitParams`, or the `Reader` monad | [Implicit parameters](./implicit-parameters.md) |
| Koka, OCaml 5, Flix, or Eff effect handlers | [Algebraic effects](./algebraic-effects.md) |
| PureScript rows, OCaml polymorphic variants, or structural typing | [Row polymorphism](./row-polymorphism.md) |
| Spring, Guice, Dagger, or constructor injection | [Dependency injection](./dependency-injection.md) |
| Python, Ruby, JavaScript, Smalltalk, vtables, or prototypes | [Dynamic dispatch](./dynamic-dispatch.md) |
| Bevy reflection, Zig `comptime`, or Rust's reflection work | [Reflection](./reflection.md) |
| Object capabilities, `cap-std`, WASI, Effekt, or Scala capture checking | [Capabilities](./capabilities.md) |

## What every page shares

Each page begins with a vocabulary map and a brief account of the related idea. It then shows
how CGP expresses comparable behavior, identifies the context and target in its examples, and
explains where the analogy stops. Separate sections cover costs, cases favoring the other tool,
and behavior that may differ from your expectations.

Each comparison cites its sources and records how its examples were checked. The Sources section
distinguishes compiled snippets from examples checked against documentation or taken from a
proposal. CGP examples have compiled counterparts in the website's verification suite.

## Where to go next

The [Concepts](/docs/concepts/) pages explain CGP's own ideas without reference to another paradigm,
and each comparison links the concept pages it rests on. The [tutorials](/docs/tutorials/hello) build
working programs, and the [reference](/docs/reference/) specifies each construct a comparison names.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
