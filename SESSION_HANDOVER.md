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

### セッション 14: タイトルバーのドメイン集約、Aestheticsモーダル化、およびデフォルトサイズ1600x1600
- **タイトルバーの肥大化・混雑解消（ドメイン別集約 & アイコン単体化の排除）**:
  - **課題**: タイトルバーにテーマ選択（20テーマ）、システムフォント選択、フォントサイズ変更（`A-`/`A+`）、および個別のアクションボタンが並び、幅が狭い場合や解像度によってボタン同士が詰まり視認性が悪化していた。また、単にアイコンのみのボタンにしてしまうと機能が直感的に判別しにくくなる。
  - **解決策**:
    - **「アイコン化は行わない」制約の遵守**: アイコン＋明瞭なテキストラベルを併記した5つのドメイン集約ボタンに統合。
      - `󰈔 File ▾`: New File, Open File, Open Folder, Save File, Save File As, Close Tab
      - `󰧑 Edit ▾`: Undo, Redo, Cut, Copy, Paste, Select All, Comment, Delete Line, Duplicate Line
      - `󰈈 View ▾`: Split / Single Pane, File Tree Sidebar, Markdown Preview
      - `󰚩 AI ▾`: Toggle AI Chat, Attach Selection, Attach File, Trigger FIM Completion
      - `󰒓 Aesthetics`: 外観設定モーダル表示（Themes, Fonts, Size, Transparency, AI Models）
    - `ActiveHeaderMenu` によるフローティングカード型ドロップダウンオーバーレイと、全画面バックドロップによる自然なクリックキャンセルを実装。
    - `header_end` を完全に空とし、タイトルバーの混雑・圧迫感を完全解消。
- **Aesthetics（外観設定）モーダルへの完全集約**:
  - 20種類のテーマ選択ドロップダウン、システムフォント選択ドロップダウン、フォントサイズ増減（`A-` / `A+`）、Waylandウィンドウ透明度スライダー、背景ディミングスライダー、ローカルAIモデル選択ドロップダウンをすべて1つの中央ダイアログ（`Aesthetics Preferences`）に集約。
  - エディタ画面を広く保ち、設定項目を一元管理できるように改良。
- **デフォルトウィンドウサイズの最適化**:
  - `src/main.rs` でデフォルトウィンドウサイズを `1600.0 x 1600.0` に設定。
  - コンテキストメニュー等の画面端クランプ座標を `1550.0` / `1500.0` に最適化。
- **テスト・静的解析の完全遵守**:
  - `cargo test`: 27 core tests + 3 ollama tests = **30/30 passed (0 failed)**。
  - `cargo clippy --all-targets -- -D warnings`: **0 errors, 0 warnings**。
  - `cargo build --release`: 完全クリーンビルド成功。

### セッション 15: 包括的セキュリティ・堅牢性強化（パストラバーサル防止、アトミック保存、機密ファイル保護、DoS防止）
- **セキュリティ・堅牢性の全項目改善**:
  1. **パストラバーサル・ディレクトリ脱出の防止**:
     - `is_valid_file_or_folder_name` を新設し、ファイル名・フォルダ名入力およびリネームにおいて、パス区切り文字（`/`, `\`）、`..`、`.`、NULバイトを完全に拒否。
     - 意図せぬワークスペース外へのファイル作成・上書き・リネーム移動を根本遮断。
  2. **危険な再帰削除の防止（ルート・ホーム保護 & シンボリックリンク安全処理）**:
     - `ConfirmDelete` において、ルート `/`、ホームディレクトリ、ワークスペースルートの削除を検出し、安全違反として遮断。
     - `symlink_metadata()` によりシンボリックリンク判定を行い、ディレクトリリンクであってもリンク先への再帰削除ではなく単一ファイル削除（リンク自体の解除）を実行。
  3. **ファイル保存の原子的置換（Atomic Write）**:
     - `EditorTab::atomic_write_file` を実装。
     - 同一ディレクトリ内に一時ファイル（`.{filename}.tmp.{pid}`）を同期書き込み (`sync_all`) し、既存ファイルのパーミッションを保持した上で `rename` による原子的上書き置換を実行。保存途中のOSクラッシュや電源断による0バイト破損を根絶。
  4. **大容量ファイル・デバイスファイル読み込み制限（50MB上限）**:
     - `load_file` で `std::fs::metadata` のファイルサイズを検証。50MBを超える場合は `ErrorKind::FileTooLarge` で安全に拒否し、メモリ枯渇 (OOM kill) やUIフリーズを未然に防止。
  5. **機密ファイルのAI自動補完シールド（プライバシー保護）**:
     - `is_sensitive_file` を新設（`.env*`, `id_rsa`, `id_ed25519`, `credentials`, `*.pem`, `*.key` 等を自動識別）。
     - 機密ファイルを開いている際、Ollamaへの自動FIM補完リクエスト送信を自動的にバイパス。
  6. **AIストリーミング受信バッファのメモリ上限保護**:
     - `chat_generate_stream` で改行のない異常なストリーム受信時に `line_buffer` が無制限に肥大化しないよう、1MBのセーフティガードを設定。
  7. **フォントキャッシュ Mutex のパニック耐性向上**:
     - `intern_font_name` の `.unwrap()` を `.unwrap_or_else(|e| e.into_inner())` に変更し、万一の Mutex Poisoning 時でもエディタがクラッシュせず描画継続可能に。
  8. **ファイルツリー再帰深度上限ガード**:
     - `FileTree::scan_dir` に `depth > 48` のガードを設け、循環シンボリックリンクによるスタックオーバーフローを徹底防止。
- **テスト・静的解析の完全遵守**:
  - `test_filename_sanitization_and_path_traversal_guards`, `test_sensitive_file_ai_protection`, `test_atomic_save_and_file_size_limits` を追加。
  - `cargo test`: 30 core tests + 3 ollama tests = **33/33 passed (0 failed)**。
  - `cargo clippy --all-targets -- -D warnings`: **0 errors, 0 warnings**。
### セッション 16: 公式SVGアプリケーショングラフィック（タイトルバー＆ドックアイコン）の完全統合とマット版アイコンへの差し替え
- **アプリ公式SVGアイコンの組み込みとWayland / COSMIC Dock完全連携**:
  1. **タイトルバー（HeaderBar）へのベクターアイコン組み込み**:
     - `/images/Rooney-matte-icon.svg`（25,259 bytes）を `include_bytes!` でコンパイル時にバイナリ直接埋め込み。
     - `cosmic::iced::widget::svg::Handle::from_memory` を用い、`cosmic::widget::svg` ウィジェットとして先頭にレンダリング（22px x 22px、パディング [0, 5]）。
     - ドロップダウンメニュー（`File ▾`, `Edit ▾`, `View ▾`, `AI ▾`）のオーバーレイ `menu_x` 座標をアイコン幅に合わせて均等シフト（`44.0`, `128.0`, `212.0`, `300.0`）し、ボタン真下への正確な展開を維持。
  2. **Wayland `APP_ID` と COSMIC Dock / ランチャーの完全一致**:
     - `App::APP_ID` を `"org.pop_os.CosmicCode"` から `"rooney"` へ更新。
     - `~/.local/share/applications/rooney.desktop` の `StartupWMClass=rooneyk` の誤記を `rooney` に修正し、`Icon=rooney` を設定。
     - `App::init()` 起動時に `ensure_system_icons()` を実行。`~/.local/share/icons/hicolor/scalable/apps/` 内の `rooney.svg`, `Rooney-icon.svg`, `Rooney-matte-icon.svg`, `org.pop_os.CosmicCode.svg` を自動生成・同期。COSMIC Dockやアプリ一覧、Alt+Tabスイッチャーで鮮明な公式マット版ベクターアイコンが表示されるように統合。
- **テスト・静的解析の完全遵守**:
  - `test_rooney_icon_integration` を更新（25,259 bytes のマット版 SVG の検証）。
  - `cargo test`: 31 core tests + 3 ollama tests = **34/34 passed (0 failed)**。
  - `cargo clippy --all-targets -- -D warnings`: **0 errors, 0 warnings**。
### セッション 17: ドキュメント（README）の冗長性解消と Waddle 準拠の構造刷新
- **Waddle（`/home/susie/GitHUB/wammed/Waddle/README.md`）の設計・レイアウトに準拠した構成再編**:
  1. **日英分離によるスリム化**:
     - 従来の単一ファイル内に日本語と英語の長文を同居させ、同一内容を冗長に繰り返していた 35KB の構成を廃止。
     - GitHub 標準のバイリンガル構成（`README.md`: 英語 約7.4KB、`README.ja.md`: 日本語 約9.4KB）へ分離。
  2. **モダンなビジュアル＆バッジヘッダー**:
     - `libcosmic`、Rust 1.80+、Wayland Native、Ollama Local AI、Linux (COSMIC/Wayland)、AI Vibe Coding、MIT License バッジをセンタリング配置。
     - バナー画像 (`images/Rooney-banner.svg`) およびキャッチーなタグラインを配置。
  3. **High-Impact なセクション構成**:
     - **💡 Highlights**: コア機能（COSMICネイティブ、ローカルAI FIM/Chat、Ropey、Tree-sitter 12+言語、2分割＆Markdown、日本語IME、ファイルツリー、20テーマ、セキュリティ）を絵文字付き箇条書きで凝縮。
     - **🚀 Quick Start**: 前提環境、Ollamaセットアップ、開発起動（`cargo run`）、リリースインストール（`~/.local/bin/rooney`）の簡潔な手順。
     - **⌨️ Keybindings**: 最重要ショートカットを整理した明瞭な Markdown テーブル。
     - **🔒 Security & Architecture**: オフライン性、アトミック保存、機密ファイルシールド、パストラバーサル保護の要約。
     - **🤖 About This Project (AI Vibe Coding)**: Antigravity (Gemini) とのバイブコーディング実績コールアウト。
     - **📄 License**: MIT License。

### セッション 18: ファイルツリーとタイトルバーの独立透過度設定（Independent Opacity Controls）
- **ユーザー要求**:
  - ファイルツリー（サイドバー）とタイトルバー（ヘッダーバー）の透過度を独立して設定できるようにする。
- **実装内容とアーキテクチャ設計**:
  1. **透過度モデルとカラー算出の拡張 (`src/theme/themes.rs`)**:
     - `EditorTheme` に `file_tree_opacity: f32` および `title_bar_opacity: f32` フィールドを追加。
     - `sidebar_with_alpha(&self) -> Color` を更新し、`self.file_tree_opacity` をアルファ値として適用。
     - `title_bar_with_alpha(&self) -> Color` を新設し、`self.config.gutter_bg` を基底色に `self.title_bar_opacity` をアルファ値として適用。
  2. **設定ファイルへの永続化と後方互換性 (`src/config.rs`, `src/app/state.rs`)**:
     - `AppConfig` に `file_tree_opacity` と `title_bar_opacity` を追加。
     - `#[serde(default = "default_opacity_val")]`（1.0 を返却）を指定し、旧バージョンの設定ファイル読み込み時にも互換性を完全維持。
     - `save_config()` で各透過度値を自動保存。
  3. **COSMIC ネイティブタイトルバーのカスタム透過コンテナ化 (`src/app/mod.rs`, `src/app/ui/header.rs`, `src/app/ui/mod.rs`)**:
     - `libcosmic` 標準のヘッダーバーラッパー（`core.window.show_headerbar = true` 時）は透過しない不透明背景で囲まれる仕様のため、`App::init` で `core.window.show_headerbar = false;` に設定。
     - 代わりに `src/app/ui/header.rs` にて `render_title_bar(&self) -> Element<'_, Message>` を実装し、`cosmic::widget::header_bar()` を直接生成してウィンドウ制御（ドラッグ、最大化、最小化、閉じる）とアクションボタン群をマウント。
     - ヘッダーバーを `cosmic::theme::Container::Custom` でラップし、`self.theme.title_bar_with_alpha()` を背景色として適用することで 0%〜100% の独立透過を実現。
    - ヘッダーバーが `render_view` 最上部に配置されたことに伴い、ドロップダウンメニュー（`File ▾`, `Edit ▾`, `View ▾`, `AI ▾`）の縦位置 `menu_y` を `46.0` に調整し、ボタン直下にピタリと揃えて展開。
  4. **ファイルツリーコンテナの独立透過背景化 (`src/ui/file_tree_view.rs`)**:
     - ファイルツリーのスクロールコンテナを `cosmic::theme::Container::Custom` でラップし、`self.theme.sidebar_with_alpha()` を背景色として適用。
  5. **外観設定モーダル（Aesthetics Preferences）へのスライダー追加 (`src/app/ui/modal.rs`)**:
     - `Editor Window Opacity`（エディタ全体）、`File Tree Opacity`（ファイルツリー）、`Title Bar Opacity`（タイトルバー）、`Background Dimming Overlay`（背景ディミング）の独立調整スライダーを配置。
  6. **テストとドキュメント同期**:
     - `tests/core_tests.rs` に `test_independent_opacity_settings` を追加し、アルファ計算・新旧TOMLシリアライズ互換性を検証（35/35 テスト全件通過）。
     - `README.md` および `README.ja.md` のハイライトとキーバインド説明を更新。

