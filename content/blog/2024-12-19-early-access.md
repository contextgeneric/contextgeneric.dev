+++

title = "Announcing Context-Generic Programming (Early Access)"

+++

# The Beginning of a New Paradigm

Welcome everyone! This blog post marks the beginning of the _context-generic programming_ (CGP) project,
to introduce a new modular programming paradigm for Rust.

If you are new here, you should check out the [project homepage](/) for a proper introduction of CGP.
Instead of rehearsing the introduction here, this blog post will cover some background about the
project, the current status, and what to expect from here on.

# How It All Started

My name is [Soares Chen](https://maybevoid.com/soarschen), a.k.a. [MaybeVoid](https://maybevoid.com),
and I am the creator of CGP. Even though this project is still new to the public, it has been ongoing
for a while. The work for CGP first started at around July 2022, when I was working on the
[Hermes IBC Relayer](https://github.com/informalsystems/hermes) at [Informal Systems](https://informal.systems/).

I started developing the techniques used in CGP to help writing large-scale generic applications in Rust.
At that time, the generic code in our code base all share a large monolithic trait called
[`ChainHandle`](https://github.com/informalsystems/hermes/blob/master/crates/relayer/src/chain/handle.rs#L398),
which contains dozens of methods that are hard to implement and also hard to change. I then started
exploring on using Rust traits with blanket implementations as a form of _dependency injection_ to
hide the constraints used on the implementation side. This way, a generic code can require the minimal
subset of dependencies that it needs, and can be reused by implementations that provide the given subset
of dependencies.

As time goes on, I developed more and more design patterns to help further modularize the code,
which collectively forms the basis for CGP. The work I done on Hermes also slowly gets decoupled
from the main code base, eventually becoming its own project called
[Hermes SDK](https://github.com/informalsystems/hermes-sdk). If you compare both codebases,
you may notice that the way context-generic programs are written in Hermes SDK is almost completely
different than the original Hermes, even though both implement the same functionality.
Compared to the original version, we are able to extend and customize Hermes SDK much more easily
to support projects with different very requirements, including host environments, APIs, encodings,
cryptographic primitives, protocols, concurrency strategy, and many more.