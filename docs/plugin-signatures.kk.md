# Plugin Signatures

Тілдер: [English](plugin-signatures.en.md), [Русский](plugin-signatures.ru.md), [中文](plugin-signatures.zh.md), [Қазақша](plugin-signatures.kk.md)

`signature.algorithm` `sha256` болса, AgentHub plugin package signature тексереді. Trusted install үшін verified cryptographic signature керек.

## Signing Flow

Алдымен placeholder value бар signature metadata қосыңыз:

```yaml
signature:
  algorithm: sha256
  signer: Example Team
  value: pending
```

Digest есептеу:

```bash
agenthub plugins digest ./marketplace/skill-packs/my-pack
```

Шыққан digest мәнін `signature.value` ішіне қойып, тексеріп орнатыңыз:

```bash
agenthub plugins inspect ./marketplace/skill-packs/my-pack
agenthub plugins install ./marketplace/skill-packs/my-pack --trust trusted
```

Digest `signature.value` бос canonical manifest және manifest-тен басқа барлық package files қамтиды. Local packages unsigned бола алады немесе `algorithm: none` қолдана алады; `--trust trusted` үшін `sha256` керек.
