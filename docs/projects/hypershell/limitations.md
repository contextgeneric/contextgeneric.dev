---
sidebar_label: 'Limitations'
sidebar_position: 10
description: 'The limits of Hypershell: a proof of concept, programs fixed at compile time, a nightly toolchain, and when a shell or plain Rust fits better.'
---

# Limitations

Hypershell is a proof of concept built to show what CGP makes possible, and its design has limits
that are worth knowing before reading further. [Hypershell](./index.md) is a small shell-scripting
language built with [CGP](/docs/). This page describes those limits, and the cases where another
tool is the better choice.

## It is a demonstration, not a tool to depend on

Hypershell is an experiment. It is not a replacement for a shell, it is lightly tested, and it is
not meant for production use. Its value is as a worked example: the most complete program on this
site that shows CGP's wiring used to build and extend a language. Read it to learn the patterns, and
expect rough edges if you try to build on it.

## Programs are fixed at compile time

A Hypershell program is a Rust type, so it is checked and compiled with the rest of the program.
That is the source of its guarantees, since a pipeline whose stages do not fit is a compile error
rather than a failure halfway through a run. It also means a program cannot be read from a file,
typed by a user, or changed while an application runs. Values a program needs at run time, such as a
URL or a file name, come from fields of the context, but the shape of the pipeline itself is fixed.

## Typed stages are stricter than a shell

A shell passes bytes between any two commands. Hypershell stages pass typed values, such as bytes or
one of several kinds of stream, and two stages connect only if the second accepts what the first
produces. Most stages accept bytes and any kind of stream, so they chain freely, but some pairings
need an adapter stage in between, which the program has to name. The [streams and input
dispatch](./architecture/streams-and-input-dispatch.md) page lists what each stage accepts.

## Extensions add to the language rather than replace it

Anyone can add new syntax to Hypershell, from their own crate, by publishing a namespace that
inherits the standard one or by adding entries to a single context. What an extension cannot do is
give the standard syntax a different implementation, because CGP does not let a context or an
inheriting namespace override a choice the shared namespace has already made. So a context decides
what new syntax means, but the base language's own syntax always means what the standard wiring
says. The [assembly](./architecture/assembly.md) page explains the layers this follows from, and the
[namespaces](/docs/concepts/namespaces) page explains the rule.

## Building it needs nightly Rust

The repository builds with nightly Rust and the compiler's next-generation trait solver. The largest
examples nest whole programs inside others, and on stable Rust they exhausted the compiler's memory
while type-checking. A crate of your own with programs of that size needs the same toolchain. Large
programs also take noticeable time to compile, since the compiler resolves every piece of every
program.

## Mistakes produce long compile errors

A mistake in a program or its wiring is a compile error, and the compiler's raw message lists every
step it took through the wiring before it failed. Checking a program with `check_components!` and
reading the error with [`cargo cgp check`](/docs/cargo-cgp/check) gets to the cause faster:
`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet
reshape every class. The [debugging guide](./guides/debugging.md)
shows the common mistakes.

## When another tool is the better choice

Most tasks Hypershell can express are simpler another way:

- **A shell script** is shorter to write, runs without compiling, and can be changed at any time.
  For a pipeline written once and run by hand, it is the right tool.
- **Plain Rust with `std::process::Command`** runs a command from a program with no framework and no
  nightly toolchain, and a few lines of it are easier to read than a Hypershell program.

Hypershell's design earns its cost where a pipeline's parts should be reusable and replaceable, and
checked by the compiler. Those are also the properties that make it worth reading as an example of
CGP, even for a reader who never writes a Hypershell program.

## Where to go next

- [How Hypershell works](./architecture/index.md): the design these limits follow from.
- [Type-level DSLs](/docs/concepts/type-level-dsls): the CGP technique, and where it pays off in
  general.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
