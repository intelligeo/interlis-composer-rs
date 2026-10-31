# INTERLIS Model Composer (Rust)

INTERLIS Model Composer is a Rust + Axum service with a static web UI for:

- authoring INTERLIS model structures
- validating semantic and structural constraints
- importing and exporting `.ili`, `.imcp`, and `.xtf`-related payloads
- generating Markdown documentation from the active model

## Documentation map

- Start here: [Getting Started](getting-started.md)
- End-to-end usage: [Guide](guide.md)
- Runtime and toolchain settings: [Configuration](configuration.md)
- HTTP endpoints: [API Reference](api.md)
- UI interactions and sync model: [UI Workflow](ui-workflow.md)
- Validation strategy and smoke checks: [Validation](validation.md)
- Deploying to Docker/Render: [Deployment](deployment.md)
- Operational help: [Troubleshooting](troubleshooting.md)

## Quick links

- Health endpoint: `/health`
- Main API scope: `/api/*`
- Validation playbook script: `scripts/smoke-api-validation.ps1`

## Intended audience

This documentation is for:

- backend contributors
- UI contributors
- DevOps engineers deploying the service
- QA teams validating model and instance workflows
