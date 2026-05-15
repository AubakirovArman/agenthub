# python.data_artifact

Creates a data artifact with Python and validates the result.

## Example AgentSpec

```yaml
skills: [python.data_artifact]
workspace:
  type: data.git
scope:
  allow: ["outputs/**"]
verify:
  commands: ["python scripts/validate.py outputs/result.json"]
```

Success test: artifact exists and validates. Failure test: missing artifact fails.
