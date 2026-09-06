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

### セッション 9〜10: 入力時カーソルずれ解消・設定永続化・編集メニュー・マウスコピペ
- **課題**:
  - 入力時にカーソルの位置がおかしい（日本語/全角文字を入力するごとにカーソルが最大2文字分ほど前方にずれる）。
  - テーマ、フォント、透明度、ペインレイアウトなどの設定がアプリ終了時にリセットされる（設定の永続化）。
  - エディタ内に一般的な編集メニュー（Undo, Redo, Cut, Copy, Paste, Select All）がない。
  - マウス操作での範囲選択およびコピー＆ペーストを行いたい。
- **解決策**:
  - **カーソル位置ずれ（CJK文字幅ズレ）の根本解決**:
    - 等幅フォント（JetBrains Mono等）において、ASCII半角文字のアドバンスは `0.60 * font_size`（14px時 約8.4px）だが、CJK全角文字のアドバンスは `1.0 * font_size`（14px時 14.0px）である。
    - 従来の「全角=半角の2倍（2 × 8.4 = 16.8px または 2 × 8.0 = 16.0px）」という計算では、1文字ごとに約2.8pxの過大評価が発生し、5〜7文字入力すると14〜20px（約2文字分）前方にカーソルが飛び出していた。
    - `EditorCanvas::char_advance(c, font_size)` を導入し、半角（`font_size * 0.60`）、全角/CJK（`font_size * 1.0`）、タブ（`4.0 * font_size * 0.60`）の正確なサブピクセル幅を算出。
    - 折り返し計算（`build_visual_rows`）、カーソル位置（`cursor_screen_pos`）、クリック/ドラッグ逆引き（`pos_to_char_coords`）、IME Preedit描画端のすべてで `char_advance` を一貫して適用。入力中のカーソル位置ずれを完全解消。
  - **設定の永続化 (`~/.config/rooney/config.toml`)**:
    - `src/config.rs` (`AppConfig`) を新設。
    - テーマ、フォント、フォントサイズ、Wayland透明度、背景ディミング、分割ペイン配置、ファイルツリー開閉状態、AI有効化、AIモデルを `~/.config/rooney/config.toml` に自動保存・復元。
    - ユーザーがUI上で設定を変更するたびに `save_config()` が自動実行され、再起動後も直前の作業環境を完全復元。
  - **エディタ内編集メニュー & 右クリックコンテキストメニュー**:
    - ヘッダーバーに「`󰧑 Edit ▾`」ボタンを新設。ワンクリックで Undo, Redo, Cut, Copy, Paste, Select All の編集ツールバーを展開。
    - エディタペイン上のマウス右クリックで、カーソル位置にコンテキストメニューをフロート表示（コピー、切り取り、貼り付け、全選択、元に戻す、やり直し）。
    - **右クリックメニュー表示崩れの解消**: 従来はコンテナ背景が透過していたため背景のコード文字と重なって乱れていた問題を、`cosmic::theme::Container::Card` による不透明カード背景、クリーンな左右揃え（アイコン/名称 + ショートカットキー）、画面端での位置クランプ、およびメニュー外クリックで即座に閉じる全画面バックドロップによって刷新。
  - **マウス操作でのドラッグ選択 & Wayland クリップボード完全統合**:
    - マウス左ドラッグによる自由なテキスト範囲選択と、テーマに調和した半透明の選択ハイライト描画。
    - `libcosmic` / `iced` の非同期クリップボード API（`clipboard::write`, `clipboard::read`）を統合し、`Ctrl + C`、`Ctrl + X`、`Ctrl + V`、マウス右クリック、Editメニューすべてでシステムクリップボードとシームレスに同期。
  - **テンキー（Numpad）の入力・操作完全対応（日本語IME時と英字入力時の挙動差の根本解消）**:
    - **原因究明**:
      - **日本語IME使用時**: Fcitx5/Mozc等のIMEがコンポジタ側でテンキーを捕捉し、確定テキストを `cosmic::iced::advanced::input_method::Event::Commit(text)` として直接エディタに注入していたため正常に入力できていた。
      - **英字（直接）入力時**: 入力イベントが `KeyPressed` を通過する。Wayland / Pop!_OS COSMIC 環境ではクライアント側 XKB に NumLock の状態フラグが同期されない場合が多く、クライアントには NumLock オフ状態の Keysym（`Named::ArrowLeft` [4], `ArrowRight` [6], `ArrowUp` [8], `ArrowDown` [2], `Home` [7], `End` [1], `PageUp` [9], `PageDown` [3], `Delete` [.]）として渡されていた。
      - Rooneyのキー判定順序において、Delete や矢印・Home・End 移動が文字入力判定より**前**にあったため、テンキー数字キーがすべてカーソル移動や削除に奪われていた。
      - さらに `0` や `5` では `text` が `Some("")`（空文字列）となり、従来の `if let Some(t) = text { if !t.is_empty() { ... } else { None } }` のネストにより `else if physical_key` にフォールバックせず無視されていた。
    - **解決策**:
      - `src/editor/mod.rs` に `resolve_numpad_char(physical_key: &Physical) -> Option<&'static str>` を新設。
      - `app.rs` の `KeyPressed` において、`!modifiers.control() && !modifiers.alt()` の場合、**Delete や矢印・Home・End などのナビゲーション判定よりも前**で `resolve_numpad_char` を評価し、数字および四則演算子記号を直接バッファに挿入するよう変更。
      - 独立した専用カーソルキー（`ArrowLeft` 等）や専用 `Delete` キーは `resolve_numpad_char` で `None` となるため、移動・削除操作が一切損なわれず完璧に共存。
      - 通常文字入力フォールバックでも `.filter(!empty).or_else(...)` を採用し、空文字列によるフォールバック握りつぶしを根本排除。
      - テンキーの **Enter**（`Code::NumpadEnter`）もエディタ改行およびモーダル確定の両方でシームレスに動作。
      - `tests/core_tests.rs::test_numpad_key_resolution` で 0〜9, +, -, *, /, ., ,, = の正引き、および専用矢印・文字キーの `None` 判定を自動テスト化。

