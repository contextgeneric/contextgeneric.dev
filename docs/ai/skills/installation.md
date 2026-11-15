---
title: Installing the CGP Skill
description: How to install and update the /cgp agent skill in Claude Code, Codex, Cursor, GitHub Copilot, Gemini CLI, OpenCode, Pi, or another agent, from the main branch that supports cgp v0.8.0 and later.
sidebar_label: Installation
sidebar_position: 2
---

# Installing the CGP Skill

The `/cgp` skill teaches a coding agent to read, write, and debug
[Context-Generic Programming (CGP)](/docs/) code, and this page shows how to install it in the agent
you use. The skill consists of Markdown files, with no scripts to run, so installing it means
putting those files where your agent looks for skills.

Every installation method below uses the `main` branch of the
[cgp-skills repository](https://github.com/contextgeneric/cgp-skills). That branch supports `cgp`
v0.8.0 and later, but not older releases. The skill keeps improving between releases, so
[update it from time to time](#updating-the-skill).

## Ask your agent to install it

Your agent already knows where it keeps its skills, so the simplest way to install the skill is to
ask it. Give it the repository's URL and a request such as this one:

```text
Install the /cgp agent skill from
https://github.com/contextgeneric/cgp-skills,
following the installation steps in the repository's README.
Ask me first whether to install it for me in every project
or only in this project.
```

The repository's README gives the agent installation steps. Following those steps, the agent asks
you where the skill should be available, then uses its own installer if there is one. Otherwise, it
copies the skill into its skills directory.

Some agents, including Claude Code and Codex, protect their skills folders, so expect to approve the
step that copies the skill there. Review that command, and any others the agent proposes, before it
runs them. After installation, [check that the skill loaded](#checking-that-it-loaded).

## Installing it yourself

If you would rather run the commands yourself, find your agent below. Each section describes its
installer, if it has one, and the directories you can copy the skill into. The first directory makes
the skill available to you in every project; the second limits it to a single project. See
[Copying the skill by hand](#copying-the-skill-by-hand) for the copy commands.

Choose one installation method per agent. Installing both a plugin and a separate copy gives the
agent two copies of the skill.

### Claude Code

Claude Code installs the skill as a plugin from the repository, which serves as its own
marketplace. Run these commands inside Claude Code:

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

Cursor reads skills from folders, so copy the skill into `~/.cursor/skills/cgp`. Cursor syncs this
folder to its Cloud Agents when you turn on *Sync Skills for Cloud Agents*. For a single project,
copy the skill into `.cursor/skills/cgp` and commit it. Cursor's automations also load skills from
that project folder. The installers in [Any other agent](#any-other-agent) can place the skill for
you with `-a cursor` or `--agent cursor`.

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

This command uses the repository's default branch, `main`. To copy the skill instead, use
`~/.gemini/skills/cgp`, or `.gemini/skills/cgp` in a project.

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

If your agent supports skills, the `skills` tool or the GitHub CLI can usually install this one for
you. To use the `skills` tool, run it through `npx`:

```sh
npx skills add contextgeneric/cgp-skills --skill cgp
```

This installs into the current project unless you add `-g`; use `-a` to name the agent. The tool
sends anonymous usage data unless you set `DISABLE_TELEMETRY=1`.

You can also use the GitHub CLI with `gh skill install contextgeneric/cgp-skills cgp`, choosing the
agent with `--agent`. If neither installer recognizes your agent, copy the skill into
`~/.agents/skills/cgp`, a folder that many agents read.

## Copying the skill by hand

Every agent listed above can load a copy of the skill. Clone the repository, then copy the `cgp/`
directory to the path listed for your agent. For example, these commands install it for your use in
every Claude Code project:

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
it. Codex, Cursor, and Pi read `.agents/skills/cgp`, while Claude Code reads `.claude/skills/cgp`.
Every collaborator and cloud agent that checks out the repository then gets the same copy, and
updates to the skill become commits like any other change.

## Checking that it loaded

Start a new session after installing, since most agents read their skills when a session begins.
Then ask the agent which skills it has, or open your agent's list of skills. It should name `cgp`,
or `cgp:cgp` for a plugin. A question about CGP code, such as what a `delegate_components!` entry
does, should then lead the agent to load the skill. The skill tells the agent to compare your
project's `cgp` version with the releases it describes, and to tell you when a newer `cgp` or a
newer skill is available.

The skill teaches the agent, but it does not make the agent's code correct. Review what the agent
writes with it as you would any other change.

## Updating the skill

Fixes and better explanations reach `main` between `cgp` releases, so update the skill from time to
time and whenever your project moves to a newer `cgp`. To help you keep up with release changes,
the skill tells the agent to compare your project's `cgp` version with the releases it describes.
If your version is newer, the agent checks `main` and tells you when a newer skill is available.

Use the update command for your installation method:

- **Claude Code:** run `claude plugin marketplace update cgp-skills`, then
  `claude plugin update cgp@cgp-skills`, and start a new session. To have it update by itself, open
  `/plugin`, select `cgp-skills` in the **Marketplaces** tab, and choose **Enable auto-update**.
  This setting is off for marketplaces outside Anthropic's own.
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
conversation. It introduces the core model and names the reference file for each kind of task.
As the agent needs those files, you can attach them from the
[references directory](https://github.com/contextgeneric/cgp-skills/tree/main/cgp/references).
You can also read the [skill's pages](/docs/ai/skills/cgp/SKILL) in your browser. They show the same
files as they stood when this site was last built.

---

*An AI agent wrote and revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
