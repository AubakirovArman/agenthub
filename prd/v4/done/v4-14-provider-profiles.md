# V4.14 Provider Profiles

## Status

Done for OpenAI-compatible local/team endpoint profiles.

## Completed

- Added `agenthub providers add openai-http --name <name> --url <url>`.
- Added optional `--model` and `--api-key-env` fields.
- Stored profiles in `.agent/config.yaml` under `provider.profile.<name>.*`.
- Made profiles appear in provider status and doctor provider checks.
- Allowed profiles to be selected with `providers setup`, `providers set <role>`, and `providers fallback`.
- Made `providers diagnose <profile>` show profile kind, endpoint, model, and API key environment marker.
- Made `providers test <profile>` perform the same OpenAI-compatible completion test as `openai-http`.
- Added shell shortcut syntax: `/providers add openai-http <name> <url> [model] [api_key_env]`.
- Updated provider docs in English, Russian, Chinese, and Kazakh.

## Evidence

- `cargo test providers_add_named_openai_http_profile`

## 1.0 Relevance

This makes local model setups such as `local-vllm`, `ollama`, `lm-studio`, `openrouter`, and company proxy endpoints reusable without relying on a single global `AGENTHUB_OPENAI_COMPAT_BASE_URL`.
