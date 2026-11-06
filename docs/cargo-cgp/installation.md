---
title: 'Install cargo-cgp with cargo or Nix'
sidebar_label: 'Installation'
sidebar_position: 2
description: 'Install cargo-cgp v0.1.0 with cargo and rustup or with Nix, check that it runs, update it, use it in CI, and remove it again.'
---

# Installation

To install [`cargo-cgp`](./index.md), choose the path that fits your existing Rust environment:
cargo with rustup, or the Nix flake. This page covers version 0.1.0, built for CGP v0.8.

The cargo path installs the command first, then prepares the compiler it needs. `cargo-cgp` runs
a separate `cargo-cgp-driver`, which embeds the Rust compiler and must match one exact nightly.
Setup chooses that nightly for you. It is used for the tool's checks without changing your
project's toolchain.

## Which path to use

Match what your machine already has to the path for it:

| You have | Use | What it does |
|---|---|---|
| rustup | [With cargo](#with-cargo) | Installs the small front end, then provisions the nightly and the driver |
| Nix | [With Nix](#with-nix) | Builds both binaries against the pinned nightly; no rustup needed |
| Nix, and want to try it once | [Without installing](#without-installing) | Runs the tool from the flake, putting nothing on your `PATH` |
| Neither | [Install rustup or Nix first](#neither-rustup-nor-nix) | No packaged path fits |

## With cargo

Install the front end, then let it prepare the matching driver and nightly:

```sh
cargo install --locked cargo-cgp
cargo cgp setup
```

The first command builds an ordinary binary using your existing toolchain, which must be
Rust 1.96 or newer. Its `--locked` flag keeps the dependency versions tested with the release.
The compiler-linked driver is built separately in the setup step.

During setup, rustup installs the pinned nightly with `rustc-dev` and `llvm-tools`. The tool then
builds the matching version of `cargo-cgp-driver` under that nightly and puts it beside the front
end in `~/.cargo/bin`. Allow time for both the toolchain download, which is more than a gigabyte,
and local driver compilation. This step needs rustup and a network connection.

Once setup finishes, ordinary checks only test that the toolchain and driver are present and
match. They do not download or install anything. If that read-only test fails, the message tells
you to run `cargo cgp setup` again; [Troubleshooting](./troubleshooting.md#the-preflight-rejects-the-setup)
explains the possible failures.

To read what `setup` or `update` will do, pass `--help`. Neither command accepts other options:
an unrecognized argument produces an error before installation begins.

## With Nix

With Nix, the flake builds both binaries against the pinned nightly and wraps them with the
library paths they need. This supplies the complete setup without rustup:

```sh
nix profile install github:contextgeneric/cargo-cgp/v0.1.0
```

Including `/v0.1.0` keeps this installation on the documented release. Omitting it selects the
repository's default branch instead, which is appropriate when following development rather
than a release.

To pin the tool inside another project's flake, add it as an input and take its default package:

```nix
inputs.cargo-cgp.url = "github:contextgeneric/cargo-cgp/v0.1.0";
# then, in a devShell or CI derivation:
#   packages = [ cargo-cgp.packages.${system}.default ];
```

## Without installing

To try the tool on a project without installing anything, run the flake's default app from that
project's directory. Everything after `--` is passed to the tool, so this runs `cargo cgp check`:

```sh
cd /path/to/your/project
nix run github:contextgeneric/cargo-cgp/v0.1.0 -- check
```

## Neither rustup nor Nix

An operating-system Rust package alone does not supply the matching nightly and compiler
libraries the driver needs. The installation paths here obtain those through rustup or the Nix
flake. If you use neither, the usual next step is installing rustup alongside your packaged Rust.
CGP itself still works without either installer on stable Rust 1.89 or newer.

## Checking your install

To confirm the front end is in place, ask it for its version:

```sh
cargo cgp --version
```

```text
cargo-cgp 0.1.0
```

`cargo cgp --help` prints the same line first, followed by the list of commands.

To test whether the driver itself can start, ask for its version too. It must load the compiler's
libraries before answering, so this query also tests library loading:

```sh
cargo-cgp-driver --version
```

With a cargo installation, running the driver directly uses your default toolchain's environment.
It can therefore fail to load a library even when checks work: the front end normally supplies the
pinned nightly's library path. To supply that path yourself, run:

```sh
LD_LIBRARY_PATH="$(rustc +nightly-YYYY-MM-DD --print sysroot)/lib" cargo-cgp-driver --version
```

Here `nightly-YYYY-MM-DD` is the pinned nightly, the one `cargo cgp setup` installed and
`rustup toolchain list` now shows. A Nix-installed driver needs no library path and runs as it is. On
success, the driver prints three lines:

```text
cargo-cgp-driver 0.1.0
pinned-toolchain: nightly-2026-11-05
built-against-rustc: rustc 1.101.0-nightly (2d20b42b8 2026-11-04)
```

Read the nightly from `pinned-toolchain:` and the compiler identity from `built-against-rustc:`
in your own output. The dates in this example may differ from those in your installation.

## Supported platforms and CGP versions

The tool is tested on Linux, and it has not yet been tested on macOS or Windows.

The supported CGP target is v0.8. Recognition depends on CGP's trait and crate names, so older
CGP code is still compiled as it would be under `cargo check`, but rewriting its errors has
not been tested.

## Updating

On the cargo path, one command updates the tool:

```sh
cargo cgp update
```

The update command checks the published versions on crates.io and leaves an up-to-date install
alone. If it finds a newer version, it reinstalls the front end and runs that version's `setup`,
bringing the driver and pinned nightly into agreement with it.

The Nix path has no `cargo cgp update`. Instead, change the version in the flake reference and reinstall
the profile, or run `nix flake update cargo-cgp` in a flake that pins the tool as an input.

## Using it in CI

In CI, keep your own toolchain's `cargo check` as the build gate and add the tool for readable
diagnostics. Pin its installation so later runs use the same version: either
`cargo install cargo-cgp --version 0.1.0 --locked` followed by `cargo cgp setup`, or the tagged
Nix reference.

A fresh CI machine repeats the nightly download and driver compilation. If the service supports
caching, retaining `~/.cargo/bin` and the pinned nightly in `~/.rustup/toolchains/` avoids that
part of setup on later runs. These instructions have not been tested with a particular CI service.

## Uninstalling

There is no `cargo cgp uninstall`, so remove the binaries with whatever placed them:

```sh
cargo uninstall cargo-cgp cargo-cgp-driver    # cargo path: both, since setup installed the driver
nix profile remove cargo-cgp                  # Nix path
```

Removing the binaries leaves the nightly installed, since other tools may use it. If nothing
else needs it, take the name from the `pinned-toolchain:` line of `cargo-cgp-driver --version`
and remove it with rustup:

```sh
rustup toolchain uninstall nightly-YYYY-MM-DD
```

Project build artifacts also remain after uninstalling the binaries. A normal `cargo clean`
removes them along with the rest of the project's target directory. To keep the normal build and
remove only the tool's artifacts, run `cargo clean --target-dir target/cgp` from the project root.

## From source

To run the tool from a checkout, for example to work on it, clone and build. The repository's
`rust-toolchain.toml` selects the pinned nightly for you, which needs rustup:

```sh
git clone https://github.com/contextgeneric/cargo-cgp
cd cargo-cgp
cargo build
```

To use that build in another project, point the front end at the built driver and disable the
preflight that expects an installed setup. Disabling management also stops the automatic toolchain
switch, so select the pinned nightly yourself to let the driver load its libraries:

```sh
RUSTUP_TOOLCHAIN=nightly-YYYY-MM-DD CARGO_CGP_NO_MANAGE=1 \
  CARGO_CGP_DRIVER=/path/to/cargo-cgp/target/debug/cargo-cgp-driver \
  /path/to/cargo-cgp/target/debug/cargo-cgp check
```

The pinned nightly is the `channel` in the checkout's `rust-toolchain.toml`. The Nix flake handles all of
this for you, so it is usually the easier way to run a checkout: from the project you want to check, run
`nix run /path/to/cargo-cgp -- check`. The [command reference](./command-reference.md#environment-variables)
describes both variables.

## A coding assistant can set it up

If you work with a coding assistant that has [CGP's agent skill](/docs/ai/skills) loaded, it checks
whether `cargo-cgp` is installed before debugging a CGP error and recommends it when it is missing. The
skill tells the assistant to ask before installing, because setup downloads a toolchain and builds a
compiler-linked binary, and to mention Nix only when you use it.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
