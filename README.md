# CosmicCode (Rooney) 🚀
**Cosmic-Native Lightweight Code & Markdown Editor**

Pop!_OS COSMIC DE ネイティブの超高速・軽量・多機能テキスト＆Markdownエディタです。
外部LSPや子プロセスのオーバーヘッドを一切排除し、インプロセス構文ハイライト（Tree-sitter）、Ropeyバッファ、20種類のClassic & Neonテーマ、Waylandネイティブアルファ透過、左右2分割レイアウト、Nerd Font対応リッチファイルツリー、そしてローカルOllamaによるFill-in-the-Middle（FIM）インラインAI補完を統合しています。

---

## 主な特徴 ✨

1. **左右2分割レイアウト（Dual-Pane Split View）**
   - ツールバーまたは `Ctrl + \` / `Ctrl + E` で、シングルペインと左右2分割ペインを瞬時に切り替え。
   - 左右で別々のファイルを開いて同時編集、または片方をMarkdownリアルタイムプレビューとして表示。

2. **Nerd Font の全面採用 & システムフォント選択**
   - システム内のフォント（`JetBrainsMono Nerd Font` 等）を自動検出。
   - ファイル種別ごとの鮮やかな Nerd Font アイコン（Rust ``, Markdown ``, TOML ``, Python ``, JS `` など）。
   - ヘッダーバーのドロップダウンからシステムフォントおよびフォントサイズをリアルタイム変更可能。

3. **常時表示＆リッチなファイルツリー（Sidebar）**
   - 階層フォルダのインデントガイド、展開/折りたたみ（`` / ``）。
   - クリックでアクティブペインに即座にファイルオープン。
   - `Ctrl + B` で開閉トグル。

4. **インプロセス構文ハイライト & Markdown プレビュー**
   - `tree-sitter`（Rust 等）によるインプロセスの高速構文解析。
   - `pulldown-cmark` による見出し、コードブロック、リスト、引用などのリッチなMarkdownプレビュー。

5. **20種類の Classic & Neon テーマ**
   - **Classic (10)**: Tokyo Night, Catppuccin Mocha, Catppuccin Latte, Nord, Gruvbox Dark, Gruvbox Light, Solarized Dark, Solarized Light, One Dark, Monokai Pro
   - **Neon / Cyberpunk (10)**: Synthwave '84, Cyberpunk Neon, Matrix Green, Vaporwave, Dracula Neon, Acid Rain, Sunset Glow, Deep Ocean, Neon Violet, Amber CRT

6. **Wayland ネイティブ透過 & 背景画像ディミング**
   - Wayland ウィンドウのアルファ透明度（0.1〜1.0）スライダー。
   - 背景ディミングオーバーレイ（0%〜100%）。

7. **Local AI FIM (Fill-in-the-Middle) 補完**
   - ローカルの Ollama（`http://localhost:11434`）と連携。
   - `deepseek-coder-v2:16b` や `gemma4-coder:latest` 等のモデルを自動検出。
   - カーソル位置でのインラインゴーストテキストサジェスト。
   - `Tab` キーで即座に確定挿入、`Esc` で破棄。

---

## キーボードショートカット ⌨️

| ショートカット | 動作 |
|---|---|
| `Tab` | AI補完（ゴーストテキスト）の確定挿入 / インデント |
| `Esc` | AI補完の破棄 / 選択解除 |
| `Ctrl + S` | ファイル保存 |
| `Ctrl + B` | ファイルツリーサイドバーの表示/非表示 |
| `Ctrl + \` または `Ctrl + E` | 左右2分割（Split / Single）レイアウト切り替え |
| `Ctrl + M` | Markdownプレビューの切り替え |
| `Ctrl + Space` | Local AI FIM 補完の手動トリガー |
| `Ctrl + Z` | 元に戻す (Undo) |
| `Ctrl + Y` または `Ctrl + Shift + Z` | やり直す (Redo) |
| `Ctrl + A` | 全選択 |
| `A-` / `A+` | フォントサイズの縮小 / 拡大 |

---

## ビルド＆起動方法 🛠️

```bash
# ビルド
cargo build --release

# 実行
cargo run

# テスト実行
cargo test
```
