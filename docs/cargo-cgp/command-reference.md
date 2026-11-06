---
title: 'cargo-cgp command reference: commands, options, variables'
sidebar_label: 'Command reference'
sidebar_position: 9
description: 'Every cargo-cgp command and option, the environment variables it reads and sets, the files it creates, and its exit status, in one place.'
---

# Command reference

This page collects the commands, options, environment variables, files, and exit statuses of
[`cargo-cgp`](./index.md) version 0.1.0. Use it to look up an exact flag or setting; the other
pages in this section develop working examples.

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
| `-V`, `--version` | Prints the version of the tool, as `cargo-cgp 0.1.0`. |

## Commands

Use `check` and `expand` to inspect a project. The other commands, `setup` and `update`, prepare
and maintain the tool's own installation:

| Command | Does | Details |
|---|---|---|
| `check` | Runs `cargo check` and rewrites the CGP errors it recognizes | [Check](./check.md) |
| `expand` | Prints the Rust one target's macros generate | [Expand](./expand.md) |
| `setup` | Installs the pinned nightly and builds the driver | [Installation](./installation.md#with-cargo) |
| `update` | Upgrades the tool to the newest version its release channel allows | [Installation](./installation.md#updating) |

### `check`

```sh
cargo cgp check [CARGO CHECK ARGS]...
```

`check` forwards its arguments to `cargo check`, which validates them. It therefore accepts
cargo's check flags without adding flags of its own, and `cargo cgp check --help` displays cargo's
help instead of checking a project. Two options are especially useful when configuring a check:

- **`--target-dir <DIR>`:** sets exactly where the build goes. Without it, the tool builds into the `cgp`
  subdirectory of your project's own target directory, which it reads from `cargo metadata`, so the location
  is the same wherever in the project you run it.
- **`--message-format=json`:** prints the errors as JSON, rewritten the same way as the text output. An editor
  needs this form, as [Editor integration](./editor-integration.md) explains.

### `expand`

```sh
cargo cgp expand [--item <PATH>] [CARGO RUSTC ARGS]...
```

| Option | Effect |
|---|---|
| `--item <PATH>` | Prints only one module or item instead of the whole target. The path is `::`-separated, names something inside the crate, and may start with `crate::`. A module gives its contents, a type its declaration and every impl written for it, and a trait its definition and every impl of it. |
| `-h`, `--help` | Prints the help for `expand`, including `--item`. |

After handling `--item`, `expand` passes the remaining arguments to `cargo rustc`. Select the
package, target, and features through cargo's options such as `-p`, `--lib`, `--bin`, and
`--features`; a package with multiple targets needs `--lib` or `--bin <NAME>`. The build uses
cargo's `check` profile unless you request `--profile` or `--release`. The expansion is written
to standard output, leaving other messages on standard error.

### `setup`

```sh
cargo cgp setup
```

Setup uses rustup to install the pinned nightly with `rustc-dev` and `llvm-tools`, then builds
the matching `cargo-cgp-driver` and installs it beside the front end. It needs a network connection
and can be run again when the binaries and toolchain are already present.

### `update`

```sh
cargo cgp update
```

Update checks crates.io for a newer version in your release channel. If one is available, it
reinstalls the front end and invokes that version's `setup`, keeping the driver in agreement.
The command needs a network connection.

`setup` and `update` take no options. Each answers `-h` or `--help` with a description of what it does instead
of running, and refuses any other argument with an error naming it.

## Environment variables

When running a build outside the setup-managed installation, such as a source checkout, these
variables select the driver and toolchain environment. `CARGO_TARGET_DIR` also controls where
ordinary project artifacts go:

| Variable | Effect |
|---|---|
| `CARGO_CGP_NO_MANAGE` | When set, skips the preflight test of the toolchain and driver, and does not switch to the pinned nightly. The check then runs under whatever toolchain is active, which must be the driver's nightly. |
| `CARGO_CGP_DRIVER` | The path of the driver to run, instead of looking beside the front end. |
| `CARGO_CGP_TOOLCHAIN` | A nightly to use in place of the pinned one, for `setup`, `check`, and `expand`. The driver must be built against it. It has no effect when `CARGO_CGP_NO_MANAGE` is set. |
| `CARGO_TARGET_DIR` | cargo's own variable. It moves the project's target directory, and the tool builds in that directory's `cgp` subdirectory. |

During `check` and `expand`, the tool passes the following environment to cargo so it uses the
driver and matching compiler libraries:

| Variable | Set to |
|---|---|
| `RUSTC_WORKSPACE_WRAPPER` | The driver. This replaces any value you set, so a wrapper of your own given through this variable does not run under the tool, which prints a warning saying so. |
| `RUSTUP_TOOLCHAIN` | The pinned nightly, unless `CARGO_CGP_NO_MANAGE` is set. |
| `LD_LIBRARY_PATH` | The `lib` directory of the toolchain the check runs under, added in front of any existing value, so the driver can load the compiler. |

## Files and directories

Installation and checks create the files listed below. Uninstalling through cargo removes the
two binaries, while the toolchain and build directories remain; [Uninstalling](./installation.md#uninstalling)
explains how to remove those separately.

| Path | What it holds |
|---|---|
| `~/.cargo/bin/cargo-cgp` | The front end, on the cargo path. |
| `~/.cargo/bin/cargo-cgp-driver` | The driver, which `setup` installs beside the front end. |
| `~/.rustup/toolchains/nightly-…` | The pinned nightly, which `setup` installs through rustup. The driver's `--version` names it. |
| `target/cgp` | The check's build artifacts, inside the project's target directory. |

## Exit status

`check` returns the exit status of `cargo check`, with `0` indicating no errors. For `expand`,
success means reaching the end of macro expansion without an error; type checking has not run,
so that result can include a crate that would fail it.

If an error occurs before expansion completes, the command returns cargo's status even when a
partial listing was printed. The tool returns `1` for an unmatched `--item` path or one of its
own failures, including an unsuccessful preflight or an unknown command.

## The driver

The front end normally invokes `cargo-cgp-driver` for you in place of the compiler. When checking
an installation, you can query the driver directly with these options:

| Option | Effect |
|---|---|
| `--version`, `-V` | Prints three lines: its version, the `pinned-toolchain:` it needs, and the `built-against-rustc:` compiler it was built with. |
| `--help`, `-h` | Prints a short description of the driver. Running it with no arguments does the same. |

With a cargo installation, a direct driver invocation needs the pinned nightly's `lib` directory
on `LD_LIBRARY_PATH`. [Troubleshooting](./troubleshooting.md#the-driver-cannot-load-the-compiler)
shows how to supply it.

---

*An AI agent wrote this page using the CGP knowledge base, and checked it against the tool's source.
See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
