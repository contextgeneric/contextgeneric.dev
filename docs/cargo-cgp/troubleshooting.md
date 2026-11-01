---
title: 'Troubleshooting cargo-cgp'
sidebar_label: 'Troubleshooting'
sidebar_position: 7
description: 'Match an error from cargo-cgp itself to its cause and fix: the setup check, a driver that will not load or cannot be found, wrong invocations, and slow checks.'
---

# Troubleshooting

This page is for when [`cargo-cgp`](./index.md) itself will not run, or behaves in a way you did not
expect. If instead you need help reading a CGP error the tool printed about your code, see
[Reading the output](./reading-output.md).

Most failures come from one fact about the tool: `cargo-cgp` is two programs and one exact nightly
toolchain, and all three must match. The `cargo-cgp` front end looks for `cargo-cgp-driver` in its own
directory, and the driver loads the compiler's libraries from the pinned nightly. The tool is tested on
Linux, so the library paths on this page are the Linux ones.

## Start with the symptom index

Find the distinctive part of your error in the left column, then read the section it points to.

| Error fragment | Cause | Section |
|---|---|---|
| `toolchain … is not available` | the pinned nightly is not installed | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `could not run under toolchain` | the driver was built against another nightly | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `out of lockstep` | the front end and driver have different versions | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `was built against` | the pinned nightly changed since the driver was built | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `could not parse the` … `--version` output | an old or unrelated driver binary | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `rustup was not found on PATH` | no rustup for the preflight or `setup` | [The preflight rejects the setup](#the-preflight-rejects-the-setup), [Setup fails](#setup-fails) |
| `error while loading shared libraries: librustc_driver-…` | library path unset, or the wrong toolchain | [The driver cannot load the compiler](#the-driver-cannot-load-the-compiler) |
| `failed to run the cargo-cgp-driver` | driver missing | [The driver cannot be found](#the-driver-cannot-be-found) |
| `could not execute process` … `(never executed)` | driver path wrong, with the preflight turned off | [The driver cannot be found](#the-driver-cannot-be-found) |
| `` no such command: `frobnicate` `` with the four commands listed | wrong command name | [The command itself fails](#the-command-itself-fails) |
| `` no such command: `cgp` `` | the tool is not installed, or not on `PATH` | [The command itself fails](#the-command-itself-fails) |
| `could not find Cargo.toml` | run outside a cargo package | [The command itself fails](#the-command-itself-fails) |
| `unexpected argument` | a flag `cargo check` does not know | [The command itself fails](#the-command-itself-fails) |
| `is cargo on PATH?` | cargo is not installed, or not on `PATH` | [The command itself fails](#the-command-itself-fails) |
| `RUSTC_WORKSPACE_WRAPPER is set` | a compiler wrapper of your own will not run | [Your own compiler wrapper stopped running](#your-own-compiler-wrapper-stopped-running) |

Some problems print no error at all: a check that disagrees with `cargo check`, and a slow first check.
Each has a section near the end of the page.

## Narrowing it down

If your error is not in the index, run the check once more and read the first line of what it prints.
Before every check, the front end runs a quick, read-only test of the toolchain and the driver, called the
preflight. A message that ends with `` help: run `cargo cgp setup` `` comes from that test, and the
[preflight section](#the-preflight-rejects-the-setup) covers each one.

If the preflight passes, run the check verbosely:

```sh
cargo cgp check -v
```

Each `Running` line that shows a `cargo-cgp-driver … rustc …` command is cargo calling the driver for one
crate. So an error before those lines is a front-end or setup problem, and an error from one of them is a
driver or compiler problem.

## The preflight rejects the setup

Almost every message in this section is resolved by running `cargo cgp setup`, which installs the pinned
nightly and rebuilds the driver against it at the front end's version. Even so, the messages are worth
reading, because they tell you which part was wrong. The examples show `nightly-2026-10-10`, and yours names
the nightly your build is pinned to. Each message about the driver also names the driver's path and where it
was found: from `CARGO_CGP_DRIVER`, beside `cargo-cgp`, or on `PATH`.

The first thing the preflight checks is that the pinned nightly is installed. When it is not, the preflight
quotes rustup's own reason:

```text
error: toolchain `nightly-2026-10-10` is not available:

  error: toolchain 'nightly-2026-10-10-x86_64-unknown-linux-gnu' is not installed
  help: run `rustup toolchain install nightly-2026-10-10-x86_64-unknown-linux-gnu` to install it

help: run `cargo cgp setup`
```

The front end finds the pinned nightly through rustup, so a machine without rustup gets a different message.
Here `setup` cannot help, since it needs rustup too:

```text
error: rustup was not found on PATH, and cargo-cgp installs and selects its toolchain `nightly-2026-10-10` through rustup

help: install rustup, or install cargo-cgp through its Nix flake, which needs no rustup
```

The [Nix flake](./installation.md#with-nix) runs the front end without the preflight.

Next, the driver has to run under the toolchain. When the toolchain is installed but the driver cannot load
it, the driver was almost always built against a different nightly, and the message keeps the loader's words:

```text
error: the cargo-cgp-driver at /home/you/.cargo/bin/cargo-cgp-driver (beside cargo-cgp) could not run under toolchain `nightly-2026-10-10` (it was likely built against a different nightly):

  /home/you/.cargo/bin/cargo-cgp-driver: error while loading shared libraries: librustc_driver-1a6e695c833ebcd1.so: cannot open shared object file: No such file or directory

help: run `cargo cgp setup`
```

A driver that runs can still be the wrong one. After a partial upgrade, or when an older driver is found first,
its version differs from the front end's:

```text
error: the cargo-cgp-driver at /home/you/.cargo/bin/cargo-cgp-driver (beside cargo-cgp) is version 0.0.9, but this cargo-cgp is 0.1.0 (the two are out of lockstep)

help: run `cargo cgp setup`
```

Even when their versions match, the driver and the toolchain may no longer belong together, if a different
compiler built the driver than the pinned nightly now provides:

```text
error: the cargo-cgp-driver at /home/you/.cargo/bin/cargo-cgp-driver (beside cargo-cgp) was built against `rustc 1.99.0-nightly (0000000 2026-01-01)`, but toolchain `nightly-2026-10-10` provides `rustc 1.101.0-nightly (32dba69d6 2026-10-09)`

help: run `cargo cgp setup`
```

Finally, the program found as the driver may not answer like one at all, such as an unrelated binary with the
same name:

```text
error: could not parse the `--version` output of the cargo-cgp-driver at /home/you/.cargo/bin/cargo-cgp-driver (beside cargo-cgp)

help: run `cargo cgp setup`
```

If you are deliberately running a build that `setup` did not install, from a source checkout for example, set
`CARGO_CGP_NO_MANAGE=1` to skip the preflight. The tool then trusts your environment, and keeping the toolchain
matched is up to you. The next section describes what happens when it is not.

## The driver cannot load the compiler

This failure comes from the operating system's loader, before the driver's own code runs, which is why it looks
unlike any other message from the tool:

```text
cargo-cgp-driver: error while loading shared libraries: librustc_driver-1a6e695c833ebcd1.so: cannot open shared object file: No such file or directory
```

The driver links `librustc_driver-<hash>.so` from the nightly it was built against, and the hash is fixed when
the driver is built, so yours differs from the one above. The loader could not find that exact library, for one
of two reasons.

One reason is that you ran the driver yourself, without the library path. The front end sets the path every time
it runs the driver, so a working install still fails this way when you run `cargo-cgp-driver` directly. To fix
that, supply the pinned nightly's `lib` directory:

```sh
LD_LIBRARY_PATH="$(rustc +nightly-YYYY-MM-DD --print sysroot)/lib" cargo-cgp-driver --version
```

Put the pinned nightly in place of `nightly-YYYY-MM-DD`. A driver installed with Nix has the path built in and
does not need this.

The other reason is that the check ran under a different toolchain from the driver's. With the preflight turned
off, the tool uses whichever toolchain is active, and when that is not the driver's nightly, cargo's first call
to the driver fails:

```text
error: process didn't exit successfully: `…/cargo-cgp-driver …/stable-…/bin/rustc -vV` (exit status: 127)
--- stderr
…/cargo-cgp-driver: error while loading shared libraries: librustc_driver-1a6e695c833ebcd1.so: cannot open shared object file: No such file or directory
```

In that case, unset `CARGO_CGP_NO_MANAGE` so the tool switches to the pinned nightly itself, or make that nightly
the active toolchain, for example with `RUSTUP_TOOLCHAIN`. Running through the Nix flake also handles it.

## The driver cannot be found

The front end looks for the driver in its own directory, unless `CARGO_CGP_DRIVER` names another path, and looks
on your `PATH` only when the driver is not there. When it finds no driver at all, the preflight reports it:

```text
error: failed to run the cargo-cgp-driver (none is beside cargo-cgp or on PATH): No such file or directory (os error 2)

help: run `cargo cgp setup`
```

With the preflight turned off by `CARGO_CGP_NO_MANAGE`, the missing driver reaches cargo instead, which reports a
compiler wrapper it could not start:

```text
error: could not execute process `/path/to/cargo-cgp-driver …/stable-…/bin/rustc -vV` (never executed)

Caused by:
  No such file or directory (os error 2)
```

Either way, the fix is to make the driver reachable. Run `cargo cgp setup` to install it beside the front end,
point `CARGO_CGP_DRIVER` at the real binary (`target/debug/cargo-cgp-driver` in a source checkout), or run through
the Nix flake, which installs both together. Keep the two in the same directory, because when the driver is
missing there, the front end falls back to any `cargo-cgp-driver` on your `PATH`, which may be an older one.

## The command itself fails

Some failures happen before the driver is involved. An unknown command is reported with the list the tool accepts:

```text
error: no such command: `frobnicate`

help: the commands are `check`, `expand`, `setup`, and `update`
```

When the unknown command is `cgp` itself, the message comes from cargo rather than the tool. Cargo found no
`cargo-cgp` on `PATH`, so the tool is not installed, or not on this shell's `PATH`:

```text
error: no such command: `cgp`
```

Running outside a cargo package also produces cargo's error rather than the tool's, because the tool runs
`cargo check`:

```text
error: could not find `Cargo.toml` in `/some/dir` or any parent directory
```

Similarly, cargo rejects a flag `cargo check` does not know, since every argument after `check` is passed to it:

```text
error: unexpected argument '--nope' found
```

If cargo itself is not on `PATH`, the front end reports that when it tries to run the check:

```text
error: failed to run `cargo check` (is cargo on PATH?)

Caused by:
  No such file or directory (os error 2)
```

## Setup fails

`cargo cgp setup` installs the toolchain through rustup, so without rustup it stops at once:

```text
  Installing toolchain `nightly-2026-10-10` with rustc-dev and llvm-tools
error: rustup was not found on PATH, and `cargo cgp setup` installs the toolchain `nightly-2026-10-10` through rustup

help: install rustup, or install cargo-cgp through its Nix flake, which needs no rustup
```

As the message says, you can install through the [Nix flake](./installation.md#with-nix) instead, or install
rustup first. `setup` also downloads the toolchain and the driver's source, so it needs a network connection. A
failure during either step comes from rustup or cargo, and their message says what went wrong.

## The check disagrees with `cargo check`

Your own toolchain's `cargo check` decides whether your code compiles. A check runs a different compiler, a pinned
nightly, with Rust's next-generation trait solver turned on, so on code that relies on nightly-only or
solver-specific behavior the two can disagree. For example, two namespaces that inherit from each other are rejected
by stable Rust and pass the check. When the two disagree, trust `cargo check`, and use the tool for what it is for:
reading the CGP errors both of them report.

## The first check is slow

The first check in a project builds every dependency once more, into `target/cgp` inside your project's target
directory, because the check uses its own toolchain and keeps its artifacts apart from your normal build. Later
checks reuse them, from wherever in the project you run them.

## Your own compiler wrapper stopped running

The tool sets cargo's `RUSTC_WORKSPACE_WRAPPER` to its driver for the length of a check, which replaces any value
you set yourself, so a wrapper you configured through that variable does not run during `cargo cgp check`. The tool
says so when it happens, and your normal builds are unaffected:

```text
warning: RUSTC_WORKSPACE_WRAPPER is set to `/usr/local/bin/my-wrapper`; cargo-cgp replaces it with its driver for this run, so that wrapper does not run
```

A wrapper given through `RUSTC_WRAPPER`, such as `sccache`, does keep running alongside the driver.

## Still stuck

Most of these failures share one root: the nightly the driver embeds is not the toolchain present when it runs. So
if you are between sections, `cargo cgp setup` puts the expected pair back in place on the cargo path, and reinstalling
the profile does the same on the Nix path.

Failures not covered here are worth reporting on the
[issue tracker](https://github.com/contextgeneric/cargo-cgp/issues). Include the output of `cargo cgp check -v`,
`cargo cgp --version`, and the driver's `--version` run as shown
[above](#the-driver-cannot-load-the-compiler).

---

*An AI agent wrote this page using the CGP knowledge base, and its messages were produced by running the
tool. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
