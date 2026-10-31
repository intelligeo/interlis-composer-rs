# UI Workflow

## Main panels

- Structure Tree: hierarchical model navigation and node creation
- Properties Editor: type/cardinality/metadata editing
- Diagram Editor: visual arrangement and relation tuning
- Input: raw model JSON view/edit
- Output: compiled INTERLIS textual output
- Docs: markdown preview and references

## Sync model

The UI uses model-change events and debounced cross-panel synchronization to keep:

- JSON input state
- diagram state
- output INTERLIS text
- docs preview

aligned with the current model.

## Expected Output panel behavior

The Output panel is expected to contain the latest compiled INTERLIS text.

If a new compile attempt fails, the UI can keep the last valid snapshot to avoid showing an empty panel.

## Recommended operator sequence

1. Modify model in Structure/Properties.
2. Run validation.
3. Inspect Output panel for generated INTERLIS.
4. Export `.ili` and docs markdown.

## Common UX pitfalls

- Editing raw JSON with invalid syntax interrupts full sync.
- Missing external tool binaries can degrade diagnostics quality.
- Long-running tool calls can look like stale UI without progress feedback.