### セッション 11: Tree-sitter多言語拡張・検索置換・行編集・パフォーマンス最適化・Clippy警告完全解消
- **Tree-sitter 言語・設定ファイルの大幅拡張（12+言語）**:
  - `tree-sitter-python`, `tree-sitter-javascript`, `tree-sitter-typescript`, `tree-sitter-c`, `tree-sitter-cpp`, `tree-sitter-bash`, `tree-sitter-fish`, `tree-sitter-json`, `tree-sitter-toml-ng`, `tree-sitter-yaml` を導入。
  - Rust, Python, JS/TS, C/C++, Bash, Fish, TOML, YAML, JSON, INI/Conf, Markdown を完全ハイライト。
  - ファイルツリーアイコンに `.fish` (`󰈺`), `.yaml` (``), `.ini` (``), `.tsx`/`.jsx` (``) 等を追加。
- **インクリメンタルファイル内検索 (`Ctrl + F`) & リアルタイムハイライト**:
  - `Ctrl + F` でエディタ上部に検索バーを展開。
  - マッチ箇所をエディタ内で黄色ハイライト描画、アクティブマッチを白枠強調。
  - `Enter` / `▼`（次の一致）、`Shift + Enter` / `▲`（前の一致）、`Esc` / `✕`（閉じる）。
- **高度なカーソル移動・行編集ショートカット**:
  - `Ctrl + Left` / `Ctrl + Right`: 単語単位移動（`+Shift` で単語選択）。
  - `Ctrl + /`: 行コメントトグル（言語別の `//` または `#` を自動判定）。
  - `Ctrl + Shift + K`: 行削除。
  - `Ctrl + D`: 行複製。
  - `Tab` / `Shift + Tab`（選択時）: 複数行の一括インデント / アンインデント（4スペース）。
- **メモリリークの根絶 & パフォーマンス最適化**:
  - `canvas_editor.rs` の毎フレーム `Box::leak` を `intern_font_name`（`OnceLock<Mutex<HashSet<&'static str>>>` キャッシュ）に置換し、メモリリークを完全解消。
  - `pane.on_content_changed` において、Markdownプレビューが非アクティブな時はMarkdown ASTパースをスキップし、毎打鍵の遅延を極小化。
  - `FileTree::scan_dir` で `node_modules`, `.venv`, `target`, `dist`, `build`, `.idea`, `.vscode` 等の大規模ディレクトリを自動除外。
