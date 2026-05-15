# Done PRD Areas

These PRD areas are implemented in the current repository and have tests, docs, or runtime examples.

- Roadmap phases 1-14.
- Transaction kernel with isolated worktrees, commit/rollback, reports, and journal.
- Memory staging, committed memory, failed attempts, compaction.
- AgentSpec YAML, AgentIR, and execution DAG compiler.
- CLI runtime commands and VS Code extension surface.
- Terminal TUI dashboard for transactions, DAG, verifier, cost, memory, and approvals.
- Static browser dashboard for transactions, timeline, agent trace, memory graph, skills, policies, costs, and reports.
- Aggregated metrics dashboard for reliability, context, quality, trust, and cost KPIs.
- Standalone AAL parser, diagnostics, AgentSpec YAML output, and language reference.
- Natural language `ask` preview with defaults, clarification questions, and approval marking.
- Context maps and least-context context pack trace.
- Agent adapters with command/dry-run integration, prompts, transcripts, and role routes.
- Runtime smoke checks and bounded repair loop.
- Advanced topologies for planner/executor, generator/critic, reviewer/repair, and swarm research.
- Manager / Worker topology with fan-out DAG roles and route traces.
- Tournament topology with contestant fan-in, judge role, executor application, and route traces.
- Code, Content, Data, and Infra workspaces.
- MediaWorkspace with media memory schema and `media_render` verifier profile.
- ResearchWorkspace with source, citation, graph, critic, report validation, and research memory schema.
- Domain verifiers for content, data, infra, media, and research.
- Specialized `backend_tdd` verifier profile with test and API response artifact checks.
- Specialized `db_migration` verifier profile with migration, dry-run, rollback, and seed artifact checks.
- Plugin ecosystem with package manifest, scaffold, trust model, optional signature metadata, and locks.
- Cryptographic plugin package signature verification with SHA-256 digest checks and trusted-install enforcement.
- Enterprise RBAC, policy source, audit log, secret checks, runner inventory, private model route metadata, and compliance reports.
- Networked enterprise policy server with HTTP policy loading, built-in YAML server, and token env support.
- Command policy enforcement for `safe`, `needs_approval`, and restricted command lists at transaction preflight.
- Sandbox Level 0/1 execution controls plus Level 2/3 remote runner dispatch and result collection.
- Remote runner execution for sandbox Level 2/3 with `local://` and `ssh://` endpoints.
- README and feature docs on English, Russian, Chinese, and Kazakh for recent phases.