### セッション 19: ファイルツリークリック・右クリック時のスクロールリセット不具合の解消
- **ユーザー報告課題**:
  - ファイルツリークリック時の挙動が安定しない。
  - スクロールした状態でアイテムをクリックすると、表示がデフォルト（ホームディレクトリのトップ）に戻ってしまう。
  - 右クリック時も同様にトップに戻る。
- **根本原因の特定**:
  1. **Scrollable に一意の Id が未付与**:
     - `file_tree_view.rs` 内の `scrollable` に静的 `Id` が付与されておらず、メニュー開閉やアイテム選択時にウィジェット再生成に伴いスクロールオフセット (0, 0) にリセットされていた。
  2. **Sidebar Header のスクロール巻き込み**:
     - ファイルツリー上部のワークスペースヘッダー（` ROOT` や新規・親フォルダ移動ボタン）が `items_col` と同一の `scrollable` 内に含まれていたため、スクロールするとヘッダーが画面外に消え、スクロールリセット時に「ホームディレクトリのトップが現れる」現象をより顕著にしていた。
  3. **Scrollable 内部の `Space(Fill, Fill)` によるレイアウト破綻**:
     - アイテム末尾に `Space::new().height(Length::Fill)` を配置していたため、スクロール可能な無限高さの中で不定なレイアウト計算が発生し、レイアウト再評価のたびにスクロールオフセットが 0 にクランプされていた。

### セッション 20: 右クリック時スクロールリセット＆ウィンドウ下限メニュー隠れ不具合の完全解消
- **ユーザー報告課題**:
  1. 「右クリック時にはまたトップに戻る」（右クリックメニュー開閉時にファイルツリーのスクロール位置がトップに戻ってしまう）。
  2. 「ファイルツリー下部で右クリックした時、メニューがウィンドウ下限に隠れる」（下限付近で右クリックするとメニュー下部が見切れてクリックできない）。
- **根本原因の特定**:
  1. **ルートウィジェットの動的型変更（`Column` ⇄ `Stack`）による全State破棄**:
     - `render_view()` において、メニュー非表示時は `base_view`（`Column`）を直接返却し、メニュー表示時は `cosmic::iced::widget::stack(...)` でラップして返却していた。
     - iced の内部 diff アルゴリズム（`tree.diff`）はルート要素の Tag が `Column` ⇄ `Stack` で切り替わるたびにツリー全体を不一致と判定し、`*self = Tree::new(...)` を実行して全ウィジェットの内部状態（State）を破棄・再生成していた。
     - これにより、右クリックした瞬間およびメニューを閉じた瞬間に `Scrollable` の State が新規作成され、スクロールオフセットが (0, 0) にリセットされていた。
  2. **ハードコードされた 1500px 画面境界チェックによる下部見切れ**:
     - `context_menu.rs` のメニュー配置ロジックで `if cm.y + menu_h > 1500.0` と 1500px がハードコードされていた。
     - 一般的な 800〜1080px 高さのディスプレイではこの判定が常に false となり、ウィンドウ下部（例: y=700px）で右クリックしても下方向へ展開され、メニューの半分以上がウィンドウ下限外に隠れてしまっていた。
