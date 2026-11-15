---
title: Installing the CGP Skill
description: How to install and update the /cgp agent skill in Claude Code, Codex, Cursor, GitHub Copilot, Gemini CLI, OpenCode, Pi, or another agent, from the main branch that supports cgp v0.8.0 and later.
sidebar_label: Installation
sidebar_position: 2
---

# Installing the CGP Skill

The `/cgp` skill teaches a coding agent to read, write, and debug
[Context-Generic Programming (CGP)](/docs/) code, and this page shows how to install it in the agent
you use. The skill is a directory of Markdown files, with no scripts to run, so installing it means
putting that directory where your agent looks for skills. Every route below takes it from the
`main` branch of the [cgp-skills repository](https://github.com/contextgeneric/cgp-skills), which
supports `cgp` v0.8.0 and every later release, though not an older `cgp`. The skill keeps improving
between releases, so update it now and then, as [Updating the skill](#updating-the-skill)
describes.

## Ask your agent to install it

Your agent already knows where its own harness keeps skills, so the simplest route is to ask it.
Give it the repository's URL and a request such as this one:

```text
Install the /cgp agent skill from
https://github.com/contextgeneric/cgp-skills,
following the installation steps in the repository's README.
Ask me first whether to install it for me in every project
or only in this project.
```

The repository's README holds steps written for an agent: it asks you for the scope, uses your
harness's own installer when there is one, and otherwise copies the skill into the harness's skills
directory. Some harnesses, Claude Code and Codex among them, protect their skills folders, so expect
to approve the step that copies the skill there; review that command, and any other the agent
proposes, before it runs. Then check the result as
[Checking that it loaded](#checking-that-it-loaded) describes.

## Installing it yourself

If you would rather run the commands yourself, find your harness below. Each section gives the
harness's own installer where it has one, then the directory to copy the skill into, first for you
in every project and then for a single project. The copy itself is described in
[Copying the skill by hand](#copying-the-skill-by-hand). Choose one route per harness, because a
plugin and a copy installed together give the agent two copies of the skill.

### Claude Code

Claude Code installs the skill as a plugin from the repository, which serves as its own
marketplace. Run these two commands inside Claude Code:

```text
/plugin marketplace add contextgeneric/cgp-skills
/plugin install cgp@cgp-skills
```

A plugin's skills carry the plugin's name, so you invoke this one as `/cgp:cgp`. A copy in
`~/.claude/skills/cgp`, or `.claude/skills/cgp` in a project, is invoked as `/cgp` instead.

### Codex

Codex reads the same marketplace. Run these commands in a terminal:

```sh
codex plugin marketplace add contextgeneric/cgp-skills
codex plugin add cgp@cgp-skills
```

Codex then lists the skill as `cgp:cgp`. Its built-in `$skill-installer` can also install
`https://github.com/contextgeneric/cgp-skills/tree/main/cgp` when you ask it to. A copy goes in
`~/.agents/skills/cgp`, or `.agents/skills/cgp` in a project, where Codex lists it as `cgp`.

### Cursor

Cursor reads skills from folders, so copy the skill into `~/.cursor/skills/cgp`. That folder is the
one Cursor syncs to its Cloud Agents when you turn on *Sync Skills for Cloud Agents*. For a single
project, copy it into `.cursor/skills/cgp` and commit it, which is also the folder Cursor's
automations load. The installers in [Any other agent](#any-other-agent) can place it for you with
`-a cursor` or `--agent cursor`.

### GitHub Copilot and VS Code

The GitHub CLI installs the skill for Copilot:

```sh
gh skill install contextgeneric/cgp-skills cgp --agent github-copilot --scope user
```

Without `--scope user`, it installs into the current project. To copy the skill instead, use
`~/.copilot/skills/cgp`, or `.github/skills/cgp` in a project.

### Gemini CLI

Gemini CLI installs a skill from a directory in a Git repository:

```sh
gemini skills install https://github.com/contextgeneric/cgp-skills.git --path cgp
```

It takes the repository's default branch, `main`, which is the one to install. To copy the skill
instead, use `~/.gemini/skills/cgp`, or `.gemini/skills/cgp` in a project.

### OpenCode

OpenCode reads skills from folders. Copy the skill into `~/.config/opencode/skills/cgp`, or
`.opencode/skills/cgp` in a project.

### Pi

Pi installs the repository as a package:

```sh
pi install git:github.com/contextgeneric/cgp-skills
```

Add `-l` to install it for the current project only. Pi also reads skills copied into
`~/.agents/skills/cgp`, or `.agents/skills/cgp` in a project.

### Any other agent

Two installers place the skill for most agents that load skills. The `skills` tool runs through
`npx`:

```sh
npx skills add contextgeneric/cgp-skills --skill cgp
```

It installs into the current project unless you add `-g`, and `-a` names the agent. It sends
anonymous usage data unless you set `DISABLE_TELEMETRY=1`. The GitHub CLI does the same with
`gh skill install contextgeneric/cgp-skills cgp`, choosing the agent with `--agent`.
If neither knows your agent, copy the skill into `~/.agents/skills/cgp`, a folder that many agents
read.

## Copying the skill by hand

A copy works in every harness above. Clone the repository, then copy the `cgp/` directory to the
path your harness's section gives. For Claude Code, installing for yourself, the commands are:

```sh
git clone --depth 1 https://github.com/contextgeneric/cgp-skills.git
mkdir -p ~/.claude/skills
cp -R cgp-skills/cgp ~/.claude/skills/cgp
```

Keep `SKILL.md` and its `references/` directory together, because the skill links between them.
Copy the directory rather than linking to the clone: Codex finds the plugin manifest beside a linked
skill and names it `cgp:cgp`, while a copy is named `cgp` everywhere. Remove an older copy before
copying a new one, since `cp -R` puts the new directory inside an existing one.

## Installing it for a team

To give everyone on a project the same skill, copy it into the project's skills folder and commit
it, such as `.agents/skills/cgp`, which Codex, Cursor, and Pi read, and `.claude/skills/cgp` for
Claude Code. Every collaborator, and every cloud agent that checks out the repository, then gets the
same copy, and updating it is a commit like any other.

## Checking that it loaded

Start a new session after installing, since most agents read their skills when a session begins.
Then ask the agent which skills it has, or open your harness's list of skills: it should name `cgp`,
or `cgp:cgp` for a plugin. A question about CGP code, such as what a `delegate_components!` entry
does, should then lead the agent to load the skill. The skill tells the agent to compare your
project's `cgp` version with the releases it describes, and to tell you when a newer `cgp` or a
newer skill is available.

The skill teaches the agent, but it does not make the agent's code correct. Review what the agent
writes with it as you would any other change.

## Updating the skill

Fixes and better explanations reach `main` between `cgp` releases, so update the skill from time to
time, and whenever your project moves to a newer `cgp`. The skill helps with the second case: when
your project's `cgp` is newer than the releases it describes, it has the agent check `main` and tell
you when a newer skill is there, with the command for your route:

- **Claude Code:** run `claude plugin marketplace update cgp-skills`, then
  `claude plugin update cgp@cgp-skills`, and start a new session. To have it update by itself, open
  `/plugin`, select `cgp-skills` in the **Marketplaces** tab, and choose **Enable auto-update**,
  which is off for marketplaces outside Anthropic's own.
- **Codex:** run `codex plugin marketplace upgrade cgp-skills`, then
  `codex plugin add cgp@cgp-skills`.
- **Gemini CLI:** it has no update command, so run `gemini skills uninstall cgp`, then install the
  skill again.
- **Pi:** run `pi update --extensions`. A plain `pi update` updates only Pi itself.
- **`npx skills` and the GitHub CLI:** run `npx skills update cgp`, adding `-g` for an install in
  every project, or `gh skill update cgp`.
- **A copy:** remove the old directory, then copy `cgp/` again from a fresh clone.

## Agents without skill support

If your agent cannot load skills, attach the skill's main file,
[SKILL.md](https://raw.githubusercontent.com/contextgeneric/cgp-skills/main/cgp/SKILL.md), to the
conversation. It introduces the core model and names the reference file that covers each kind of
task, and you can attach those files from the
[references directory](https://github.com/contextgeneric/cgp-skills/tree/main/cgp/references) as
the agent needs them. The [skill's pages](/docs/ai/skills/cgp/SKILL) on this site show the same
files to read in the browser, as they stood when the site was last built.

---

*An AI agent wrote and revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
