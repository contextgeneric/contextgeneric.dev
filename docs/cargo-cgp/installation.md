---
title: 'Install cargo-cgp with cargo or Nix'
sidebar_label: 'Installation'
sidebar_position: 2
description: 'Install cargo-cgp v0.1.0 with cargo and rustup or with Nix, check that it runs, update it, use it in CI, and remove it again.'
---

# Installation

Install [`cargo-cgp`](./index.md) with cargo if your Rust comes from rustup, or with Nix if you use
Nix. This page covers version 0.1.0 of the tool, which is built for CGP v0.8.

On the cargo path, installing takes two commands, because the tool comes in two parts. `cargo-cgp` is
the cargo subcommand you type. `cargo-cgp-driver` is the program it runs in place of the compiler, and
it embeds the compiler itself, so it has to be built against one exact nightly toolchain. You never
type that nightly's date, and **your project does not switch to it**: the tool uses the nightly for its
own check only.

## Which path to use

Match what your machine already has to the path for it:

| You have | Use | What it does |
|---|---|---|
| rustup | [With cargo](#with-cargo) | Installs the small front end, then provisions the nightly and the driver |
| Nix | [With Nix](#with-nix) | Builds both binaries against the pinned nightly; no rustup needed |
| Nix, and want to try it once | [Without installing](#without-installing) | Runs the tool from the flake, putting nothing on your `PATH` |
| Neither | [Install rustup or Nix first](#neither-rustup-nor-nix) | No packaged path fits |

## With cargo

```sh
cargo install cargo-cgp
cargo cgp setup
```

**`cargo install cargo-cgp` installs the front end.** It is an ordinary binary with no compiler
inside, so it builds with whatever toolchain you already have.

**`cargo cgp setup` does the rest.** It asks rustup to install the pinned nightly with the `rustc-dev`
and `llvm-tools` components, then builds `cargo-cgp-driver` at the same version as the front end under
that nightly, and places it beside the front end in `~/.cargo/bin`. It needs rustup and a network
connection, and it takes a while: the toolchain is more than a gigabyte, and the driver is compiled on
your machine.

Run `setup` once after installing, and again only when the tool tells you to. Before every check the
front end runs a quick, read-only test of the toolchain and the driver. If either is missing or the two
do not match, it stops with a message naming `cargo cgp setup` as the fix, rather than downloading
anything on its own. [Troubleshooting](./troubleshooting.md#the-preflight-rejects-the-setup) lists those
messages.

`setup` and `update` take no options. Each answers `--help` with what it does instead of running, and
refuses any other argument rather than starting an install you did not ask for.

## With Nix

The flake builds both binaries against the pinned nightly and wraps them so they find its libraries
without rustup. A machine with only Nix can run the tool this way.

```sh
nix profile install github:contextgeneric/cargo-cgp/v0.1.0
```

The `/v0.1.0` suffix pins the install to that release. Without it, the flake tracks the repository's
default branch, which is what you want only if you are following the tool's development.

To pin the tool inside another project's flake, add it as an input and take its default package:

```nix
inputs.cargo-cgp.url = "github:contextgeneric/cargo-cgp/v0.1.0";
# then, in a devShell or CI derivation:
#   packages = [ cargo-cgp.packages.${system}.default ];
```

## Without installing

To try the tool on a project without installing anything, run the flake's default app from that
project's directory. Everything after `--` is passed to the tool, so this is `cargo cgp check`:

```sh
cd /path/to/your/project
nix run github:contextgeneric/cargo-cgp/v0.1.0 -- check
```

## Neither rustup nor Nix

**If your Rust comes from somewhere else, such as an operating system package, neither path works as
it stands.** The driver has to be built against one exact nightly with the compiler's development
libraries, and only rustup or the Nix flake provides that. Installing rustup alongside a packaged Rust
is the usual way forward. CGP itself does not need either: it compiles on any stable Rust 1.89 or
newer.

## Checking your install

Ask the front end for its version:

```sh
cargo cgp --version
```

```text
cargo-cgp 0.1.0
```

`cargo cgp --help` prints the same line first, followed by the list of commands.

The driver has its own version query, and it doubles as a test that the driver can run at all, because
the driver loads the compiler's libraries before printing anything:

```sh
cargo-cgp-driver --version
```

On the cargo path that command runs under your default toolchain rather than the pinned nightly, so it
can fail with a library-loading error even when the install is fine: the front end sets up the
library path for the driver every time it runs it. Run the probe the way the front end would:

```sh
LD_LIBRARY_PATH="$(rustc +nightly-YYYY-MM-DD --print sysroot)/lib" cargo-cgp-driver --version
```

where `nightly-YYYY-MM-DD` is the pinned nightly, the one `cargo cgp setup` installed and
`rustup toolchain list` now shows. A Nix-installed driver needs no library path and runs as it is. On
success it prints three lines:

```text
cargo-cgp-driver 0.1.0
pinned-toolchain: nightly-2026-09-14
built-against-rustc: rustc 1.100.0-nightly (4b6d04e70 2026-09-13)
```

The second line names the nightly the driver needs, and the third the compiler it was built with. Your
dates may differ from these; the lines are the way to read them.

## Supported platforms and CGP versions

**The tool is tested on Linux.** It has not yet been tested on macOS or Windows.

**It is built for CGP v0.8.** The tool recognizes CGP's errors by the names of the traits and crates
CGP defines, so a project on an older CGP version compiles under it as it would under `cargo check`,
but whether its errors are reshaped has not been tested.

## Updating

On the cargo path, one command updates the tool:

```sh
cargo cgp update
```

It looks up the published versions on crates.io and does nothing if you already have the newest. When
there is a newer one, it reinstalls the front end and runs the new version's `setup`, so the driver and
the pinned nightly move with it. It stays within your release channel: a stable install moves only to
stable releases, and a pre-release install to newer pre-releases or a stable release.

On the Nix path there is no `cargo cgp update`. Change the version in the flake reference and reinstall
the profile, or run `nix flake update cargo-cgp` in a flake that pins the tool as an input.

### Upgrading from the v0.1.0-alpha pre-release

**An install of the `v0.1.0-alpha` pre-release cannot update itself to 0.1.0.** `update` runs the code
of the version you have, and the pre-release only ever looks for newer pre-releases. Reinstall instead:

```sh
cargo install cargo-cgp
cargo cgp setup
```

On the Nix path, install the `v0.1.0` reference as above.

## Using it in CI

**Keep your own toolchain's `cargo check` as the gate, and add the tool for readable failures.** The
commands are the same as on your machine; pin them so a build does not change under you, either with
`cargo install cargo-cgp --version 0.1.0 --locked` followed by `cargo cgp setup`, or with the tagged
Nix reference.

On the cargo path every fresh machine downloads the pinned nightly and compiles the driver, which is
the slowest part of the install. If your CI service can cache directories between runs, caching
`~/.cargo/bin` and the pinned nightly under `~/.rustup/toolchains/` avoids repeating that. This page does
not cover any particular CI service, and these setups have not been run there.

## Uninstalling

Remove the binaries with whatever placed them; there is no `cargo cgp uninstall`.

```sh
cargo uninstall cargo-cgp cargo-cgp-driver    # cargo path: both, since setup installed the driver
nix profile remove cargo-cgp                  # Nix path
```

Neither removes the pinned nightly, since another tool may use the same toolchain. Remove it yourself
when nothing else needs it; its name is the `pinned-toolchain:` line of `cargo-cgp-driver --version`:

```sh
rustup toolchain uninstall nightly-YYYY-MM-DD
```

Neither removes the `target/cgp` directories the tool built in your projects either. Delete them as you
would any build output, for example with `cargo clean --target-dir target/cgp` from the project's root.

## From source

To run the tool from a checkout, for example to work on it, clone and build. The repository's
`rust-toolchain.toml` selects the pinned nightly for you, which needs rustup:

```sh
git clone https://github.com/contextgeneric/cargo-cgp
cd cargo-cgp
cargo build
```

Then run the built front end in another project, pointing it at the built driver and turning off the
read-only test, which expects an installed setup:

```sh
CARGO_CGP_NO_MANAGE=1 CARGO_CGP_DRIVER=/path/to/cargo-cgp/target/debug/cargo-cgp-driver \
  /path/to/cargo-cgp/target/debug/cargo-cgp check
```

With the test turned off, the tool does not switch to the pinned nightly for you either, so the project
you check must be using that nightly too, or the driver will fail to load. The Nix flake handles this
for you, so it is usually the easier way to run a checkout: from the project you want to check, run
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
