# core.fix_build

Fixes a failing verifier with the smallest useful change.

## Example AgentSpec

```yaml
skills: [core.fix_build]
scope:
  allow: ["src/**", "tests/**"]
verify:
  commands: ["cargo test --locked"]
```

Success test: verifier passes after patch. Failure test: verifier failure triggers rollback.
