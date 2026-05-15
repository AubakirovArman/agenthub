# code.rust.add_test

Adds a focused Rust test.

## Example AgentSpec

```yaml
skills: [code.rust.add_test]
scope:
  allow: ["src/**", "tests/**"]
verify:
  commands: ["cargo test --locked"]
```

Success test: test proves behavior. Failure test: assertion-free test is rejected.
