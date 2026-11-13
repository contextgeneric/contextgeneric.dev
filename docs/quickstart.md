---
title: 'Quickstart: install and run a first program'
sidebar_label: 'Quickstart'
sidebar_position: 2
description: 'Install Context-Generic Programming and get a working Rust program running in a few minutes, with nothing to configure and no concepts to learn first.'
---

# Quickstart

[Context-Generic Programming](/docs/) (CGP) enables pluggable trait implementations in Rust, and
this page gets a program that uses it compiling and running on your machine. It explains nothing
along the way; the [Hello World tutorial](/docs/tutorials/hello) teaches the idea behind the code
below.

You need Rust 1.89 or newer on the stable toolchain, and nothing else. Check with `rustc --version`
first: on an older toolchain the build fails, often because `cargo add` picks an old release of
`cgp` that lacks what this program uses. `rustup update stable` brings the toolchain up to date.

## Install

Create a project and add the crate:

```bash
cargo new hello-cgp
cd hello-cgp
cargo add cgp
```

That writes the dependency into your `Cargo.toml`:

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

```bash
cargo run
```

You should see:

```text
Hello, World!
```

That is the whole program. CGP is installed and working in your project.

## Next

The [Hello World tutorial](/docs/tutorials/hello) walks through a program like this one and then
shows what makes it worth using: it runs `greet` unchanged on a second, different context.

---

*An AI agent wrote this page using the CGP knowledge base, and its program was verified by compiling
and running it. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
