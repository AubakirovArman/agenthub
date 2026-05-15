# Partial PRD Areas

These PRD areas have an implemented foundation but are not complete compared with the full long-term PRD.

- Product vision as universal agent OS: foundation exists, full product surface is not complete.
- AgentHub layers: CLI, TUI, and VS Code exist; web dashboard is not implemented.
- AAL: AgentSpec and AgentIR exist, but a standalone AAL grammar/parser is not implemented.
- Agent Lock: `.agent/agent.lock` exists, but deeper central lock governance is future work.
- Transaction Manager effect tracking: local git/file rollback exists, but not all external effects have rollback handlers.
- Verifier layer: command/runtime/content/data/infra checks exist; specialized backend_tdd and db_migration profiles are not complete.
- Agent orchestration: several topology kinds exist; manager/worker and tournament remain open.
- LLM Gateway: traces, metadata, redaction, and cost estimates exist; not a full provider network gateway.
- Security policy: RBAC and diff/scope controls exist; command allowlist enforcement and strong sandbox levels are not complete.
- Domain profiles: Code, Infra, Data, Content are implemented; Research is partial and Media is open.
- Success metrics: artifacts expose data; aggregated metrics and dashboards are not implemented.
- WAL connection: transaction journal exists, but a formal WAL subsystem is not complete.
