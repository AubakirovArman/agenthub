# PRD Completion Audit

Source: [`../../prd.md`](../../prd.md)

## Overall Verdict

The staged roadmap in section 21 is done: Phase 1 through Phase 14 are in [`../done`](../done), and [`../todo`](../todo) has no phase files.

The full PRD is broader than those phases. Some long-term product points remain partial or open: web dashboard, full AAL grammar, MediaWorkspace, full Research profile, Manager/Worker and Tournament topologies, specialized database/backend verifiers, full command policy enforcement, sandbox levels beyond local control, real remote runner execution, cryptographic package signing, metrics dashboards, and a formal WAL layer.

## Top-Level Section Matrix

| PRD Section | Status | Evidence / Gap |
|---|---|---|
| 0. Document Purpose | Done | PRD split into `prd/source`; tracker and audit now exist. |
| 1. Executive Summary | Mostly done | Core transaction runtime exists; some future UI/domain breadth remains. |
| 2. Product Vision | Partial | Foundation implemented, but full "agent OS" vision is long-term. |
| 3. Positioning | Done | Product is positioned as runtime/orchestrator, not a model replacement. |
| 4. Problem Statement | Done | State drift, context bloat, transactionality, memory pollution, verification, observability, and cross-agent continuity are addressed by phases 1-14. |
| 5. Target Users | Partial | Developers, agent builders, content/data/infra, and enterprise flows exist; media/research users are not fully covered. |
| 6. Core Principles / Laws | Mostly done | All core transaction laws are implemented; domain breadth is partial. |
| 7. High-Level Architecture | Mostly done | Runtime architecture exists; future visual/web surfaces remain. |
| 8. AgentHub Layers | Partial | CLI, TUI, VS Code, intent, AgentSpec, AgentIR exist; web dashboard and full AAL grammar are open. |
| 9. VCM-OS Memory Layer | Mostly done | Staging, committed, failed attempts, compaction, and schemas exist; media/research memory is partial. |
| 10. Context Pack System | Done | Context pack, trace, least-context policy, maps, and selected map context are implemented. |
| 11. Agent Lock | Mostly done | `.agent/agent.lock` exists with project, policies, skills, plugins, enterprise, verifiers, and commands; deeper lock governance remains future. |
| 12. Skill Registry | Done | Skill manifests, dependency loading, plugin packages, version locks, trust model, and scaffold exist. |
| 13. Workspace Runtime | Partial | Code, Content, Data, and Infra workspaces exist; MediaWorkspace is open. |
| 14. Transaction Manager | Mostly done | Lifecycle, rollback, sync, diff guard, repair, reviewer gate, reports, memory promotion exist; full external effect rollback handlers remain partial. |
| 15. Verifier Layer | Partial | command checks, runtime smoke, infra/data/content domain verifiers exist; backend_tdd and db_migration are not specialized profiles yet. |
| 16. Agent Orchestration | Partial | single, planner/executor, generator/critic, reviewer/repair, and swarm research exist; manager/worker and tournament are open. |
| 17. LLM Gateway and Observability | Mostly done | Redaction, raw traces, token/cost estimates, metadata, reports exist; it is not a full provider network gateway yet. |
| 18. `.agent/` Project Structure | Done | Runtime directories, locks, maps, policies, schemas, memory, tx artifacts, plugins, and enterprise files exist. |
| 19. Security and Policy | Partial | RBAC, diff guard, secret checks, and enterprise policy exist; command allowlist enforcement and sandbox levels 1-3 are open. |
| 20. Domain Profiles | Partial | Code, Infra, Data, Content exist; Media is open and Research is partial. |
| 21. Development Roadmap | Done | Phases 1-14 are done with commit evidence in `prd/status.md`. |
| 22. Technical Stack | Mostly done | Rust core and VS Code extension exist; research/ML plugins are future. |
| 23. Success Metrics | Partial | Artifacts expose reliability/context/cost data, but metric aggregation and dashboards are open. |
| 24. Major Risks | Mostly mitigated | Transaction, verifier, memory, security, cost, and skill risks have mitigations, but remain ongoing product risks. |
| 25. Open Questions | Partial | Several decisions are answered; some long-term questions remain open. |
| 26. Reference Scenario | Partial | Transaction examples exist; a full real web-app add-page scenario is not shipped as an end-to-end fixture. |
| 27. WAL and VCM-OS Connection | Partial | Journal and VCM memory exist; formal WAL subsystem is not complete. |
| 28. Final Product Thesis | Partial | Foundation is real, but full product thesis is broader than current implementation. |
| 29. Immediate Next Documents | Partial | Many docs exist in 4 languages; full AAL spec, plugin author guide depth, and marketplace governance docs can expand. |
| 30. Current Decision Snapshot | Mostly done | Rust, AgentSpec/YAML, worktrees, verifier profiles, plugin ecosystem, and enterprise policy are implemented. |
| 31. Short North Star | Partial | The direction is implemented as a foundation, not yet as a complete universal agent OS. |

