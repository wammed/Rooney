# 🛡️ Rooney Security & Robustness Model

Detailed security principles, privacy models, and defensive boundaries in Rooney.

---

## 📑 Security Principles
1. [100% Offline & Zero Cloud Telemetry](#1-100-offline--zero-cloud-telemetry)
2. [Automated Sensitive File Exclusion](#2-automated-sensitive-file-exclusion)
3. [Atomic Safe Writes](#3-atomic-safe-writes)
4. [Path Traversal & Boundary Protection](#4-path-traversal--boundary-protection)
5. [Resource Safeguards & Zero Remote I/O Markdown](#5-resource-safeguards--zero-remote-io-markdown)

---

## 1. 100% Offline & Zero Cloud Telemetry
- **No Remote Telemetry**:
  - Rooney does not collect usage analytics, send crash dumps, or make network calls to external cloud services.
- **Local-Only AI Routing**:
  - Code completion and chat features communicate exclusively with the locally running Ollama daemon (`http://127.0.0.1:11434`).
  - Code fragments, keystrokes, and prompts never leave your local system.

---

## 2. Automated Sensitive File Exclusion
To protect developers from accidentally passing credentials or secrets to AI inference engines, Rooney automatically suppresses AI suggestions and context extraction when opening files matching sensitive patterns:

- `.env*` (Environment variables)
- SSH keys and credentials (`id_rsa`, `id_ed25519`, `credentials`, `*.pem`, `*.key`, `*.keystore`, `*.jks`)
- Cloud and package tokens (`.npmrc`, `.pypirc`, `kubeconfig`, `*.token`, `*secret*`, `.aws/credentials`, `.kube/config`)

---

## 3. Atomic Safe Writes
- **Corruption Prevention**:
  - Documents and preferences (`~/.config/rooney/config.toml`) are never truncated in-place.
  - New contents are written to a temporary sibling file (`.{filename}.tmp.{pid}`), flushed to persistent disk storage via `sync_all`, and replaced via an atomic `rename` system call.
  - This ensures files are never left in a partially written or corrupted state if power is lost or the application is killed during a write.

---

## 4. Path Traversal & Boundary Protection
- **Path Sanitization**:
  - File/folder creation and renaming via the sidebar validate and sanitize traversal characters (`..`, `/`) to prevent directory escape.
- **Destructive Deletion Guards**:
  - Root (`/`) and user home (`~`) paths are explicitly blocked from recursive deletion routines.
- **Standardized Base Paths**:
  - Safe home directory and config paths are resolved using `directories::BaseDirs`.

---

## 5. Resource Safeguards & Zero Remote I/O Markdown

### Zero Remote I/O Markdown Policy
> **Markdown rendering performs zero remote I/O.**  
> Rooney's Markdown preview engine never fetches remote network assets, never launches an external web browser, and executes zero JavaScript.

- **Safe Text Fallbacks for Images**:
  - Image markup (`![alt](url)` and `<img>`) is rendered as a clean, offline badge (`󰋩 [Image: alt]`) without issuing HTTP requests.
- **Display-Only Links**:
  - Links are rendered with distinct visual styling for reference, but are intentionally non-clickable to eliminate inadvertent external navigation.

### Resource Safeguards
- **50MB File Size Threshold**:
  - Protects the editor from memory exhaustion or UI freezing when opening large binary dumps.
- **Bounded Buffers**:
  - Caps streaming AI responses at 1MB and bounds the Undo/Redo history via a circular `VecDeque` ring buffer to keep memory usage strictly bounded.

---

<p align="center">
  <a href="PORTAL.md">← Back to Portal</a> | <a href="SECURITY.ja.md">日本語版セキュリティ仕様書 →</a>
</p>
