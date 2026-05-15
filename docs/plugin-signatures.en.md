# Plugin Signatures

Languages: [English](plugin-signatures.en.md), [Русский](plugin-signatures.ru.md), [中文](plugin-signatures.zh.md), [Қазақша](plugin-signatures.kk.md)

AgentHub verifies plugin package signatures when `signature.algorithm` is `sha256`. Trusted installs require a verified cryptographic signature.

## Signing Flow

Add signature metadata with a placeholder value:

```yaml
signature:
  algorithm: sha256
  signer: Example Team
  value: pending
```

Compute the digest:

```bash
agenthub plugins digest ./marketplace/skill-packs/my-pack
```

Put the printed digest into `signature.value`, then verify and install:

```bash
agenthub plugins inspect ./marketplace/skill-packs/my-pack
agenthub plugins install ./marketplace/skill-packs/my-pack --trust trusted
```

The digest covers a canonical manifest with `signature.value` empty plus every package file except the manifest itself. Local packages may stay unsigned or use `algorithm: none`; `--trust trusted` requires `sha256`.
