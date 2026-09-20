---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'CGP placed against the ideas its readers already know, one page per idea.'
---

# Comparisons

CGP is a language extension for Rust, with pluggable trait implementations at compile-time. It is a
library on stable Rust, and a CGP trait is an ordinary Rust trait; the [Introduction](/docs/) says
what that means. These pages are for a reader who already knows a related idea from another language
or paradigm and wants CGP placed in it. Each page maps CGP into that idea's vocabulary, shows the two
side by side in code, states what each costs, and says where the reader's own tool is the better
choice.

## Which page to read

Start from what you already know. The first two pages are the ones a Rust programmer compares CGP
to before anything else; the rest are grouped by the tradition they come from.

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

Each page follows the same shape, so a reader who has read one can skim the next by habit. It opens
with a short table that maps the idea's vocabulary onto CGP's. It refreshes the idea in a few cited
snippets, shows the same problems written in CGP, and names which of CGP's context shapes each
snippet uses. It then states what each approach costs, in the words its own community uses for its
costs, and gives the cases where the compared tool is the better choice their own section. A closing
section lists the expectations a reader of that background holds that CGP does not meet, each with
the reason CGP is arranged as it is.

Every page cites its sources, and every snippet in another language was compiled against the
toolchain the page names. The CGP code on every page compiles against the current release.

## Where to go next

The [Concepts](/docs/concepts/) pages explain CGP's own ideas without reference to another paradigm,
and each comparison links the concept pages it rests on. The [tutorials](/docs/tutorials/hello) build
working programs, and the [reference](/docs/reference/) specifies each construct a comparison names.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
