---
title: 'Quickstart: install and run a first program'
sidebar_label: 'Quickstart'
sidebar_position: 2
description: 'Install Context-Generic Programming and get a working Rust program running in a few minutes, with nothing to configure and no concepts to learn first.'
---

# Quickstart

[Context-Generic Programming](/docs/) (CGP) enables pluggable trait implementations in Rust, and
this page gets a program that uses it compiling and running on your machine. It explains nothing
along the way, because the [Hello World tutorial](/docs/tutorials/hello) does that next, with a
program much like this one.

The only thing you need is Rust 1.89 or newer on the stable toolchain, so check your version with
`rustc --version` before you start. The version matters because on an older toolchain `cargo add`
often picks an old release of `cgp`, one that lacks what this program uses, and the build fails.
`rustup update stable` brings the toolchain up to date.

## Install

Create a project and add the crate to it:

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

Replace the contents of `src/main.rs` with this program:

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

If it does, CGP is installed and working in your project.

## Next

The [Hello World tutorial](/docs/tutorials/hello) walks through a program like this one, then shows
what makes it worth using: the same `greet` runs unchanged on a second, different context.

---

*An AI agent wrote this page using the CGP knowledge base, and its program was verified by compiling
and running it. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
