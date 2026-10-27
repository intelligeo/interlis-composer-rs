# interlis-composer-rs

Rust + Axum backend with a static frontend for INTERLIS model authoring, validation, export, and collaboration support.

## Features

- Project lifecycle: new, import/export `.imcp`
- INTERLIS import/export: `.ili`
- Model validation with structured diagnostics
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
