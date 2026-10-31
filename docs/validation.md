# Validation

## Validation layers

Validation combines:

- internal model checks
- optional `ili2c` compilation checks
- optional `ilivalidator` checks for instance workflows

## Playbook

Use the validation playbook and smoke script as baseline quality checks:

- `docs/VALIDATION_PLAYBOOK.md` (workspace-level reference)
- `scripts/smoke-api-validation.ps1`

## Suggested validation pipeline

1. Run model validation from UI/API.
2. Export INTERLIS text.
3. Re-validate with strict toolchain in CI.
4. Run XTF import/export checks for integration scenarios.

## Smoke script usage

```powershell
pwsh -File scripts/smoke-api-validation.ps1 -SkipStartServer
pwsh -File scripts/smoke-api-validation.ps1 -StartServer
```

## CI gates

Recommended minimum:

- formatting (`cargo fmt --check`)
- linting (`cargo clippy -D warnings`)
- tests (`cargo test`)
- smoke validation script
