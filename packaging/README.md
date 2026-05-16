# AgentHub Packaging Templates

These templates prepare package-manager manifests for release maintainers.
They are not published package repositories by themselves.

Render manifests after release archives and `.sha256` files exist:

```bash
AGENTHUB_PACKAGE_DIST=dist scripts/render-package-manifests.sh
```

Generated output:

```text
target/package-manifests/homebrew/agenthub.rb
target/package-manifests/scoop/agenthub.json
target/package-manifests/winget/AubakirovArman.AgentHub*.yaml
```

Current preview assets:

- Linux x86_64
- macOS Apple Silicon
- Windows x86_64

Intel macOS assets are intentionally not published.
