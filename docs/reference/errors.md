---
sidebar_label: 'Compile errors'
sidebar_position: 90
---

# Compile errors

The errors CGP produces after macro expansion, grouped by what went wrong: a dependency a provider needs and the context does not supply, a wiring table that does not resolve, and a shorthand that lowered into something the compiler rejects.

:::info

### Not written yet

This page is still being written. In the meantime, run [`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) in place of `cargo check`: it un-hides the root cause the default trait solver suppresses and leads with it, for the error classes it recognizes.

:::
