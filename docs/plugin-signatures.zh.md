# Plugin Signatures

语言: [English](plugin-signatures.en.md), [Русский](plugin-signatures.ru.md), [中文](plugin-signatures.zh.md), [Қазақша](plugin-signatures.kk.md)

当 `signature.algorithm` 为 `sha256` 时，AgentHub 会验证 plugin package signature。Trusted install 必须有 verified cryptographic signature。

## Signing Flow

先添加带占位值的 signature metadata：

```yaml
signature:
  algorithm: sha256
  signer: Example Team
  value: pending
```

计算 digest：

```bash
agenthub plugins digest ./marketplace/skill-packs/my-pack
```

把输出的 digest 写入 `signature.value`，然后验证并安装：

```bash
agenthub plugins inspect ./marketplace/skill-packs/my-pack
agenthub plugins install ./marketplace/skill-packs/my-pack --trust trusted
```

Digest 覆盖一个把 `signature.value` 置空后的 canonical manifest，以及除 manifest 本身以外的所有 package files。Local packages 可以保持 unsigned 或使用 `algorithm: none`；`--trust trusted` 要求 `sha256`。
