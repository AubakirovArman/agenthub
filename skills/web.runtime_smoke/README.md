# web.runtime_smoke

Verifies a web route through a running local server.

## Example AgentSpec

```yaml
skills: [web.runtime_smoke]
verify:
  runtime:
    start_command: "npm run dev"
  routes:
    - { path: "/courses", expect: 200 }
```

Success test: expected route responds. Failure test: route mismatch fails verification.
