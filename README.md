# interlis-composer-rs

Rust + Axum backend with a static frontend for INTERLIS model authoring, validation, export, and collaboration support.

## Features

- Project lifecycle: new, import/export `.imcp`
- INTERLIS import/export: `.ili`
- Model validation with structured diagnostics
- External INTERLIS toolchain integration:
  - `ili2c` for robust `.ili` compilation checks during import/export/validate
  - `ilivalidator` for `.xtf` validation against the active model
- Documentation export to Markdown
- Catalog validation presets with governance metadata
- Collaboration baseline endpoints:
  - `/api/collab/status`
  - `/api/collab/git-refs`
  - `/api/collab/commit-draft`
  - `/api/collab/model-diff`

## Local run

```bash
cargo run
```

Default port: `8080`

## Docker

```bash
docker build -t interlis-composer-rs .
docker run --rm -p 8080:8080 interlis-composer-rs
```

## CI

GitHub Actions workflow is in `.github/workflows/ci.yml` and runs:

- `cargo fmt --check`
- `cargo clippy -D warnings`
- `cargo build --release`
- `cargo test`
- Docker image build

## Render blueprint

This repository includes `render.yaml` for Blueprint deployment on Render.

### Deploy on Render

1. In Render, choose **New +** -> **Blueprint**.
2. Connect this repository.
3. Select branch `main`.
4. Apply `render.yaml`.
5. Deploy.

Health endpoint: `/health`

## External toolchain configuration

The backend can call `ili2c` and `ilivalidator` as external commands. This improves
robustness of INTERLIS interpretation beyond internal MVP parsing.

Environment variables:

- `ILI2C_ENABLED` (`true|false`, default: `true`)
- `ILI2C_BIN` (default: `ili2c`)
- `ILI2C_ARGS` (optional, whitespace-separated)
- `ILI2C_STRICT` (`true|false`, default: `false`)
- `ILIVALIDATOR_ENABLED` (`true|false`, default: `true`)
- `ILIVALIDATOR_BIN` (default: `ilivalidator`)
- `ILIVALIDATOR_ARGS` (optional, whitespace-separated)
- `ILIVALIDATOR_STRICT` (`true|false`, default: `false`)
- `IOX_ILI_ENABLED` (`true|false`, default: `false`)
- `IOX_ILI_BIN` (default: `iox-ili`)
- `IOX_ILI_ARGS` (optional, whitespace-separated)
- `IOX_ILI_STRICT` (`true|false`, default: `false`)
- `IMC_TOOLCHAIN_TIMEOUT_MS` (default: `30000`)

Example (JAR-based setup):

```bash
ILI2C_BIN=java
ILI2C_ARGS="-jar /tools/ili2c.jar"
ILIVALIDATOR_BIN=java
ILIVALIDATOR_ARGS="-jar /tools/ilivalidator.jar"
cargo run
```

Notes:

- If a tool is unavailable and `*_STRICT=false`, the API returns warnings and continues.
- If `*_STRICT=true`, unavailable tool execution is treated as an error.
- `IOX_ILI_*` config is used by the Rust-native adapter for `/api/instance/export/xtf`.
