# Deployment

## Docker

Build and run:

```bash
docker build -t interlis-composer-rs .
docker run --rm -p 8080:8080 interlis-composer-rs
```

Health endpoint:

- `/health`

## Render Blueprint

This repository includes `render.yaml`.

Deployment steps:

1. Render -> New + -> Blueprint
2. Connect repository
3. Select `main`
4. Apply `render.yaml`
5. Deploy

## Production recommendations

1. Configure external toolchain binaries explicitly.
2. Set timeout and repository mirror values.
3. Use strict mode in production CI checks.
4. Keep proxy settings aligned with network policy.

## Documentation publishing with Material for MkDocs

### Local build

```bash
pip install mkdocs-material
mkdocs serve
```

### Static build

```bash
mkdocs build
```

This uses repository-level `mkdocs.yml` and pages under `docs/`.