- **解決策**:
  1. **恒久的ルート `Stack` アーキテクチャの確立 (`src/app/ui/mod.rs`)**:
     - `render_view()` の返却値を常に `cosmic::iced::widget::stack(layers).width(Length::Fill).height(Length::Fill)` に固定。
     - `layers[0]` は常に不変の `base_view`（`Column`）とし、メニュー表示時は直接 `layers[1]` に `backdrop`、`layers[2]` に `positioned_menu` を重層配置。
     - メニュー開閉時にルートの Tag（`Stack`）および `layers[0]` の Tag（`Column`）が完全に一致し続けるため、iced の差分更新がツリーを破棄せず、`Scrollable` のスクロール位置が 100% 保持される。
  2. **ウィンドウサイズ動的追跡 (`src/app/mod.rs`, `src/app/keybindings.rs`)**:
     - `App` に `window_size: (f32, f32)` を追加し、`iced::Event::Window(Resized/Opened)` および `CursorMoved` イベントを通じてリアルタイムのクライアント領域サイズを追跡。
  3. **スマート境界クランプ・上方向フリップ展開 (`src/app/ui/context_menu.rs`)**:
     - メニューがウィンドウ下限（ステータスバー手前 `max_h - 36.0`）をはみ出る場合、自動的にカーソル上方向（`menu_y = cm.y - menu_h`）へ反転展開。
     - 上部タイトルバー（40px）から下部ステータスバー（`max_h - 30.0`）の間に収まるようクランプし、全メニュー項目が常に画面内に完全に表示されるように修正。エディタ右クリックメニューにも同一ロジックを適用。
- **検証結果**:
  - `tests/core_tests.rs`: `test_context_menu_boundary_clamping` を追加し、上下左右の反転・クランプ動作を単体テストで検証。
  - `cargo clippy --all-targets -- -D warnings`: 警告 0 件。
  - `cargo test`: 36 件全テスト成功（33 core tests + 3 ollama tests）。
  - `cargo build --release && install -m 755 target/release/rooney ~/.local/bin/rooney`: インストール完了。

### セッション 21: Markdown プレビューの GFM 対応および設定画面への仕様切替トグル追加
- **ユーザー要求**:
  - Markdownプレビューの既存CommonMark動作を維持しつつ、GFM（GitHub Flavored Markdown）拡張機能（テーブル、タスクリスト、アラート、打ち消し線、自動リンク）を追加。
  - 設定画面（Aesthetics & Preferences モーダル）から「CommonMark」と「GFM」（推奨デフォルト値）を切り替え可能にし、TOML設定ファイルに永続化。
  - プレビュー画面上にはトグルボタンを配置せず、設定変更時にプレビューペインを即座に再描画する。
  - GFM / CommonMark の両仕様において `breaks: false`（単一改行で `<br>` を作らず半角スペースで連結）を厳密に遵守。
- **実装内容とアーキテクチャ設計**:
  1. **設定モデルと永続化の拡張 (`src/config.rs`)**:
     - `MarkdownSpec` enum（`Gfm`, `CommonMark`）を新設。
     - `AppConfig.markdown_spec`（デフォルト `Gfm`）を追加し、serdeシリアライズ（`"GFM"`, `"CommonMark"` および小文字エイリアス）を実装。
     - 設定モーダルで選択変更された際に即座に `config.toml` に永続化。
  2. **ASTブロック拡張とパーサー仕様分岐 (`src/markdown/renderer.rs`, `src/markdown/mod.rs`)**:
     - `ColumnAlignment`、`TableBlock`、`AlertKind` を新設。
     - `MarkdownBlock` に `Table(TableBlock)`、`Alert { kind, text }`、`ListItem { depth, text, task_status: Option<bool> }` を追加。
     - `MarkdownDocument::parse(source: &str, spec: MarkdownSpec)` を実装。
     - GFMモード: `ENABLE_TABLES | ENABLE_TASKLISTS | ENABLE_STRIKETHROUGH | ENABLE_GFM | ENABLE_FOOTNOTES` を適用。
     - CommonMarkモード: `Options::empty()` を適用（テーブルやタスクリスト、アラートは標準テキスト・標準引用として処理）。
     - 打ち消し線（`~~text~~`）はUnicode結合文字（`\u{0336}`）を付与してレンダリング。
     - GitHubアラート（`[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`）を専用Calloutブロックへ解析。
     - `Event::SoftBreak` は `' '` を挿入し、GFM/CommonMark双方で `breaks: false` を一貫して維持。
  3. **ペイン・タブへの仕様伝播と即時再描画 (`src/editor/pane.rs`, `src/app/update.rs`, `src/app/mod.rs`)**:
     - `EditorTab` および `EditorPane` に `markdown_spec` を持たせ、全タブへのカスケード更新および `refresh_markdown()` による即時ゼロ遅延プレビュー再描画を実現。
  4. **リッチなプレビューUIスタイリング (`src/ui/markdown_view.rs`)**:
     - テーブル: カラム最大文字幅に応じた動的幅計算、アクセントカラーヘッダー、区切り線、水平スクロールラッパー。
     - タスクリスト: チェック済み（`󰱒 ` / コメント色）、未完了（`󰄱 ` / 前景色）、通常リスト（`• ` / アクセント色）。
     - GitHubアラート: 各種別の固有アイコン（`󰋽 Note`, `󰌵 Tip`, `󰅒 Important`, `󰀦 Warning`, `󰳦 Caution`）と左アクセントバー付きコールアウトボックス。
  5. **設定モーダルへのUI追加 (`src/app/ui/modal.rs`)**:
     - 外観・設定モーダル（Aesthetics & Preferences）に「Markdown Specification」ドロップダウンを追加。
  6. **テストと検証**:
     - `tests/core_tests.rs`: `test_markdown_spec_commonmark_vs_gfm`（テーブル、タスクリスト、アラート、打ち消し線、breaks: false の差異検証）、`test_markdown_spec_config_persistence`（TOML往復シリアライズ・エイリアス検証）を追加。
     - `cargo test`: 38/38 全テスト通過（35 core tests + 3 ollama tests）。
     - `cargo clippy --all-targets -- -D warnings`: 警告 0 件。
     - `cargo build --release && install -m 755 target/release/rooney ~/.local/bin/rooney`: インストール完了。

### セッション 22: タイトルバーメニュー・モーダルの背景不透明化とテーマ色同期
- **ユーザー要求**:
  - タイトルバーをクリックした際に表示されるメニュー（たとえば Aesthetic をクリックした際の設定メニューや、タイトルバードロップダウン）の背景を透過処理無し（alpha = 1.0）でテーマに沿ったカラーにする。
- **背景と課題**:
  - 従来、`render_settings_modal`（Aesthetics & Preferences）および各モーダル（New File, New Folder, Rename, Delete）の `modal_box` はデフォルトの `container(column)` でラップされており背景指定が存在しなかったため、100% 透明（透過）になっていた。そのため、エディタのソースコードや行番号の上に設定文字が直接重なって表示され、視認性が著しく悪化していた。
  - タイトルバーのドロップダウンメニュー（`File ▾`, `Edit ▾`, `View ▾`, `AI ▾`）およびコンテキストメニューは `Container::Card` を使用しており、Wayland / COSMIC のシステムテーマに依存した透過や色ズレが生じていた。
- **実装内容とアーキテクチャ設計**:
  1. **モーダルの不透明化とバックドロップ調和 (`src/app/ui/modal.rs`)**:
     - `wrap_modal` ヘルパー関数を新設。
     - モーダルダイアログの `modal_box` 背景を `Color { a: 1.0, ..theme.config.gutter_bg }` に設定し、テーマ境界線 `theme.config.border`（角丸 8px、線幅 1px）を付与して 100% 不透明（透過処理無し）なソリッドサーフェス化。
     - ダイアログ背面には 60% のダークディミングバックドロップ（`rgba(0, 0, 0, 0.60)`）を配置し、背後のエディタコードを程よく減光してモーダルの視認性と立体感を飛躍的に向上。
     - 全モーダル（Aesthetics & Preferences, New File, New Folder, Rename, Delete）に適用。
     - `render_settings_modal` 内の全ラベルテキストに `theme.config.fg` カラーを適用し、不透明背景上でのコントラストと視認性を最大化。
  2. **タイトルバードロップダウンメニューの不透明化 (`src/app/ui/header.rs`)**:
     - `render_header_menu` の `menu_box` を `Container::Custom` に切り替え、`Color { a: 1.0, ..theme.config.gutter_bg }` および `theme.config.border`（角丸 6px）を適用。
  3. **コンテキストメニューの不透明化 (`src/app/ui/context_menu.rs`)**:
     - エディタおよびファイルツリーの右クリックコンテキストメニューも同様に `Container::Custom` + `gutter_bg`（alpha = 1.0）へ切り替え、全メニュー・ポップオーバーの不透明度とテーマ色を一貫統一。
