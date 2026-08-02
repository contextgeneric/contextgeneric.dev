---
sidebar_position: 5
---

# Troubleshooting

`cargo-cgp` is two binaries plus one exact nightly compiler, held together by a sibling-path lookup and
a couple of environment variables. When it will not run, the fault sits at one of those seams. This
page maps the error you see to the seam it came from.

## Start with the symptom index

Match the distinctive fragment of your error, then read the section.

| Error fragment | Cause | Section |
|---|---|---|
| `unknown cargo-cgp subcommand` | wrong invocation | [The command itself fails](#the-command-itself-fails) |
| `could not find Cargo.toml` | run outside a cargo package | [The command itself fails](#the-command-itself-fails) |
| `is cargo on PATH?` | cargo missing from `PATH` | [The command itself fails](#the-command-itself-fails) |
| `failed to run the cargo-cgp-driver at …` | driver missing | [The driver cannot be found](#the-driver-cannot-be-found) |
| `could not execute process` … `(never executed)` | driver path wrong | [The driver cannot be found](#the-driver-cannot-be-found) |
| `error while loading shared libraries: librustc_driver-…` | library path unset, or toolchain mismatch | [The driver cannot load the compiler](#the-driver-cannot-load-the-compiler) |
| `--print sysroot` failed with `exit status: 127` | Nix install shadowed by a same-version rustup toolchain | [A Nix install fails its sysroot probe](#a-nix-install-fails-its-sysroot-probe) |
| `the pinned toolchain is not installed` | nightly absent | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `could not run under toolchain …` | driver built against another nightly | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `out of lockstep` / `now provides` | front-end and driver disagree | [The preflight rejects the setup](#the-preflight-rejects-the-setup) |
| `rustup was not found on PATH` | no rustup for `setup` | [Provisioning fails](#provisioning-fails) |

## Isolating the failure

Two probes narrow almost anything before you read further.

**First, ask the driver for its version.** It loads the compiler library before printing, so this
doubles as a test that it can run at all:

```sh
cargo-cgp-driver --version
```

If that fails, the problem is the driver or its library path, not the front-end — jump to
[The driver cannot load the compiler](#the-driver-cannot-load-the-compiler). On success it prints the
`pinned-toolchain:` it needs and the `built-against-rustc:` it was built with, which is the fastest way
to see which nightly a given driver actually wants.

**Then run the check verbosely:**

```sh
cargo cgp check -v
```

Each `Running …` line showing a `cargo-cgp-driver … rustc …` command is the driver being invoked. An
error *before* those lines is a front-end or preflight problem; an error *from* one of them is a driver
or compiler problem.

## The command itself fails

The simplest failures never reach the driver. An unknown or missing subcommand is reported with the
list of what is accepted:

```text
cargo-cgp: unknown cargo-cgp subcommand `frobnicate` (expected `check`, `expand`, `setup`, or `update`)
```

If your build lists only three and omits `expand`, that is not a fault — it predates that command. See
[Installation](./installation.md#expand-is-newer-than-the-published-release).

Running outside a cargo package produces cargo's error rather than the tool's, because the front-end
forwards to `cargo check`:

```text
error: could not find `Cargo.toml` in `/some/dir` or any parent directory
```

Run it from inside the package or workspace you mean to check. And if cargo is not on `PATH` at all,
the front-end says so while trying to launch the build:

```text
failed to run `cargo check` (is cargo on PATH?)
```

## The driver cannot be found

The front-end looks for the driver **beside itself**, in the same directory, unless `CARGO_CGP_DRIVER`
says otherwise. When it is missing, the message depends on whether the preflight is running.

Normally the preflight catches it and names the fix:

```text
cargo-cgp: failed to run the cargo-cgp-driver at /path/to/cargo-cgp-driver: No such file or directory (os error 2)

Run `cargo cgp setup`.
```

With the preflight skipped — the from-source and Nix paths set `CARGO_CGP_NO_MANAGE` — the bad path
reaches cargo instead, which reports a wrapper it could not execute:

```text
error: could not execute process `/path/to/cargo-cgp-driver …/rustc -vV` (never executed)

Caused by:
  No such file or directory (os error 2)
```

Either way, make the driver reachable: run `cargo cgp setup` to reinstall it beside the front-end, point
`CARGO_CGP_DRIVER` at the real binary (for a source build, `target/debug/cargo-cgp-driver`), or run
through the Nix flake, which places both binaries together.

## The driver cannot load the compiler

This is the most common failure and the most confusing, because the driver aborts *before its own code
runs* and the message comes from the operating system's loader:

```text
cargo-cgp-driver: error while loading shared libraries: librustc_driver-c29d28819724b6fa.so:
cannot open shared object file: No such file or directory
```

The driver links `librustc_driver-<hash>.so` from the nightly it was built against, and that hash is
fixed at build time. The loader cannot find *that exact library*, for one of two reasons.

**The library path is not set.** You ran the driver directly, without the search-path setup the
front-end normally provides. A Nix-built driver has the path baked into its wrapper and never hits
this; a from-source driver run by hand does. Supply the pinned toolchain's `lib` directory —
`DYLD_FALLBACK_LIBRARY_PATH` on macOS, `LD_LIBRARY_PATH` elsewhere:

```sh
SYSROOT=$(rustc --print sysroot)          # under the pinned toolchain
LD_LIBRARY_PATH=$SYSROOT/lib cargo-cgp-driver --version
```

**Or the path is set to the wrong toolchain.** The exact library is absent because the active toolchain
is not the one the driver was built against — the driver wants a dated nightly, the environment offers
stable:

```text
error: process didn't exit successfully: `…/cargo-cgp-driver …/stable/…/rustc -vV` (exit status: 127)
--- stderr
…/cargo-cgp-driver: error while loading shared libraries: librustc_driver-c29d28819724b6fa.so
```

A normal install forces the right nightly for you. If you are running unmanaged, either make the pinned
nightly active — work inside the `cargo-cgp` checkout, whose `rust-toolchain.toml` selects it, or set
`RUSTUP_TOOLCHAIN` — or use the Nix flake, which forces the matching nightly from any directory.

## A Nix install fails its sysroot probe

A Nix-installed tool can fail before compiling anything, on the query it makes to locate the
toolchain's libraries:

```text
cargo-cgp: `/nix/store/…-rust-minimal-…/bin/rustc --print sysroot` failed with status exit status: 127:

rustc: error while loading shared libraries: libz.so.1: cannot open shared object file
```

Two things make this one recognizable: **the same command run by hand succeeds**, and it fails only in
*some* projects. Both follow from one cause — a foreign toolchain's library directory reaching the Nix
toolchain's binaries. Invoked as `cargo cgp`, the entry point is rustup's `cargo` shim, which exports
the project's active toolchain's `lib` directory to everything it spawns, and the loader searches that
ahead of the binary's own path.

Whether it matters depends on the project, because a rustc shared library is named for its Rust version
rather than a content hash. A project on a *different* version collides with nothing. A project pinning
the **same** version as the tool's own nightly has a library with exactly the name the Nix `rustc` is
looking for, so it wins the lookup — and being a non-Nix build, it then wants a system `libz.so.1` the
Nix loader cannot resolve. The failure therefore appears in whichever project happens to track the same
Rust version as the tool, which is the opposite of the intuition that a closely matched toolchain is
the safe case.

The fix ships in the flake, whose wrapper puts the pinned toolchain's `lib` first. Upgrade the install
(`nix profile upgrade cargo-cgp`, or remove and re-add it). To confirm the diagnosis before upgrading,
run the probe with that directory in front — it should succeed where the bare command failed:

```sh
NIX_RUSTC=/nix/store/…-rust-minimal-…/bin/rustc
LD_LIBRARY_PATH=$(dirname "$NIX_RUSTC")/../lib "$NIX_RUSTC" --print sysroot
```

A rustup-managed install does not hit this.

## The preflight rejects the setup

Before each check the front-end verifies the toolchain and the driver, read-only. Every failure names
`cargo cgp setup` as the fix, and the messages tell the cases apart.

The pinned nightly is not installed:

```text
cargo-cgp: toolchain `nightly-2026-07-16` is not available (exit status: 1)

The pinned toolchain is not installed. Run `cargo cgp setup`.
```

The toolchain is there, but the driver cannot run under it — almost always a driver built against a
different nightly:

```text
cargo-cgp: the cargo-cgp-driver could not run under toolchain `nightly-2026-07-16`
(it was likely built against a different nightly). Run `cargo cgp setup`.
```

The driver runs but the two are out of lockstep — a partial upgrade, or a stale binary earlier on
`PATH`:

```text
the installed cargo-cgp-driver is version 0.1.0, but this cargo-cgp is 0.2.0 (the two are out of lockstep)

Run `cargo cgp setup`.
```

Every one of these is resolved by `cargo cgp setup`, which reinstalls the pinned toolchain and rebuilds
the driver against it. If you are deliberately running an unprovisioned build, set `CARGO_CGP_NO_MANAGE`
to skip the preflight — but then keeping the toolchain matched is your responsibility, per
[the section above](#the-driver-cannot-load-the-compiler).

## Provisioning fails

`cargo cgp setup` manages toolchains through rustup, so on a machine without it the command stops
plainly rather than failing obscurely:

```text
rustup was not found on PATH; cargo-cgp requires rustup to manage toolchains
```

Install through the [Nix flake](./installation.md#with-nix) instead, which provisions the toolchain at
build time and needs no rustup.

## Still stuck?

The root cause of most of these is a single thing: **a mismatch between the nightly the driver embeds
and the toolchain present when it runs.** Several different-looking errors trace back to it, so if you
are between sections, run `cargo-cgp-driver --version` and compare its `pinned-toolchain:` line against
`rustc --version` in the directory where the check fails.

Failures that are not covered here are worth reporting on the
[issue tracker](https://github.com/contextgeneric/cargo-cgp/issues), with the output of
`cargo cgp check -v` and `cargo-cgp-driver --version`.

---

*This page was written by an AI agent from the CGP knowledge base — see
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
