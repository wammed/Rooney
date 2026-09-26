# 📚 Rooney Documentation Portal

Welcome to the official documentation portal for Rooney.  
Rooney is a 100% offline, local-AI-integrated, next-generation Linux code and Markdown editor built natively from scratch for Pop!_OS COSMIC Desktop and Linux Wayland environments.

Use this hub to quickly navigate to the detailed guides, specifications, and architecture documents suited to your needs.

---

## 🧭 Master Documentation Index

| Document | Primary Topics | Recommended Audience |
| :--- | :--- | :--- |
| **[Quick Start](../README.md#-quick-start)** | Prerequisites, Ollama setup, build, launch, configuration | First-time users & developers installing Rooney |
| **[⌨️ Shortcuts & Navigation](SHORTCUTS.md)** | Complete keyboard shortcuts, multi-pane splitting, editing, AI controls | Users looking to master keyboard-driven workflow |
| **[💡 Feature Specifications (FEATURES)](FEATURES.md)** | Full feature specifications, GFM Markdown renderer, theme customizations | Users wanting in-depth feature & Markdown specs |
| **[📐 System Architecture (ARCHITECTURE)](ARCHITECTURE.md)** | Generation-tracked async pipelines, virtual viewport, Rope buffer, CJK metrics | Developers interested in internals and performance |
| **[🛡️ Security Model (SECURITY)](SECURITY.md)** | Zero cloud telemetry, sensitive file exclusion, atomic safe writes, zero remote I/O | Security auditors and privacy-conscious users |
| **[📄 Licenses & Notices](../LICENSES.md)** | MIT license, third-party OSS dependencies, redistribution notes | Legal & compliance review |

---

## 🎯 Task-Oriented Guides

### 1. Getting Started Immediately
- Head over to [README.md: Quick Start](../README.md#-quick-start) to launch Rooney via `cargo run` or build an optimized binary with `cargo build --release`.
- For offline AI autocompletion and conversational assistance, install Ollama and run `ollama serve` and `ollama pull qwen2.5-coder`.

### 2. Streamlining Keyboard Workflow
- Consult [SHORTCUTS.md](SHORTCUTS.md) for full keybinding coverage including tab switching, side-by-side splitting (`Ctrl + \` / `Ctrl + E`), Markdown live preview toggle (`Ctrl + M`), streaming AI chat panel (`Ctrl + Shift + A`), and FIM inline ghost completions (`Ctrl + I`).

### 3. Exploring Markdown & Editor Features
- Check [FEATURES.md](FEATURES.md) to explore GFM callouts/alerts, real-glyph metric tables, task lists, zero-remote-I/O security guarantees, 20 bundled color themes, and independent pane opacity controls.

### 4. Deep-Diving into Architecture & Performance
- Explore [ARCHITECTURE.md](ARCHITECTURE.md) to learn how Rooney achieves sub-millisecond layout times (< 0.40 ms) and 144+ FPS on massive 50MB files via viewport virtualization, integer generation counters (`parse_generation`, `markdown_generation`, `search_generation`), and `cosmic-text` shaped glyph caching.

### 5. Reviewing Security & Privacy Guarantees
- Review [SECURITY.md](SECURITY.md) for detailed descriptions of Rooney's zero-cloud privacy architecture, automated sensitive file exclusion patterns (`.env*`, private keys), atomic write guarantees, and directory path traversal guards.

---

<p align="center">
  <a href="../README.md">← Back to Root README</a> | <a href="PORTAL.ja.md">日本語ポータルへ →</a>
</p>