- **検証結果**:
  - `cargo clippy --all-targets -- -D warnings`: 警告 0 件。
  - `cargo test`: 38/38 全テスト通過（35 core + 3 ollama）。
### セッション 23: OPUS レビューに基づく全面改善・セキュリティ強化・パフォーマンス最適化
- **レビュー指摘への全面対応 (`Rooney_project_review_by_OPUS.md`)**:
  1. **設定ファイル（`config.toml`）のアトミック保存**:
     - `src/config.rs` の `AppConfig::save()` を従来の直接 `fs::write()` から、`EditorTab::atomic_write_file(&path, &content)` による原子的置換に変更。
     - 一時ファイル書き込み (`.{file_name}.tmp.{pid}`)、`sync_all()` によるディスク同期、`rename()` によるアトミック置換を徹底し、OSクラッシュや強制終了による設定ファイル破損リスクを根絶。
  2. **機密ファイル AI シールドの拡充 (`src/app/update.rs`)**:
     - `is_sensitive_file()` を大幅拡張。
     - `.keystore`, `.jks` (Java KeyStore)、`.npmrc` (npm 認証トークン)、`.pypirc` (PyPI 認証)、`kubeconfig`, `.kubeconfig` (Kubernetes 設定)、`token`, `.token`、ファイル名に `secret` を含む全ファイル、および `.aws` (`.aws/credentials`), `.kube` ディレクトリ配下の全ファイルを自動検出し、AI への機密漏洩を未然に遮断。
  3. **安全なホームディレクトリ解決 (`src/app/mod.rs`)**:
     - `ensure_system_icons()` において、環境変数 `std::env::var_os("HOME")` を直接信用せず、`directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf())` で安全にホームディレクトリを解決。
  4. **外部コマンド（`fc-list`）のフォールバック強化 (`src/font/manager.rs`)**:
     - `FontManager::new()` において、`fc-list` の実行失敗時にシステムの絶対パス `/usr/bin/fc-list` へのフォールバック試行を追加。
  5. **Undo/Redo スタックの $O(1)$ 化 (`src/editor/buffer.rs`)**:
     - `undo_stack` および `redo_stack` の内部実装を `Vec<Rope>` から `std::collections::VecDeque<Rope>` に変更。
     - 100 件上限超過時の先頭削除を `Vec::remove(0)`（$O(n)$ 要素シフト）から `VecDeque::pop_front()`（$O(1)$ リングバッファ操作）に改善し、タイピング時の Undo 記録レイテンシを最小化。
  6. **型安全な AI 構造化エラー型 `OllamaError` の導入 (`src/ai/ollama.rs`)**:
     - 従来の非構造化 `String` エラーから、専用の構造化 enum `OllamaError` (`Disabled`, `HttpError`, `StatusError(u16, String)`, `ParseError`, `StreamError`, `BufferExceeded`) を導入。`std::fmt::Display` および `std::error::Error` を実装し、`Result<_, OllamaError>` による堅牢なエラーハンドリングを実現。
  7. **Tree-sitter 増分構文解析（Incremental Parsing）の有効化 (`src/syntax/highlighter.rs`)**:
     - `update_source` で `parser.parse(source, self.tree.as_ref())` とし、既存の構文木参照を渡すことで、タイピング時に変更されていない構文ノードを再利用する増分パースを有効化。
  8. **依存関係の健全性向上 (`Cargo.toml`)**:
     - `tokio`: 不要な依存を含む `features = ["full"]` から、必要な機能のみ (`features = ["rt-multi-thread", "time", "net", "macros"]`) に絞り込みビルドフットプリントを最適化。
     - `libcosmic`: `git HEAD` 参照から `rev = "d4d71fd53e5ed6bd3a430089114dffa2da3cd498"` でコミットハッシュを明示固定し、ビルド再現性を保証。
  9. **テストスイート拡充 (`tests/core_tests.rs`)**:
     - `test_sensitive_file_ai_protection` を拡充（`.npmrc`, `.pypirc`, `kubeconfig`, `.jks`, `.keystore`, `.token`, `secret`, `.aws`, `.kube` の検出検証）。
     - `test_undo_redo_stack_depth_and_performance` を追加（120回の連続編集による 100 件 FIFO 上限と LIFO アンドゥの完全検証）。
     - `test_buffer_edge_cases` を追加（空バッファでの全編集・カーソル動作、10万文字以上の超長行の編集・クランプ検証）。
     - `test_atomic_config_save` を追加（`config.toml` の原子的保存と一時ファイル自動清掃の検証）。
### セッション 24: エディタ内自動スクロール（Auto-scrolling to Cursor）＆右端スクロールバー（Independent Right-edge Scrollbars）
- **ユーザー要求**:
  - エディタ内でカーソルが上端/下端に到達した際、画面が自動でスクロールしない問題を解消する。
  - エディタ右端にスクロールバーをつける（左右2分割時はそれぞれ独立して表示・動作）。
- **背景と技術的課題**:
  - 従来、矢印キーやタイピングでカーソルがエディタ領域の上下限を超えても、`scroll_y` が追従せずカーソルが画面外に見切れていた。
  - `libcosmic` / `iced` の `Widget::draw` では `&self`（不変参照）しか渡されないため、描画タイミングで正確な行折り返し（`visual_rows`）やピクセル高さ（`bounds.height`）に基づいてスクロール位置を調整するには、内部可変性（Interior Mutability）が必要であった。
  - 単純に常時カーソル位置へ追従させると、ユーザーがマウスホイールやスクロールバーで他行を閲覧している最中に勝手にカーソル位置へ引き戻される問題が発生するため、操作意図に応じた自動スクロールの能動/休止状態管理が必要であった。
- **実装内容とアーキテクチャ設計**:
  1. **内部可変性によるゼロフレーム遅延スクロール管理 (`src/editor/pane.rs`)**:
     - `EditorTab` の `scroll_y` および `scroll_x` を `std::cell::Cell<f32>` に変更。
     - 自動スクロール要求フラグ `needs_scroll_to_cursor: std::cell::Cell<bool>` を新設。
     - `mark_cursor_moved(&self)` メソッドを追加し、カーソル移動時に `needs_scroll_to_cursor.set(true)` を即座に通知。
     - 新規ファイル読込時（`load_file`）や編集時（`on_content_changed`）にも自動で `needs_scroll_to_cursor = true` を設定。
  2. **キーバインドおよび各種操作でのカーソル移動追従 (`src/app/keybindings.rs`, `src/app/update.rs`)**:
     - 単語移動（`Ctrl + Left/Right`）、矢印キー（`Up/Down/Left/Right`）、`Home`、`End`、`PageUp/PageDown` で `pane.mark_cursor_moved()` を呼び出し。
     - マウスクリック（`ClickPane`）、ドラッグ範囲選択（`DragSelect`）、検索ジャンプ（`NextSearchMatch`, `PrevSearchMatch`）でも `mark_cursor_moved()` を実行。
     - マウスホイール操作（`ScrollPane`）やスクロールバーのドラッグ操作（`SetScrollY`）では `needs_scroll_to_cursor.set(false)` を設定し、ユーザーがスクロールして閲覧している間はカーソルへ勝手に引き戻されないよう自然なUXを保証。
  3. **自動スクロールの境界クランプとマージン計算 (`src/ui/canvas_editor.rs`)**:
     - `draw_frame` の冒頭で、`needs_scroll_to_cursor` が有効な場合にカーソルの表示行インデックス（`cursor_v_idx`）を検出。
     - 2行分のマージン（`margin = (2.0 * line_height).min(bounds.height * 0.25).max(0.0)`）を設け、カーソルが上端マージンより上または下端マージンより下へ移動した場合にのみ、滑らかに `scroll_y` を更新。
  4. **独立した右端スクロールバーの描画とインタラクション (`src/ui/canvas_editor.rs`)**:
     - **スクロールバー描画**: コンテンツ高さがペイン高さを超える場合、右端12pxに背景トラック（`rgba(0,0,0,0.15)`）と境界線（`theme.config.border`）を描画。つまみ（Thumb）は全行数に応じたプロポーショナル高さ（最小28px）で描画し、アクティブペイン時はアクセントカラー（透過75%）、非アクティブ時はボーダーカラー（透過60%）で表示。
     - **マウスインタラクション**: 右端14pxにカーソルが進入した際、自動で `mouse::Interaction::Pointer`（ポインターカーソル）に切り替え。
     - **ドラッグ操作**: つまみをクリックしてドラッグする際、マウスが左右に多少ずれても追従を維持（`global_pos.y` に基づくクランプ計算）。
     - **トラッククリックジャンプ**: つまみ以外のトラックをクリックした際、クリック位置が即座につまみ中心となるようスムーズにジャンプ移動。
     - **分割モード（Split Layout）完全対応**: 各ペインは独立した `EditorCanvas` インスタンスとして描画されるため、左右ペインそれぞれが右端に独立したスクロールバーを持ち、独立してスクロール・操作可能。
  5. **テストスイート拡充 (`tests/core_tests.rs`)**:
     - `test_editor_auto_scroll_flag_and_coordinates`: 新規作成時、描画消費時、明示的カーソル移動時、マウスホイール休止時、文字入力時、タブ切替時の一連の `needs_scroll_to_cursor` 挙動を検証。
     - `test_scrollbar_proportions_and_thumb_mapping`: 100行バッファ（2400px）でのプロポーショナルつまみ高さ（150px）、上・中・下の位置マッピング、上下マージン自動スクロール計算（1752px / 72px）、および5万行超長ファイルでの最小高さクランプ（28px）を厳密に検証。