- **コード品質向上**:
  - `cargo clippy`: 全15件の警告を修正し、**0警告** を達成。
  - `cargo test`: 19件のユニットテスト + 2件のOllamaテスト = **全21件パス (0 failure)**。

### セッション 10〜11: タブ形式マルチバッファ管理と `src/app/` モジュール分割
- **タブ形式のマルチバッファ管理**:
  - `EditorTab` および `EditorPane` による左右ペインそれぞれの独立複数タブ管理。
  - 未保存バッファのインジケータ（`●`）、タブ追加・閉じるボタン、`Ctrl + T`, `Ctrl + W`, `Ctrl + Tab`, `Ctrl + Shift + Tab` ショートカット。
  - 最後のタブを閉じた際のクリーンな Untitled タブ自動フォールバック。
- **`src/app/` の責任分割**:
  - 巨大化していた単一の `src/app.rs` を `src/app/` サブモジュール（`mod.rs`, `keybindings.rs`, `update.rs`, `state.rs`, `message.rs`, `ui/`）へ分割。
  - 責務ごとの明確な分離により、拡張性とテスト容易性を飛躍的に向上。

### セッション 12: ファイルツリー操作・AIチャットストリーミングパネル・セッション永続化（新機能群）
- **ファイルツリー コンテキスト操作 & ディレクトリ管理機能**:
  - サイドバーヘッダーおよび各アイテムにクイックアクション（新規ファイル `＋`, フォルダ作成 ``, リネーム ``, 削除 ``）を配置。
  - 画面中央のモーダルダイアログでファイル名・フォルダ名・リネーム・削除確認（誤操作防止）を安全に実行。
  - ファイルやディレクトリのリネーム時、左右両ペインで開いている該当タブのファイルパスおよびタブ表示名をリアルタイムに自動同期。
  - ファイルやディレクトリの削除時、開いていた該当タブを安全に自動クローズ。
- **AI チャット / 複数行コード生成パネル & リアルタイム・ストリーミング（SSE）**:
  - ヘッダーの `󰭹 Chat` ボタンまたは `Ctrl + Shift + A` で展開・格納できる右側AIアシスタントパネル。
  - `stream: true` による非同期チャンク受信（`futures_channel::mpsc` & `cosmic::task::stream`）。1トークンずつリアルタイムに流れるタイピングアニメーション（`▋` カーソル付き）を実装。
  - いつでも生成を中断できる `󰓛 Stop` ボタン（および生成中の Enter キーでの即時キャンセル）を完備。
  - `Attach Selection`（選択範囲コード添付）および `Attach File`（アクティブファイル全体添付）機能。
  - 生成されたコードブロックをワンクリックでカーソル位置へ挿入（Insert at Cursor）またはクリップボードコピー。
- **ワークスペースセッションの自動永続化**:
  - 左右ペインで開いていたタブ一覧、アクティブタブ番号、カーソル位置（行・列）、ワークスペースルートディレクトリ、AIチャットパネル開閉状態を `~/.config/rooney/config.toml` に自動保存。
  - 次回エディタ起動時に、前回の作業状態が寸分違わずそのまま自動復元。
- **テスト・静的解析の完全遵守**:
  - セッションシリアライズ・復元テスト、タブ同期テスト、AI Chatストリーミング蓄積テスト、Ollamaチャットストリーミング実走テストを追加。
  - `cargo test`: 24 core tests + 3 ollama tests = **27/27 passed**。
  - `cargo clippy --all-targets -- -D warnings`: **0 errors, 0 warnings**。
