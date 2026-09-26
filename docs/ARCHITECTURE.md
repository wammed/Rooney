# 📐 Rooney System Architecture

In-depth technical architecture of Rooney's rendering pipeline, asynchronous generation tracking, and low-latency buffer systems.

---

## 📑 Architecture Overview
1. [Layered System Architecture](#1-layered-system-architecture)
2. [Unified Generation-Tracked Async Pipelines](#2-unified-generation-tracked-async-pipelines)
3. [Viewport Virtualization & Cumulative Wrap Model (`LineWrapModel`)](#3-viewport-virtualization--cumulative-wrap-model-linewrapmodel)
4. [Piece-Tree Rope Buffer & Editing Mechanics](#4-piece-tree-rope-buffer--editing-mechanics)
5. [Shaped Glyph Metrics Caching (`cosmic-text`)](#5-shaped-glyph-metrics-caching-cosmic-text)
6. [Offline Local AI Integration (Ollama SSE Streaming)](#6-offline-local-ai-integration-ollama-sse-streaming)

---

## 1. Layered System Architecture

Rooney is constructed strictly in Rust and `libcosmic` across four modular layers:

```text
┌────────────────────────────────────────────────────────┐
│                   UI / View Layer                      │
│ (libcosmic, Titlebar, TabBar, SidebarTree, SplitPane)  │
└──────────────────────────┬─────────────────────────────┘
                           │ Event Dispatch / Message Flow
┌──────────────────────────▼─────────────────────────────┐
│                 Editor Canvas Core                     │
│  - Viewport Virtualization (< 0.40 ms Layout)          │
│  - LineWrapModel (Sparse O(log W) Cumulative Wrap)     │
│  - Caret & Selection Engine (Unicode Graphemes)        │
└──────────────┬──────────────────────────┬──────────────┘
               │                          │
┌──────────────▼────────────┐ ┌───────────▼──────────────┐
│ Buffer Layer (Piece-Tree) │ │ Generation-Tracked Async │
│ - ropey::Rope             │ │ - Tree-sitter AST Parser │
│ - O(1) Max Line Tracking  │ │ - Markdown GFM Parser    │
│ - Atomic Safe File I/O    │ │ - Regex & String Search  │
└───────────────────────────┘ └──────────────────────────┘
```

---

## 2. Unified Generation-Tracked Async Pipelines

Performing deep Tree-sitter AST queries or document-wide regex searches synchronously on massive files (> 2MB to 50MB) can freeze the UI thread for dozens of milliseconds. Rooney eliminates frame drops via **generation-tracked background pipelines**:

- **Background Worker Threads**:
  - Tree-sitter AST incremental parsing
  - Markdown GFM document parsing
  - Full-buffer regex and string searches
  All intensive workloads are offloaded to `tokio::task::spawn_blocking`.
- **Deterministic Generation Counters (`GenerationId`)**:
  - `parse_generation`: Incremented on text modifications
  - `markdown_generation`: Incremented on preview toggles, edits, or spec changes
  - `search_generation`: Incremented on search query input
- **100% Stale Result Rejection**:
  - When worker threads complete and return results to the UI thread, the returned generation is validated against the active buffer generation. If they differ, the result is immediately dropped without relying on timestamps or clocks.
  - This eliminates highlight flashing and race conditions during fast typing or rapid tab switches.

---

## 3. Viewport Virtualization & Cumulative Wrap Model (`LineWrapModel`)

### Viewport Virtualization
- Even on 50MB documents containing 1.8M lines, layout calculation and glyph shaping are strictly constrained to **the visible viewport plus a small buffer margin**.
- Layout calculations consistently take **less than 0.40 ms**, maintaining 144+ FPS butter-smooth scrolling regardless of total document size.

### Sparse Cumulative Wrap Model (`LineWrapModel`)
- Naive wrap height calculations often result in visual row overlap between wrapped subrows and subsequent lines.
- Rooney uses a sparse $O(\log W)$ `LineWrapModel` that accumulates wrapped subrow counts, guaranteeing non-overlapping Y coordinate layout and sub-pixel accurate mouse hit-testing.

---

## 4. Piece-Tree Rope Buffer & Editing Mechanics

- **Piece-Tree (`ropey`)**:
  - Documents are modeled as balanced B-trees of text chunks, avoiding massive contiguous memory relocations on edits.
- **Constant-Time Line Tracking**:
  - Line lengths are tracked in $O(1)$ time, enabling rapid recalculations of soft wraps when resizing windows or split panes.
- **Atomic Writes**:
  - File saves write to temporary staging files (`.{filename}.tmp.{pid}`), invoke `sync_all`, and execute atomic renames, preventing truncated writes on power interruption or process termination.

---

## 5. Shaped Glyph Metrics Caching (`cosmic-text`)

- **Moving Beyond Fixed-Width Approximations**:
  - Conventional terminal and editor assumptions that fullwidth characters are exactly 2× single-width fail on fullwidth dashes (`――`), ambiguous CJK symbols, and combining diacritics.
- **Direct Shaped Glyph Inspection**:
  - Rooney inspects exact glyph widths computed by `cosmic-text`.
  - Backed by an `RwLock` thread-safe cache and fast paths for standard Tab/ASCII characters, ensuring absolute caret alignment without compromising drawing speed.

---

## 6. Offline Local AI Integration (Ollama SSE Streaming)

- **Local-Only Communication**:
  - Connects strictly to the local daemon at `http://127.0.0.1:11434` via `reqwest`.
- **FIM Autocompletion**:
  - Extracts the pre-caret `prefix` and post-caret `suffix` to build standard Fill-in-the-Middle prompts.
- **Server-Sent Events (SSE)**:
  - Streaming chat responses arrive as asynchronous token events for instant on-screen rendering and immediate cancellation support.

---

<p align="center">
  <a href="PORTAL.md">← Back to Portal</a> | <a href="ARCHITECTURE.ja.md">日本語版設計書 →</a>
</p>
