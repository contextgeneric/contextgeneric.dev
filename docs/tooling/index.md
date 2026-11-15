---
title: 'Tooling for Context-Generic Programming'
sidebar_label: 'Overview'
sidebar_position: 0
description: 'The programs you can run beside the cgp library, starting with cargo-cgp, which explains CGP compile errors and expands CGP macros.'
---

# Tooling

[Context-Generic Programming (CGP)](/docs/) is a Rust library, so you build, check, and test CGP
code with the same cargo commands as any other Rust project. This section covers the programs you
can run beside those commands. They are optional, because CGP compiles on stable Rust without them.

## cargo-cgp

[`cargo-cgp`](/docs/cargo-cgp) helps you understand compile errors and macro expansions in CGP
code. Run `cargo cgp check` in place of `cargo check` when an error is hard to read:
`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet
reshape every class. To read the Rust code that CGP's macros generate, run `cargo cgp expand`.

Its pages start with [installation](/docs/cargo-cgp/installation), then cover
[checking a project](/docs/cargo-cgp/check), [reading the output](/docs/cargo-cgp/reading-output),
and the [error codes](/docs/cargo-cgp/error-codes) the tool prints.

## More tools to come

`cargo-cgp` is the only tool in this section today. More tooling for CGP will be added here in the
future.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
