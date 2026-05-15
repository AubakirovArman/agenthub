# web.reuse_component

Keeps web UI changes consistent with existing components.

## Example AgentSpec

```yaml
skills: [web.reuse_component]
scope:
  allow: ["src/**"]
verify:
  commands: ["npm run build"]
```

Success test: build passes with reused patterns. Failure test: duplicated component pattern is reported.