### セッション 25: CLI 引数処理 & file:// URI 対応によるデスクトップファイルマネージャからのファイルオープン対応
- **ユーザー要求**:
  - デスクトップ環境（COSMIC）のファイルマネージャからテキストファイルをダブルクリックして Rooney を起動した際、空白エディタが立ち上がるだけでファイルが開かれない不具合の解消。
  - コマンドライン引数（CLI 引数 / argv）を受け取り、起動直後にそのファイルの内容を読み込んでエディタに表示する。
  - 相対パスの場合は絶対パスに解決（正規化）して開く。
  - `file://` スキームが付与された URI 形式（`file:///...` や URL エンコード `%20`, `%E3%83%86...` 等）も適切にローカルパスに変換して開けるようにする。
  - 引数が渡されなかった（通常起動の）場合は、これまで通り新規の空白エディタ / セッション復元を行う。
  - ビルドおよび `/home/susie/.local/bin/rooney` への配置。
- **背景と根本原因**:
  - `src/main.rs` において `cosmic::app::run::<App>(settings, ())` のように flags に空タプル `()` がハードコードされており、`std::env::args()` の取得および `App::init` への受け渡し処理が一切存在していなかった。
  - そのため、`.desktop` ファイルの `Exec=rooney %f` やファイルマネージャからファイル引数・URI が渡されても無視され、常に直前のセッション復元またはデフォルトの Welcome バッファが開かれていた。
- **実装内容とアーキテクチャ設計**:
  1. **CLI 引数および URI パーサーの新設 (`src/app/flags.rs`)**:
     - `normalize_path(&Path) -> PathBuf`: 実在するファイルの場合は `canonicalize()` で実体を解決。新規作成ファイル等で未実在の場合は `directories::BaseDirs` による `~`（チルダ展開）およびカレントディレクトリ（`current_dir()`）との結合を行い、`Component` 解析により `.` や `..` を論理的に正規化。
     - `parse_path_or_uri(&str) -> Option<PathBuf>`:
       - `file://` スキームで始まる場合、WHATWG URL 規格準拠の `url::Url::parse` および `to_file_path()` を用いてパーセントデコード（半角スペース `%20` や日本語 UTF-8 `%E3%83%86%E3%82%B9%E3%83%88` 等）を完全処理。
       - 非標準 URI への安全なフォールバックパーセントデコーダー（`percent_decode_str`）を実装。
       - 通常の絶対パス・相対パス・チルダパスを正確に判定・正規化。
     - `AppFlags`: `pub files: Vec<PathBuf>` を保持し、`from_args` で `-h`, `--help`, `-v`, `--version`, `--` 等のオプションフラグを除外して対象ファイル群を抽出。
  2. **エントリーポイントの更新 (`src/main.rs`)**:
     - `std::env::args().skip(1)` を取得。
     - `-h` / `--help` で使用方法（Usage / Options）を表示して終了。
     - `-v` / `--version` でバージョン番号を表示して終了。
     - `AppFlags::from_args(args)` を生成し、`cosmic::app::run::<App>(settings, flags)` へ引き渡し。
  3. **アプリケーション初期化でのファイルロード統合 (`src/app/mod.rs`)**:
     - `cosmic::Application::Flags` を `AppFlags` に設定。
     - `App::init` において `flags.files` が指定されている場合：
       - 指定されたパスがディレクトリであればファイルツリールート（`file_tree.set_root`）に設定。
       - ファイルであれば、最初のファイルの親ディレクトリに合わせてファイルツリーのフォーカス・選択（`file_tree.select`）を行い、エディタペインで `left_pane.open_file(file_path)` を実行して即座にロード＆表示。
       - 起動時ステータスメッセージを `Opened <ファイル名>` に更新。
       - モーダル用の初期作成先ディレクトリ（`new_file_target_dir`, `new_folder_target_dir`）を開いたファイルのフォルダに同期。
     - `flags.files` が渡されなかった場合（通常起動）は、従来通り直前のセッション復元または Welcome バッファを表示。
  4. **エディタペインのタブ再利用・新規ファイル対応強化 (`src/editor/pane.rs`)**:
     - `open_file`: 初期起動時の「Welcome」または未変更の「Untitled」タブが存在する場合、不要な孤立タブを残さずそのタブを再利用して開いたファイルに切り替え。
     - まだディスク上に存在しない新規ファイルパス（`rooney new.txt` 等）が渡された場合でも、そのパスを保持したタブを初期化し、`Ctrl+S` で即座にそのパスへ保存できるよう対応。
  5. **テストスイート拡充 (`tests/core_tests.rs`)**:
     - `test_cli_flags_and_path_resolution`: `file:///...`、`file://localhost/...`、スペース含む `%20`、日本語 `%E3%83%86%E3%82%B9%E3%83%88`、絶対パス、相対パス、ドット正規化、空文字列、フラグスキップの全パターンを網羅検証。
     - `test_editor_pane_open_file_welcome_tab_reuse_and_non_existent`: Welcome タブの自動再利用、未実在ファイルパスのタブ初期化、複数タブ展開、既存タブへのスイッチを検証。
