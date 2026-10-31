# INTERLIS Model Composer Comprehensive Guide

This guide is the operational and contributor handbook for the Rust implementation of INTERLIS Model Composer.

## 1. Scope

The application provides:

- a static web UI for model authoring
- a Rust + Axum backend for model operations and validation
- import/export support for INTERLIS-centered workflows
- collaboration helper endpoints

## 2. Architecture at a glance

## Frontend

- Served from `static/`
- Panel-based workspace (structure, properties, diagram, input, output, docs, sync)
- Event-driven synchronization between panels

## Backend

- Axum HTTP API in `src/main.rs`
- domain model definitions in `src/model.rs`
- external toolchain adapter in `src/toolchain_adapter.rs`

## Integrations

- `ili2c` for compile-grade INTERLIS checks
- `ilivalidator` for XTF validation
- `iox-ili` adapter path for instance export flows

## 3. Runtime workflows

## Modeling

1. Create new project or import `.imcp`.
2. Build structure in tree and properties panels.
3. Validate continuously.
4. Inspect generated INTERLIS output.
5. Export artifacts.

## Import and export

- Import model text: `.ili`
- Export model text: `.ili`
- Export project bundle: `.imcp`
- Export docs: markdown
- Import/export instance baseline: `.xtf/.xml` API paths

## 4. API overview

Core groups:

- Health and ops
- Project lifecycle
- Model import/validate/export
- Docs export
- Instance import/export
- Collaboration helper endpoints

For endpoint-level details, see [docs/api.md](docs/api.md).

## 5. Configuration strategy

The system is highly configurable via environment variables:

- tool enablement
- command names and args
- strictness policy
- timeout control
- model repositories
- proxy routing

For full variable list and examples, see [docs/configuration.md](docs/configuration.md).

## 6. Quality and validation strategy

Recommended baseline:

- `cargo fmt --check`
- `cargo clippy -D warnings`
- `cargo test`
- smoke script: `scripts/smoke-api-validation.ps1`

Validation behavior is influenced by strictness flags and external tool availability.

## 7. Deployment patterns

Supported paths:

- local development with `cargo run`
- containerized run via Docker
- blueprint deployment via Render (`render.yaml`)

See [docs/deployment.md](docs/deployment.md) for deployment details.

## 8. Documentation system (Material for MkDocs)

This repository includes:

- `mkdocs.yml` as site configuration
- `docs/` as navigation and content root

Recommended docs lifecycle:

1. Update docs in same PR as code changes.
2. Keep API docs aligned with route changes.
3. Add troubleshooting notes from real incidents.

## 9. Contributor checklist

Before pushing changes:

1. Run format, lint, and tests.
2. Validate a representative model scenario through UI.
3. Confirm Output panel and docs export are synchronized.
4. Update docs pages and this guide if behavior changed.

## 10. Known operational risks

- external tool version drift across environments
- proxy constraints in restricted networks
- temporary desync from invalid raw JSON edits

Mitigations:

- pin tool versions where possible
- standardize env configuration per environment
- retain fallback snapshots for generated output

## 11. Where to continue

- Getting started tasks: [docs/getting-started.md](docs/getting-started.md)
- End-user UI sequence: [docs/ui-workflow.md](docs/ui-workflow.md)
- Failure handling and fixes: [docs/troubleshooting.md](docs/troubleshooting.md)
