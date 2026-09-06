# Rooney 🚀
**Cosmic-Native Lightweight Code & Markdown Editor**

![Banner](./images/Rooney-banner.svg)

> [!NOTE]
> **🤖 AI Vibe Coding Project**  
> このプロジェクトは、ユーザーとAIアシスタントの協調（バイブコーディング / Vibe Coding）によってゼロから設計・実装されました。  
> *This project was iteratively designed, prototyped, and implemented from scratch through AI Vibe Coding in pair-programming collaboration with Antigravity / Gemini.*

[日本語](#日本語) | [English](#english)

---

<a name="日本語"></a>
## 日本語 🇯🇵

> **🤖 バイブコーディングによる開発 (About Vibe Coding)**  
> 本プロジェクト「Rooney (CosmicCode)」は、ユーザーの要求とフィードバックを基に、AIとの対話的なバイブコーディング（Vibe Coding）によって構築されました。Rust、libcosmic、Wayland ネイティブの低レイヤープロトコル（IME text-input、Tree-sitter構文解析、Ropeyバッファ、XDG Portalファイル操作等）の高度な組み合わせを、AIとの高速な反復試行を通じて実現しています。

Pop!_OS COSMIC DE（libcosmic / Wayland）ネイティブの超高速・軽量・多機能コード＆Markdownエディタです。
外部LSPや重厚な子プロセスのオーバーヘッドを一切排除し、インプロセス構文ハイライト（Tree-sitter）、Ropeyバッファ、日本語IME（Fcitx5 / IBus）完全対応、左右2分割レイアウト、Nerd Font対応リッチファイルツリー、XDG Portalネイティブファイル操作、そしてローカルOllamaによるFill-in-the-Middle（FIM）インラインAI補完を統合しています。

### 主な特徴 ✨

1. **左右2分割レイアウト（Dual-Pane Split View）**
   - ツールバーまたは `Ctrl + \` / `Ctrl + E` で、シングルペインと左右2分割ペインを瞬時に切り替え。
   - 左右ペインで別々のファイルを開いて独立してスクロール・編集可能。
   - アクティブペインの視覚的強調（アクセントカラー枠・タブ強調）、独立したカーソル表示。
   - 画面端でのソフト折り返し（Soft-wrap）およびペイン境界での厳密なクリッピングにより、文字のはみ出しを防止。

2. **日本語入力（IME）ネイティブ対応 & サブピクセル正確なカーソル位置**
   - Wayland text-input プロトコル（libcosmic InputMethod）による Fcitx5 / IBus 完全対応。
   - インライン変換（Preedit）のプレビュー表示および確定コミット処理。
   - `Ctrl + Space` は IME のオン/オフ切り替え専用に解放（AI補完ショートカットと競合しません）。
   - **サブピクセル高精度カーソル位置計算**: 等幅フォントの半角文字アドバンス（`font_size * 0.60`）と全角/CJK文字アドバンス（`font_size * 1.0`）の厳密な実寸幅を個別合算。日本語を何文字入力してもカーソル位置が前方にずれることなく、文字直下に寸分の狂いなく吸い付きます。

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
   - デフォルトウィンドウサイズ 2400x2400 の広々とした作業領域。

7. **Local AI FIM (Fill-in-the-Middle) 補完**
   - ローカルの Ollama（`http://localhost:11434`）と高速非同期連携。
   - `deepseek-coder-v2`, `gemma4-coder`, `qwen2.5-coder` 等の利用可能なモデルを自動検出。
   - 入力中の文脈に応じたゴーストテキストサジェスト。
   - `Tab` キーで即座に確定挿入、`Esc` で破棄。
   - `Ctrl + I` または `Alt + Enter` で手動生成トリガー。

8. **設定の自動永続化 (`~/.config/rooney/config.toml`)**
   - 選択したテーマ、フォント、フォントサイズ、Wayland透明度、背景ディミング、分割ペイン配置（Single/Split）、ファイルツリーの開閉状態、AI有効化/モデルを `~/.config/rooney/config.toml` に自動保存。
   - アプリを再起動しても直前の作業環境が完全復元されます。

9. **エディタ内編集メニュー & マウス操作（ドラッグ選択・右クリック・Wayland クリップボード）**
   - **ヘッダー編集メニュー**: ヘッダーの「`󰧑 Edit ▾`」ボタンから、元に戻す、やり直し、切り取り、コピー、貼り付け、全選択を素早く操作。
   - **右クリックコンテキストメニュー**: エディタ上で右クリックすると、カーソル位置にコンテキストメニュー（Copy, Cut, Paste, Select All, Undo, Redo）をポップアップ表示。不透明カード背景（`Container::Card`）、整列したショートカット表示、画面端クランプ、全画面バックドロップによるクリックキャンセルを完備。
   - **マウスドラッグ選択**: マウスで直感的にテキスト範囲を選択（視覚的ハイライト表示）。
   - **Wayland システムクリップボード連携**: `Ctrl + C`、`Ctrl + X`、`Ctrl + V`、右クリックメニュー、Editメニューすべてでシステムクリップボードとリアルタイム同期。

10. **テンキー（Numpad）の完全サポート & Wayland NumLock 互換性**
    - テンキーの数字（`0`〜`9`）、四則演算子（`+`, `-`, `*`, `/`, `.`, `,`, `=`）、および **Enter** を完全サポート。
    - Wayland / Pop!_OS COSMIC 環境において、クライアントに NumLock の有効状態が同期されない場合でも、物理キーコードを自動識別して直接英字入力時・IME使用時の両方で数字・記号を確実に入力。
    - 独立した専用カーソルキー（矢印、Delete、Home、End、PageUp、PageDown）との競合を完全に排除し、操作性を両立。

11. **多言語 Tree-sitter & 設定ファイル構文ハイライト（12+言語対応）**
    - 外部LSP不要で、インプロセスAST構文解析による高精度・高速ハイライトを実現。
    - **対応言語**:
      - **システム / プログラミング言語**: Rust (`.rs`), Python (`.py`/`.pyi`), JavaScript (`.js`/`.jsx`/`.mjs`/`.cjs`), TypeScript (`.ts`/`.tsx`/`.mts`/`.cts`), C (`.c`/`.h`), C++ (`.cpp`/`.hpp`/`.cc`/`.cxx`)
      - **シェルスクリプト**: Bash / POSIX Shell (`.sh`/`.bash`/`.zsh`), Fish (`.fish`, `config.fish`)
      - **設定ファイル & データ形式**: TOML (`.toml`, `Cargo.lock`), YAML (`.yaml`/`.yml`), JSON (`.json`/`.jsonc`), INI / Conf (`.ini`/`.conf`/`.cfg`/`.gitconfig`)
      - **マークダウン**: Markdown (`.md`/`.markdown`)
    - ファイルツリーにも各言語ごとのカラー Nerd Font アイコン（Fish `󰈺`, YAML ``, INI ``, TSX `` 等）を自動表示。

12. **インクリメンタルファイル内検索 (`Ctrl + F`) & 画面ハイライト**
    - `Ctrl + F` でエディタ上部にミニマルで洗練された検索バーを展開。
    - 入力と同時にファイル内の一致箇所をリアルタイム走査し、エディタ内で黄色ハイライト表示。
    - `Enter` / `▼` で次の一致へ、`Shift + Enter` / `▲` で前の一致へスムーズにカーソルジャンプ。
13. **高度なカーソル移動 & 行編集ショートカット**
    - **単語移動**: `Ctrl + Left` / `Ctrl + Right` で単語区切り単位ジャンプ（`Shift`併用で単語単位選択）。
    - **行コメントのトグル**: `Ctrl + /` で選択行または現在行を言語に応じたコメント記号（`//` または `#`）で一括トグル。
    - **行削除**: `Ctrl + Shift + K` でカーソル行を丸ごと削除。
    - **行複製**: `Ctrl + D` でカーソル行を下行に複製。
    - **一括インデント**: 複数行選択時の `Tab` で一括インデント（4スペース）、`Shift + Tab` で一括アンインデント。

14. **タブ形式のマルチバッファ管理（Multi-Tab Buffer Management）**
    - 左右ペインそれぞれで独立した複数タブの切り替え・管理に対応。
    - 未保存バッファの視覚的インジケータ（`●`）、アクティブタブのハイライト表示。
    - `Ctrl + T` で新規タブ作成、`Ctrl + W` でアクティブタブを閉じる、`Ctrl + Tab` / `Ctrl + Shift + Tab` でスムーズに巡回。
    - 全タブを閉じても自動的にクリーンな Untitled タブへフォールバックし、クラッシュを完全防止。

15. **ファイルツリーの右クリックコンテキストメニュー & ディレクトリ管理**
    - **右クリックコンテキストメニュー**: ファイル、ディレクトリ、またはツリーの余白/ルートを右クリックすると、カーソル位置に専用のコンテキストメニューがポップアップ（New File, New Folder, Rename, Delete, Refresh）。
    - **スクロールバー干渉を根絶したガターマージン設計**: アクションボタン（``, ``, ``）とスクロールバー・左右ペイン境界の間に十分な余白（16px）を確保。アイコンが重なって隠れる問題を解消し、スムーズで快適なクリック操作を実現。
    - **新規作成（`` / ``）**: 選択ディレクトリ直下にファイルまたはフォルダをモーダルダイアログから作成。
    - **リネーム（``）**: ファイルやディレクトリの名前を変更。開いているタブのパス・タブ名もリアルタイムに自動同期。
    - **安全な削除（``）**: 誤操作を防ぐ確認モーダルを表示し、確認後にファイルまたはディレクトリ（再帰的）を完全削除。開いていた該当タブも安全に自動クローズ。

16. **AI チャット / 複数行コード生成パネル（`Ctrl + Shift + A` / `󰭹 Chat`）**
    - ヘッダーの `󰭹 Chat` ボタンまたは `Ctrl + Shift + A` で展開・格納できる専用の右側AIアシスタントパネル。
    - **リアルタイム・トークンストリーミング & タイピングアニメーション**:
      - Ollama の `stream: true`（ndjson / SSE）を非同期ストリーミング受信し、1トークンずつリアルタイムに流れるように描画。
      - 生成中は文末にアニメーションタイピングカーソル（`▋`）および思考状態（`󰚩 Thinking...`）を表示。
      - いつでも生成を瞬時に中断できる `󰓛 Stop` ボタン（または生成中の Enter 打鍵）を搭載。
    - **ワンクリック文脈添付**:
      - `󰈔 Attach Selection`: エディタ上で選択しているコード行をMarkdownコードブロック形式でチャット入力欄に即座に添付。
      - `󰈙 Attach File`: 現在アクティブなファイル全体のコードを自動添付。
    - **AIレスポンスのワンクリック挿入 & コピー**:
      - `󰈔 Insert`: 生成されたコードブロックをエディタのアクティブカーソル位置へ直接挿入。
      - `󰆏 Copy`: 生成されたコードをクリップボードにコピー。

17. **ワークスペースセッションの自動永続化**
    - 前回開いていた左右ペインの全タブ一覧、アクティブタブ番号、カーソル行・列位置、ルートフォルダ、およびAIチャットパネルの開閉状態を `~/.config/rooney/config.toml` に自動記録。
    - 次回エディタ起動時に、前回の作業状態が寸分違わずそのまま自動復元されます。

---

### キーボードショートカット & マウス操作 ⌨️

| 操作 / ショートカット | 動作 |
|---|---|
| `Ctrl + N` | 新規ファイル作成（ファイル名入力ダイアログ） |
| `Ctrl + O` | ファイルを開く（システムファイルダイアログ） |
| `Ctrl + Shift + O` | フォルダを開く（ツリールート変更） |
| `Ctrl + S` | ファイル保存（未命名バッファは名前を付けて保存） |
| `Ctrl + Shift + S` | 名前を付けて保存（Save As） |
| `Ctrl + T` | 新規タブ作成 |
| `Ctrl + W` | アクティブタブを閉じる |
| `Ctrl + Tab` / `Ctrl + PageDown` | 次のタブへ切り替え |
| `Ctrl + Shift + Tab` / `Ctrl + PageUp` | 前のタブへ切り替え |
| `Ctrl + F` | ファイル内検索バーの表示 / 非表示 |
| `Enter` / `Shift + Enter`（検索時） | 次の一致 / 前の一致へジャンプ |
| `Ctrl + /` | 行コメントのトグル（言語別自動判定） |
| `Ctrl + Shift + K` | 現在行の丸ごと削除 |
| `Ctrl + D` | 現在行の複製 |
| `Ctrl + Left` / `Ctrl + Right` | 単語単位のカーソル移動（+Shiftで単語選択） |
| `Tab` / `Shift + Tab`（選択時） | 選択範囲の一括インデント / アンインデント |
| `Ctrl + C` | 選択テキストのコピー（システムクリップボード） |
| `Ctrl + X` | 選択テキストの切り取り（システムクリップボード） |
| `Ctrl + V` | 貼り付け（システムクリップボードから挿入） |
| `Ctrl + A` | バッファ全選択 |
| `Ctrl + Shift + A` | AI チャットパネルの開閉トグル |
| `Ctrl + Z` | 元に戻す (Undo) |
| `Ctrl + Y` または `Ctrl + Shift + Z` | やり直す (Redo) |
| マウス左ドラッグ | テキスト範囲選択（ビジュアルハイライト） |
| エディタ上マウス右クリック | エディタコンテキストメニュー表示（Copy, Cut, Paste, Select All, Undo, Redo） |
| ファイルツリー上マウス右クリック | ファイルツリーコンテキストメニュー表示（New File, New Folder, Rename, Delete, Refresh） |
| ヘッダー `󰧑 Edit` | 編集ツールバーの表示/非表示切り替え |
| ヘッダー `󰭹 Chat` | AIチャットパネルの表示/非表示切り替え |
| チャット `󰓛 Stop` | AIコード生成の即時中断・ストリーミング停止 |
| テンキー `0`〜`9` / 記号 (`+`, `-`, `*`, `/`, `.`, `,`, `=`) | 数字および演算子記号の直接入力（英字/IME両モード完全対応） |
| テンキー `Enter` | 改行の挿入 / 新規ファイル作成モーダルの確定 |
| `Tab`（未選択時） | AI補完の確定挿入 / 4スペース挿入 |
| `Esc` | 検索バー終了 / AI補完破棄 / モーダル終了 / 選択解除 / メニューキャンセル |
| `Ctrl + B` | ファイルツリーサイドバーの表示/非表示 |
| `Ctrl + \` または `Ctrl + E` | 左右2分割（Split / Single）レイアウト切り替え |
| `Ctrl + M` | Markdownプレビューの切り替え |
| `Ctrl + I` または `Alt + Enter` | Local AI FIM 補完の手動トリガー（`Ctrl + Space` は IME 専用に解放） |
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

# テスト実行 (27テスト)
cargo test
```

---

<a name="english"></a>
## English 🇬🇧 🇺🇸

> **🤖 Built via AI Vibe Coding**  
> "Rooney (CosmicCode)" was designed, architected, and continuously iterated through AI Vibe Coding. Complex low-level systems programming in Rust—including libcosmic/Wayland desktop protocols, custom IME text-input methods, Tree-sitter in-process syntax parsing, Ropey buffers, and XDG Portal async dialogs—were built in rapid pair-programming collaboration with AI.

A blazing-fast, lightweight, feature-rich code & markdown editor built natively for Pop!_OS COSMIC DE (libcosmic / Wayland).
Zero LSP overhead and zero heavy child processes: features in-process Tree-sitter syntax highlighting, a high-performance Ropey text buffer, native Japanese IME (Fcitx5 / IBus) integration, side-by-side dual-pane split editing, Nerd Font file tree, native XDG Desktop Portal file dialogs, and local Ollama-powered Fill-in-the-Middle (FIM) inline AI completions.

### Key Features ✨

1. **Dual-Pane Split View**
   - Toggle instantly between Single Pane and Side-by-Side Split View via the toolbar or `Ctrl + \` / `Ctrl + E`.
   - Edit different files simultaneously in left and right panes with independent scrolling, cursors, and syntax highlighters.
   - Clear visual focus indication with accent borders and active tab highlights.
   - Clean soft-wrapping and bounded layer clipping to prevent text from overflowing across panes.

2. **Native Japanese IME Support & Sub-Pixel Precise Cursor Alignment**
   - Full support for Wayland text-input protocol (libcosmic InputMethod) supporting Fcitx5 and IBus.
   - Live pre-edit underline preview and commit event handling.
   - `Ctrl + Space` is unmapped from editor shortcuts and dedicated strictly to system IME toggle.
   - **Sub-Pixel Exact Cursor Calculations**: Calculates advance dynamically per character: half-width monospace glyphs (`font_size * 0.60`) vs. full-width CJK glyphs (`font_size * 1.0`). Eliminates the common caret drift bug where the cursor shifted 1–2 characters ahead during CJK typing.

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
   - Generous default window size (2400x2400) optimized for modern high-resolution displays.

7. **Local AI FIM (Fill-in-the-Middle) Code Completion**
   - Direct asynchronous communication with local Ollama (`http://localhost:11434`).
   - Automatically detects installed coding models (e.g., `deepseek-coder-v2`, `gemma4-coder`, `qwen2.5-coder`).
   - Context-aware inline ghost text suggestions.
   - Press `Tab` to accept, `Esc` to dismiss.
   - Trigger suggestions manually with `Ctrl + I` or `Alt + Enter`.

8. **Automatic Configuration Persistence (`~/.config/rooney/config.toml`)**
   - Theme, font family, font size, Wayland opacity, dimming overlay, dual-pane layout, sidebar visibility, and local AI model settings are automatically persisted to `~/.config/rooney/config.toml`.
   - Your entire workspace environment is seamlessly restored when relaunching the editor.

9. **Edit Menu & Mouse Clipboard Integration (Selection, Context Menu, Wayland Clipboard)**
   - **Header Edit Menu**: Quick-access dropdown toolbar for Undo, Redo, Cut, Copy, Paste, and Select All via the header `󰧑 Edit ▾` button.
   - **Right-Click Context Menu**: Right-click anywhere in the editor to bring up a floating context menu at the mouse cursor. Features opaque card styling (`Container::Card`), aligned icons, labels, and shortcuts, edge-of-screen clamping, and full-screen dismissal on backdrop click.
   - **Mouse Drag Selection**: Intuitive click-and-drag visual text selection with theme-matched highlighting.
   - **Wayland System Clipboard**: Full bi-directional integration with the system clipboard via `Ctrl + C`, `Ctrl + X`, `Ctrl + V`, right-click menu, and the header Edit menu.

10. **Full Numeric Keypad (Numpad) Support & Wayland NumLock Compatibility**
    - Complete support for numpad digits (`0`–`9`), operators (`+`, `-`, `*`, `/`, `.`, `,`, `=`), and **Numpad Enter**.
    - Solves Wayland/XKB NumLock state desynchronization: intelligently maps physical keycodes so numeric keypad input works seamlessly across both direct English input and IME modes.
    - Dedicated navigation keys (arrows, Delete, Home, End, PageUp, PageDown) remain 100% functional and unhindered.

11. **Multi-Language Tree-sitter & Configuration Syntax Highlighting (12+ Languages)**
    - Zero-LSP in-process AST syntax parsing for high speed and pinpoint highlighting accuracy.
    - **Supported Languages**:
      - **Systems & Programming**: Rust (`.rs`), Python (`.py`/`.pyi`), JavaScript (`.js`/`.jsx`/`.mjs`/`.cjs`), TypeScript (`.ts`/`.tsx`/`.mts`/`.cts`), C (`.c`/`.h`), C++ (`.cpp`/`.hpp`/`.cc`/`.cxx`)
      - **Shell Scripts**: Bash / POSIX Shell (`.sh`/`.bash`/`.zsh`), Fish (`.fish`, `config.fish`)
      - **Configs & Formats**: TOML (`.toml`, `Cargo.lock`), YAML (`.yaml`/`.yml`), JSON (`.json`/`.jsonc`), INI / Conf (`.ini`/`.conf`/`.cfg`/`.gitconfig`)
      - **Markdown**: Markdown (`.md`/`.markdown`)
    - Rich colored Nerd Font file icons in the file tree for each file type (Fish `󰈺`, YAML ``, INI ``, TSX ``, etc.).

12. **Incremental In-File Search (`Ctrl + F`) & Canvas Highlights**
    - Press `Ctrl + F` to reveal a floating, minimalist in-pane search bar.
    - Real-time full-buffer search with instant canvas match highlights.
    - Jump between matches with `Enter` / `▼` (Next) and `Shift + Enter` / `▲` (Previous).
    - Match count indicator (e.g. `3/12`), close with `Esc` / `✕` to return to editing.

13. **Advanced Cursor Navigation & Line Editing Shortcuts**
    - **Word Navigation**: `Ctrl + Left` / `Ctrl + Right` jump word-by-word (`Shift` modifier for word selection).
    - **Comment Toggle**: `Ctrl + /` toggles line comments using the appropriate syntax (`//` or `#`).
    - **Line Deletion**: `Ctrl + Shift + K` deletes the current line entirely.
    - **Line Duplication**: `Ctrl + D` duplicates the current line below.
    - **Block Indent**: Select multiple lines and press `Tab` to indent (4 spaces) or `Shift + Tab` to unindent.

14. **Tabbed Multi-Buffer Management & Tab Bar**
    - Open multiple files per pane (both Single and Dual-Pane Split modes) with fluid tab switching.
    - Features colored file icons, filename, unsaved dirty indicator (`●`), individual close buttons (`✕`), and add tab button (`＋`).
    - Smart tab switching: opening an already-opened file from the file tree or chooser focuses its existing tab without duplication.
    - Full keyboard navigation: `Ctrl + T` (New Tab), `Ctrl + W` (Close Tab), `Ctrl + Tab` / `Ctrl + PageDown` (Next Tab), `Ctrl + Shift + Tab` / `Ctrl + PageUp` (Previous Tab).

15. **File Tree Right-Click Context Menu & Directory Management**
    - **Right-Click Context Menu**: Right-click any file, folder, or empty background in the file tree to open a native floating context menu (New File, New Folder, Rename, Delete, Refresh).
    - **Scrollbar Gutter Margin Design**: Dedicated 16px right gutter space between row action buttons (``, ``, ``) and the vertical scrollbar / pane boundary, ensuring zero overlap and smooth clicking.
    - **Create New (`` / ``)**: Create files or folders directly in the selected directory via centered modal dialog.
    - **Rename (``)**: Rename files or directories on disk; all open tabs referencing the file or children within the directory automatically synchronize their file paths and tab titles in real time.
    - **Safe Deletion (``)**: Confirm removal with a modal dialog to prevent accidental deletion, then remove files or directories recursively. Automatically and safely closes any active tabs referencing deleted files.

16. **AI Chat & Code Generation Panel (`Ctrl + Shift + A` / `󰭹 Chat`)**
    - Collapsible dedicated AI assistant panel on the right side of the editor.
    - **Real-Time Token Streaming & Typing Animation**:
      - Asynchronous token streaming via Ollama's `stream: true` (ndjson / SSE), rendering generated responses line-by-line in real time.
      - Features an animated typing cursor (`▋`) and thinking status indicator (`󰚩 Thinking...`).
      - Instant generation cancellation via the dynamic `󰓛 Stop` button or by hitting `Enter` during streaming.
    - **Context Attachment**:
      - `󰈔 Attach Selection`: Attaches the currently selected code in the editor into the chat input as a markdown code block.
      - `󰈙 Attach File`: Attaches the full text of the active file into the chat prompt.
    - **Code Insertion & Clipboard Copy**:
      - `󰈔 Insert`: Inserts the generated code block directly at the current cursor position in the active editor buffer.
      - `󰆏 Copy`: Copies the generated code to the system clipboard.

17. **Automatic Workspace Session Persistence**
    - Automatically serializes and saves open tabs in both left and right panes, active tab indices, cursor positions (line and column), workspace root folder, and AI chat panel visibility to `~/.config/rooney/config.toml`.
    - Seamlessly restores your entire editing session on application relaunch.

---

### Keyboard Shortcuts & Mouse Operations ⌨️

| Action / Shortcut | Description |
|---|---|
| `Ctrl + T` | Open new blank tab |
| `Ctrl + W` | Close active tab |
| `Ctrl + Tab` / `Ctrl + PageDown` | Switch to next tab |
| `Ctrl + Shift + Tab` / `Ctrl + PageUp` | Switch to previous tab |
| `Ctrl + N` | Create New File (opens filename prompt dialog) |
| `Ctrl + O` | Open File (native system file chooser) |
| `Ctrl + Shift + O` | Open Folder (switch file tree root) |
| `Ctrl + S` | Save File (prompts Save As if buffer is untitled) |
| `Ctrl + Shift + S` | Save File As |
| `Ctrl + F` | Toggle Find in File search bar |
| `Enter` / `Shift + Enter` (in search) | Jump to next / previous search match |
| `Ctrl + /` | Toggle line comment (auto-detected syntax) |
| `Ctrl + Shift + K` | Delete current line |
| `Ctrl + D` | Duplicate current line |
| `Ctrl + Left` / `Ctrl + Right` | Word-by-word cursor jump (+Shift to select) |
| `Tab` / `Shift + Tab` (with selection) | Multi-line block indent / unindent (4 spaces) |
| `Ctrl + C` | Copy selected text to system clipboard |
| `Ctrl + X` | Cut selected text to system clipboard |
| `Ctrl + V` | Paste from system clipboard |
| `Ctrl + A` | Select All in active buffer |
| `Ctrl + Shift + A` | Toggle AI Chat Panel |
| `Ctrl + Z` | Undo |
| `Ctrl + Y` or `Ctrl + Shift + Z` | Redo |
| Mouse Left Drag | Drag-select text with visual highlight |
| Editor Right Click | Open editor context menu (Copy, Cut, Paste, Select All, Undo, Redo) |
| File Tree Right Click | Open file tree context menu (New File, New Folder, Rename, Delete, Refresh) |
| Header `󰧑 Edit` | Toggle Edit toolbar visibility |
| Header `󰭹 Chat` | Toggle AI Chat panel visibility |
| Chat `󰓛 Stop` | Stop/cancel active AI generation |
| Numpad `0`–`9` / Operators (`+`, `-`, `*`, `/`, `.`, `,`, `=`) | Direct numeric and operator entry (both English and IME modes) |
| Numpad `Enter` | Insert newline / Confirm modal dialog |
| `Tab` (no selection) | Accept AI suggestion / Insert 4 spaces |
| `Esc` | Close search / Dismiss AI suggestion / Close modals / Clear selection / Cancel dialogs |
| `Ctrl + B` | Toggle File Tree sidebar visibility |
| `Ctrl + \` or `Ctrl + E` | Toggle Dual-Pane Split / Single layout |
| `Ctrl + M` | Toggle Markdown Live Preview |
| `Ctrl + I` or `Alt + Enter` | Manually trigger Local AI FIM (`Ctrl + Space` reserved for IME) |
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

# Run unit tests (27 tests)
cargo test
```

---

### Architecture & Tech Stack 🏗️

- **GUI & Windowing**: [libcosmic](https://github.com/pop-os/libcosmic), `cosmic-text`, `iced`
- **Configuration & Persistence**: `serde`, `toml`, `directories` (XDG Base Directory standard)
- **File Dialogs**: `rfd` (XDG Desktop Portal, Wayland native)
- **Text Buffer**: [Ropey](https://github.com/cessen/ropey) (Piece-tree rope buffer for handling large files)
- **Syntax Highlighting**: [tree-sitter](https://tree-sitter.github.io/tree-sitter/) (in-process tree parsing)
- **Markdown**: `pulldown-cmark`
- **AI Integration**: `reqwest`, `tokio` (local Ollama FIM endpoint)
- **Character Metrics**: `unicode-width` (CJK full-width display handling)
