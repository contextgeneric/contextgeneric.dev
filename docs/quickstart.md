---
title: 'Quickstart: install CGP and run a program'
sidebar_label: 'Quickstart'
sidebar_position: 2
description: 'Install Context-Generic Programming and get a working Rust program running, with nothing to configure and no concepts to learn first.'
---

# Quickstart

Context-Generic Programming (CGP) is a language extension for Rust, with pluggable trait
implementations at compile-time. This page does one thing: it gets a program that uses CGP compiling
and running on your machine. It explains nothing along the way. The [Introduction](/docs/) says what
CGP is, and the [Hello World tutorial](/docs/tutorials/hello) teaches the idea behind the code below.

You need Rust installed, and nothing else. CGP is an ordinary library on the stable toolchain.

## Install

In a new or existing project, add the crate:

```bash
cargo add cgp
```

That writes the dependency into your `Cargo.toml`:

```toml title="Cargo.toml"
[dependencies]
cgp = "0.8.0"
```

## The program

Put this in `src/main.rs`:

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

The [Hello World tutorial](/docs/tutorials/hello) walks through the same program and then does the
thing that makes it worth using: it runs `greet` unchanged on a second, different context.

---

*An AI agent wrote this page using the CGP knowledge base, and its program was verified by compiling
and running it. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
