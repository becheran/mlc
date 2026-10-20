# Examples

Snippets for using *mlc* with AI coding agents and git hooks. Copy what you need into your own project. See [AI agents and automation](../README.md#ai-agents-and-automation) for the details about the output and exit codes.

| Folder | What it does |
| --- | --- |
| [agents/](./agents/AGENTS-snippet.md) | Rule for your `AGENTS.md` or `CLAUDE.md` |
| [claude-code/](./claude-code/settings.json) | Hook which checks markup files right after Claude Code edited them |
| [claude-code/skills/check-links/](./claude-code/skills/check-links/SKILL.md) | Claude Code skill which teaches Claude how to run *mlc* and fix the findings |
| [pre-commit/](./pre-commit/.pre-commit-config.yaml) | [pre-commit](https://pre-commit.com) hook which checks staged markup files |

All examples expect `mlc` to be on the `PATH`. See the [installation instructions](../README.md#install-locally).
