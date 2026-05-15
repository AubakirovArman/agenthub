# core.docs.update

Updates documentation alongside behavior changes.

## Example AgentSpec

```yaml
skills: [core.docs.update]
scope:
  allow: ["README.md", "docs/**"]
execution:
  commands: ["printf '\\n## New Usage\\n' >> README.md"]
verify:
  commands: ["test -s README.md"]
```

Success test: docs and examples are present. Failure test: required localized docs are missing.
