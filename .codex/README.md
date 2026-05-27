# Codex hook wrappers

This directory provides project-level wrappers for Codex-style commit/push gates:

- `hooks/pre-commit.sh`
- `hooks/pre-push.sh`

Both delegate to the shared quality gate entrypoint:

```bash
scripts/quality-gate.sh
```

So Git hooks, Claude hooks, and Codex wrappers stay aligned with one source of truth.

