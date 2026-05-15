# core.file.create

Creates a new file inside the declared transaction scope.

## Example AgentSpec

```yaml
skills: [core.file.create]
scope:
  allow: ["generated/**"]
execution:
  commands: ["mkdir -p generated && printf ok > generated/file.txt"]
verify:
  commands: ["test -f generated/file.txt"]
```

Success test: allowed file is created and committed. Failure test: denied path is blocked by diff guard.