### セッション 25: 外部レビュー対応（Unicode 座標補正・レンダリング性能改善・README適正化・Roadmap新設）
- **外部コードレビュー（総合評価 8.3/10）への対応**:
  1. **Tree-sitter Byte Offset ↔ Char Index 厳密変換の実装 (`src/syntax/highlighter.rs`)**:
     - `ByteCharMapper` および `byte_to_char_idx` を新設。
     - Tree-sitter の `Point.column`（UTF-8 バイトオフセット）を描画側の文字インデックス（Char Index）へ正確に変換。
     - ASCII 行は $O(1)$ かつメモリ確保ゼロでバイトオフセットをそのまま処理し、マルチバイト文字（日本語・全角記号・絵文字）を含む行では `char_indices` に基づくバイナリサーチで厳密にマッピング。
     - 同一行内で日本語文字列リテラルの後に続くコード（例: `let s = "こんにちは世界 🚀"; let count = 100;`）でも、後続トークンのハイライトが手前・奥へ一切ずれないことを保証。
     - 複数行にまたがるブロックコメントや複数行文字列についても、行境界で正しくクリッピングして各行にハイライトを適用。
     - `fallback_lexical_highlight`、`highlight_markdown_line`、`highlight_ini_line` 内のバイト長と文字数の混同を解消。
  2. **描画ループの `Vec<char>` アロケーション完全排除 (`src/ui/canvas_editor.rs`)**:
     - 従来毎フレーム大量に呼び出されていた `chars().collect::<Vec<char>>()` を以下の 6 箇所すべてで完全排除：
       1. `build_visual_rows`: `chars().enumerate()` の直接走査へ移行。
       2. `cursor_screen_pos`: `skip().take()` による直接文字幅加算へ移行。
       3. `pos_to_char_coords`: `skip().take().enumerate()` による直接走査へ移行。
       4. 選択範囲ハイライト描画: イテレータ直接走査へ移行。
       5. 検索一致ハイライト描画: イテレータ直接走査へ移行。
       6. テキスト描画ループ: `slice_by_char_indices` を導入し、行全体の `Vec<char>` を生成せず必要なトークン文字列スライスのみを抽出。
  3. **README の記述適正化 & Roadmap セクション新設 (`README.md`, `README.ja.md`)**:
     - 日英完全同期を維持しつつ、過度な主張（「ゼロコピー」「鉄壁のセキュリティ」「0ms瞬時起動」「0バイト破損根絶」）を実装実態に即した表現へトーンダウン。
     - 冒頭タグラインでスタック構成の強み（COSMIC/Wayland × Ropey × Tree-sitter × 日本語 IME × Ollama）を明瞭化。
     - 「Features（現在利用可能な機能）」と「Roadmap（今後の最適化・拡張予定）」を明確に分離（ビューポート仮想スクロール、`cosmic-text` 実グリフメトリクス連携、File Watcher、インライン Markdown AST、Undo/Redo 差分化）。
  4. **テストスイート拡充 (`tests/core_tests.rs`)**:
     - `test_treesitter_multibyte_and_emoji_highlight_coordinates`: 日本語・絵文字混在環境でのトークン範囲と文字スライスの厳密一致を検証。
     - `test_byte_char_mapper_and_slice_by_char_indices`: 各種マルチバイト文字・絵文字境界でのマッピングとスライスの正確性を検証。
     - `test_multiline_block_comment_highlighting`: 複数行コメントの各行ハイライト適用を検証。
### セッション 26: コアレンダリング仮想化・Tree-sitter Incremental Edit・行キャッシュ・マルチスケール性能ベンチマーク
- **外部レビュー重要課題（コア最適化）の完全実装**:
  1. **Tree-sitter 厳密インクリメンタル編集 (`InputEdit`) 実装 (`src/editor/buffer.rs`, `src/syntax/highlighter.rs`, `src/editor/pane.rs`)**:
     - `TextBuffer` に `pub last_edit: Option<tree_sitter::InputEdit>` を追加。
     - `compute_input_edit(rope, start_char_idx, end_char_idx, inserted_text) -> InputEdit` ヘルパーを新設。
     - `char_to_byte`、`char_to_line`、`line_to_byte` を用いて、挿入・削除・置換操作における `start_byte`、`old_end_byte`、`new_end_byte`、`start_position`、`old_end_position`、`new_end_position`（行・列バイトオフセット）を厳密算出。
     - `insert_char`、`insert_str`、`delete_backspace`、`delete_forward`、`delete_selection`、`delete_line`、`duplicate_line` に `last_edit` 計算を統合（Undo/Redo やバッチ操作時は None で完全再パースを保証）。
     - `Highlighter` に `has_pending_edit: bool` フラグおよび `apply_edit(&mut self, edit: &InputEdit)` を実装。`tree.edit(edit)` を呼び出し後、`update_source` で `parser.parse(source, Some(&tree))` による真のインクリメンタル AST 解析を実行。
  2. **論理行単位の構文ハイライトキャッシュ導入 (`src/syntax/highlighter.rs`)**:
     - `CachedLine { hash: u64, spans: Vec<HighlightSpan> }` および `Highlighter.cache: RefCell<HashMap<usize, CachedLine>>` を導入。
     - `highlight_line` 呼び出し時、行文字列のハッシュ値と行インデックスでキャッシュを即座に参照。同一行テキストであれば AST 探索・トークナイズをスキップし、~1 µs（マイクロ秒）の極小レイテンシでキャッシュヒットを返却。
     - ソフト折り返しによる複数行描画や、マウス移動・カーソル点滅に伴う毎フレームの同一行再ハイライト負荷を完全に排除。テキスト編集時（`apply_edit` / `update_source`）にはキャッシュを自動クリア。
  3. **`build_viewport_visual_rows()` による Viewport 仮想化と $O(1)$ スクロール (`src/ui/canvas_editor.rs`)**:
     - 従来のバッファ全行を毎フレーム走査・生成していた `build_visual_rows()` を刷新。
     - `total_content_height(&self) -> f32` を実装し、スクロールバーのつまみプロポーショナル計算・ドラッグ追従・トラックジャンプを $O(1)$ の瞬時計算に高速化。
     - `build_viewport_visual_rows(bounds_width, scroll_y, bounds_height)` を導入。`scroll_y` と `bounds.height` から画面内に現在表示される可視論理行範囲（+ 上下マージン5行）のみを抽出し、画面外のレイアウト計算を完全スキップ。
     - `VisualRow` に絶対座標 `y: f32` を持たせ、描画ループ内でのインデックス掛け算を排除。
     - `cursor_screen_pos` をカーソル対象行（`cursor.0`）のみのローカルレイアウトに最適化し、$O(N)$ から $O(\text{カーソル行長})$ へ短縮。
     - `pos_to_char_coords` をマウスクリック対象行のみのローカルレイアウトに最適化し、$O(N)$ から $O(\text{クリック行長})$ へ短縮。
  4. **マルチスケール性能ベンチマークテストの整備 (`tests/benchmark_tests.rs`)**:
     - 10KB、100KB、1MB、10MB、50MB（185万行）の各スケールで、バッファ読み込み、初期 Tree-sitter 構文解析、1文字インクリメンタル編集＆差分解析、ビューポートレイアウト計算、ハイライトキャッシュヒットを実測するテストハーネスを構築。
     - **実測ベンチマーク結果**:
       - **Viewport Layout**: 10KB（326 µs）、100KB（365 µs）、1MB（383 µs）、10MB（359 µs）、50MB（427 µs）。50MB / 185万行でも **0.43 ms 以内** で完了し、2,300 FPS 以上のスループットを実証。
       - **Highlight Cache Hit**: 全スケールで **0.8〜2.0 µs** の超高速アクセス。
       - **Incremental Edit & Parse**: 10KB / 100KB で **1.0〜2.2 ms** で差分解析完了。
### セッション 27: 抜本的グリフメトリクス連携による全角記号・ダッシュカーソル位置ズレの根本解消
- **背景と課題**:
  - `「正確に見せる」――この2方向で` などの日本語文において、ダッシュ記号 `――`（U+2015 Horizontal Bar, East Asian Ambiguous）の表示幅とエディタのカーソル進捗幅（`char_advance`）に乖離が生じ、ダッシュの直後からカーソルが文字の上に重なってずれる現象が発生。
  - 原因: `unicode-width` の `UnicodeWidthChar::width()` は非 CJK コンテキスト（デフォルト）で Ambiguous 文字を幅 1（8.4px）と判定していたが、GUI レンダリング（`cosmic-text` + JetBrainsMono Nerd Font / Noto Sans CJK JP）では 14.0px（全角幅）でラスタライズされていたため、2文字で 11.2px のズレが累積していた。
- **抜本的解決策（Roadmap: Direct Glyph Metrics Integration の実現）**:
  - 固定幅や簡易ヒューリスティクスに頼らず、描画エンジンである `cosmic-text` の実際のグリフシェーピング結果（`Buffer::layout_runs() -> glyph.w`）を直接測定するアーキテクチャ `measure_glyph_advance(c, font_size, font_name)` を `src/ui/canvas_editor.rs` に実装。
  - ASCII 印字可能文字（`' '..='~'`）は `font_size * 0.60` のファストパスで高速処理。
  - 非 ASCII 文字は `cosmic_text::FontSystem` による実シェーピングを行い、スレッドセーフな `OnceLock<GlyphMetricsMap>`（`(char, u32, &'static str) -> f32`）にキャッシュ。
  - 取得コストは初回計測時のみで、キャッシュヒット時は ~10ns。エディタの 2,000+ FPS レンダリング性能を一切損なわない。
  - `EditorCanvas::char_advance_with_font` / `glyph_advance` を導入し、Viewport レイアウト計算、カーソル座標計算（`cursor_screen_pos`）、マウスクリック文字判定（`pos_to_char_coords`）、選択範囲ハイライト、検索マッチハイライト、文字描画セグメント、IME プレエディット幅の全箇所を統一。
