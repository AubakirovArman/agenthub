# content.article_outline

Creates a structured article outline for content workflows.

## Example AgentSpec

```yaml
skills: [content.article_outline]
workspace:
  type: content.git
scope:
  allow: ["content/**"]
verify:
  commands: ["test -s content/outline.md"]
```

Success test: outline exists and follows structure. Failure test: missing audience is flagged.
