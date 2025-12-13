# Agent Instructions for Whitehall

## Issue Tracking with bd (beads)

This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs or task lists.

### Quick Start

```bash
# Find ready work
bd ready

# Create issues
bd create "Issue title" -t bug|feature|task -p 0-4

# Claim and work
bd update <id> --status in_progress

# Complete
bd close <id> --reason "Completed"
```

### Priorities

- `0` - Critical (blocking bugs, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default)
- `3` - Low (polish, nice-to-have)
- `4` - Backlog

### Workflow

1. `bd ready` - see what to work on
2. `bd update <id> --status in_progress` - claim it
3. Work on it
4. `bd close <id> --reason "Done"` - mark complete
5. Commit `.beads/` changes with your code

### Rules

- Use bd for ALL task tracking
- Link discovered issues with `--deps discovered-from:<parent-id>`
- Check `bd ready` before asking "what should I work on?"
- Do NOT create markdown TODO lists