### セッション 13: ファイルツリー右クリックコンテキストメニュー & アクションアイコン配置・スクロールバー重なり解消
- **ファイルツリー右クリックコンテキストメニュー（Pop!_OS COSMIC / Wayland ネイティブ）**:
  - `cosmic::widget::mouse_area` による右クリックイベントディスパッチを実装。
  - 各ファイル、ディレクトリ、ツリールートヘッダー、空きスペースへの右クリック（`RightClick(path, is_dir)`, `RightClickRoot`）を検出し、マウス座標へ美しいフローティングカードメニューを表示。
  - **ディレクトリ**: `  New File Here...`, `  New Folder Here...`, `  Rename Folder...`, `  Delete Folder...`, `  Refresh Tree`
  - **ファイル**: `󰈙  Open File`, `  Rename File...`, `  Delete File...`, `  Refresh Tree`
  - **ワークスペースルート / 余白**: `  New File in Root...`, `  New Folder in Root...`, `  Open Folder...`, `  Refresh Tree`
  - 背景（Backdrop）クリックまたは `Esc` キーで自然に即座キャンセル・クローズ。
- **ファイルツリー右側アイコンの重なり解消 & ガターマージン設計**:
  - 各行のアクションボタン（``, ``, ``）の右側に `Space::new().width(Length::Fixed(16.0))` の専用ガターマージンを新設。
  - スクロールバーが表示されてもボタンと完全に分離され、重なりや誤クリックを徹底防止。
  - ヘッダー右端にも `14.0px` の専用スペースを確保し、サイドバー区切り線との干渉を解消。
  - アイコンボタンのパディングを `[1, 3]` に最適化し、サイドバー最小幅を `280.0px` に拡張して視認性と操作性を大幅向上。
  - エディタペインとの境界スペースを `2.0px` に拡張。
- **テスト・静的解析の完全遵守**:
  - `test_file_tree_context_menu_state_and_right_click` および `test_file_tree_width_and_item_gutter` 単体テストを追加。
  - `cargo test`: 26 core tests + 3 ollama tests = **29/29 passed**。
  - `cargo clippy --all-targets -- -D warnings`: **0 errors, 0 warnings**。
  - `cargo build --release`: クリーンビルド完了。

---

## 3. ファイル構成と役割

```
Rooney/
├── Cargo.toml               # 依存関係定義 (libcosmic, ropey, tree-sitter多言語, futures-channel, rfd, ollama, etc.)
├── README.md                # 日本語・英語バイリンガル公式ドキュメント (Vibe Coding明記)
├── SESSION_HANDOVER.md      # 本ファイル (次回再開用完全ハンドオーバー)
├── src/
│   ├── main.rs              # アプリ起動エントリーポイント (ウィンドウサイズ 2400x2400 設定)
│   ├── config.rs            # AppConfig & SessionConfig (セッション・設定の ~/.config/rooney/config.toml 永続化)
│   ├── app/
│   │   ├── mod.rs           # App 構造体定義、cosmic::Application 実装、init() によるセッション復元
│   │   ├── message.rs       # Message 列挙型定義 (タブ、ファイルツリー、モーダル、AIチャットストリーミング等)
│   │   ├── keybindings.rs   # handle_key_event (キーボードショートカット、モーダルキーハンドリング)
│   │   ├── update.rs        # handle_update (非同期Task/Streamディスパッチ、ファイルCRUD、タブ同期、AIチャット)
│   │   ├── state.rs         # save_config, open_file, クリップボード, 行編集ヘルパー
│   │   └── ui/
│   │       ├── mod.rs       # view() ルートUIマウント (ヘッダー、サイドバー、エディタ、AIチャット、モーダル)
│   │       ├── header.rs    # render_header (タイトル、Editメニュー、Chatトグル、フォント、テーマ)
│   │       ├── tab_bar.rs   # render_tab_bar (タブ切り替え、未保存●、閉じる、新規タブボタン)
│   │       ├── editor_area.rs# render_editor_area (左右分割ペイン、Markdownプレビューコンテナ)
│   │       ├── context_menu.rs# render_context_menu (右クリック浮動メニュー)
│   │       ├── modal.rs     # render_active_modal (新規ファイル、新規フォルダ、リネーム、削除確認モーダル)
│   │       ├── ai_chat.rs   # render_ai_chat_panel (ストリーミング描画、タイピング▋、Stop/Sendボタン、文脈添付)
│   │       └── settings.rs  # render_settings_view (テーマ、透過度、ディミングスライダー)
│   ├── editor/
│   │   ├── mod.rs           # resolve_numpad_char (Waylandテンキー物理キーコード解決ユーティリティ)
│   │   ├── buffer.rs        # TextBuffer (Ropey、単語移動、行削除/複製、コメントトグル、インデント、Undo/Redo)
│   │   └── pane.rs          # EditorPane & EditorTab (タブ管理、ファイル読込/保存、検索マッチ、言語判別)
│   ├── syntax/
│   │   ├── mod.rs
│   │   └── highlighter.rs   # Highlighter & classify_node (12+言語のTree-sitter/字句解析エンジン)
│   ├── markdown/
│   │   ├── mod.rs
│   │   └── renderer.rs      # MarkdownDocument (pulldown-cmark によるAST構築)
│   ├── fs/
│   │   ├── mod.rs
│   │   └── tree.rs          # FileTree (除外パターン、Nerd Font アイコン、ディレクトリ走査)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── canvas_editor.rs # EditorCanvas (intern_font_name、検索ハイライト、サブピクセル文字幅、IME)
│   │   ├── file_tree_view.rs# view_file_tree (サイドバーUI、/// アクションボタン)
│   │   └── markdown_view.rs # view_markdown (リッチMarkdownプレビューコンテナ)
│   ├── theme/
│   │   ├── mod.rs
│   │   └── themes.rs        # 20種類の Classic & Neon テーマ定義、アルファ透過・ディミング
│   ├── font/
│   │   └── mod.rs           # FontManager (fontconfig によるシステムNerd Font検出とサイズ管理)
│   └── ai/
│       ├── mod.rs           # ChatMessage, ChatRole, ChatStreamEvent 再エクスポート
│       └── ollama.rs        # OllamaClient (FIM補完、chat_generate_stream ストリーミング、モデル自動検出)
└── tests/
    ├── core_tests.rs        # 24件のユニットテスト (セッション復元、タブ同期、AIストリーミング蓄積、構文、検索等)
    └── ollama_tests.rs      # 3件の統合テスト (Ollama 接続性、FIM生成、チャットストリーミング実走テスト)
```

