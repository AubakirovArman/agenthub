# code.rust.fix_clippy

Fixes `cargo clippy -- -D warnings` failures.

## Example AgentSpec

```yaml
skills: [code.rust.fix_clippy]
scope:
  allow: ["src/**"]
verify:
  commands: ["cargo clippy --locked -- -D warnings"]
```

Success test: clippy passes. Failure test: behavior-changing patch is rejected in review.
