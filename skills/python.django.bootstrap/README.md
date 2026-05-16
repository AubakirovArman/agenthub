# python.django.bootstrap

Creates a scoped Django starter project without installing packages during the transaction.

## Example AgentSpec

```yaml
task:
  id: create_django_app
  type: code.django_scaffold
  title: Create Django web application
  target: agenthub_site

workspace:
  type: code.git
  isolation: git_worktree

skills:
  - python.django.bootstrap

execution:
  commands:
    - python -m compileall -q manage.py agenthub_site web

scope:
  allow:
    - manage.py
    - requirements.txt
    - agenthub_site/**
    - web/**
    - templates/**
    - static/**
    - docs/django-quickstart.md
  deny:
    - .agent/**
    - .env*

verify:
  profile: code_build
  commands:
    - python -m compileall -q manage.py agenthub_site web
```

The natural-language request `create a Django web application` generates the full scaffold command, adds a quickstart document, and verifies syntax plus required file presence.
