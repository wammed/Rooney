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
  <a href="README.md">English</a> | <a href="README.ja.md">日本語</a> | <a href="LICENSES.md">Licenses</a>
</p>

</div>

---

## 💡 Features

- ⚡ **COSMIC & Wayland-Native Core**: Pure Rust `libcosmic` desktop integration with custom matte icon, Wayland client-side decorations, seamless dark/light theme integration, and fast startup without heavy external language server daemons.
- 🏎️ **Virtualized Viewport & Cumulative Visual Row Model**: Sub-millisecond layout computation (< 0.40 ms even on 50MB / 1.8M-line documents) rendering strictly the visible window plus margins, delivering 144+ FPS butter-smooth scrolling. Features a sparse $O(\log W)$ cumulative wrap model (`LineWrapModel`) that strictly places subsequent lines below all wrapped subrows with non-overlapping Y coordinates, accurate visual row mapping, and sub-pixel mouse hit-testing.
- 🌳 **Unified Generation-Tracked Async Architecture (Tree-sitter, Markdown & Search)**: Heavy Tree-sitter parsing for massive files (> 2MB–50MB), Markdown document parsing (on toggle, edit, or spec changes across all active and inactive tabs), and search queries are offloaded to background workers (`tokio::task::spawn_blocking`), completely eliminating UI thread freezing. All asynchronous pipelines are strictly protected by integer generation counters (`parse_generation`, `markdown_generation`, and `search_generation`), guaranteeing that in-flight results from typing, rapid file switches, save-as renames, or undo/redo are cleanly validated and stale results are discarded without timestamp drift. Keeps keystroke latency down to **~3.9 µs** in release mode (< 0.1 ms in debug) even on 50MB buffers.
- 🤖 **100% Local AI Intelligence**: Real-time Fill-in-the-Middle (FIM) ghost suggestions (`Ctrl + I` / `Alt + Enter`), streaming token-by-token interactive chat panel (`Ctrl + Shift + A`) with cancellation and model switching powered completely offline by Ollama.
- 📜 **Piece-Tree Rope Buffer**: Powered by `ropey`, enabling efficient character and line manipulations across documents without large-scale contiguous memory copies, backed by $O(1)$ max line length tracking.
- 🪟 **Dual-Pane Split Editing & GFM-Oriented Markdown Live Preview**: Side-by-side editing (`Ctrl + \`), independent multi-tabs, synchronized editing, and live Markdown preview (`Ctrl + M`) with dynamic specification switching between **GFM** (GitHub Flavored Markdown) and standard **CommonMark**.
  - **GFM-Oriented Markdown Support (Text-Focused Rendering)**: Pure native Rust renderer engineered without bloated browser engines (WebView), CSS layout engines, or JavaScript. Delivers rich and accurate coverage of GFM tables (with shaped glyph metrics alignment and horizontal scrolling), alerts (Callouts: `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`), nested task lists, composite inline formatting (nested bold, italic, strikethrough, inline code), display-only styled links and autolinks, footnotes, and strict `breaks: false` adherence. External links are display-only / non-clickable by deliberate design. List items preserve nesting through depth and source order, while nested block content such as code blocks is retained in each item's children.
  - **Zero Remote I/O Markdown Policy**: **Markdown rendering performs zero remote I/O.** It does not fetch remote images, execute scripts, launch external browsers, or make HTTP/network requests. (Note: Rooney communicates solely over `localhost:11434` with the local Ollama daemon for AI completions, and never performs remote cloud I/O).
  - **Intentional Image-Free Design & Safe HTML Fallback**: Images are rendered as safe textual fallback: image syntax (`![alt](url)` / `<img>`) is represented using a lightweight offline textual/badge fallback (`󰋩 [Image: alt]`). HTML tags are safely parsed to textual fallbacks (e.g. `<br>` to line breaks) without full HTML browser rendering.
- 🧭 **Smooth Auto-Scrolling & Dual Interactive Scrollbars**: Cursor-following auto-scroll keeps the editing cursor visible at top/bottom margins during typing and navigation. Each pane features an independent, proportional right-edge scrollbar supporting smooth dragging, mouse wheel disengagement, and track jumping with $O(1)$ content height calculations.
- 🇯🇵 **Unicode Grapheme Cluster Integrity & Real Glyph Metrics**: Full Wayland text-input protocol support (Fcitx5 / Mozc / IBus), live pre-edit preview, and strict Grapheme Cluster boundary enforcement (`unicode-segmentation`) ensuring combining characters (`é`) and emoji ZWJ sequences (`👨‍💻`) move, delete, and select indivisibly. Backed by `cosmic-text` shaped glyph metrics with `RwLock` shared concurrency, fast-path CJK / Tab handling, and `clear_glyph_cache`, fully preventing cursor drift on dashes (`――`) and CJK symbols.
- 📂 **Rich File Tree Explorer**: Workspace navigation, Nerd Font file icons, right-click context menu (New File, New Folder, Rename, Safe Delete) with scroll position retention and screen-boundary clamping, and XDG Desktop Portal integration.
- 🎨 **20 Premium Classic & Neon Themes**: Tokyo Night, Catppuccin, Gruvbox, Synthwave '84, Cyberpunk Neon, and more, with independent opacity controls for editor window, file tree, and title bar, plus background dimming unified in the `󰒓 Aesthetics & Preferences` modal.
- 🔒 **Security & File Integrity**: Atomic file replacement (`.{file}.tmp.{pid}`) to reduce partial write risks on sudden failures, pattern-based exclusion of known sensitive files (`.env*`, `id_rsa`, `*.pem`) from AI context, 50MB file size limits, and path traversal guards.

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

### 6. Persistent Configuration

Rooney automatically preserves all settings—theme, font, font size, independent opacities, dimming, local AI model, Markdown specification, and multi-tab session state—in `~/.config/rooney/config.toml`. Your editing environment is instantly restored every time you launch.

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
| `Ctrl + ,` | Open **Aesthetics Preferences** (Theme, Font, Size, Opacities, Local AI, Markdown Spec: GFM/CommonMark) |
| `Esc` | Close active modal, search bar, dropdown, or cancel completion |

---

## 🔒 Security & Architecture (Overview)

Rooney operates under a strict **100% offline, local-first** model:
- **Zero Cloud Leakage**: No telemetry, external cloud APIs, or tracking; all AI prompts and code streams remain strictly local via Ollama.
- **Pattern-Based Sensitive File AI Guard**: Automatically detects known sensitive file patterns (`.env*`, `id_rsa`, `id_ed25519`, `credentials`, `*.pem`, `*.key`, `*.keystore`, `*.jks`, `.npmrc`, `.pypirc`, `kubeconfig`, `*.token`, `*secret*`, `.aws/credentials`, `.kube/config`) and excludes them from AI completion requests to prevent accidental exposure.
- **Atomic Crash-Resistant Writing**: Saves editor files and `config.toml` via sibling temporary files (`.{filename}.tmp.{pid}`) with `sync_all` and atomic rename, reducing the risk of partial or zero-byte writes during unexpected system crashes.
- **Path Traversal & Boundary Protection**: Sanitizes file/directory creation and renaming to prevent directory escape (`..`, `/`), while safeguarding root (`/`) and home directories against accidental recursive deletion. Safe home directory resolution via `directories::BaseDirs`.
- **Resource Limits, Bounded Memory & Zero Remote I/O Markdown**: 50MB file size ceiling to prevent OOM freezes, 1MB streaming buffer cap, bounded $O(1)$ ring-buffer Undo/Redo stack (`VecDeque`), and heap-efficient direct iterator traversal during rendering. Additionally, **Markdown rendering performs zero remote I/O**: it deliberately omits heavy image decoders and WebViews, eliminating memory bloat and decoder exploit risks from untrusted external image files.

---

## 🗺️ Roadmap

The following architectural optimizations and feature enhancements are planned for upcoming milestones:

- [x] 🏎️ **Viewport-Based Virtualization**: Layout and visual row computation limited strictly to the visible viewport, achieving sub-millisecond layout (< 0.45 ms) on 10KB–50MB buffers.
- [x] ⚡ **Incremental Syntax Highlighting & Line Cache**: AST delta parsing via Tree-sitter `InputEdit` and per-line highlight token caching to eliminate reparsing and tokenization overhead on typing.
- [x] 📊 **Multi-Scale Performance Benchmark Suite**: Built-in benchmark harness spanning 10KB to 50MB files (verifying buffer load, initial parse, incremental edit, viewport layout, and cache hits).
- [x] 📐 **Direct Glyph Metrics Integration**: Integrating real layout measurements from `cosmic-text` to precisely measure glyph advances and eliminate cursor position drift on dashes (`――`) and CJK symbols.
- 👁️ **Filesystem Change Monitoring (Watcher)**: Asynchronous workspace directory notifications to automatically refresh the file tree and notify users of external file modifications.
- 📝 **Rich Inline Markdown Rendering**: Direct in-canvas styling of inline Markdown elements (bold, italic, inline code, and styled link representations).
- 💾 **Undo/Redo Delta Compression**: Transitioning from Rope snapshot checkpoints to compact delta-based action logs for memory efficiency on huge edit sessions.

---

## 🤖 About This Project (AI Vibe Coding)

> [!IMPORTANT]
> ### 💡 AI Vibe Coding Project
> **Rooney** is an **AI Vibe Coding** project created through real-time interactive pair programming with **Google DeepMind's Antigravity (Gemini)**.
> Combining human architectural vision with agentic AI pair programming, the entire system—from low-level Rust `libcosmic` Wayland text-input protocols, in-process Tree-sitter AST parsing, Piece-Tree Ropey buffers, sub-pixel CJK font metrics, local Ollama SSE token streaming, to the dual-pane desktop editor interface—was designed, tested, and implemented in full creative flow.

---

## 📄 License

Rooney's own source code is licensed under the [MIT License](LICENSE).

Third-party dependencies utilized by Rooney are subject to their respective upstream licenses, including `MPL-2.0`, `Apache-2.0`, `MIT`, and `GPL-3.0-only`. For comprehensive details on licensing policies, the source-only distribution model, pinned Git dependency revisions, and downstream redistribution guidance, please see **[LICENSES.md](LICENSES.md)** ([日本語版: LICENSES.ja.md](LICENSES.ja.md)) and [THIRD_PARTY_LICENSES/README.md](THIRD_PARTY_LICENSES/README.md).

<p align="center">
  Crafted via <strong>AI Vibe Coding</strong> 🚀 · Built with ❤️ for Pop!_OS COSMIC & Linux Developers
</p>
