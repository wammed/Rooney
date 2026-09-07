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
  <strong>High-Performance Piece-Tree Rope × Tree-sitter In-Process Highlighting × 100% Local AI (Ollama) × Wayland IME Native</strong><br>
  A blazing-fast, private, and aesthetic lightweight code & markdown editor built natively for Pop!_OS COSMIC Desktop and Linux Wayland.
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.ja.md">日本語</a>
</p>

</div>

---

## 💡 Highlights

- ⚡ **COSMIC & Wayland-Native Core**: Pure Rust `libcosmic` desktop integration with client-side decorations, seamless dark/light theme integration, and 0ms instant startup without heavy child processes or LSP daemon overhead.
- 🤖 **100% Local AI Intelligence**: Real-time Fill-in-the-Middle (FIM) ghost suggestions (`Ctrl + I` / `Alt + Enter`), streaming token-by-token interactive chat panel (`Ctrl + Shift + A`) with cancellation and model switching powered completely offline by Ollama.
- 📜 **High-Performance Piece-Tree Rope**: Powered by `ropey`, handling large files with zero copy lag and instantaneous buffer operations.
- 🌳 **In-Process Tree-sitter Highlighting**: AST-based syntax highlighting for 12+ languages (Rust, Python, JS, TS, C, C++, Bash, Fish, JSON, TOML, YAML, Markdown) with zero external daemon overhead.
- 🪟 **Dual-Pane Split Editing & Live Markdown Preview**: Side-by-side editing (`Ctrl + \`), independent multi-tabs, synchronized editing, and live GFM Markdown preview (`Ctrl + M`).
- 🇯🇵 **Pixel-Perfect Japanese IME Support**: Full Wayland text-input protocol support (Fcitx5 / Mozc / IBus), live pre-edit underline preview, and sub-pixel advance calculations preventing cursor drift.
- 📂 **Rich File Tree Explorer**: Real-time workspace navigation, Nerd Font file icons, right-click context menu (New File, New Folder, Rename, Safe Delete), and XDG Desktop Portal integration.
- 🎨 **20 Premium Classic & Neon Themes**: Tokyo Night, Catppuccin, Gruvbox, Synthwave '84, Cyberpunk Neon, and more, with window transparency and background dimming unified in the `󰒓 Aesthetics` modal.
- 🔒 **Ironclad Local Security**: Atomic file replacement (`.{file}.tmp.{pid}`), 50MB file size limits, path traversal sanitization, and automatic AI shielding for sensitive files (`.env*`, `id_rsa`, `*.pem`).

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

### 2. Set Up Ollama

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
# Build optimized release binary
cargo build --release

# Install binary to user path
cp target/release/rooney ~/.local/bin/rooney
chmod +x ~/.local/bin/rooney
```
*Standalone binary output: `~/.local/bin/rooney`.*

---

## ⌨️ Keybindings

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + N` | **New File** (modal filename dialog) |
| `Ctrl + O` | **Open File** (XDG Desktop Portal native dialog) |
| `Ctrl + Shift + O` | **Open Folder** (switch file tree workspace) |
| `Ctrl + S` | **Save File** (atomic safe save) |
| `Ctrl + Shift + S` | **Save File As...** |
| `Ctrl + T` | **New Tab** |
| `Ctrl + W` | **Close Active Tab** |
| `Ctrl + Tab` / `Ctrl + Shift + Tab` | Next / Previous Tab |
| `Ctrl + \` or `Ctrl + E` | Toggle **Dual-Pane Split View** |
| `Ctrl + M` | Toggle **Markdown Live Preview** |
| `Ctrl + B` | Toggle **File Tree Sidebar** |
| `Ctrl + F` | Toggle **In-File Incremental Search** |
| `Ctrl + Shift + A` | Toggle **AI Copilot Chat Panel** |
| `Ctrl + I` or `Alt + Enter` | Trigger **Local AI FIM Inline Completion** |
| `Ctrl + /` | Toggle **Line Comment** (language-aware) |
| `Ctrl + Shift + K` | **Delete Current Line** |
| `Ctrl + D` | **Duplicate Current Line** |
| `Ctrl + ,` | Open **Aesthetics Preferences** (Theme, Font, Size, AI) |
| `Esc` | Close active modal, search bar, dropdown, or cancel completion |

---

## 🔒 Security & Architecture (Overview)

Rooney operates under a strict **100% offline, local-first** model:
- **Zero Cloud Leakage**: No telemetry, external telemetry APIs, or cloud models; all AI prompts and code streams remain strictly local via Ollama.
- **Sensitive File AI Shield**: Automatically detects and blocks AI completion requests on sensitive files (`.env*`, `id_rsa`, `id_ed25519`, `credentials`, `*.pem`, `*.key`).
- **Atomic File Writing**: Saves files via sibling temporary files (`.{filename}.tmp.{pid}`) with `sync_all` and atomic rename, eliminating zero-byte corruption on unexpected shutdowns.
- **Path Traversal & Boundary Protection**: Sanitizes file/directory creation and renaming to prevent directory escape (`..`, `/`), while safeguarding root (`/`) and home directories against accidental recursive deletion.
- **Resource Protection**: 50MB file size ceiling to prevent OOM freezes, 1MB streaming buffer cap, and recursive directory scan depth limits.

---

## 🤖 About This Project (AI Vibe Coding)

> [!IMPORTANT]
> ### 💡 AI Vibe Coding Project
> **Rooney** is an **AI Vibe Coding** project created through real-time interactive pair programming with **Google DeepMind's Antigravity (Gemini)**.
> Combining human architectural vision with agentic AI pair programming, the entire system—from low-level Rust `libcosmic` Wayland text-input protocols, in-process Tree-sitter AST parsing, Piece-Tree Ropey buffers, sub-pixel CJK font metrics, local Ollama SSE token streaming, to the dual-pane desktop editor interface—was designed, tested, and implemented in full creative flow.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

<p align="center">
  Crafted via <strong>AI Vibe Coding</strong> 🚀 · Built with ❤️ for Pop!_OS COSMIC & Linux Developers
</p>
