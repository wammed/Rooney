# 💡 Rooney Feature Specifications

Comprehensive technical specifications and capabilities of Rooney.

---

## 📑 Feature Index
1. [COSMIC & Wayland Desktop Integration](#1-cosmic--wayland-desktop-integration)
2. [Virtualized Viewport & Cumulative Visual Row Model](#2-virtualized-viewport--cumulative-visual-row-model)
3. [Unified Generation-Tracked Async Architecture](#3-unified-generation-tracked-async-architecture)
4. [100% Local AI Intelligence (Ollama)](#4-100-local-ai-intelligence-ollama)
5. [Piece-Tree Rope Buffer](#5-piece-tree-rope-buffer)
6. [Dual-Pane Split & GFM Markdown Renderer](#6-dual-pane-split--gfm-markdown-renderer)
7. [Smooth Auto-Scrolling & Dual Interactive Scrollbars](#7-smooth-auto-scrolling--dual-interactive-scrollbars)
8. [Unicode Grapheme Integrity & Shaped Glyph Metrics](#8-unicode-grapheme-integrity--shaped-glyph-metrics)
9. [File Explorer & XDG Portal Integration](#9-file-explorer--xdg-portal-integration)
10. [20 Built-in Themes & Independent Opacity Controls](#10-20-built-in-themes--independent-opacity-controls)
11. [Data Integrity & Security Safeguards](#11-data-integrity--security-safeguards)

---

## 1. COSMIC & Wayland Desktop Integration
- **Zero-Bloat Startup**: Built purely with Rust and `libcosmic`, free from heavy Electron runtimes or bulky background LSP server dependencies.
- **System Theme Synchronization**: Seamlessly follows COSMIC dark/light desktop preferences, with custom matte app icons and Wayland Client-Side Decorations (CSD).
- **Wayland text-input**: First-class support for Japanese IME pre-edit composition (Fcitx5 / Mozc / IBus).

---

## 2. Virtualized Viewport & Cumulative Visual Row Model
- **Sub-millisecond Viewport Layout**: Only computes layout and geometry for rows visible within the viewport plus margin buffers (< 0.40 ms on 50MB / 1.8M-line files), ensuring 144+ FPS butter-smooth scrolling.
- **Sparse Cumulative Row Model (`LineWrapModel`)**: Sparse $O(\log W)$ data structure accurately computes multi-row visual wraps without overlapping Y coordinates, powering precise sub-pixel mouse hit-testing.

---

## 3. Unified Generation-Tracked Async Architecture
- **Non-blocking UI Thread**: Offloads heavy Tree-sitter AST parsing, Markdown document parsing, and full-buffer search queries to thread workers (`tokio::task::spawn_blocking`).
- **Deterministic Integer Generation Counters**: `parse_generation`, `markdown_generation`, and `search_generation` guarantee 100% stale result rejection during rapid typing, file switches, or undo/redo sequences without timestamp drift.
- **Minimal Keystroke Latency**: Sustains ~3.9 µs typing latency in release builds (< 0.1 ms in debug builds) even during active background searches.

---

## 4. 100% Local AI Intelligence (Ollama)
- **Zero Cloud Exposure**: Prompts and code never leave the local machine, communicating strictly over `localhost:11434`.
- **FIM Inline Ghost Suggestions (`Ctrl + I` / `Alt + Enter`)**: Fill-in-the-Middle autocomplete rendered in ghost text; accept with `Tab`, dismiss with `Esc`.
- **Streaming Chat Assistant (`Ctrl + Shift + A`)**: Token-by-token real-time SSE streaming dialogue with cancellation and instant model switching.

---

## 5. Piece-Tree Rope Buffer
- **Scalable Editing**: Powered by `ropey`, avoiding large contiguous memory copies on large file manipulations.
- **$O(1)$ Line Length Tracking**: Tracks line lengths in constant time, allowing instant soft-wrap recalculations on window resize.

---

## 6. Dual-Pane Split & GFM Markdown Renderer
- **Side-by-Side Editing (`Ctrl + \` / `Ctrl + E`)**: View or edit two different files simultaneously, or split the current document side-by-side.
- **GFM-Oriented Native Markdown Support**:
  - **Pure Native Rust Renderer**: Completely eliminates WebView overhead and JavaScript attack surfaces.
  - **Comprehensive Syntax**: Accurately formats GFM tables, alerts (`[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`), task lists, nested inline styles, footnotes, and `breaks: false`.
  - **Zero Remote I/O Policy**: External images fall back safely to textual badges (`󰋩 [Image: alt]`), external links are display-only, and remote HTTP queries are prohibited.

---

## 7. Smooth Auto-Scrolling & Dual Interactive Scrollbars
- **Margin Auto-Scrolling**: Keeps caret comfortably visible away from screen edges during typing and navigation.
- **Proportional Scrollbars**: Independent scrollbars on both panes supporting drag tracking, track jump clicks, and $O(1)$ height recalculations.

---

## 8. Unicode Grapheme Integrity & Shaped Glyph Metrics
- **Grapheme Cluster Enforcement**: `unicode-segmentation` prevents partial deletion or broken selections on combining characters (`é`) and emoji ZWJ sequences (`👨‍💻`).
- **Real Glyph Metrics Alignment**: Caches exact layout metrics via `cosmic-text` with `RwLock` concurrency, eliminating cursor drift on wide CJK characters and fullwidth dashes (`――`).

---

## 9. File Explorer & XDG Portal Integration
- **Workspace Navigation**: Asynchronous directory scanning with Nerd Font icons.
- **Right-Click Context Menus**: New File, New Folder, Rename, and Safe Delete with boundary-clamped popup menus.
- **XDG Desktop Portal**: Native Linux file open / directory selection dialogs.

---

## 10. 20 Built-in Themes & Independent Opacity Controls
- **Curated Color Schemes**: Tokyo Night, Catppuccin, Gruvbox, Synthwave '84, Cyberpunk Neon, Nord, Dracula, and more.
- **Independent Window Opacities**: Adjust editor canvas, sidebar tree, and title bar opacities individually via `Ctrl + ,`.

---

## 11. Data Integrity & Security Safeguards
- **Atomic Writes**: Writes to temporary file (`.{file}.tmp.{pid}`) before replacement to prevent corruption on crash.
- **Sensitive File Exclusion**: Blocks AI suggestions when editing `.env*` or private key files.
- **50MB Safety Limit**: Protects against accidental hangs when opening excessive binary files.

---

<p align="center">
  <a href="PORTAL.md">← Back to Portal</a> | <a href="FEATURES.ja.md">日本語版仕様書 →</a>
</p>
