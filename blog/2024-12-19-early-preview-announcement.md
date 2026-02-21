---
slug: early-preview-announcement
title: 'Announcing Context-Generic Programming (Early Preview)'
authors: [soares]
tags: [release]
---

Welcome everyone! This blog post marks the launch of the _context-generic programming_ (CGP) project,
to introduce a new modular programming paradigm for Rust.

<!-- truncate -->

## A Quick Overview of Context-Generic Programming

As its name implied, CGP makes it possible to write _context-generic_ programs that can work
with any context type in Rust, i.e. the type that we typically refer to as `Self`.

Compared to regular generic programming, CGP supercharges how we can write generic programs,
by lifting many restrictions imposed by Rust and its trait system. CGP alleviates the needs
to explicitly specify all generic parameters by position, and allows them to be referenced
similar to named parameters. CGP also makes it possible to specify overlapping and orphaned
trait instances, by introducing _provider traits_ replace any reference to `Self` with
an explicit `Context` generic type.

Compared to object-oriented programming (OOP), CGP makes it possible to write Rust programs
following popular OOP patterns, such as inheritance and mixins, but in better ways that
fit Rust's type system. With CGP, one can write highly expressive programs that may look similar
to dynamic-typed programs, while ensuring that the programs remain type-safe at compile time,
without any sacrifice to runtime performance.

As a new programming paradigm, CGP significantly changes how Rust programs can be written.
Because of this, CGP programs may look very different from regular Rust programs, and appear
intimidating to even experienced Rust programmers. CGP introduces new programming concepts
in the form of Rust macros, which makes it not as elegant as it could have been if we were
to introduce them as native language constructs in Rust, or with a whole new programming language.
Hence, we can think of the current form of CGP as an _experimentation_ for introducing new
language concepts into Rust, or for programming languages in the future.

It would take too much space of this blog post to give a full picture of how CGP works.
That would require dedication of an entire website, and several books to cover the entirety of CGP.
If you are new here, you should check out the [CGP homepage](/) for a proper introduction of CGP.
Instead of rehearsing the full introduction, this blog post will cover some background about the
project, the current status, and what to expect from here on.

## How It All Started

