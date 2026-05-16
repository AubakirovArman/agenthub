# AgentHub v0.4 API-native runtime

## Цель

AgentHub должен перестать быть оболочкой над чужими CLI. В v0.4 внешний LLM слой ограничен двумя API-провайдерами: DeepSeek и Kimi. Это дает собственные логи, контролируемую память, предсказуемый retry/fallback и возможность строить sub-agent orchestration внутри AgentHub.

## Режимы запуска

- Chat mode: обычный `agenthub` в папке без проекта не требует Git и `.agent`; история, индекс чатов и command logs пишутся в глобальный AgentHub home.
- DevOps mode: пользователь может обсуждать сервер и запускать `!command` без создания файлов проекта.
- Project mode: `.agent` и Git нужны только когда пользователь запускает transaction с изменением файлов, verifier, rollback, commit и memory promotion.

## Провайдеры

- `deepseek`: OpenAI-compatible DeepSeek API, ключ `DEEPSEEK_API_KEY`; legacy `ANTHROPIC_AUTH_TOKEN` можно переиспользовать для DeepSeek-compatible deployments.
- `kimi`: Kimi/Moonshot API, ключ `KIMI_API_KEY` или `MOONSHOT_API_KEY`.
- `command`: внутренний deterministic runner для transaction kernel и тестов; это не внешний AI provider.

На сервере можно положить ключи в `.deepseek` и `.kimi` в project directory или любой parent directory. AgentHub читает эти файлы как runtime secrets и не сохраняет содержимое в config.

DeepSeek API, Kimi API, legacy aliases и generic custom profiles больше не являются user-facing provider surface.

## Текущий статус

- Non-project shell уже не инициализирует Git и `.agent` автоматически.
- Chat/history/index/command logs получают global home fallback.
- Non-project plain messages идут напрямую в DeepSeek/Kimi API, если ключ настроен, и печатают streaming SSE output.
- Project transaction routes для `deepseek`/`kimi` используют API-native JSON command executor: provider возвращает command plan, AgentHub валидирует команды, запускает их в isolated worktree и пишет `.agent/tx/<tx-id>/api_execution_<role>.json`.

## Следующие этапы

1. Streaming SSE events для dashboard, не только terminal chat.
2. Tool-calling loop: structured shell/file/read/diff/verifier tools вместо одноразового JSON command plan.
3. Sub-agent manager/worker orchestration внутри AgentHub, без внешних CLI.
4. Memory policy: global user memory, project memory, failed-attempt warnings, promotion rules.
5. UI rewrite: отдельные Chat, DevOps и Project transaction screens.
6. Cost/token accounting per provider, request, role and transaction.
