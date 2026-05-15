# web.add_page

Adds a page or route to an existing web app.

## Example AgentSpec

```yaml
skills: [web.add_page]
scope:
  allow: ["src/app/courses/**"]
verify:
  routes:
    - { path: "/courses", expect: 200 }
```

Success test: route builds and smoke passes. Failure test: smoke catches missing route.