---

## 4. キーボードショートカット & マウス操作一覧

| 操作 / ショートカット | 動作 |
|---|---|
| `Ctrl + N` | 新規ファイル作成（ファイル名・パス指定モーダル起動） |
| `Ctrl + O` | ファイルを開く（XDG Desktop Portal ネイティブダイアログ） |
| `Ctrl + Shift + O` | フォルダを開く（サイドバーのツリールートを変更） |
| `Ctrl + S` | ファイル保存（未命名バッファの場合は自動で Save As） |
| `Ctrl + Shift + S` | 名前を付けて保存（Save As） |
| `Ctrl + T` | 新規タブを開く |
| `Ctrl + W` | 現在のタブを閉じる |
| `Ctrl + Tab` / `Ctrl + PageDown` | 次のタブへ切り替え |
| `Ctrl + Shift + Tab` / `Ctrl + PageUp` | 前のタブへ切り替え |
| `Ctrl + F` | ファイル内検索バーの表示 / 非表示トグル |
| `Enter` / `Shift + Enter`（検索時） | 次の一致 / 前の一致へジャンプ |
| `Ctrl + /` | 行コメントのトグル（言語別自動判定） |
| `Ctrl + Shift + K` | 現在行の丸ごと削除 |
| `Ctrl + D` | 現在行の複製 |
| `Ctrl + Left` / `Ctrl + Right` | 単語単位のカーソル移動（+Shiftで単語選択） |
| `Tab` / `Shift + Tab`（選択時） | 選択範囲の一括インデント / アンインデント（4スペース） |
| `Ctrl + C` | 選択テキストのコピー（Wayland システムクリップボード） |
| `Ctrl + X` | 選択テキストの切り取り（Wayland システムクリップボード） |
| `Ctrl + V` | 貼り付け（Wayland システムクリップボードから挿入） |
| `Ctrl + A` | バッファ全選択 |
| `Ctrl + Shift + A` | AI チャットパネルの開閉トグル |
| `Ctrl + Z` | 元に戻す (Undo) |
| `Ctrl + Y` または `Ctrl + Shift + Z` | やり直す (Redo) |
| マウス左ドラッグ | テキスト範囲選択（ビジュアルハイライト） |
| エディタ上マウス右クリック | エディタコンテキストメニュー表示（Copy, Cut, Paste, Select All, Undo, Redo） |
| ファイルツリー上マウス右クリック | ファイル/ディレクトリ/ルートのコンテキストメニュー表示（New File, New Folder, Rename, Delete, Refresh） |
| ヘッダー `󰧑 Edit` | 編集ツールバーの表示/非表示切り替え |
| ヘッダー `󰭹 Chat` | AIチャットパネルの表示/非表示切り替え |
| チャット `󰓛 Stop` | AIコード生成の即時中断・ストリーミング停止 |
| テンキー `0`〜`9` / 記号 (`+`, `-`, `*`, `/`, `.`, `,`, `=`) | 数字および四則演算子記号の直接入力（英字/IME両モード完全対応） |
| テンキー `Enter` | 改行の挿入 / 新規ファイル作成モーダルの確定 |
| `Tab`（未選択時） | AI補完（ゴーストテキスト）の確定挿入 / インデント（4スペース） |
| `Esc` | 検索バー終了 / AI補完破棄 / モーダル終了 / コンテキストメニュー終了 / 選択解除 |
| `Ctrl + B` | ファイルツリーサイドバーの表示/非表示トグル |
| `Ctrl + \` または `Ctrl + E` | 左右2分割（Split / Single）レイアウト切り替え |
| `Ctrl + M` | Markdownプレビューの切り替え |
| `Ctrl + I` または `Alt + Enter` | Local AI FIM 補完の手動トリガー（`Ctrl + Space` は IME 専用に解放） |
| `A-` / `A+` | フォントサイズの縮小 / 拡大 |

---

## 5. 現在のビルドおよびテスト状態

- `cargo clippy --all-targets -- -D warnings`: **0 errors, 0 warnings** (完全クリーン)
- `cargo test`: **29/29 passed (0 failed)**
  - `test_char_advance_ascii_and_cjk` ... ok
  - `test_buffer_selection_and_deletion` ... ok
  - `test_file_type_icons_extended` ... ok
  - `test_file_tree_navigation` ... ok
  - `test_file_tree_scanning` ... ok
  - `test_file_type_icons_nerd_font` ... ok
  - `test_markdown_parsing` ... ok
  - `test_multi_language_detection` ... ok
  - `test_numpad_key_resolution` ... ok
  - `test_editor_pane_saving` ... ok
  - `test_japanese_text_and_width` ... ok
  - `test_search_matches_and_navigation` ... ok
  - `test_app_config_roundtrip` ... ok
  - `test_text_buffer_basic_operations` ... ok
  - `test_text_buffer_fim_extraction` ... ok
  - `test_theme_system_20_themes` ... ok
  - `test_text_buffer_multiline_and_cursor` ... ok
  - `test_pane_tab_management` ... ok
  - `test_word_navigation_and_line_operations` ... ok
  - `test_tree_sitter_highlighting_all_languages` ... ok
  - `test_session_serialization_roundtrip` ... ok
  - `test_ai_chat_data_structures` ... ok
  - `test_ai_chat_streaming_accumulation` ... ok
  - `test_tab_synchronization_on_rename_and_delete` ... ok
  - `test_file_tree_context_menu_state_and_right_click` ... ok (新規追加)
  - `test_file_tree_width_and_item_gutter` ... ok (新規追加)
  - `test_ollama_connectivity_and_models` ... ok
  - `test_ollama_chat_streaming` ... ok
  - `test_ollama_fim_generation` ... ok
- `cargo build --release`: **Clean release build (Code 0)**

---

## 6. 次回再開時の推奨作業・機能拡張案

1. **ファイルツリー内でのドラッグ＆ドロップ（DnD）移動**:
   - ファイルを別のフォルダへドラッグして移動するGUI操作。
2. **ミニマップ（Minimap）またはアウトライン表示**:
   - コード全体の縮小表示や、Tree-sitter関数一覧（Symbol Outline）のサイドパネル。
3. **Git 変更行ガターハイライト（Diff Gutter）**:
   - 変更・追加・削除行を行番号脇にカラーバー表示。



