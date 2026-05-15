# Plugin Signatures

Языки: [English](plugin-signatures.en.md), [Русский](plugin-signatures.ru.md), [中文](plugin-signatures.zh.md), [Қазақша](plugin-signatures.kk.md)

AgentHub проверяет подписи plugin package, когда `signature.algorithm` равен `sha256`. Trusted install требует verified cryptographic signature.

## Signing Flow

Добавьте signature metadata с временным value:

```yaml
signature:
  algorithm: sha256
  signer: Example Team
  value: pending
```

Посчитайте digest:

```bash
agenthub plugins digest ./marketplace/skill-packs/my-pack
```

Запишите напечатанный digest в `signature.value`, затем проверьте и установите:

```bash
agenthub plugins inspect ./marketplace/skill-packs/my-pack
agenthub plugins install ./marketplace/skill-packs/my-pack --trust trusted
```

Digest покрывает canonical manifest с пустым `signature.value` и все package files, кроме самого manifest. Local packages могут быть unsigned или использовать `algorithm: none`; `--trust trusted` требует `sha256`.
