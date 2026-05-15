# core.file.edit

Edits an existing file with a narrow diff.

## Example AgentSpec

```yaml
skills: [core.file.edit]
scope:
  allow: ["README.md"]
execution:
  commands: ["printf '\\nUpdated by AgentHub\\n' >> README.md"]
verify:
  commands: ["grep -q AgentHub README.md"]
```

Success test: allowed file changes. Failure test: denied file edit rolls back.
