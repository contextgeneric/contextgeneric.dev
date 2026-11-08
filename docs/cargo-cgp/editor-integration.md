---
title: 'Use cargo-cgp in VS Code with rust-analyzer'
sidebar_label: 'Editor integration'
sidebar_position: 6
description: 'Make rust-analyzer run cargo cgp check when you save, so VS Code shows CGP errors with their root cause first, and learn what the switch changes.'
---

# Editor integration

You can have rust-analyzer run `cargo cgp check` when you save, bringing the rewritten CGP errors
into your editor. One setting replaces its usual `cargo check` command, so errors from the check
lead with the same root causes as they do in a terminal.

This page covers VS Code with the
[rust-analyzer extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
Before configuring it, [install](./installation.md) the [`cargo-cgp`](./index.md) toolchain for
[Context-Generic Programming (CGP)](/docs/) and confirm that `cargo cgp check` works in a terminal
inside your project.

## Set it up

Open the project's `.vscode/settings.json` through **Preferences: Open Workspace Settings (JSON)**
in the command palette. Add the override to this workspace settings file at the project root:

```json
{
  "rust-analyzer.check.overrideCommand": [
    "cargo", "cgp", "check", "--workspace", "--all-targets", "--message-format=json"
  ]
}
```

The next time you save a Rust file, rust-analyzer runs this command instead of `cargo check`. If the
errors you see do not change, run **rust-analyzer: Restart server** from the command palette.

Each part of the command has a job:

- **`cargo cgp check`:** the check itself, the same command you run in a terminal.
- **`--workspace`:** checks every crate in the workspace, as rust-analyzer's own check does.
- **`--all-targets`:** includes tests, examples, and benchmarks, so an error in a test shows up too.
- **`--message-format=json`:** makes cargo print its errors as JSON, which is the only form rust-analyzer
  reads. The tool writes its rewritten errors in the same JSON format as the compiler, so nothing else is
  needed for them to reach the editor.

The full command belongs in
[`check.overrideCommand`](https://rust-analyzer.github.io/book/configuration.html#check.overrideCommand),
which runs it as written. The shorter
[`check.command`](https://rust-analyzer.github.io/book/configuration.html#check.command) setting
accepts only a cargo subcommand such as `clippy`. Giving it `cgp` would put rust-analyzer's flags
directly after `cgp`, where the tool expects `check`.

### Workspace or user settings

Workspace settings limit this check to the project where you enable it.
[User settings](https://code.visualstudio.com/docs/configure/settings) instead apply across projects,
including ones without CGP. Those projects still check correctly, but incur a separate nightly
build with no CGP diagnostics to rewrite. Since workspace settings override user settings, a
project can also enable or disable the command locally.

Sharing the settings file also makes installation a requirement for collaborators' save checks.
Only commit `.vscode/settings.json` if everyone opening the project has the tool. Otherwise,
rust-analyzer warns on each save instead of showing check errors until they install it or remove
the override.

## What the editor shows

Rewritten check errors use the same editor locations as ordinary check errors: file underlines,
the Problems panel, and the hover over an underline. In the
[Check page's program](./check.md#try-it-on-a-deliberate-mistake), the underline marks `area` in
`rectangle.area()`. Its diagnostic identifies the absent `height` field required by the provider:

```text
[CGP-E001] missing field `height` on `Rectangle`
root cause: [CGP-E110] missing field `height` on `Rectangle`
this is required through the dependency chain:
  [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
  └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
    └─ [CGP-E110] missing field `height` on `Rectangle`
add the field `height: f64` to `Rectangle`
```

The compiler's own code, `E0599` here, stays attached to the error, as it does in a terminal.
[Reading the output](./reading-output.md) explains each part.

## What changes when the editor runs the tool

The override changes how the check runs as well as the wording of its diagnostics. These
changes explain some differences you may notice after enabling it.

The first save can take longer because the pinned nightly builds every dependency into its own
`target/cgp` directory. Later saves reuse that build. Keeping it separate also lets the editor
check while ordinary terminal builds use their own target directory, without either waiting on
the other's cargo lock.

Because the override runs exactly as written, rust-analyzer does not add its usual check options.
Settings such as [`check.features`](https://rust-analyzer.github.io/book/configuration.html#check.features),
`check.allTargets`, `check.workspace`, and `check.extraArgs` therefore need corresponding flags
in the command itself. For example, add `"--all-features"` or `"--features", "my-feature"` after
`"--all-targets"`.

Alongside the check, rust-analyzer analyzes code as you type using its own engine. Errors from
that analysis do not pass through the tool, so their wording stays the same. The rewritten
compiler diagnostics arrive after saving, when the override command runs.

The separate compiler can also make the editor's result differ from your normal build.
The tool uses its pinned nightly and Rust's next-generation trait solver, which occasionally
accept or reject code differently from your own toolchain. Keep `cargo check` on that toolchain
as the build authority; the [Check page](./check.md#what-a-check-does-differently) explains the
distinction.

## Turning it off

Delete the `rust-analyzer.check.overrideCommand` entry from the settings file you added it to.
rust-analyzer goes back to running `cargo check` on save.

## When it does not work

If the check command fails to start, rust-analyzer reports a warning beginning
`cargo check failed to start`, even though you configured a different command. The rest contains
the command's output. For a missing setup, that includes the tool's own diagnostic:

```text
cargo check failed to start: Cargo watcher failed, the command produced no valid metadata (exit code: ExitStatus(unix_wait_status(256))):
error: toolchain `nightly-2026-11-05` is not available:

  error: toolchain 'nightly-2026-11-05-x86_64-unknown-linux-gnu' is not installed
  help: run `rustup toolchain install nightly-2026-11-05-x86_64-unknown-linux-gnu` to install it

help: run `cargo cgp setup`
```

The nightly and platform in your warning depend on your installation.
[Troubleshooting](./troubleshooting.md) explains these messages and their fixes. To separate a
tool failure from editor configuration, run the same check in a terminal at the opened project's root:

```sh
cargo cgp check --workspace --all-targets
```

If the warning instead ends with cargo's `no such command` error, the editor cannot find the tool:

```text
error: no such command: `cgp`
```

If the terminal command works but the editor reports this missing-command error, compare their
`PATH` environments. A desktop launcher may start VS Code without the shell setup that adds the
tool's location, such as a Nix profile. Starting VS Code from a terminal where
`cargo cgp --version` works lets it inherit that environment. Save again to retry the check.

If only some errors are rewritten, you are seeing the tool's limit rather than the editor's.
`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet reshape
every class. [Reading the output](./reading-output.md#errors-that-pass-through-unchanged) lists the errors you
are most likely to see as the compiler wrote them.

## What this does not do

Only the on-save check uses this command. Completion, hover types, go-to-definition, and
as-you-type diagnostics come from rust-analyzer's own analysis, so this override does not
change them.

Verification covered the rust-analyzer language server on Linux, started with the configuration
VS Code would pass. Saving the example produced the rewritten diagnostic, the quoted startup
warnings, and the observed omission of rust-analyzer's other check options. The VS Code window
itself was not tested. Other rust-analyzer editors can express `check.overrideCommand` in their
own configuration formats, but those editors, macOS, and Windows have not been tried.

---

*An AI agent wrote this page using the CGP knowledge base, and checked its setup against the
rust-analyzer language server. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