My name is [Soares Chen](https://maybevoid.com/soareschen), a.k.a. [MaybeVoid](https://maybevoid.com),
and I am the creator of CGP. Even though this project is still new to the public, it has been ongoing
for a while. The work for CGP first started at around July 2022, when I was working on the
[Hermes IBC Relayer](https://github.com/informalsystems/hermes) at [Informal Systems](https://informal.systems/).

I started developing the techniques used in CGP to help writing large-scale generic applications in Rust.
At that time, the generic code in our code base all share a large monolithic trait called
[`ChainHandle`](https://github.com/informalsystems/hermes/blob/master/crates/relayer/src/chain/handle.rs#L398),
which contains dozens of methods that are hard to implement and also difficult to evolve. I then started
experimenting on using Rust traits with blanket implementations as a form of _dependency injection_ to
hide the constraints used on the implementation side. This way, a generic code can require the minimal
subset of dependencies that it needs, and can be reused by implementations that provide only the given subset
of dependencies.

As time goes on, I developed more and more design patterns to help further modularize the code,
which collectively form the basis for CGP. The work I done on Hermes also slowly gets decoupled
from the main code base, eventually becoming its own project called
[Hermes SDK](https://github.com/informalsystems/hermes-sdk). If you compare both codebases,
you may notice that the way context-generic programs are written in Hermes SDK is almost completely
different than the original Hermes, even though both implement the same functionality.
Compared to the original version, we are able to extend and customize Hermes SDK much more easily
to support projects with different very requirements, including host environments, APIs, encodings,
cryptographic primitives, protocols, concurrency strategy, and many more.

But even before my work at Informal Systems, I have spent over 20 years of my programming journey
experimenting on various design patterns to enable modular programming. My previous projects include
the implementation of a [dynamic-typed component system in JavaScript](https://github.com/quiverjs/quiverjs),
and an extensible
[algebraic effects library in Haskell using implicit parameters](https://github.com/maybevoid/casimir).
Compared to my previous attempts, I am hopeful that Rust serves as a sweetspot to be a host
programming language for modular design patterns, thanks to its advanced type systems as well as
its rapidly expanding ecosystem.

## Current Status

This blog post serves as an early preview announcement, and kickstarts many efforts that are
still needed before we can be ready for a full release.
Previously, I have demonstrated the technical feasibility of various CGP programming techniques in
Hermes SDK.
In this new phase, I will start adding documentation and learning resources to help spread the
knowledge of CGP.

For starter, I have created the [project website](https://contextgeneric.dev)
and finished the first section of my [first book on CGP](https://patterns.contextgeneric.dev).
However, there are still a lot more work needed before I can make CGP accessible enough
to the mainstream programming community. Nevertheless, I would like to make use of this
early announcement to start building an early adopter community to help me continue
growing CGP.

Depending on my time availability, it may take a year or more before I am ready for an official release
of CGP. But in the meanwhile, I will start posting regular updates on my development process,
which may be of interest for some of you reading this blog post.

## Plans for 2025

In the upcoming new year 2025, I have many plans laid out to prepare for an official release of
CGP. This section is less about making promises, but more about making you aware of how much work
is still needed before you should consider using CGP seriously.

### Finish the CGP Book

The most important goal I have for CGP is to finish writing my first book,
[Context-Generic Programming Patterns](https://patterns.contextgeneric.dev).
This book will serve as the minimal knowledge transfer for anyone to fully understand CGP.
My hope is that the book will help reduce the bus factor of CGP, so that even if I became
unavailable to continue working on CGP, someone could still use the book as a basis
to continue the work.

### Improve Error Diagnostics

A critical blocker that makes it challenging for me to teach about CGP is the poor error
reporting returned from the Rust compiler, when there is any error arise from unsatisfied constraints.
CGP makes heavy use of blanket implementations to facilitate the wiring of components and provide
dependency injections. But due to its unconventional use of Rust's trait systems, the error case
is not handled well by the current Rust compiler. This is a major issue, because without proper
error reporting, it is very tedious to figure out what went wrong inside the code that use CGP.

To improve the error message from Rust, I have taken the initiative to file issue
[#134346](https://github.com/rust-lang/rust/issues/134346), and attempted a preliminary fix
[#134348](https://github.com/rust-lang/rust/pull/134348) that is made of ~30 lines of code.
Currently, the fix somewhat works, by at least showing sufficient information to allow
debugging to continue. However, it is not yet general enough to not affect general Rust
programs that do not use CGP.

I plan to eventually dive deeper into Rust's error reporting code, and write a better patch
that can report CGP-related errors in better ways. But until I have the patch ready and merged,
any serious use of CGP would require the use of a fork of Rust compiler that applies my temporary patch.

The progress on improving the error messages is tracked on CGP's GitHub issue
[#44](https://github.com/contextgeneric/cgp/issues/44), and more information on how to use
the forked compiler is documented in the
[CGP book](https://patterns.contextgeneric.dev/debugging-techniques.html#improving-the-compiler-error-message).

### Document the `cgp` Crate

I have done quite a bit of writing about CGP on the project website and the book. But if you
look at the Cargo documentation for the [`cgp` crate](https://docs.rs/cgp/), you would see
almost no documentation about any CGP core construct provided by the crate.

A main reason I haven't focused on documenting the `cgp` crate is that I wanted to avoid
explaining the full CGP concepts inside the crate documentation. Instead, I plan to finish
the CGP book first, and then provide links inside the `cgp` crate for readers to learn
about relevant concepts.

That said, I do plan to provide at least minimal documentation inside the `cgp` crate,
to help onboarding programmers to projects that use the `cgp` crate.

### Public Speaking

An effective way to spread the awareness of CGP is for me to speak about it at Rust conferences.
I plan to apply to speak at major Rust conferences located in Europe, and hopefully I will get
accepted into at least some of them. If you know of a conference that I should speak at, do let
me know about it.

As an alternative, I also consider talking about CGP by sharing video recording on YouTube, or
by organizing online meetups. However, this would subject to my time availability and interest
from the community, as producing tech videos is not exactly my area of expertise.
But in case if you are interested in such content, do let me know what you would like to see produced.

### Improve the CGP Macros

The proc macros provided by the `cgp` crate were written in haste as quick proof of concepts
to simplify the syntax for writing CGP programs. As a result, they are not that high in quality,
and do not provide good UX when there are errors in using the macros. In most cases,
the macro would just panic, and do not provide much clue to users on what went wrong.

The CGP macros are also not comprehensive enough to support all possible ways users may define
CGP components. For instance, the use of const generics or associated constants may result in
macro panics. Other than that, there are known bugs when merging generic parameters coming from
multiple sources.

When I have the time, I plan to learn more about how to properly implement the proc macros,
and implement them correctly with proper test coverage. This is important to provide good
user experience, as developers will use the macros all the time when programming in CGP.

### Developer Tooling

For CGP to gain mainstream adoption, it is not sufficient to just make CGP powerful enough
to solve difficult programming problems. In addition to that, we also need to make CGP
_easy_ enough for even beginner programmers to easily pick up. And to move toward that
goal, we can slowly make CGP easier by building better _tools_ to assist programming in CGP.

Although CGP makes heavy use of Rust's trait system to power its component system, the
heavy machinery are _not_ strictly necessary for its _users_ who use CGP to build modular applications.
If we were to implement CGP as native language constructs, we could in principle not require
beginner programmers to understand anything about traits when they start to learn about CGP.
But even if CGP is not native Rust constructs, there are probably ways for us to build tools
that provide first class support for CGP.

One way we can provide such support is to build analyzers that give special treatment to CGP
traits such as `DelegateComponent`. Our tools can then perform analysis on the component
dependencies directly, and provide help in performing any necessary wiring.

Ideally, I would like to implement IDE features similar to Rust Analyzer, so that most of
the cognitive burden of wiring CGP components can be automated by the IDE. But it may take
much longer than one year for me to implement such features. In the meanwhile, I will probably
explore on simpler options, such as building simple CLI tools for CGP.

### Implement Advanced CGP features

Aside from improving CGP macros, there are a few more advanced core constructs that I need to
implement in the `cgp` crate to enable CGP to solve more use cases.
In particular, I plan to introduce constructs for context-generic construction of struct fields
(product types), and context-generic matching of enum variants (sum types).
These constructs are commonly needed in complex applications, and currently they are commonly
solved using OOP patterns such as factories and visitors.
CGP offers better alternatives than the existing OOP design patterns, but I have yet able to
find the time to implement and document them.

In case if you have interest in topics such as row polymorphism, datatype-generic programming,
and category theory, you might be interested to follow my progress on how I make use of these
advanced concepts in CGP.

### More Documentation

It would be a big milestone if I am able to finish the first CGP book and document the `cgp`
crate by the end of 2025. But I also hope to write more documentation in other forms, to
explain CGP in different ways to different audiences.

It would be helpful if I can write some tutorial series for teaching CGP to beginners, or
for programmers coming from imperative programming background. But it may be challenging to
write such tutorials, without first improving the toolings and error handling for CGP.
Alternatively, I may focus on writing use-case oriented series to explain how to use CGP
to solve real world problems, such as building web applications, training AI models,
or programming microcontrollers.

On one hand, I would like to avoid giving the impression that CGP is specifically
designed to solve a specific application domain. On the other hand, it is a bit tough
to demonstrate on my own how CGP can be used for all kinds of problem domains, while
I myself is clearly not an expert in all of them.

Perhaps the best way for me to approach this is for the community to guide me on
what kind of content I should produce for CGP. If you have a specific programming
problem that you think CGP may help solving, I would love to hear more about it.
This can help inform me what kind of topics is popular, and allows me to better
prepared to produce content for those topics.

## How You Can Help

If you have read till the end of this blog post, thank you for taking your time!
If you are interested in CGP, the project homepage [lists many ways](/docs/contribute)
you can help me continue my development on CGP.
I look forward to see you again in the future updates for CGP!

Following are some links to the discussions on this blog post:

- [Reddit](https://www.reddit.com/r/rust/comments/1hkzaiu/announcing_contextgeneric_programming_a_new/)
- [Lobsters](https://lobste.rs/s/a5wfid/context_generic_programming)
- [Hacker News](https://news.ycombinator.com/item?id=42498176)