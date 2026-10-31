# Configuration

## Environment variables

### Tool enablement

- `ILI2C_ENABLED` (`true|false`, default `true`)
- `ILIVALIDATOR_ENABLED` (`true|false`, default `true`)
- `IOX_ILI_ENABLED` (`true|false`, default `false`)

### Tool binaries

- `ILI2C_BIN` (default `ili2c`)
- `ILIVALIDATOR_BIN` (default `ilivalidator`)
- `IOX_ILI_BIN` (default `iox-ili`)

### Tool arguments

- `ILI2C_ARGS` (whitespace-separated)
- `ILIVALIDATOR_ARGS` (whitespace-separated)
- `IOX_ILI_ARGS` (whitespace-separated)

### Strictness

- `ILI2C_STRICT` (`true|false`, default `false`)
- `ILIVALIDATOR_STRICT` (`true|false`, default `false`)
- `IOX_ILI_STRICT` (`true|false`, default `false`)

### Timeouts and model repositories

- `IMC_TOOLCHAIN_TIMEOUT_MS` (default `30000`)
- `IMC_MODEL_REPOSITORIES` (default `https://models.interlis.ch/;https://models.geo.admin.ch/`)

### Proxy support

- `IMC_HTTP_PROXY`
- `IMC_HTTPS_PROXY` (falls back to `IMC_HTTP_PROXY`)
- `IMC_NO_PROXY`

## Example: Java JAR toolchain

```bash
ILI2C_BIN=java
ILI2C_ARGS="-jar /tools/ili2c.jar"
ILIVALIDATOR_BIN=java
ILIVALIDATOR_ARGS="-jar /tools/ilivalidator.jar"
IMC_HTTP_PROXY="http://proxy-bvcol.admin.ch:8080"
IMC_HTTPS_PROXY="http://proxy-bvcol.admin.ch:8080"
cargo run
```

## Behavior model

- If a tool is missing and strict mode is off, the API returns warnings.
- If strict mode is on, missing/failed tool execution is treated as error.

## Operational recommendations

1. Use strict mode in CI and release pipelines.
2. Keep strict mode off in early local prototyping.
3. Persist validated binary versions in deployment manifests.