- **検証結果**:
  - `tests/core_tests.rs` に `test_cosmic_text_glyph_layout` を追加。
  - `EditorCanvas::char_advance_with_font('―', 14.0, "JetBrainsMono Nerd Font") == 14.0px` を確認。
  - 対象文 `「正確に見せる」――この2方向で` の文字進捗累積和（218.40px）が、`cosmic-text` の単一行レイアウト幅（218.40px）と 0.00px 差で完全一致することを検証。
  - `cargo clippy --all-targets -- -D warnings`: **警告 0 件**。
  - `cargo test`: **50/50 全テスト通過 (1 benchmark + 46 core + 3 ollama, 0 failed)**。
  - `cargo build --release && install -m 755 target/release/rooney ~/.local/bin/rooney`: インストール完了。
  - `README.md` & `README.ja.md` の Roadmap チェックボックス更新（`Direct Glyph Metrics Integration` 完了）および機能ハイライト反映。

### セッション 28: 50MB 入力レイテンシのプロファイリング分解・Glyph Cache 最適化・複合文字実GUI検証
- **50MB 入力レイテンシ（444ms）のプロファイリング分解とボトルネック特定**:
  - 50MB（1,852,606 行 / 52,428,810 バイト）での 1 文字入力処理の各ステップをマイクロ秒単位で実測分解：
    1. `Rope` 挿入 & Undo 履歴クローン: **18.9 µs** (0.018 ms) — 極めて高速
    2. `tree.edit(&InputEdit)`: **7.4 µs** (0.007 ms) — 極めて高速
    3. ソース供給 `full_text()`: **11.33 ms** — 52MB の String を毎キーストローク全走査・新規ヒープ確保・コピー（大きな負荷）
    4. `parser.parse()` 差分解析: **232.57 ms (Release) / 422.63 ms (Debug)** — **最大のボトルネック**（C言語内部で編集点以降の全 1,000 万ノードのオフセット調整と新AST構築を同期的に実行）
    5. （参考検証）`parser.parse_with()` コールバック: **9,133 ms** (9.13 秒) — FFI コールバック往復オーバーヘッドのため連続メモリ渡しより 40 倍遅延
- **ハイブリッド同期/デバウンス解析の実装 (`src/editor/pane.rs`, `src/app/update.rs`)**:
  - ファイルサイズ ≤ 2MB: 即時同期インクリメンタル更新（< 4ms、144+ FPS を維持）。
  - ファイルサイズ > 2MB〜50MB: 入力時同期処理を `TextBuffer` 挿入と `apply_edit`（合計 **28 µs**）のみにとどめ、52MB `full_text()` と Tree-sitter 全域パースを 100ms デバウンス（`Message::Tick`）へ遅延。
  - **実測結果**: 50MB ファイルでの 1 文字入力タイピング遅延が **444 ms → 0.028 ms（28.59 µs、15,000倍以上の高速化）** を達成。
- **`measure_glyph_advance` の `RwLock` 化・ライフサイクル最適化 (`src/ui/canvas_editor.rs`, `src/app/update.rs`)**:
  - `Mutex` から `std::sync::RwLock` に移行し、共有リードロック（~15ns）で複数スレッド・複数ペインのロック競合を完全排除。
  - `pub fn clear_glyph_cache()` を新設し、フォントサイズ変更（`IncreaseFontSize` / `DecreaseFontSize`）やテーマ切り替え時に安全にキャッシュ破棄。
- **複合文字（Mixed-Script）の実座標・選択・IME 整合性テスト体系化 (`tests/core_tests.rs`)**:
  - `test_mixed_script_coordinates_and_roundtrip` を追加。
  - 和文約物（`「正確に見せる」――この2方向で……（検証中）！`）、絵文字 ZWJ（`🦀 Rust 🚀 and 👨‍💻 Hacker`）、結合文字（`Cafe\u{0301} & か\u{3099}`）、タブ混在（`\tfn main() {\t// こんにちは世界！`）の全ケースで：
    1. カーソル X 座標が厳密に単調増加（`x_{i+1} > x_i`）
    2. `cursor_screen_pos` と文字累積幅が完全一致
    3. `pos_to_char_coords` の左右半分クリック判定が正確にラウンドトリップ（Round-trip）
- **検証結果**:
  - `cargo clippy --all-targets -- -D warnings`: **警告 0 件**。
  - `cargo test`: **51/51 全テスト通過 (1 benchmark + 47 core + 3 ollama, 0 failed)**。
  - `cargo build --release && install -m 755 target/release/rooney ~/.local/bin/rooney`: インストール完了。

---

## 3. ファイル構成と役割

