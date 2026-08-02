---
sidebar_position: 2
---

# Installation

`cargo-cgp` is two binaries plus one exact nightly compiler, and installing it means getting all three
in place and matched. The `cargo-cgp` front-end is the subcommand you type; the `cargo-cgp-driver` it
calls links the compiler's internals, so it has to be built against the nightly it embeds.

You never type that nightly date, and **your own project never adopts it** — the tool forces its
toolchain only for its own check, and your project keeps whatever it already uses for ordinary builds.

## Which path to use

| You have | Use | Why |
|---|---|---|
| rustup | [cargo](#with-cargo) | Installs a small front-end, then provisions the rest for you |
| Nix | [Nix](#with-nix) | Builds both binaries against the pinned nightly; no rustup needed |
| Nix, and want to try it once | [without installing](#without-installing) | Runs from the flake in place |
| A coding assistant | [the agent skill](#let-your-coding-assistant-do-it) | It detects the tool is missing and offers to install it |

## With cargo

Two commands, and the first is one you can type from memory because it carries no version in it:

```sh
cargo install cargo-cgp
cargo cgp setup
```

They divide the work deliberately. `cargo install cargo-cgp` builds the front-end, which has no
compiler linkage and so installs under whatever toolchain you already have. `cargo cgp setup` then does
everything heavyweight: it installs the pinned nightly with its `rustc-dev` and `llvm-tools`
components, builds the driver against that exact nightly, and places it beside the front-end. `setup`
needs rustup, since that is what it manages toolchains through.

You run `setup` once, and again only when the tool tells you to. Before each check the front-end runs a
fast read-only preflight; if the driver is missing, the toolchain absent, or the two out of step, it
stops and names `cargo cgp setup` as the fix rather than doing slow or stateful work behind your back.

## With Nix

The flake builds both binaries against the pinned nightly and wraps them so they run without rustup —
so a machine with only Nix can run the tool, and the project you check needs no toolchain of its own.

```sh
nix profile install github:contextgeneric/cargo-cgp
```

Append a released tag to pin an exact version rather than the default branch, which is the form to
prefer when you want the install reproducible:

```sh
nix profile install github:contextgeneric/cargo-cgp/v0.1.0-alpha
```

## Without installing

To try the tool on a project without putting anything on your `PATH`, run the flake's default app from
that project's directory. Everything after `--` is forwarded to the check:

```sh
cd /path/to/your/project
nix run github:contextgeneric/cargo-cgp -- check
```

This is also the form to use in CI. Add a `/v0.1.0-alpha` suffix to pin the release, or leave it off to
track the default branch.

To pin the tool inside another project's own flake, take its `packages.default`:

```nix
inputs.cargo-cgp.url = "github:contextgeneric/cargo-cgp/v0.1.0-alpha";
# then, in a devShell or CI derivation:
#   packages = [ cargo-cgp.packages.${system}.default ];
```

## Let your coding assistant do it

If you work with an LLM assistant that has [CGP's agent skill](/docs/ai/skills) loaded, you do not have
to drive this yourself. The skill tells the assistant to check whether `cargo-cgp` is available before
debugging any CGP compile error, to recommend it when it is missing, and to prefer it over plain
`cargo check` when diagnosing a wiring failure. Asking your assistant to set up CGP tooling is enough.

Two things the skill is deliberately careful about, so you are not surprised:

- **It asks before installing.** Provisioning a nightly toolchain and building a compiler-linked driver
  is heavy, so the skill instructs the assistant to get your approval rather than run `setup` on its
  own.
- **It prefers cargo, and only mentions Nix if you have it.** The skill reaches for the Nix path when a
  `nix` command or a `flake.nix` is actually present, and otherwise does not bring it up.

## Checking what you have

```sh
cargo cgp
```

With no subcommand, the front-end prints its version and the commands it accepts. **Read the command
list rather than the version number**, because the two can disagree: a build from the default branch
still reports the released version while carrying commands that release does not have. If `expand` is
absent from the list, see the note below.

The driver has its own version query, which loads the compiler library before printing and so doubles
as a check that it can run at all:

```sh
cargo-cgp-driver --version
```

On success it prints three lines — its own version, the `pinned-toolchain:` it needs, and the
`built-against-rustc:` compiler it was built with. That first pair is the fastest way to see which
nightly a driver actually wants.

:::note

### `expand` is newer than the published release

`cargo cgp expand` landed after `v0.1.0-alpha` was tagged, so an install from crates.io does not carry
it — `cargo cgp check` is unaffected. Until the next release, get it from the flake with the tag
dropped, which tracks the default branch:

```sh
nix run github:contextgeneric/cargo-cgp -- expand --lib
```

or from a [source build](#from-source).

:::

## From source

To run the current code, clone and build. The repository's own `rust-toolchain.toml` selects the right
nightly automatically:

```sh
git clone https://github.com/contextgeneric/cargo-cgp
cd cargo-cgp
cargo build
```

Then run the freshly built front-end against another project, pointing it at the built driver and
skipping the preflight:

```sh
CARGO_CGP_NO_MANAGE=1 CARGO_CGP_DRIVER=$PWD/target/debug/cargo-cgp-driver \
  /path/to/cargo-cgp/target/debug/cargo-cgp check
```

For most from-source use the Nix flake is easier, because it wires these overrides for you.

## Updating

On the cargo path, one command:

```sh
cargo cgp update
```

It reads the crates.io index, stays within your current release channel — a stable install never jumps
to a pre-release, and a pre-release install stays on pre-releases — and does nothing if you are already
current. When there is something newer it reinstalls the front-end and re-runs `setup` so the driver and
toolchain move with it. On Windows the running binary cannot replace itself, so `update` prints the two
commands to run by hand instead.

On the Nix path there is no `cargo cgp update`: upgrade the profile (`nix profile upgrade`), or run
`nix flake update cargo-cgp` where you pinned it as an input.

## Uninstalling

There is no `cargo cgp uninstall`; you remove the binaries with whatever placed them.

```sh
nix profile remove cargo-cgp                  # Nix path
cargo uninstall cargo-cgp cargo-cgp-driver    # cargo path — both, since setup installed the driver
```

Neither removes the pinned nightly, because another tool may still want that dated toolchain. Remove it
by hand only when nothing else needs it; its name is the `pinned-toolchain:` line of
`cargo-cgp-driver --version`:

```sh
rustup toolchain uninstall <pinned-nightly>
```

---

*This page was written by an AI agent from the CGP knowledge base and verified against the tool — see
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
