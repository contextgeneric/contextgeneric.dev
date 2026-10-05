---
title: 'Use cargo-cgp in VS Code with rust-analyzer'
sidebar_label: 'Editor integration'
sidebar_position: 6
description: 'Make rust-analyzer run cargo cgp check when you save, so VS Code shows CGP errors with their root cause first, and learn what the switch changes.'
---

# Editor integration

rust-analyzer runs `cargo check` every time you save a file and shows the errors in your editor. One
setting makes it run `cargo cgp check` instead, so the CGP errors VS Code underlines lead with their
root cause, exactly as they do in a terminal. This page covers VS Code with the
[rust-analyzer extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
It is part of [`cargo-cgp`](./index.md), the error toolchain for [CGP](/docs/), and it assumes the
tool is [installed](./installation.md) and that `cargo cgp check` already works in a terminal in your
project.

## Set it up

**Add the setting to your project's workspace settings**, the `.vscode/settings.json` file at the
project's root. Open it with **Preferences: Open Workspace Settings (JSON)** from the command palette,
and add:

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

- **`cargo cgp check`** is the check itself, the same command you run in a terminal.
- **`--workspace`** checks every crate in the workspace, as rust-analyzer's own check does.
- **`--all-targets`** includes tests, examples, and benchmarks, so an error in a test shows up too.
- **`--message-format=json`** makes cargo print its errors as JSON, which is the only form
  rust-analyzer reads. The tool writes its rewritten errors in the same JSON format as the compiler,
  so nothing else is needed for them to reach the editor.

**The command goes in `check.overrideCommand`, not `check.command`.** The
[`check.command`](https://rust-analyzer.github.io/book/configuration.html#check.command) setting takes
a single cargo subcommand, such as `clippy`, and rust-analyzer puts its own flags straight after it,
so `cgp` would be followed by `--workspace` rather than by `check`.
[`check.overrideCommand`](https://rust-analyzer.github.io/book/configuration.html#check.overrideCommand)
takes the whole command line and runs it as written.

### Workspace or user settings

**Prefer the workspace settings, so the tool runs only on projects that use CGP.**
[User settings](https://code.visualstudio.com/docs/configure/settings) apply to every project you open.
The tool checks a project without CGP correctly, but it still builds that project a second time under
its pinned nightly, which costs time and disk on a project that has no CGP errors to rewrite. Workspace settings
override user settings, so a project can also turn the check on or off for itself.

**Commit `.vscode/settings.json` only if everyone who opens the project has the tool.** On a machine
without `cargo-cgp`, the check fails on every save, so that person gets a warning from rust-analyzer
instead of their check errors until they install the tool or remove the setting.

## What the editor shows

The errors appear where `cargo check`'s errors would: underlined in the file, listed in the Problems
panel, and in the hover over the underline. Their text is what `cargo cgp check` prints in a terminal.
For the program on the [Check page](./check.md#try-it-on-a-deliberate-mistake), where `Rectangle` lacks
the `height` field its area provider reads, the underline sits on `area` in `rectangle.area()`, and the
message is the rewritten one:

```text
[CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
root cause: [CGP-E106] missing field `height` on `Rectangle`
this is required through the dependency chain:
  [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
  └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
    └─ [CGP-E106] missing field `height` on `Rectangle`
```

The compiler's own code, `E0599` here, stays attached to the error, as it does in a terminal.
[Reading the output](./reading-output.md) explains each part.

## What changes when the editor runs the tool

Running the tool in place of `cargo check` changes four things about how rust-analyzer behaves:

- **The first save is slow.** The check builds into its own `target/cgp` directory under the tool's
  pinned nightly, so the first one compiles every dependency again. Later saves reuse that build, and
  because it is separate from `target`, your terminal builds and the editor's checks do not undo each
  other's work.
- **rust-analyzer's own check options are ignored.** With an override in place, rust-analyzer runs
  the command exactly as written, so settings such as
  [`check.features`](https://rust-analyzer.github.io/book/configuration.html#check.features),
  `check.allTargets`, and `check.extraArgs` do not reach the check. Put the flags you need in the
  command itself, for example `"--all-features"` or `"--features", "my-feature"` after
  `"--all-targets"`.
- **Two kinds of error appear, and only one is rewritten.** rust-analyzer also analyzes your code as
  you type, with its own engine and without running any compiler, and reports some errors of its own.
  Those come from rust-analyzer rather than from the check, so they read as they always have. The
  rewritten ones are the errors that arrive after a save.
- **The check and your toolchain can disagree.** The check compiles under the tool's pinned nightly
  with Rust's next-generation trait solver, so on rare code the editor can show an error your own
  `cargo check` does not, or the reverse. Your toolchain's `cargo check` is the authority on whether
  the code compiles, as the [Check page](./check.md#what-a-check-does-differently) explains.

## Turning it off

Delete the `rust-analyzer.check.overrideCommand` entry from the settings file you added it to.
rust-analyzer goes back to running `cargo check` on save.

## When it does not work

**When the command fails, rust-analyzer says so in a warning.** The warning begins
`cargo check failed to start`, whatever the command is, and ends with what the command printed. A
missing setup, for example, arrives as the tool's own message:

```text
cargo check failed to start: Cargo watcher failed, the command produced no valid metadata (exit code: ExitStatus(unix_wait_status(256))):
cargo-cgp: toolchain `nightly-2026-09-14` is not available (exit status: 1)

The pinned toolchain is not installed. Run `cargo cgp setup`.
```

Your message names the nightly your build is pinned to. Look the message up on
[Troubleshooting](./troubleshooting.md), which lists each one with its fix. To tell whether the problem
is the tool or the editor, run the same command in a terminal from the root of the project you opened
in VS Code:

```sh
cargo cgp check --workspace --all-targets
```

**If the warning ends with cargo's `no such command` error, the editor cannot find the tool:**

```text
error: no such command: `cgp`
```

When the terminal command works, VS Code is running with a different `PATH` from your terminal, so the
`cargo` it starts does not see the tool. This happens most often when VS Code is started from a desktop
launcher rather than from a shell, and the tool is installed somewhere only your shell's start-up files
add to `PATH`, as with a Nix profile. Start VS Code from a terminal in which `cargo cgp --version`
works, and save again.

**If only some errors are rewritten**, that is the tool's limit rather than the editor's.
`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet
reshape every class. [Reading the output](./reading-output.md#errors-that-pass-through-unchanged) lists
the errors that arrive as the compiler wrote them.

## What this does not do

**It changes only the errors from the check that runs on save.** Completion, hover types,
go-to-definition, and rust-analyzer's own as-you-type errors are unaffected, because none of them run
the check.

**It has been tested on Linux, with the rust-analyzer language server.** The setting was verified by
running the server the way VS Code runs it, saving the program above, and reading what the server
reported: the rewritten error, the warnings quoted above, and the check options it leaves out
under the override. The VS Code window itself was not part of that test. Other editors that use rust-analyzer accept the
same setting, `check.overrideCommand`, in their own configuration format, but none of them has been
tried, and neither have macOS and Windows.

---

*An AI agent wrote this page using the CGP knowledge base, and checked its setup against the
rust-analyzer language server. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
