# Beads - AI-Native Issue Tracking

## Project persistence

This workspace uses embedded Dolt; no server or scheduled task is required.
Normal task operations use `bd ready`, `bd update --claim`, and `bd close`,
with `--json` for programmatic callers. A coordinator manages claims and
dependencies while workers operate on their assigned files.

From the repository root, export portable issue records with:

```powershell
python .\scripts\export-beads.py
git add -f -- .beads\backup\issues.jsonl
```

The exporter preserves task IDs, dependencies, statuses, assignments and other
issue fields, but omits the local `owner` routing field. It is not a general
redactor: task descriptions and evidence references must not contain secrets,
private user data or recovered vendor code. It never commits or pushes.

Only this JSONL file belongs in Git. The other backup files, embedded database,
locks and runtime files remain ignored. JSONL is an interchange snapshot, not a
complete Dolt-history backup. Full local backups can be made explicitly:

```powershell
bd backup init .reverse-engineering\beads-backup
bd backup sync
```

Auto-backup is disabled for this workspace; these operations are manual.

Welcome to Beads! This repository uses **Beads** for issue tracking - a modern, AI-native tool designed to live directly in your codebase alongside your code.

## What is Beads?

Beads is issue tracking that lives in your repo, making it perfect for AI coding agents and developers who want their issues close to their code. No web UI required - everything works through the CLI and integrates seamlessly with git.

**Learn more:** [github.com/steveyegge/beads](https://github.com/steveyegge/beads)

## Quick Start

### Essential Commands

```bash
# Create new issues
bd create "Add user authentication"

# View all issues
bd list

# View issue details
bd show <issue-id>

# Update issue status
bd update <issue-id> --claim
bd update <issue-id> --status done

# Sync with Dolt remote
bd dolt push
```

### Working with Issues

Issues in Beads are:
- **Git-native**: Stored in Dolt database with version control and branching
- **AI-friendly**: CLI-first design works perfectly with AI coding agents
- **Branch-aware**: Issues can follow your branch workflow
- **Sync-ready**: Uses Dolt remotes for backup and team sharing

## Why Beads?

✨ **AI-Native Design**
- Built specifically for AI-assisted development workflows
- CLI-first interface works seamlessly with AI coding agents
- No context switching to web UIs

🚀 **Developer Focused**
- Issues live in your repo, right next to your code
- Works offline, syncs when you push
- Fast, lightweight, and stays out of your way

🔧 **Git Integration**
- Dolt-native sync via bd dolt push / bd dolt pull
- Branch-aware issue tracking
- Dolt-native three-way merge resolution

## Get Started with Beads

Try Beads in your own projects:

```bash
# Install Beads
curl -sSL https://raw.githubusercontent.com/steveyegge/beads/main/scripts/install.sh | bash

# Initialize in your repo
bd init

# Create your first issue
bd create "Try out Beads"
```

## Learn More

- **Documentation**: [github.com/steveyegge/beads/docs](https://github.com/steveyegge/beads/tree/main/docs)
- **Quick Start Guide**: Run `bd quickstart`
- **Examples**: [github.com/steveyegge/beads/examples](https://github.com/steveyegge/beads/tree/main/examples)

---

*Beads: Issue tracking that moves at the speed of thought* ⚡
