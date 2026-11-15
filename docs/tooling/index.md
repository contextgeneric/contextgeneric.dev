---
title: 'Tooling for Context-Generic Programming'
sidebar_label: 'Overview'
sidebar_position: 0
description: 'These pages cover the optional programs you run beside the cgp library, starting with cargo-cgp for reading CGP compile errors and macro expansions.'
---

# Tooling

[Context-Generic Programming (CGP)](/docs/) is a Rust library, so the cargo commands you already
use build, check, and test CGP code as they do any other Rust code. Now and then, though, you need
more than those commands give you. CGP's macros generate ordinary Rust traits, so a compile error
may name that generated code while leaving the failed requirement unclear, or you may want to read
the code a macro produced. The programs in this section help at those moments. None of them is
required, because CGP compiles on stable Rust without them.

## cargo-cgp

[`cargo-cgp`](/docs/tooling/cargo-cgp) helps with both. When an error is hard to read, run
`cargo cgp check` in place of `cargo check`. `cargo cgp check` leads with the root cause for the
classes it recognizes, and the tool does not yet reshape every class. When you want to see the code
behind a macro, [`cargo cgp expand`](/docs/tooling/cargo-cgp/expand) prints the Rust that CGP's macros
generate.

To start using it, follow the [installation guide](/docs/tooling/cargo-cgp/installation), then try it on a
project with [the check page](/docs/tooling/cargo-cgp/check). Once the tool reports an error,
[reading the output](/docs/tooling/cargo-cgp/reading-output) explains each part of it, and
[the error-codes page](/docs/tooling/cargo-cgp/error-codes) explains each code it prints.

## More tools to come

For now, `cargo-cgp` is the only program in this section. More tooling for CGP will be added here
in the future.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
