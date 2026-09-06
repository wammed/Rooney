# SESSION_HANDOVER.md - Rooney (CosmicCode) 🚀

このドキュメントは、これまでの開発経緯、アーキテクチャ設計、解決された課題、現在のコードベース状態、および次回セッションで即座に作業を再開するための完全なハンドオーバー情報を記録したものです。

---

## 1. プロジェクト概要

- **プロジェクト名**: Rooney (`CosmicCode`)
- **リポジトリ**: `wammed/Rooney`
- **開発形態**: **AI Vibe Coding**（ユーザーとAntigravity / Gemini による対話的ペアプログラミング）
- **主要技術スタック**:
  - **GUI / Windowing**: `libcosmic` (Pop!_OS COSMIC Desktop / Wayland native), `cosmic-text`, `iced`
  - **テキストバッファ**: `ropey 1.6` (ピースツリー型ロープバッファ)
  - **構文解析**: `tree-sitter 0.24`, `tree-sitter-rust 0.23` (インプロセス高速ハイライト)
  - **Markdown**: `pulldown-cmark 0.12`
  - **ファイルダイアログ**: `rfd 0.17.2` (XDG Desktop Portal, Wayland native)
  - **ローカルAI連携**: `tokio 1.40`, `reqwest 0.12` (Ollama FIM エンドポイント `http://localhost:11434`)
  - **文字幅・IME**: `unicode-width 0.2`, `libcosmic` InputMethod プロトコル
  - **設定・シリアライズ**: `serde`, `toml`, `directories`, `walkdir`

---

## 2. これまでの開発履歴と解決された課題

### セッション 1〜3: 初期エディタ構築とフルファイル書き出し
- Ropeyバッファ、Tree-sitterインプロセス構文ハイライト、pulldown-cmark Markdownプレビューを実装。
- 左右2分割レイアウト（Dual-Pane Split View）の基本構造を策定。
- Nerd Font 対応ファイル種別アイコン、システムフォント検出、20種類のClassic & Neonテーマを導入。
- ローカル Ollama FIM (Fill-in-the-Middle) によるゴーストテキスト補完機能を統合。

### セッション 4: 2分割ペインの視覚的・レンダリング不具合の解消
- **課題**:
  - 右ペインを選択してもハイライトされない。
  - 右ペインにカーソルや行番号が表示されない。
  - 左ペインのエディタ表示が右や下にはみ出る（折り返しが効かない）。
- **解決策**:
  - `active_pane` に応じたアクティブ枠線・タブ色・カーソル点滅の独立レンダリングを実装。
  - `unicode-width` を導入し、半角・全角文字の視覚的カラム幅に応じたソフト折り返し（Soft-wrap）を計算。
  - `renderer.with_layer(bounds, ...)` を適用し、ペイン境界を厳密にクリッピングして文字のはみ出しを防止。

### セッション 5〜6: 日本語IME完全対応とキーバインド最適化
- **課題**:
  - 日本語入力（IME）が効かない。
  - `Ctrl + Space` が AI 補完トリガーになっており、システムIME切り替えと競合。
  - デフォルトウィンドウサイズが小さい。
- **解決策**:
  - `src/main.rs` でデフォルトウィンドウサイズを `1800x1800` に設定。
  - AI FIM手動トリガーを `Ctrl + I` または `Alt + Enter` に変更し、`Ctrl + Space` を完全にIMEに解放。
  - `EditorCanvas` に `cosmic::iced::advanced::Widget` トレイトを実装。
  - `shell.request_input_method(InputMethod::Enabled { ... })` を呼び出し、Wayland text-input プロトコルを完全に有効化。
  - Preedit（変換中下線・候補文字列）および Commit（確定テキスト挿入）イベントをディスパッチ。
  - 直前のIME確定直後の Enter キーの重複改行を防ぐデバウンス（100ms）を実装。