```
Rooney/
├── Cargo.toml               # 依存関係定義 (libcosmic, ropey, tree-sitter多言語, futures-channel, rfd, ollama, url, etc.)
├── README.md                # 英語公式ドキュメント (Waddle準拠スリム構成)
├── README.ja.md             # 日本語公式ドキュメント (Waddle準拠スリム構成)
├── SESSION_HANDOVER.md      # 本ファイル (次回再開用完全ハンドオーバー)
├── src/
│   ├── main.rs              # アプリ起動エントリーポイント (CLI引数解析, --help/--version, ウィンドウサイズ設定)
│   ├── config.rs            # AppConfig & SessionConfig (セッション・設定の ~/.config/rooney/config.toml 永続化)
│   ├── app/
│   │   ├── mod.rs           # App 構造体定義、cosmic::Application 実装、init() によるセッション復元
│   │   ├── flags.rs         # AppFlags, normalize_path, parse_path_or_uri (CLI引数, file:// URI解析)
│   │   ├── message.rs       # Message 列挙型 (ActiveHeaderMenu, タブ, ファイルツリー, モーダル, AIチャット)
│   │   ├── keybindings.rs   # handle_key_event (キーボードショートカット、モーダル・メニューキーハンドリング)
│   │   ├── update.rs        # handle_update (非同期Task/Streamディスパッチ、メニュー開閉、ファイルCRUD、タブ同期)
│   │   ├── state.rs         # save_config, open_file, クリップボード, 行編集ヘルパー
│   │   └── ui/
│   │       ├── mod.rs       # view() ルートUIマウント (ヘッダーメニューオーバーレイ、モーダル、エディタ)
│   │       ├── header.rs    # render_header_start (5つの集約ボタン), render_header_menu_overlay (ドロップダウン)
│   │       ├── tab_bar.rs   # render_tab_bar (タブ切り替え、未保存●、閉じる、新規タブボタン)
│   │       ├── context_menu.rs# render_context_menu (右クリック浮動メニュー, 1600x1600クランプ対応)
│   │       ├── modal.rs     # render_active_modal (新規ファイル, 新規フォルダ, リネーム, 削除, Aestheticsモーダル)
│   │       ├── ai_chat.rs   # render_ai_chat_panel (ストリーミング描画、タイピング▋、Stop/Sendボタン、文脈添付)
│   │       └── search_bar.rs# render_search_bar (Ctrl+F ファイル内検索バー)
│   ├── editor/
│   │   ├── mod.rs           # resolve_numpad_char (Waylandテンキー物理キーコード解決ユーティリティ)
│   │   ├── buffer.rs        # TextBuffer (Ropey、単語移動、行削除/複製、コメントトグル、インデント、Undo/Redo)
│   │   └── pane.rs          # EditorPane & EditorTab (タブ管理、ファイル読込/保存、検索マッチ、言語判別)
│   ├── syntax/
│   │   ├── mod.rs
│   │   └── highlighter.rs   # Highlighter & classify_node (12+言語のTree-sitter/字句解析エンジン)
│   ├── markdown/
│   │   ├── mod.rs
│   │   └── renderer.rs      # MarkdownDocument (pulldown-cmark によるAST構築、GFM & CommonMark動的仕様切替、テーブル、タスクリスト、アラート、打ち消し線)
│   ├── fs/
│   │   ├── mod.rs
│   │   └── tree.rs          # FileTree (除外パターン、Nerd Font アイコン、ディレクトリ走査)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── canvas_editor.rs # EditorCanvas (intern_font_name、検索ハイライト、サブピクセル文字幅、IME)
│   │   ├── file_tree_view.rs# view_file_tree (サイドバーUI、/// アクションボタン、スクロール位置保持)
│   │   └── markdown_view.rs # view_markdown (リッチMarkdownプレビュー、水平スクロールテーブル、タスクリスト、GitHubアラート)
│   ├── theme/
│   │   ├── mod.rs
│   │   └── themes.rs        # 20種類の Classic & Neon テーマ定義、ウィンドウ・ツリー・ヘッダー独立アルファ透過・ディミング
│   ├── font/
│   │   └── mod.rs           # FontManager (fontconfig によるシステムNerd Font検出とサイズ管理)
│   └── ai/
│       ├── mod.rs           # ChatMessage, ChatRole, ChatStreamEvent 再エクスポート
│       └── ollama.rs        # OllamaClient (FIM補完、chat_generate_stream ストリーミング、モデル自動検出)
└── tests/
    ├── core_tests.rs        # 35件のユニットテスト (Markdown仕様比較、設定永続化、セッション復元、タブ同期、構文、検索等)
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
| ファイルツリー上マウス右クリック | ファイル/ディレクトリ/ルートのコンテキストメニュー表示（New File, New Folder, Rename, Delete, Refresh / 画面外見切れ防止クランプ） |
| ヘッダー `󰈔 File ▾` | ファイルメニュー展開（新規、開く、フォルダ、保存、閉じる） |
| ヘッダー `󰧑 Edit ▾` | 編集メニュー展開（Undo, Redo, Cut, Copy, Paste, Select All, コメント, 削除, 複製） |
| ヘッダー `󰈈 View ▾` | 表示メニュー展開（Split/Single、ファイルツリー、Markdownプレビュー） |
| ヘッダー `󰚩 AI ▾` | AIメニュー展開（チャットパネル、選択添付、ファイル添付、FIM補完） |
| ヘッダー `󰒓 Aesthetics` | 外観・仕様設定モーダル表示（テーマ、フォント、フォントサイズ、各部独立透過度、AIモデル、Markdown仕様: GFM/CommonMark） |
| チャット `󰓛 Stop` | AIコード生成の即時中断・ストリーミング停止 |
| テンキー `0`〜`9` / 記号 (`+`, `-`, `*`, `/`, `.`, `,`, `=`) | 数字および四則演算子記号の直接入力（英字/IME両モード完全対応） |
| テンキー `Enter` | 改行の挿入 / 新規ファイル作成モーダルの確定 |
| `Tab`（未選択時） | AI補完（ゴーストテキスト）の確定挿入 / インデント（4スペース） |
| `Esc` | 検索バー終了 / AI補完破棄 / モーダル終了 / コンテキストメニュー終了 / メニュー閉じる / 選択解除 |
| `Ctrl + B` | ファイルツリーサイドバーの表示/非表示トグル |
| `Ctrl + \` または `Ctrl + E` | 左右2分割（Split / Single）レイアウト切り替え |
| `Ctrl + M` | Markdownプレビューの切り替え（GFM / CommonMark動的仕様切替対応） |
| `Ctrl + I` または `Alt + Enter` | Local AI FIM 補完の手動トリガー（`Ctrl + Space` は IME 専用に解放） |

---

## 5. 現在のビルドおよびテスト状態

- `cargo clippy --all-targets -- -D warnings`: **0 errors, 0 warnings** (完全クリーン)
- `cargo test`: **51/51 全テスト通過 (1 benchmark + 47 core + 3 ollama, 0 failed)**
  - `test_multiscale_performance_benchmarks` ... ok (10KB〜50MBマルチスケール性能ベンチマーク、50MBタイピング遅延 28µs、Viewport < 0.45ms、Cache Hit < 2µs検証)
  - `test_mixed_script_coordinates_and_roundtrip` ... ok (新規追加: 和文約物・絵文字ZWJ・結合文字・タブ混在の座標単調増加およびpos_to_char_coordsラウンドトリップ完全一致検証)
  - `test_cosmic_text_glyph_layout` ... ok (cosmic-text実グリフメトリクス測定と全角ダッシュ――・CJK記号の累積位置0.00px完全一致検証)
  - `test_char_advance_ascii_and_cjk` ... ok

  - `test_treesitter_multibyte_and_emoji_highlight_coordinates` ... ok (日本語・絵文字混在環境でのTree-sitterトークン文字範囲厳密一致検証)
  - `test_byte_char_mapper_and_slice_by_char_indices` ... ok (ByteCharMapper & slice_by_char_indices 各種境界検証)
  - `test_multiline_block_comment_highlighting` ... ok (複数行ブロックコメントの行境界クリッピング・ハイライト検証)
  - `test_cli_flags_and_path_resolution` ... ok (file:// URI, %20, 日本語%E3%83%86..., 相対/絶対パス, 正規化検証)
  - `test_editor_pane_open_file_welcome_tab_reuse_and_non_existent` ... ok (Welcomeタブ再利用, 新規未実在ファイルタブ初期化検証)
  - `test_editor_auto_scroll_flag_and_coordinates` ... ok (needs_scroll_to_cursor フラグ動作とカーソル追従検証)
  - `test_scrollbar_proportions_and_thumb_mapping` ... ok (つまみプロポーショナル計算、位置マッピング、マージン自動スクロール計算、最小高さクランプ検証)
  - `test_undo_redo_stack_depth_and_performance` ... ok (VecDeque 100件FIFO上限 & LIFOアンドゥ検証)
  - `test_buffer_edge_cases` ... ok (空バッファおよび10万文字超長行編集・クランプ検証)
  - `test_atomic_config_save` ... ok (config.toml アトミック保存・一時ファイル検証)
  - `test_sensitive_file_ai_protection` ... ok (拡充: .npmrc, .pypirc, kubeconfig, .jks, .keystore, token, secret, .aws, .kube)
  - `test_markdown_spec_commonmark_vs_gfm` ... ok
  - `test_markdown_spec_config_persistence` ... ok
  - `test_context_menu_boundary_clamping` ... ok
  - `test_independent_opacity_settings` ... ok
  - `test_rooney_icon_integration` ... ok
  - `test_filename_sanitization_and_path_traversal_guards` ... ok
  - `test_atomic_save_and_file_size_limits` ... ok
  - `test_active_header_menu_and_actions` ... ok
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
  - `test_file_tree_context_menu_state_and_right_click` ... ok
  - `test_file_tree_width_and_item_gutter` ... ok
  - `test_ollama_connectivity_and_models` ... ok
  - `test_ollama_chat_streaming` ... ok
  - `test_ollama_fim_generation` ... ok
- `cargo build --release`: **Clean release build (Code 0), installed to ~/.local/bin/rooney**

---

## 6. 次回再開時の推奨作業・機能拡張案

1. **ファイルツリー内でのドラッグ＆ドロップ（DnD）移動**:
   - ファイルを別のフォルダへドラッグして移動するGUI操作。
2. **ミニマップ（Minimap）またはアウトライン表示**:
   - コード全体の縮小表示や、Tree-sitter関数一覧（Symbol Outline）のサイドパネル。
3. **Git 変更行ガターハイライト（Diff Gutter）**:
   - 変更・追加・削除行を行番号脇にカラーバー表示。

---

## 7. ドキュメント保守運用ルール（Documentation Maintenance Policy）

今後の開発において、以下の運用ルールを厳格に適用・維持する：

1. **日英バイリンガルの完全同期**:
   - 機能追加（New Features）、バグ修正（Bug Fixes）、セキュリティ修正（Security Fixes）、キーバインド変更等を実施した際は、**必ず `README.md`（英語）と `README.ja.md`（日本語）の両方を同様の構成で更新する**。
2. **Waddle 準拠スリム構成の維持**:
   - 単一ファイルへの長文同居や冗長化を避け、`💡 Highlights`, `🚀 Quick Start`, `⌨️ Keybindings`, `🔒 Security & Architecture`, `🤖 About This Project`, `📄 License` の明確なセクション構成と高密度なスリムレイアウトを維持する。
3. **開発履歴・技術メモの分離**:
   - 各セッションの実装履歴、テスト検証結果、低レイヤの設計経緯は本ファイル（`SESSION_HANDOVER.md`）に集約記録し、`README.md` / `README.ja.md` はエンドユーザーおよびコントリビューター向けのスッキリした見通しの良い状態を保つ。
