---
title: 'Troubleshooting cargo-cgp'
sidebar_label: 'Troubleshooting'
sidebar_position: 6
description: 'Match an error from cargo-cgp itself to its cause and fix: the setup check, a driver that will not load or cannot be found, wrong invocations, and slow checks.'
---

# Troubleshooting

This page is for when [`cargo-cgp`](./index.md) itself will not run, or behaves in a way you did not
expect. For help reading a CGP error the tool printed about your code, see
[Reading the output](./reading-output.md) instead.

`cargo-cgp` is two programs and one exact nightly toolchain. The `cargo-cgp` front end looks for
`cargo-cgp-driver` in its own directory, and the driver loads the compiler's libraries from the pinned
nightly. Most failures come from one of those three not matching the others. The tool is tested on
Linux, and the library paths on this page are the Linux ones.

## Start with the symptom index

Find the distinctive part of your error in the left column, then read the section it points to.

| Error fragment | Cause | Section |
|---|---|---|
| `toolchain … is not available` | the pinned nightly is not installed | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `is rustup on PATH?` | no rustup to find the pinned nightly with | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `could not run under toolchain` | the driver was built against another nightly | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `out of lockstep` | the front end and driver have different versions | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `now provides` | the pinned nightly changed since the driver was built | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `could not parse` … `--version` output | an old or unrelated driver binary | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `error while loading shared libraries: librustc_driver-…` | library path unset, or the wrong toolchain | [The driver cannot load the compiler](#the-driver-cannot-load-the-compiler) |
| `failed to run the cargo-cgp-driver at …` | driver missing | [The driver cannot be found](#the-driver-cannot-be-found) |
| `could not execute process` … `(never executed)` | driver path wrong, with the preflight turned off | [The driver cannot be found](#the-driver-cannot-be-found) |
| `unknown cargo-cgp subcommand` | wrong command name | [The command itself fails](#the-command-itself-fails) |
| `could not find Cargo.toml` | run outside a cargo package | [The command itself fails](#the-command-itself-fails) |
| `unexpected argument` | a flag `cargo check` does not know | [The command itself fails](#the-command-itself-fails) |
| `rustup was not found on PATH` | `setup` needs rustup | [Setup fails](#setup-fails) |

Some problems print no error at all: a check that disagrees with `cargo check`, a slow first check,
and a compiler wrapper of your own that stopped running. Each has a section near the end of the page.

## Narrowing it down

**Run the check once more and read the first line of the error.** Before every check, the front end
runs a quick, read-only test of the toolchain and the driver, the preflight. A message beginning with
`cargo-cgp:` and ending with "Run `cargo cgp setup`" comes from that test, and the
[preflight section](#the-preflight-rejects-the-setup) covers each one.

**If the preflight passes, run the check verbosely:**

```sh
cargo cgp check -v
```

Each `Running` line that shows a `cargo-cgp-driver … rustc …` command is cargo calling the driver for one
crate. An error before those lines is a front-end or setup problem; an error from one of them is a
driver or compiler problem.

## The preflight rejects the setup

Almost every message in this section is resolved by running `cargo cgp setup`, which installs the pinned
nightly and rebuilds the driver against it at the front end's version. The messages tell you which part
was wrong. The examples show `nightly-2026-09-14`; yours names the nightly your build is pinned to.

**The pinned nightly is not installed:**

```text
cargo-cgp: toolchain `nightly-2026-09-14` is not available (exit status: 1)

The pinned toolchain is not installed. Run `cargo cgp setup`.
```

**There is no rustup to ask.** The front end finds the pinned nightly through rustup, so on a machine
without it the same advice appears with a different first line:

```text
cargo-cgp: failed to run `rustc` (is rustup on PATH?)

The pinned toolchain is not installed. Run `cargo cgp setup`.
```

Here `setup` cannot help, since it needs rustup too; install rustup, or use the
[Nix flake](./installation.md#with-nix), which does not run the preflight.

**The toolchain is installed, but the driver cannot run under it**, almost always because the driver
was built against a different nightly:

```text
cargo-cgp: the cargo-cgp-driver could not run under toolchain `nightly-2026-09-14` (it was likely built against a different nightly). Run `cargo cgp setup`.
```

**The driver runs, but its version differs from the front end's.** This follows a partial upgrade, or
an older driver found first:

```text
cargo-cgp: the installed cargo-cgp-driver is version 0.0.9, but this cargo-cgp is 0.1.0 (the two are out of lockstep)

Run `cargo cgp setup`.
```

**The driver was built by a different compiler than the pinned nightly now provides.** The driver and
the toolchain the front end expects no longer belong together, even though their versions match:

```text
cargo-cgp: the cargo-cgp-driver was built against `rustc 1.99.0-nightly (0123abcde 2026-08-01)`, but the pinned toolchain `nightly-2026-09-14` now provides `rustc 1.100.0-nightly (4b6d04e70 2026-09-13)`

Run `cargo cgp setup`.
```

**The program found as the driver does not answer like one**, such as an unrelated binary with the
same name:

```text
cargo-cgp: could not parse `cargo-cgp-driver --version` output. Run `cargo cgp setup`.
```

If you are deliberately running a build that `setup` did not install, from a source checkout for
example, set `CARGO_CGP_NO_MANAGE=1` to skip the preflight. The tool then trusts your environment, and
keeping the toolchain matched is up to you; the next section describes what happens when it is not.

## The driver cannot load the compiler

**This failure comes from the operating system's loader, before the driver's own code runs**, which is
why it looks unlike any other message from the tool:

```text
cargo-cgp-driver: error while loading shared libraries: librustc_driver-61d225838afd1915.so: cannot open shared object file: No such file or directory
```

The driver links `librustc_driver-<hash>.so` from the nightly it was built against, and the hash is
fixed when the driver is built, so yours differs from the one above. The loader could not find that
exact library, for one of two reasons.

**You ran the driver yourself, without the library path.** The front end sets the path every time it
runs the driver, so a working install still fails this way when you run `cargo-cgp-driver` directly.
Supply the pinned nightly's `lib` directory:

```sh
LD_LIBRARY_PATH="$(rustc +nightly-YYYY-MM-DD --print sysroot)/lib" cargo-cgp-driver --version
```

with the pinned nightly in place of `nightly-YYYY-MM-DD`. A driver installed with Nix has the path built
in and does not need this.

**The check ran under a different toolchain from the driver's.** With the preflight turned off, the
tool uses whichever toolchain is active, and when that is not the driver's nightly, cargo's first call
to the driver fails:

```text
error: process didn't exit successfully: `…/cargo-cgp-driver …/stable-…/bin/rustc -vV` (exit status: 127)
--- stderr
…/cargo-cgp-driver: error while loading shared libraries: librustc_driver-61d225838afd1915.so: cannot open shared object file: No such file or directory
```

Unset `CARGO_CGP_NO_MANAGE` so the tool switches to the pinned nightly itself, or make that nightly the
active toolchain, for example with `RUSTUP_TOOLCHAIN`. Running through the Nix flake also handles it.

## The driver cannot be found

**The front end looks for the driver in its own directory**, unless `CARGO_CGP_DRIVER` names another
path, and looks on your `PATH` only when the driver is not there. When the driver is missing, the preflight reports it:

```text
cargo-cgp: failed to run the cargo-cgp-driver at /path/to/cargo-cgp-driver: No such file or directory (os error 2)

Run `cargo cgp setup`.
```

With the preflight turned off by `CARGO_CGP_NO_MANAGE`, the missing driver reaches cargo instead, which
reports a compiler wrapper it could not start:

```text
error: could not execute process `/path/to/cargo-cgp-driver …/stable-…/bin/rustc -vV` (never executed)

Caused by:
  No such file or directory (os error 2)
```

Either way, make the driver reachable. Run `cargo cgp setup` to install it beside the front end, point
`CARGO_CGP_DRIVER` at the real binary (`target/debug/cargo-cgp-driver` in a source checkout), or run
through the Nix flake, which installs both together. Keep the two in the same directory: if the driver
is missing there, the front end falls back to any `cargo-cgp-driver` on your `PATH`, which may be an
older one.

## The command itself fails

These failures happen before the driver is involved. An unknown command is reported with the list the
tool accepts:

```text
cargo-cgp: unknown cargo-cgp subcommand `frobnicate` (expected `check`, `expand`, `setup`, or `update`)
```

**Running outside a cargo package** produces cargo's error rather than the tool's, because the tool
runs `cargo check`:

```text
error: could not find `Cargo.toml` in `/some/dir` or any parent directory
```

**A flag `cargo check` does not know** is rejected by cargo, since every argument after `check` is
passed to it:

```text
error: unexpected argument '--nope' found
```

## Setup fails

`cargo cgp setup` installs the toolchain through rustup, so without rustup it stops at once:

```text
cargo-cgp: installing toolchain `nightly-2026-09-14` (with rustc-dev, llvm-tools)…
cargo-cgp: rustup was not found on PATH; cargo-cgp requires rustup to manage toolchains
```

Install through the [Nix flake](./installation.md#with-nix) instead, which needs no rustup, or install
rustup first. `setup` also downloads the toolchain and the driver's source, so it needs a network
connection; a failure during either step comes from rustup or cargo, and their message says what went
wrong.

## The check disagrees with `cargo check`

**Your own toolchain's `cargo check` decides whether your code compiles.** A check runs a different
compiler, a pinned nightly, with Rust's next-generation trait solver turned on, so on code that relies
on nightly-only or solver-specific behavior the two can disagree. When they do, trust `cargo check`,
and use the tool for what it is for: reading the CGP errors both of them report.

## The first check is slow

**The first check in a project builds every dependency once more**, into `target/cgp` inside your
project's target directory, because the check uses its own toolchain and keeps its artifacts apart
from your normal build. Later checks reuse them, from wherever in the project you run them.

## Your own compiler wrapper stopped running

**The tool sets cargo's `RUSTC_WORKSPACE_WRAPPER` to its driver for the length of a check**, which
replaces any value you set yourself, so a wrapper you configured through that variable does not run
during `cargo cgp check`. Your normal builds are unaffected.

## Still stuck

Most of these failures share one root: **the nightly the driver embeds is not the toolchain present
when it runs.** If you are between sections, `cargo cgp setup` puts the expected pair back in place on
the cargo path, and reinstalling the profile does the same on the Nix path.

Failures not covered here are worth reporting on the
[issue tracker](https://github.com/contextgeneric/cargo-cgp/issues). Include the output of
`cargo cgp check -v`, `cargo cgp --version`, and the driver's `--version` run as shown
[above](#the-driver-cannot-load-the-compiler).

---

*An AI agent wrote this page using the CGP knowledge base, and its messages were produced by running the
tool. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