### セッション 7: ファイル操作の大幅強化（任意ファイルオープン・保存修正・新規ファイル名指定）
- **課題**:
  - カレントディレクトリ以外のファイルが開けない。
  - ファイルの保存ができない（未保存・未命名バッファで `pane.save_file()` が何も書き込まず成功を返していた）。
  - 新規ファイル（`New File`）を開いた時点で、ファイル名を即座に指定したい。
- **解決策**:
  - `rfd 0.17.2`（XDG Desktop Portal / Wayland対応）を導入。
  - **任意ファイルオープン**:
    - `Ctrl + O` / ヘッダー「`󰈔 Open`」でネイティブファイルピッカーを起動し、システム上の任意ファイルを開く。
    - `Ctrl + Shift + O` / ツリーヘッダー「``」でフォルダ選択ダイアログを起動し、ツリールートを自由に変更可能に。
    - ツリーヘッダー「``」で親ディレクトリ（`..`）に遡るナビゲーションを追加。
    - `FileTree::new` および `set_root` でパスを canonicalize し、親ディレクトリ移動を安定化。
  - **ファイル保存の修正**:
    - `EditorPane::save_file` で `file_path.is_none()` の場合に明確な `Err` を返し、自動的に「名前を付けて保存（Save As）」ダイアログをトリガー。
    - `Ctrl + Shift + S` で常に Save As ダイアログを起動可能に。
    - `save_file` および `save_file_as` で `std::fs::create_dir_all` を呼び出し、ネストしたフォルダも安全に自動生成。
  - **新規ファイル名指定モーダル**:
    - `Ctrl + N` または「` New`」クリック時、画面中央にスタイリッシュなモーダルダイアログを表示。
    - テキスト入力フィールドでファイル名（例: `main.rs`, `src/utils.rs`, `notes.md`）を入力。
    - 保存先フォルダの表示および「Change...」ボタンによる変更に対応。
    - **Enter** で即座にファイル作成＆アクティブエディタでオープン（構文ハイライト自動適用）、**Esc** でキャンセル。

### セッション 8: バイリンガル README.md の作成 & AI Vibe Coding 明記
- `README.md` を日本語および英語のバイリンガル仕様に刷新。
- AI Vibe Coding（Antigravity / Gemini による対話的バイブコーディング）によってゼロから構築されたプロジェクトであることを明記。

---

## 3. ファイル構成と役割

```
Rooney/
├── Cargo.toml               # 依存関係定義 (libcosmic, ropey, tree-sitter, rfd, ollama, etc.)
├── README.md                # 日本語・英語バイリンガル公式ドキュメント (Vibe Coding明記)
├── SESSION_HANDOVER.md      # 本ファイル (次回再開用完全ハンドオーバー)
├── src/
│   ├── main.rs              # アプリ起動エントリーポイント (ウィンドウサイズ 1800x1800 設定)
│   ├── app.rs               # COSMIC Application 実装、Messageディスパッチ、キーバインド、モーダルUI
│   ├── editor/
│   │   ├── mod.rs
│   │   ├── buffer.rs        # TextBuffer (Ropeyラッパー、カーソル移動、Undo/Redo、FIM文脈抽出)
│   │   └── pane.rs          # EditorPane (ペイン状態、ファイル読込/保存/SaveAs、言語判別)
│   ├── syntax/
│   │   └── mod.rs           # Tree-sitter Highlighter (Rust, Markdown, Python, JS, PlainText)
│   ├── markdown/
│   │   ├── mod.rs
│   │   └── renderer.rs      # MarkdownDocument (pulldown-cmark によるAST構築)
│   ├── fs/
│   │   ├── mod.rs
│   │   └── tree.rs          # FileTree (階層スキャン、Nerd Font アイコン、set_root, go_to_parent)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── canvas_editor.rs # EditorCanvas (cosmic Widget 実装、IME Protocol、境界クリッピング、描画)
│   │   ├── file_tree_view.rs# view_file_tree (サイドバーUI、// ボタン)
│   │   └── markdown_view.rs # view_markdown (リッチMarkdownプレビューコンテナ)
│   ├── theme/
│   │   ├── mod.rs
│   │   └── themes.rs        # 20種類の Classic & Neon テーマ定義、アルファ透過・ディミング
│   ├── font/
│   │   └── mod.rs           # FontManager (fontconfig によるシステムNerd Font検出とサイズ管理)
│   └── ai/
│       └── mod.rs           # OllamaClient (FIM補完リクエスト、モデル自動検出)
└── tests/
    ├── core_tests.rs        # 10件のユニットテスト (バッファ操作、日本語文字幅、ツリー移動、ファイル保存)
    └── ollama_tests.rs      # 2件の統合テスト (Ollama 接続性、FIM生成テスト)
```

