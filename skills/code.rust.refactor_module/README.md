# code.rust.refactor_module

Splits or reshapes Rust modules without changing behavior.

## Example AgentSpec

```yaml
skills: [code.rust.refactor_module]
scope:
  allow: ["src/**"]
verify:
  commands: ["cargo test --locked", "scripts/check-module-size.sh 200"]
```

Success test: tests and size gate pass. Failure test: module path break is caught.
