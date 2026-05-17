# Competitive Positioning

AgentHub 不是另一个 coding agent。它是围绕 API-native DeepSeek/Kimi provider work 和 deterministic command execution 的本地事务型 runtime。

## 定位

```text
AgentHub = provider-neutral agent control plane + transaction safety + project memory
```

第一阶段的产品承诺很窄：在本地运行 AI-agent work，并得到 verified commit、clean rollback，或明确的 human block。

## 与 Raw Agent CLI 对比

原始 agent CLI 擅长生成 edits，但通常把 safety orchestration 留给用户。AgentHub 增加：

- isolated transaction workspaces；
- diff guard 和 smart sync；
- verifier commands 和 runtime smoke checks；
- effect ledger 和 rollback report；
- 只有 verified success 后才 promotion memory；
- dashboard、TUI 和 transaction history。

## 与 IDE Assistant 对比

IDE assistants 优化 interactive editing。AgentHub 优化 auditable task execution。它可以和 IDE 一起使用，但它的工作单元是带 artifacts 的 transaction，而不是单条 editor suggestion。

## 与 CI 对比

CI 在 changes 已经存在后检查分支。AgentHub 在 commit 前运行，记录 changes 为什么发生，并可在污染 project truth 前 rollback 或 block。

## 与 Hosted Orchestration 对比

Hosted tools 适合 team centralization 和 billing。AgentHub 是 local-first：source code、memory、reports 和 provider config 默认留在项目中，除非用户主动 export。

## 不应声称的内容

不要声称 AgentHub 已经是完整的 untrusted code security sandbox。当前 local execution 提供 transaction isolation、process supervision、command policy 和 hardening reports。强隔离需要 Docker 或 remote runners 等 hardened runner backends。