---

## 4. キーボードショートカット一覧

| ショートカット | 動作 |
|---|---|
| `Ctrl + N` | 新規ファイル作成（ファイル名・パス指定モーダル起動） |
| `Ctrl + O` | ファイルを開く（XDG Desktop Portal ネイティブダイアログ） |
| `Ctrl + Shift + O` | フォルダを開く（サイドバーのツリールートを変更） |
| `Ctrl + S` | ファイル保存（未命名バッファの場合は自動で Save As） |
| `Ctrl + Shift + S` | 名前を付けて保存（Save As） |
| `Tab` | AI補完（ゴーストテキスト）の確定挿入 / インデント（4スペース） |
| `Esc` | AI補完の破棄 / 選択解除 / モーダルダイアログのキャンセル |
| `Ctrl + B` | ファイルツリーサイドバーの表示/非表示トグル |
| `Ctrl + \` または `Ctrl + E` | 左右2分割（Split / Single）レイアウト切り替え |
| `Ctrl + M` | Markdownプレビューの切り替え |
| `Ctrl + I` または `Alt + Enter` | Local AI FIM 補完の手動トリガー（`Ctrl + Space` は IME 専用に解放） |
| `Ctrl + Z` | 元に戻す (Undo) |
| `Ctrl + Y` または `Ctrl + Shift + Z` | やり直す (Redo) |
| `Ctrl + A` | 全選択 |
| `A-` / `A+` | フォントサイズの縮小 / 拡大 |

---

## 5. 現在のビルドおよびテスト状態

- `cargo check`: **0 errors, 0 warnings** (通過)
- `cargo test`: **12/12 passed (0 failed)**
  - `test_file_type_icons_nerd_font` ... ok
  - `test_file_tree_scanning` ... ok
  - `test_file_tree_navigation` ... ok
  - `test_text_buffer_basic_operations` ... ok
  - `test_text_buffer_multiline_and_cursor` ... ok
  - `test_text_buffer_fim_extraction` ... ok
  - `test_japanese_text_and_width` ... ok
  - `test_markdown_parsing` ... ok
  - `test_theme_system_20_themes` ... ok
  - `test_editor_pane_saving` ... ok
  - `test_ollama_connectivity_and_models` ... ok
  - `test_ollama_fim_generation` ... ok
- `cargo build`: **Clean dev build**

---

## 6. 次回再開時の推奨作業・機能拡張案

1. **追加言語の Tree-sitter パーサー拡充**:
   - 現在は Rust, Markdown, PlainText が稼働中。`tree-sitter-python`, `tree-sitter-javascript`, `tree-sitter-c` などを追加してハイライト精度をさらに強化可能。
2. **エディタ内検索・置換（`Ctrl + F` / `Ctrl + H`）**:
   - 簡易検索バーUIを上部または下部にオーバーレイ表示。
3. **設定の永続化**:
   - 選択したテーマ、フォント、フォントサイズ、透明度、ディミング値を `~/.config/rooney/config.toml` に自動保存・復元。
4. **タブ形式のマルチバッファ管理**:
   - 各ペインで複数のファイルをタブで切り替え可能にする拡張。
