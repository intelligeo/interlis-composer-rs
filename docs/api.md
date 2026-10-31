# API Reference

Base URL: `http://localhost:8080`

## Health and ops

### `GET /health`

Returns service liveness information.

### `GET /api/ops/metrics`

Returns operational counters and timings useful for runtime monitoring.

## Project endpoints

### `POST /api/project/new`

Creates a new baseline project payload.

### `POST /api/project/import/imcp`

Imports a serialized project bundle (`.imcp`-style content).

### `POST /api/project/export/imcp`

Exports current model and UI state as project payload.

## Model endpoints

### `POST /api/model/import/ili`

Imports INTERLIS text into the internal model structure.

### `POST /api/model/validate`

Validates model semantics and structure.

### `POST /api/model/export/ili`

Generates INTERLIS textual output from current model state.

## Documentation endpoint

### `POST /api/docs/export/markdown`

Builds markdown representation from current model state.

## Instance endpoints

### `POST /api/instance/import/xtf`

Imports or checks instance content against model constraints.

### `POST /api/instance/export/xtf`

Exports instance baseline data in XTF/XML-compatible payload form.

## Collaboration endpoints

### `GET /api/collab/status`

Returns runtime and repository collaboration status.

### `GET /api/collab/git-refs`

Returns branch/tag references visible to the service integration layer.

### `POST /api/collab/commit-draft`

Generates a commit draft proposal based on current model changes.

### `POST /api/collab/model-diff`

Compares current model against an imported collaboration bundle.

## Error model

Most API errors use JSON payloads containing one or more structured issues with:

- level (`error`, `warning`, `info`)
- code
- message
- optional path/details

## Notes

- Toolchain-based endpoints can return warnings when external binaries are unavailable and strict mode is disabled.
- For user-visible behavior, prefer validating and exporting in sequence.
