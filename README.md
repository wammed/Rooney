# Rooney 🚀
**Cosmic-Native Lightweight Code & Markdown Editor**

[日本語](#日本語) | [English](#english)

---

<a name="日本語"></a>
## 日本語 🇯🇵

Pop!_OS COSMIC DE（libcosmic / Wayland）ネイティブの超高速・軽量・多機能コード＆Markdownエディタです。
外部LSPや重厚な子プロセスのオーバーヘッドを一切排除し、インプロセス構文ハイライト（Tree-sitter）、Ropeyバッファ、日本語IME（Fcitx5 / IBus）完全対応、左右2分割レイアウト、Nerd Font対応リッチファイルツリー、XDG Portalネイティブファイル操作、そしてローカルOllamaによるFill-in-the-Middle（FIM）インラインAI補完を統合しています。

### 主な特徴 ✨

1. **左右2分割レイアウト（Dual-Pane Split View）**
   - ツールバーまたは `Ctrl + \` / `Ctrl + E` で、シングルペインと左右2分割ペインを瞬時に切り替え。
   - 左右ペインで別々のファイルを開いて独立してスクロール・編集可能。
   - アクティブペインの視覚的強調（アクセントカラー枠・タブ強調）、独立したカーソル表示。
   - 画面端でのソフト折り返し（Soft-wrap）およびペイン境界での厳密なクリッピングにより、文字のはみ出しを防止。

2. **日本語入力（IME）ネイティブ対応**
   - Wayland text-input プロトコル（libcosmic InputMethod）による Fcitx5 / IBus 完全対応。
   - インライン変換（Preedit）のプレビュー表示および確定コミット処理。
   - `Ctrl + Space` は IME のオン/オフ切り替え専用に解放（AI補完ショートカットと競合しません）。
   - `unicode-width` による全角・半角・CJK文字の正確な表示幅・カーソル位置計算。

3. **柔軟なファイル管理 & ネイティブファイルダイアログ**
   - **新規ファイル作成（`Ctrl + N` / ` New`）**:
     - 画面中央のモーダルダイアログでファイル名（例: `main.rs`, `src/utils.rs`, `notes.md`）を即座に指定。
     - 作成先ディレクトリの変更（Change...）や、Enterで即座に作成＆オープン、Escでキャンセル。
     - ファイル作成と同時に拡張子に応じた構文ハイライトを自動適用。
   - **ファイルを開く（`Ctrl + O` / `󰈔 Open`）**:
     - XDG Desktop Portal（Wayland ネイティブ）非同期ダイアログで、ファイルシステム上の任意のファイルを開くことが可能。
   - **フォルダを開く（`Ctrl + Shift + O` / ``）**:
     - サイドバーのファイルツリールートを任意のフォルダに変更。
   - **親フォルダ移動（``）**:
     - ワンクリックで親ディレクトリ（`..`）に遡るナビゲーション。
   - **ファイルの安全な保存（`Ctrl + S` / `Ctrl + Shift + S` / `󰆓 Save`）**:
     - 未命名バッファの場合は自動で「名前を付けて保存（Save As）」ダイアログを表示。
     - ネストしたフォルダが存在しない場合も自動で親ディレクトリを作成して安全に保存。

4. **Nerd Font の全面採用 & システムフォント選択**
   - システム内のフォント（`JetBrainsMono Nerd Font` 等）を自動検出。
   - ファイル種別ごとの鮮やかな Nerd Font アイコン（Rust ``, Markdown ``, TOML ``, Python ``, JS ``, TS ``, C/C++ ``, Shell ``, Git `` 等）。
   - ヘッダーバーのドロップダウンからシステムフォントおよびフォントサイズ（`A-` / `A+`）をリアルタイム変更可能。

5. **20種類の Classic & Neon テーマ**
   - **Classic (10)**: Tokyo Night, Catppuccin Mocha, Catppuccin Latte, Nord, Gruvbox Dark, Gruvbox Light, Solarized Dark, Solarized Light, One Dark, Monokai Pro
   - **Neon / Cyberpunk (10)**: Synthwave '84, Cyberpunk Neon, Matrix Green, Vaporwave, Dracula Neon, Acid Rain, Sunset Glow, Deep Ocean, Neon Violet, Amber CRT

6. **Wayland ネイティブ透過 & 背景ディミング**
   - Wayland ウィンドウのアルファ透明度（0.1〜1.0）スライダー。
   - 背景ディミングオーバーレイ（0%〜100%）。
   - デフォルトウィンドウサイズ 1800x1800 の広々とした作業領域。

7. **Local AI FIM (Fill-in-the-Middle) 補完**
   - ローカルの Ollama（`http://localhost:11434`）と高速非同期連携。
   - `deepseek-coder-v2`, `gemma4-coder`, `qwen2.5-coder` 等の利用可能なモデルを自動検出。
   - 入力中の文脈に応じたゴーストテキストサジェスト。
   - `Tab` キーで即座に確定挿入、`Esc` で破棄。
   - `Ctrl + I` または `Alt + Enter` で手動生成トリガー。

---

### キーボードショートカット ⌨️

| ショートカット | 動作 |
|---|---|
| `Ctrl + N` | 新規ファイル作成（ファイル名入力ダイアログ） |
| `Ctrl + O` | ファイルを開く（システムファイルダイアログ） |
| `Ctrl + Shift + O` | フォルダを開く（ツリールート変更） |
| `Ctrl + S` | ファイル保存（未命名バッファは名前を付けて保存） |
| `Ctrl + Shift + S` | 名前を付けて保存（Save As） |
| `Tab` | AI補完の確定挿入 / インデント（4スペース） |
| `Esc` | AI補完の破棄 / 選択解除 / ダイアログキャンセル |
| `Ctrl + B` | ファイルツリーサイドバーの表示/非表示 |
| `Ctrl + \` または `Ctrl + E` | 左右2分割（Split / Single）レイアウト切り替え |
| `Ctrl + M` | Markdownプレビューの切り替え |
| `Ctrl + I` または `Alt + Enter` | Local AI FIM 補完の手動トリガー（`Ctrl + Space` は IME 専用に解放） |
| `Ctrl + Z` | 元に戻す (Undo) |
| `Ctrl + Y` または `Ctrl + Shift + Z` | やり直す (Redo) |
| `Ctrl + A` | 全選択 |
| `A-` / `A+` | フォントサイズの縮小 / 拡大 |

---

### ビルド＆起動方法 🛠️

```bash
# 依存パッケージの確認 (Pop!_OS / Ubuntu)
sudo apt install build-essential libxkbcommon-dev libfontconfig1-dev

# ビルド
cargo build --release

# 実行
cargo run

# テスト実行 (12テスト)
cargo test
```

---

<a name="english"></a>
## English 🇬🇧 🇺🇸

A blazing-fast, lightweight, feature-rich code & markdown editor built natively for Pop!_OS COSMIC DE (libcosmic / Wayland).
Zero LSP overhead and zero heavy child processes: features in-process Tree-sitter syntax highlighting, a high-performance Ropey text buffer, native Japanese IME (Fcitx5 / IBus) integration, side-by-side dual-pane split editing, Nerd Font file tree, native XDG Desktop Portal file dialogs, and local Ollama-powered Fill-in-the-Middle (FIM) inline AI completions.

### Key Features ✨

1. **Dual-Pane Split View**
   - Toggle instantly between Single Pane and Side-by-Side Split View via the toolbar or `Ctrl + \` / `Ctrl + E`.
   - Edit different files simultaneously in left and right panes with independent scrolling, cursors, and syntax highlighters.
   - Clear visual focus indication with accent borders and active tab highlights.
   - Clean soft-wrapping and bounded layer clipping to prevent text from overflowing across panes.

2. **Native Japanese IME Support**
   - Full support for Wayland text-input protocol (libcosmic InputMethod) supporting Fcitx5 and IBus.
   - Live pre-edit underline preview and commit event handling.
   - `Ctrl + Space` is unmapped from editor shortcuts and dedicated strictly to system IME toggle.
   - Accurate visual column and cursor positioning for full-width CJK characters via `unicode-width`.

3. **Arbitrary File Operations & Native Dialogs**
   - **Create New File (`Ctrl + N` / ` New`)**:
     - Instant modal dialog in the center of the window to specify the filename immediately (e.g., `main.rs`, `src/utils.rs`, `notes.md`).
     - Supports custom folder selection ("Change..."), Enter to confirm, and Esc to cancel.
     - Automatically creates parent directories and applies syntax highlighting based on file extension.
   - **Open File (`Ctrl + O` / `󰈔 Open`)**:
     - Uses asynchronous native file chooser dialogs (XDG Desktop Portal) to open any file across the filesystem.
   - **Open Folder (`Ctrl + Shift + O` / ``)**:
     - Switch the active root of the sidebar file tree to any workspace or folder.
   - **Navigate Up (``)**:
     - One-click parent directory (`..`) navigation.
   - **Safe File Saving (`Ctrl + S` / `Ctrl + Shift + S` / `󰆓 Save`)**:
     - Automatically prompts "Save As" if the buffer is untitled or unnamed.
     - Automatically creates intermediate directories with `create_dir_all`.

4. **Nerd Font Integration & System Font Selector**
   - Auto-detects installed monospace and Nerd Fonts (e.g., `JetBrainsMono Nerd Font`, `FiraCode Nerd Font`).
   - Colored Nerd Font icons for each file extension (Rust ``, Markdown ``, TOML ``, Python ``, JS ``, TS ``, C/C++ ``, Shell ``, Git ``, etc.).
   - Switch fonts and adjust font sizes (`A-` / `A+`) on the fly from the top header bar.

5. **20 Classic & Neon Themes**
   - **Classic (10)**: Tokyo Night, Catppuccin Mocha, Catppuccin Latte, Nord, Gruvbox Dark, Gruvbox Light, Solarized Dark, Solarized Light, One Dark, Monokai Pro
   - **Neon / Cyberpunk (10)**: Synthwave '84, Cyberpunk Neon, Matrix Green, Vaporwave, Dracula Neon, Acid Rain, Sunset Glow, Deep Ocean, Neon Violet, Amber CRT

6. **Wayland Native Transparency & Background Dimming**
   - Adjustable window alpha transparency (0.1 to 1.0) and background dimming (0% to 100%).
   - Generous default window size (1800x1800) optimized for modern high-resolution displays.

7. **Local AI FIM (Fill-in-the-Middle) Code Completion**
   - Direct asynchronous communication with local Ollama (`http://localhost:11434`).
   - Automatically detects installed coding models (e.g., `deepseek-coder-v2`, `gemma4-coder`, `qwen2.5-coder`).
   - Context-aware inline ghost text suggestions.
   - Press `Tab` to accept, `Esc` to dismiss.
   - Trigger suggestions manually with `Ctrl + I` or `Alt + Enter`.

---

### Keyboard Shortcuts ⌨️

| Shortcut | Description |
|---|---|
| `Ctrl + N` | Create New File (opens filename prompt dialog) |
| `Ctrl + O` | Open File (native system file chooser) |
| `Ctrl + Shift + O` | Open Folder (switch file tree root) |
| `Ctrl + S` | Save File (prompts Save As if buffer is untitled) |
| `Ctrl + Shift + S` | Save File As |
| `Tab` | Accept AI suggestion / Indent (4 spaces) |
| `Esc` | Dismiss AI suggestion / Clear selection / Cancel dialog |
| `Ctrl + B` | Toggle File Tree sidebar visibility |
| `Ctrl + \` or `Ctrl + E` | Toggle Dual-Pane Split / Single layout |
| `Ctrl + M` | Toggle Markdown Live Preview |
| `Ctrl + I` or `Alt + Enter` | Manually trigger Local AI FIM (`Ctrl + Space` reserved for IME) |
| `Ctrl + Z` | Undo |
| `Ctrl + Y` or `Ctrl + Shift + Z` | Redo |
| `Ctrl + A` | Select All |
| `A-` / `A+` | Decrease / Increase Font Size |

---

### Build & Run Instructions 🛠️

```bash
# Install system dependencies (Pop!_OS / Ubuntu)
sudo apt install build-essential libxkbcommon-dev libfontconfig1-dev

# Build release binary
cargo build --release

# Run locally
cargo run

# Run unit tests (12 tests)
cargo test
```

---

### Architecture & Tech Stack 🏗️

- **GUI & Windowing**: [libcosmic](https://github.com/pop-os/libcosmic), `cosmic-text`, `iced`
- **File Dialogs**: `rfd` (XDG Desktop Portal, Wayland native)
- **Text Buffer**: [Ropey](https://github.com/cessen/ropey) (Piece-tree rope buffer for handling large files)
- **Syntax Highlighting**: [tree-sitter](https://tree-sitter.github.io/tree-sitter/) (in-process tree parsing)
- **Markdown**: `pulldown-cmark`
- **AI Integration**: `reqwest`, `tokio` (local Ollama FIM endpoint)
- **Character Metrics**: `unicode-width` (CJK full-width display handling)
