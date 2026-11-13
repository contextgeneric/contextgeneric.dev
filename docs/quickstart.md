---
title: 'Quickstart: install and run a first program'
sidebar_label: 'Quickstart'
sidebar_position: 2
description: 'Install Context-Generic Programming and run your first Rust program with it in a few minutes.'
---

# Quickstart

[Context-Generic Programming](/docs/) (CGP) enables pluggable trait implementations in Rust. To
try it, you will create a Rust project and run a small program that prints a greeting. You can
follow the steps without learning the concepts first, because the
[Hello World tutorial](/docs/tutorials/hello) explains them afterward.

You need Rust 1.89 or newer on the stable toolchain. Check your version with `rustc --version`
before you start. On an older toolchain, `cargo add` picks an older release of `cgp` that lacks the
features this program uses, so the build fails. Run `rustup update stable` if you need to update.

## Install

Create a project, open its directory, and add CGP:

```bash
cargo new hello-cgp
cd hello-cgp
cargo add cgp
```

`cargo add` writes the dependency into your `Cargo.toml`:

```toml title="Cargo.toml"
[dependencies]
cgp = "0.8.0"
```

## The program

With CGP installed, replace the contents of `src/main.rs` with this greeting program:

```rust title="src/main.rs"
use cgp::prelude::*;

#[cgp_fn]
pub fn greet(&self, #[implicit] name: &str) {
    println!("Hello, {name}!");
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

fn main() {
    let person = Person {
        name: "World".to_owned(),
    };

    person.greet();
}
```

## Run it

Run the program with Cargo:

```bash
cargo run
```

It should print:

```text
Hello, World!
```

If you see that greeting, CGP is working in your project.

## Next

The [Hello World tutorial](/docs/tutorials/hello) explains a program like this one, then shows how
the same `greet` function runs unchanged on a different context.

---

*An AI agent wrote this page using the CGP knowledge base, and its program was verified by compiling
and running it. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
