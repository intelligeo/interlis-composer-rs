# Troubleshooting

## Output panel is empty

Possible causes:

- compile/export failed due to model issues
- invalid JSON edits interrupted sync
- external tools unavailable in strict mode

Actions:

1. Run model validation and review issue list.
2. Open Input panel and verify JSON validity.
3. Export `.ili` manually to force regeneration.
4. Check browser and backend logs for API errors.

## `ili2c` or `ilivalidator` not found

Symptoms:

- warnings about unavailable external commands
- reduced validation depth

Actions:

1. Set `ILI2C_BIN` and `ILIVALIDATOR_BIN` correctly.
2. If using JARs, set `*_BIN=java` with `*_ARGS="-jar /path/file.jar"`.
3. Keep `*_STRICT=false` for local prototypes.

## Proxy/network related failures

Symptoms:

- model repository fetch errors
- Java toolchain cannot resolve remote dependencies

Actions:

1. Set `IMC_HTTP_PROXY` and `IMC_HTTPS_PROXY`.
2. Set `IMC_NO_PROXY` for local/internal hosts.
3. Re-run validation/export after environment update.

## Slow or unstable validation

Actions:

1. Increase `IMC_TOOLCHAIN_TIMEOUT_MS`.
2. Minimize large model changes during iterative edits.
3. Validate in smaller logical increments.

## Documentation build fails

Actions:

1. Install Material for MkDocs: `pip install mkdocs-material`
2. Validate `mkdocs.yml` syntax.
3. Run `mkdocs serve` and fix missing pages/links.
