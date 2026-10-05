---
title: 'cargo-cgp command reference: commands, options, variables'
sidebar_label: 'Command reference'
sidebar_position: 8
description: 'Every cargo-cgp command and option, the environment variables it reads and sets, the files it creates, and its exit status, in one place.'
---

# Command reference

This page lists everything [`cargo-cgp`](./index.md) version 0.1.0 accepts and changes: its commands,
their options, the environment variables it reads and sets, the files it creates, and its exit
status. The other pages in this section explain how to use them.

## Running it

The tool is a cargo subcommand, so it runs either through cargo or on its own:

```sh
cargo cgp <COMMAND> [ARGS]...
cargo-cgp <COMMAND> [ARGS]...
```

Two options stand in for a command:

| Option | Effect |
|---|---|
| `-h`, `--help` | Prints the version, the list of commands, and a line on each. Running `cargo cgp` with no command does the same. |
| `--version` | Prints the version of the tool. |

## Commands

The tool has four commands. Two read your code, and two install and update the tool itself:

| Command | Does | Details |
|---|---|---|
| `check` | Runs `cargo check` and rewrites the CGP errors it recognizes | [Check](./check.md) |
| `expand` | Prints the Rust one target's macros generate | [Expand](./expand.md) |
| `setup` | Installs the pinned nightly and builds the driver | [Installation](./installation.md#with-cargo) |
| `update` | Upgrades the tool to the newest published version | [Installation](./installation.md#updating) |

### `check`

```sh
cargo cgp check [CARGO CHECK ARGS]...
```

**Every argument is passed to `cargo check`**, which also validates them, so the command takes every
`cargo check` flag and no flags of its own. `cargo cgp check --help` prints `cargo check`'s help and
does not run a check. Two flags matter more than the rest:

- **`--target-dir <DIR>`** sets where the build goes. Without it, and without `CARGO_TARGET_DIR`, the
  tool adds `--target-dir target/cgp`, relative to the directory you run it from.
- **`--message-format=json`** prints the errors as JSON, rewritten the same way as the text output. An
  editor needs this form; see [Check](./check.md#in-your-editor).

### `expand`

```sh
cargo cgp expand [--item <PATH>] [CARGO RUSTC ARGS]...
```

| Option | Effect |
|---|---|
| `--item <PATH>` | Prints only one module or item instead of the whole target. The path is `::`-separated, names something inside the crate, and may start with `crate::`. A module gives its contents, a type its declaration and every impl written for it, and a trait its definition and every impl of it. |
| `-h`, `--help` | Prints the help for `expand`, including `--item`. |

**Every other argument is passed to `cargo rustc`**, so choosing the target, with `--lib`, `--bin`,
`-p`, or `--features`, is cargo's job. A package with more than one target needs `--lib` or
`--bin <NAME>`. Unless you pass `--profile`, the tool builds under cargo's `check` profile. The
expansion goes to standard output, and everything else goes to standard error.

### `setup`

```sh
cargo cgp setup
```

Installs the pinned nightly through rustup, with the `rustc-dev` and `llvm-tools` components, then
builds `cargo-cgp-driver` at the front end's version under that nightly and installs it beside the
front end. Running it again when both are present does no harm. It needs rustup and a network
connection.

### `update`

```sh
cargo cgp update
```

Looks up the published versions on crates.io and, if there is a newer one in your release channel,
reinstalls the front end and runs the new version's `setup`. It needs a network connection.

**`setup` and `update` take no options, and they ignore any arguments.** `cargo cgp setup --help`
therefore runs setup; use `cargo cgp --help` for help.

## Environment variables

The tool reads these variables, which are meant for running a build of the tool that `setup` did not
install, such as one from a source checkout:

| Variable | Effect |
|---|---|
| `CARGO_CGP_NO_MANAGE` | When set, skips the preflight test of the toolchain and driver, and does not switch to the pinned nightly. The check then runs under whatever toolchain is active, which must be the driver's nightly. |
| `CARGO_CGP_DRIVER` | The path of the driver to run, instead of looking beside the front end. |
| `CARGO_CGP_TOOLCHAIN` | A nightly to use in place of the pinned one, for `setup` and for the checks. The driver must be built against it. |
| `CARGO_TARGET_DIR` | cargo's own variable. When set, the tool does not add `--target-dir target/cgp`. |

For the length of a `check` or `expand`, the tool also sets variables for cargo:

| Variable | Set to |
|---|---|
| `RUSTC_WORKSPACE_WRAPPER` | The driver. This replaces any value you set, so a wrapper of your own given through this variable does not run under the tool. |
| `RUSTUP_TOOLCHAIN` | The pinned nightly, unless `CARGO_CGP_NO_MANAGE` is set. |
| `LD_LIBRARY_PATH` | The `lib` directory of the toolchain the check runs under, added in front of any existing value, so the driver can load the compiler. |

## Files and directories

The tool creates these, and removing it does not remove them; see
[Uninstalling](./installation.md#uninstalling):

| Path | What it holds |
|---|---|
| `~/.cargo/bin/cargo-cgp` | The front end, on the cargo path. |
| `~/.cargo/bin/cargo-cgp-driver` | The driver, which `setup` installs beside the front end. |
| `~/.rustup/toolchains/nightly-…` | The pinned nightly, which `setup` installs through rustup. The driver's `--version` names it. |
| `target/cgp` | The check's build artifacts, relative to the directory the command ran in. |

## Exit status

`check` exits with `cargo check`'s status, so `0` means no errors. `expand` exits with `0` when it
printed an expansion and with a non-zero status when it did not. A failure in the tool itself, such as
a preflight message or an unknown command, exits with `1`.

## The driver

You do not normally run `cargo-cgp-driver` yourself; the front end runs it in place of the compiler.
It answers two queries of its own:

| Option | Effect |
|---|---|
| `--version`, `-V` | Prints three lines: its version, the `pinned-toolchain:` it needs, and the `built-against-rustc:` compiler it was built with. |
| `--help`, `-h` | Prints a short description of the driver. Running it with no arguments does the same. |

Run directly on the cargo path, it needs the pinned nightly's `lib` directory on `LD_LIBRARY_PATH`;
[Troubleshooting](./troubleshooting.md#the-driver-cannot-load-the-compiler) shows the command.

---

*An AI agent wrote this page using the CGP knowledge base, and checked it against the tool's source.
See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
