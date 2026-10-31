# Guide

## Product overview

INTERLIS Model Composer supports a model-first workflow:

1. Create or import model state.
2. Edit structure and properties in UI panels.
3. Validate continuously.
4. Export artifacts (`.ili`, markdown docs, project bundle).
5. Optionally validate instances (`.xtf`) against the active model.

## Core concepts

### Project bundle (`.imcp`)

A project bundle captures:

- project metadata
- current model JSON
- UI state (diagram zoom, panel configuration, presets)

Use it to move work between sessions and team members.

### Model state

The backend stores and manipulates a canonical model representation used by all APIs.

### Derived artifacts

The following outputs are generated from model state:

- INTERLIS text (`/api/model/export/ili`)
- Markdown docs (`/api/docs/export/markdown`)
- diagnostics from model validation (`/api/model/validate`)

## Daily workflow

### 1. Start or import

- New project: create baseline structure.
- Import `.imcp`: continue from previous session.
- Import `.ili`: parse textual INTERLIS into model state.

### 2. Edit model

Use these panels:

- Structure Tree: navigate/create topics, classes, attributes, domains, relations
- Properties Editor: update semantic details
- Diagram Editor: adjust visual layout and relation paths

### 3. Validate

Run validation frequently to catch:

- naming issues
- cardinality errors
- consistency issues
- external toolchain findings (if configured)

### 4. Inspect output

Output panel should always show the latest compiled INTERLIS model text after sync.

### 5. Export

- Export `.ili` for INTERLIS downstream workflows
- Export markdown docs for governance and review
- Export `.imcp` for collaboration handoff

## Collaboration endpoints

Available baseline endpoints:

- `GET /api/collab/status`
- `GET /api/collab/git-refs`
- `POST /api/collab/commit-draft`
- `POST /api/collab/model-diff`

These endpoints are intended to support lightweight coordination and traceability.

## Suggested team process

1. Keep `.imcp` snapshots for significant milestones.
2. Validate before each export.
3. Attach generated docs to review tickets.
4. Use commit drafts as helper text, not as authoritative changelog.

## Quality gates

Recommended checks before merge:

- `cargo fmt --check`
- `cargo clippy -D warnings`
- `cargo test`
- smoke validation script run

## Security and operations notes

- Keep external tool binaries managed and pinned where possible.
- Set proxy variables in restricted networks.
- Use strict mode (`*_STRICT=true`) in controlled CI environments.
