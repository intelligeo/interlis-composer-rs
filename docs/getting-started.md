# Getting Started

## Prerequisites

- Rust toolchain `>= 1.88` (repository pins `1.90.0`)
- Optional external tools for stronger validation:
  - `ili2c`
  - `ilivalidator`
  - `iox-ili`

## Run locally

```bash
cargo run
```

Default address:

- `http://127.0.0.1:8080`

Health check:

```bash
curl http://127.0.0.1:8080/health
```

## Run in Docker

```bash
docker build -t interlis-composer-rs .
docker run --rm -p 8080:8080 interlis-composer-rs
```

## First manual test

1. Open the UI in your browser.
2. Create or import a model.
3. Run validation.
4. Open Output panel and confirm generated INTERLIS text.
5. Export `.ili` and docs markdown.

## Automated smoke test

```powershell
# Server already running
pwsh -File scripts/smoke-api-validation.ps1 -SkipStartServer

# Script starts server (requires cargo in PATH)
pwsh -File scripts/smoke-api-validation.ps1 -StartServer
```
