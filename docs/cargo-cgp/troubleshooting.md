---
title: 'Troubleshooting cargo-cgp'
sidebar_label: 'Troubleshooting'
sidebar_position: 7
description: 'Match an error from cargo-cgp itself to its cause and fix: the setup check, a driver that will not load or cannot be found, wrong invocations, and slow checks.'
---

# Troubleshooting

If [`cargo-cgp`](./index.md) will not start or behaves unexpectedly, begin with the symptom index
below. For a tool that runs but reports an unfamiliar CGP error, [Reading the output](./reading-output.md)
explains the diagnostic instead.

Many startup failures involve a mismatch between the front end, `cargo-cgp-driver`, and the pinned
nightly. The front end normally finds the driver beside itself, and the driver loads that nightly's
compiler libraries. The paths and loader messages here are from Linux, the platform on which the
tool has been tested.

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

When the index does not identify the problem, rerun the check and look at its first message.
Before checking code, the front end performs a read-only test of the toolchain and driver, called
the preflight. A message ending in `` help: run `cargo cgp setup` `` comes from that test;
the [preflight section](#the-preflight-rejects-the-setup) explains its possible results.

If the preflight passes, run the check verbosely:

```sh
cargo cgp check -v
```

Once cargo begins compiling crates, the verbose output includes `Running` lines invoking
`cargo-cgp-driver … rustc …`. A failure before those invocations points to the front end or setup.
A failure from an invocation points to the driver or compiler.

## The preflight rejects the setup

Most failures below can be repaired with `cargo cgp setup`, which installs the pinned nightly
and builds a driver matching both that compiler and the front end. Before rerunning setup,
read the diagnostic to identify which part failed.

The examples name `nightly-2026-10-10`; your output names your build's pinned nightly. Driver
messages also show the binary's path and where it was found: through `CARGO_CGP_DRIVER`, beside
the front end, or on `PATH`.

The preflight begins by checking for the pinned nightly. If rustup cannot find it, the message
includes rustup's explanation:

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

Once the nightly is available, the preflight tries to run the driver under it. Failure at this
point usually means the driver was built against another nightly. The diagnostic includes the
loader's message so you can see which library could not be loaded:

```text
error: the cargo-cgp-driver at /home/you/.cargo/bin/cargo-cgp-driver (beside cargo-cgp) could not run under toolchain `nightly-2026-10-10` (it was likely built against a different nightly):

  /home/you/.cargo/bin/cargo-cgp-driver: error while loading shared libraries: librustc_driver-1a6e695c833ebcd1.so: cannot open shared object file: No such file or directory

help: run `cargo cgp setup`
```

Starting successfully establishes that the driver can load, but not that it matches the front end.
A partial upgrade or an older binary found first can leave their versions out of agreement:

```text
error: the cargo-cgp-driver at /home/you/.cargo/bin/cargo-cgp-driver (beside cargo-cgp) is version 0.0.9, but this cargo-cgp is 0.1.0 (the two are out of lockstep)

help: run `cargo cgp setup`
```

Matching version numbers are not sufficient if the compiler has changed since the driver was
built. The preflight compares the build-time compiler with the one the pinned toolchain now
provides and reports a mismatch:

```text
error: the cargo-cgp-driver at /home/you/.cargo/bin/cargo-cgp-driver (beside cargo-cgp) was built against `rustc 1.99.0-nightly (0000000 2026-01-01)`, but toolchain `nightly-2026-10-10` provides `rustc 1.101.0-nightly (32dba69d6 2026-10-09)`

help: run `cargo cgp setup`
```

The version query can also reveal that the file found as the driver is an unrelated program,
or otherwise does not return the expected version information:

```text
error: could not parse the `--version` output of the cargo-cgp-driver at /home/you/.cargo/bin/cargo-cgp-driver (beside cargo-cgp)

help: run `cargo cgp setup`
```

For a build from a source checkout, `CARGO_CGP_NO_MANAGE=1` lets you bypass the installed-setup
preflight. You must then supply the matching toolchain yourself. If that environment is wrong,
the failure usually comes from library loading, as described below.

## The driver cannot load the compiler

This failure comes from the operating system's loader, before the driver's own code runs, which is why it looks
unlike any other message from the tool:

```text
cargo-cgp-driver: error while loading shared libraries: librustc_driver-1a6e695c833ebcd1.so: cannot open shared object file: No such file or directory
```

The driver needs the exact `librustc_driver-<hash>.so` it linked against when built. Your hash may
differ from the example, but the failure means the loader could not find that specific library.
The remedy depends on how the driver was started.

If you invoked `cargo-cgp-driver` directly, its environment may lack the library path that the
front end normally supplies. Even a working installation can fail this way. Retry with the
pinned nightly's `lib` directory on the path:

```sh
LD_LIBRARY_PATH="$(rustc +nightly-YYYY-MM-DD --print sysroot)/lib" cargo-cgp-driver --version
```

Put the pinned nightly in place of `nightly-YYYY-MM-DD`. A driver installed with Nix has the path built in and
does not need this.

During a check, the same missing-library error can mean the active toolchain differs from the
one the driver needs. Disabling the preflight also disables automatic nightly selection, so cargo
may try to start the driver under another toolchain:

```text
error: process didn't exit successfully: `…/cargo-cgp-driver …/stable-…/bin/rustc -vV` (exit status: 127)
--- stderr
…/cargo-cgp-driver: error while loading shared libraries: librustc_driver-1a6e695c833ebcd1.so: cannot open shared object file: No such file or directory
```

In that case, unset `CARGO_CGP_NO_MANAGE` so the tool switches to the pinned nightly itself, or make that nightly
the active toolchain, for example with `RUSTUP_TOOLCHAIN`. Running through the Nix flake also handles it.

## The driver cannot be found

An explicit `CARGO_CGP_DRIVER` selects the driver path. Otherwise, the front end first looks
beside itself and then falls back to `PATH`. If no driver is available, the preflight reports:

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

Restore a reachable driver by running `cargo cgp setup`, selecting the actual binary through
`CARGO_CGP_DRIVER` (such as `target/debug/cargo-cgp-driver` in a checkout), or using the Nix flake
that installs both programs together. Keeping the installed binaries side by side avoids the
fallback to a possibly older driver on `PATH`.

## The command itself fails

Some failures happen before the driver is involved. An unknown command is reported with the list the tool accepts:

```text
error: no such command: `frobnicate`

help: the commands are `check`, `expand`, `setup`, and `update`
```

An unknown `cgp` command comes from cargo itself: it could not find `cargo-cgp` on `PATH`.
Check that the tool is installed and visible in the shell where you invoked cargo:

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

You can resolve the missing rustup by installing it first or choosing the
[Nix flake](./installation.md#with-nix). Once rustup is available, setup still needs a network
connection for the toolchain and driver source downloads. Errors during those steps come from
rustup or cargo, whose messages identify the failure.

## The check disagrees with `cargo check`

When the checks disagree, trust your own toolchain's `cargo check` to decide whether the code
compiles. The tool uses a pinned nightly and the next-generation trait solver, so code relying on
nightly-only or solver-specific behavior can produce a different result. Mutually inheriting
namespaces, for example, pass the tool's check but are rejected by stable Rust. Use the tool to
understand CGP failures while retaining the ordinary check as the build authority.

## The first check is slow

A check uses its own toolchain and build artifacts, so its first run recompiles the project's
dependencies into `target/cgp` within the project target directory. Subsequent runs reuse that
build wherever inside the project you invoke them.

## Your own compiler wrapper stopped running

To rewrite workspace diagnostics, the tool assigns its driver to `RUSTC_WORKSPACE_WRAPPER` for
this run. Any wrapper you selected through that variable is therefore replaced, and the tool
prints a warning. Normal builds keep your original setting:

```text
warning: RUSTC_WORKSPACE_WRAPPER is set to `/usr/local/bin/my-wrapper`; cargo-cgp replaces it with its driver for this run, so that wrapper does not run
```

A wrapper given through `RUSTC_WRAPPER`, such as `sccache`, does keep running alongside the driver.

## Still stuck

If the failure suggests a mismatch but does not fit one section exactly, restoring the matching
nightly and driver is a useful next step. On the cargo path, run `cargo cgp setup`; on the Nix
path, reinstall the profile.

Failures not covered here are worth reporting on the
[issue tracker](https://github.com/contextgeneric/cargo-cgp/issues). Include the output of `cargo cgp check -v`,
`cargo cgp --version`, and the driver's `--version` run as shown
[above](#the-driver-cannot-load-the-compiler).

---

*An AI agent wrote this page using the CGP knowledge base, and its messages were produced by running the
tool. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
