<div align="center">

# 🚀 Rooney
### AI-Integrated Next-Generation Linux Code & Markdown Editor for COSMIC / Wayland

![Banner](./images/Rooney-banner.svg)

[![Built with libcosmic](https://img.shields.io/badge/libcosmic-Pop!_OS_COSMIC-24C8D8?style=for-the-badge&logo=linux&logoColor=white)](https://github.com/pop-os/libcosmic)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Wayland](https://img.shields.io/badge/Wayland-Native-5277C3?style=for-the-badge&logo=wayland&logoColor=white)](https://wayland.freedesktop.org/)
[![Ollama](https://img.shields.io/badge/Ollama-Local_AI-white?style=for-the-badge&logo=ollama&logoColor=black)](https://ollama.com/)
[![Platform](https://img.shields.io/badge/Platform-Linux_(COSMIC_/_Wayland)-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://www.kernel.org/)
[![Vibe Coding](https://img.shields.io/badge/Built_with-AI_Vibe_Coding-8A2BE2?style=for-the-badge&logo=sparkles&logoColor=white)](#-about-this-project-ai-vibe-coding)
[![License: MIT](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

<p align="center">
  <strong>COSMIC / Wayland Native × Piece-Tree Rope Buffer × Tree-sitter Highlighting × Japanese IME Native × 100% Local AI (Ollama)</strong><br>
  A fast, privacy-respecting, and aesthetic lightweight code & markdown editor built natively for Pop!_OS COSMIC Desktop and Linux Wayland.
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.ja.md">日本語</a> | <a href="docs/PORTAL.md">📚 Documentation Portal</a> | <a href="LICENSES.md">Licenses</a>
</p>

</div>

---

## 🚀 Quick Start

### 1. Prerequisites

- [Rust (Cargo)](https://rustup.rs/) (1.80+)
- Linux with Wayland / Pop!_OS COSMIC Desktop
- System build dependencies (Debian / Pop!_OS / Ubuntu):
  ```bash
  sudo apt install build-essential libxkbcommon-dev libfontconfig1-dev
  ```
- [Ollama](https://ollama.com/) (for offline AI capabilities)

### 2. Set Up Ollama (Offline Local AI)

```bash
# Start Ollama daemon
ollama serve

# Pull your preferred local coding model(s)
ollama pull qwen2.5-coder
# Or general assistant model
ollama pull llama3.2
```

### 3. Run in Development Mode

```bash
# Clone the repository
git clone https://github.com/wammed/Rooney.git
cd Rooney

# Run development build
cargo run
```

### 4. Production Build & Installation

```bash
# Build optimized release binary and install to user path
cargo build --release
install -m 755 target/release/rooney ~/.local/bin/rooney
```
*Standalone binary installed to: `~/.local/bin/rooney`.*

### 5. CLI Usage & Desktop File Association

Rooney fully supports command-line arguments, desktop file managers, and `.desktop` (`Exec=rooney %f`) integration:
```bash
# Open specific file(s)
rooney document.txt /path/to/script.rs

# Open from file:// URI (desktop file manager double-click)
rooney file:///home/user/notes.md

# Open directory in file tree
rooney /path/to/project
```

### 6. Configuration Persistence

Selected themes, fonts, font size, pane opacities, dimming, AI model, Markdown specification (GFM / CommonMark), split layout, and open tab sessions are automatically saved and restored via `~/.config/rooney/config.toml`.

---

## 💡 Key Features

- ⚡ **COSMIC & Wayland Native**: Pure Rust `libcosmic` desktop integration. Fast startup without heavy language server daemons and automatic system dark/light theme tracking.
- 🏎️ **Virtualized Viewport (< 0.40 ms)**: Renders only visible lines plus margins even on 50MB / 1.8M-line files, achieving 144+ FPS butter-smooth scrolling and an $O(\log W)$ cumulative wrap model.
- 🌳 **Unified Generation-Tracked Async Engine**: Offloads Tree-sitter parsing, Markdown parsing, and search to background worker threads. Integer generation counters eliminate UI freezes and prevent stale highlights.
- 🤖 **100% Local AI Intelligence (Ollama)**: Zero cloud telemetry. Features real-time FIM inline ghost completions (`Ctrl + I`) and streaming conversational AI chat (`Ctrl + Shift + A`).
- 📜 **Piece-Tree Rope Buffer (`ropey`)**: Avoids contiguous memory relocations on large edits, backed by $O(1)$ line-length soft-wrap recalculations.
- 🪟 **Dual-Pane Split & GFM Markdown Preview**: Side-by-side editing (`Ctrl + \`) and pure native Rust GFM rendering without WebView overhead. Strict **Zero Remote I/O** design.
- 🇯🇵 **Unicode Grapheme Integrity & Shaped Glyph Metrics**: Native Wayland IME support (Fcitx5 / Mozc / IBus). Atomic caret movement across combining characters/ZWJ emoji, plus `cosmic-text` shaped glyph caching preventing cursor drift on CJK dashes (`――`).
- 🎨 **20 Premium Themes & Independent Opacity**: Curated color schemes (Tokyo Night, Catppuccin, Gruvbox) with independent opacity tuning for editor canvas, file tree, and title bar.

> 📖 **Full Specifications & Architectural Details**:
> For complete technical feature specs, see [docs/FEATURES.md](docs/FEATURES.md). For deep internal architecture and generation tracking, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## ⌨️ Essential Keyboard Shortcuts

Frequently used keyboard shortcuts:

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + N` / `Ctrl + O` | **New File** / **Open File** |
| `Ctrl + S` / `Ctrl + Shift + S` | **Save File** (Atomic safe write) / **Save As...** |
| `Ctrl + T` / `Ctrl + W` | **New Tab** / **Close Tab** |
| `Ctrl + Tab` / `Ctrl + Shift + Tab` | Next Tab / Previous Tab |
| `Ctrl + \` or `Ctrl + E` | **Toggle Dual-Pane Split** (Split / Single) |
| `Ctrl + M` | **Toggle Markdown Preview** (GFM / CommonMark) |
| `Ctrl + B` | **Toggle File Tree Sidebar** |
| `Ctrl + F` | **Toggle In-Buffer Incremental Search** |
| `Ctrl + Shift + A` | **Toggle Streaming AI Chat Panel** |
| `Ctrl + I` or `Alt + Enter` | **Trigger Local AI FIM Inline Completion** |
| `Ctrl + /` | **Toggle Line Comment** (Language-aware) |
| `Ctrl + ,` | **Aesthetics & Preferences** (Themes, Fonts, Opacities, AI Models) |
| `Esc` | Dismiss modals, search bar, or ghost completions |

> ⌨️ **Complete Reference**:
> For editing navigation (Undo/Redo, line duplication/deletion, indentation) and full keybindings, see **[docs/SHORTCUTS.md](docs/SHORTCUTS.md)**.

---

## 📚 Documentation Portal

Comprehensive documentation for Rooney is organized into focused reference guides:

| Document | Description |
| :--- | :--- |
| **[📚 Documentation Portal](docs/PORTAL.md)** | Master hub and audience-oriented guide index |
| **[⌨️ Shortcuts & Operations Guide](docs/SHORTCUTS.md)** | Full keybinding reference, multi-pane splitting, and AI workflows |
| **[💡 Feature Specifications (FEATURES)](docs/FEATURES.md)** | Exhaustive feature capabilities, GFM Markdown specs, and themes |
| **[📐 System Architecture (ARCHITECTURE)](docs/ARCHITECTURE.md)** | Generation-tracked async pipelines, viewport virtualization, and Rope buffer |
| **[🛡️ Security & Robustness (SECURITY)](docs/SECURITY.md)** | Zero cloud telemetry, sensitive file exclusion, atomic writes, Zero Remote I/O |

---

## 🔒 Security & Privacy (Overview)

- **100% Offline & Zero Cloud Telemetry**: Zero analytics, telemetry, or remote cloud API calls. All AI tasks execute strictly against your local Ollama daemon.
- **Sensitive File Exclusion**: Blocks AI completion requests when editing `.env*` or private key files (`id_rsa`, `*.pem`, `credentials`).
- **Atomic Safe Writes**: Writes to temporary files (`.{file}.tmp.{pid}`) before replacement to prevent corruption on abrupt termination.
- **Zero Remote I/O Markdown**: Markdown preview never makes network queries or downloads external images, safely rendering textual fallback badges.

> 🛡️ **In-Depth Security Specifications**:
> For full defensive architecture and security boundaries, see **[docs/SECURITY.md](docs/SECURITY.md)**.

---

## 🗺️ Roadmap

- [x] 🏎️ **Virtualized Viewport**: Sub-millisecond (< 0.40 ms) layout rendering strictly visible lines on 50MB files.
- [x] ⚡ **Tree-sitter Incremental Parsing & Cache**: Minimizes re-parse overhead during active typing.
- [x] 📊 **Multi-Scale Benchmark Suite**: Automated benchmarks from 10KB up to 50MB across all subsystems.
- [x] 📐 **Shaped Glyph Metrics Integration**: Eliminates cursor drift on fullwidth dashes (`――`) and CJK symbols.
- [ ] 👁️ **File System Watcher**: Asynchronously monitors and syncs workspace directory changes from external tools.
- [ ] 📝 **Rich Inline Markdown AST**: Renders bold, italic, code, and links directly on the editor canvas.
- [ ] 💾 **Delta-Based Undo/Redo**: Transitions from buffer snapshot cloning to operation delta tracking for memory efficiency.

---

## 🤖 About This Project (AI Vibe Coding)

> [!IMPORTANT]
> ### 💡 Built with AI Vibe Coding
> **Rooney** was created through interactive pair programming (**AI Vibe Coding**) with **Google DeepMind's Antigravity (Gemini)**.
> Combining human architectural direction and iterative refinement with AI implementation, debugging, and profiling, the entire stack was built from scratch—ranging from low-level Rust `libcosmic` Wayland text-input integration, in-process Tree-sitter parsing, piece-tree Ropey buffer manipulation, CJK sub-pixel font layout, local Ollama SSE streaming, to the dual-pane desktop editor UI.

---

## 📄 License

Rooney's source code is licensed under the [MIT License](LICENSE).

Third-party dependencies utilized by Rooney are governed by their respective upstream licenses (`MPL-2.0`, `Apache-2.0`, `MIT`, `GPL-3.0-only`, etc.). For full licensing details, source code distribution policies, locked Git dependency commitments, and binary redistribution requirements, please refer to **[LICENSES.md](LICENSES.md)** ([Japanese: LICENSES.ja.md](LICENSES.ja.md)) and [THIRD_PARTY_LICENSES/README.md](THIRD_PARTY_LICENSES/README.md).

<p align="center">
  Crafted via <strong>AI Vibe Coding</strong> 🚀 · Built with ❤️ for Pop!_OS COSMIC & Linux Developers
</p>