## Kernel Laws

| Law | Status | Evidence / Gap |
|---|---|---|
| Law 1 Atomicity | Done | No successful verifier means no commit. |
| Law 2 Memory Consistency | Done | Memory promotion happens after successful verification. |
| Law 3 Isolation | Done | Transactions run in isolated git worktrees. |
| Law 4 Rollbackability | Done | Failed transactions roll back worktree state. |
| Law 5 Failed Experience Durability | Done | Failed attempts and error fingerprints are persisted. |
| Law 6 No Blind Merge | Done | Commit path goes through verifier, diff guard, reports, and journal. |
| Law 7 Scope Enforcement | Done | `scope.allow`, `scope.deny`, and diff limits are enforced. |
| Law 8 Observability First | Done | Journal, report, traces, DAG, context pack, verifier logs, and metadata are written. |
| Law 9 Least Context | Done | Context packs include selected map context and trace metadata. |
| Law 10 Domain via Plugins | Mostly done | Plugin package format and domain workspaces exist; media/research domain depth remains partial. |

## Detailed Feature Matrix

| Area | Done | Partial / Open |
|---|---|---|
| Interfaces | CLI, TUI, VS Code extension | web dashboard |
| Intent | natural language `ask`, defaults, clarification, approval marking | deeper intent semantics |
| Language | AgentSpec YAML, AgentIR, DAG compiler | standalone AAL grammar/parser |
| Memory | committed, staged, failed attempts, compaction | media/research schemas |
| Context | context pack, map context, trace, stale detection | larger context policy tuning |
| Agent Lock | `.agent/agent.lock` and locks for plugins/skills | central lock governance |
| Skills | skill manifests, dependency loading, plugin packages, scaffold | remote marketplace publishing workflow |
| Workspaces | Code, Content, Data, Infra | MediaWorkspace |
| Transactions | lifecycle, rollback, sync, diff guard, repair, reviewer, report | full external effect tracking/rollback |
| Verifiers | command, runtime smoke, content/data/infra domain checks | backend_tdd and db_migration specialization |
| Orchestration | single, planner/executor, generator/critic, reviewer/repair, swarm | manager/worker and tournament |
| LLM Gateway | metadata, redaction, raw trace option, cost estimates, private model routing metadata | full provider proxy |
| Enterprise | RBAC, policy source, audit, secrets check, runners inventory, compliance | real remote runner execution and network policy server |
| Security | scope/diff guards, redaction, enterprise permissions | command allowlist enforcement and strong sandbox levels |
| Domains | Code, Infra, Data, Content | Media open, Research partial |
| Metrics | artifacts include cost/tokens/status | metric dashboards and aggregated KPIs |

## Roadmap Phase Matrix

| Phase | Status | Evidence |
|---|---|---|
| Phase 1 Execution Kernel Foundation | Done | `prd/done/phase-01-execution-kernel-foundation.md` |
| Phase 2 Observability and LLM Gateway | Done | `prd/done/phase-02-observability-llm-gateway.md` |
| Phase 3 VCM-OS Core | Done | `prd/done/phase-03-vcm-os-core.md` |
| Phase 4 AgentSpec YAML and Compiler | Done | `prd/done/phase-04-agentspec-yaml-compiler.md` |
| Phase 5 Skill Registry v1 | Done | `prd/done/phase-05-skill-registry-v1.md` |
| Phase 6 Agent Adapters v1 | Done | `prd/done/phase-06-agent-adapters-v1.md` |
| Phase 7 Runtime Smoke and Repair Loop | Done | `prd/done/phase-07-runtime-smoke-repair-loop.md` |
| Phase 8 Context Maps | Done | `prd/done/phase-08-context-maps.md` |
| Phase 9 Natural Language to AgentSpec | Done | `prd/done/phase-09-natural-language-agentspec.md` |
| Phase 10 Advanced Agent Topologies | Done | `prd/done/phase-10-advanced-agent-topologies.md` |
| Phase 11 Additional Workspaces | Done | `prd/done/phase-11-additional-workspaces.md` |
| Phase 12 IDE and Visual Layer | Done | `prd/done/phase-12-ide-and-visual-layer.md` |
| Phase 13 Marketplace / Plugin Ecosystem | Done | `prd/done/phase-13-marketplace-plugin-ecosystem.md` |
| Phase 14 Enterprise Layer | Done | `prd/done/phase-14-enterprise-layer.md` |
