---
name: release-changelog
description: Generate and maintain release changelog entries from git history.
always: false
---

# Release Changelog Skill

Use this skill when preparing a release tag and updating `CHANGELOG.md`.

## Goal

Produce a clear changelog section for a target version using commit history, then commit it before tagging.

## Inputs

- Target version (for example: `v0.0.4`)
- Optional starting reference (for example: `v0.0.3`)

## Steps

1. Preview generated section:

```bash
just changelog-preview v0.0.4
```

2. If needed, use a fixed range:

```bash
bash scripts/generate-changelog.sh v0.0.4 --since v0.0.3 --dry-run
```

3. Write to `CHANGELOG.md`:

```bash
just changelog v0.0.4
```

4. Review and lightly edit wording if required (keep sections concise and factual).
5. Commit changelog before creating/pushing the release tag.

## Output format

Generated sections follow:

- `## [vX.Y.Z] - YYYY-MM-DD`
- `### Added`
- `### Changed`
- `### Fixed`
- `### Documentation`
- `### Maintenance`

## Notes

- The generator inserts entries after `<!-- changelog-entries -->` in `CHANGELOG.md`.
- If no commits are found in range, it emits a minimal `Changed` section.

