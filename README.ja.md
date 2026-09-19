<div align="center">

# 🚀 Rooney
### COSMIC / Wayland 向け AI 統合型次世代 Linux コード＆Markdown エディタ

![Banner](./images/Rooney-banner.svg)

[![Built with libcosmic](https://img.shields.io/badge/libcosmic-Pop!_OS_COSMIC-24C8D8?style=for-the-badge&logo=linux&logoColor=white)](https://github.com/pop-os/libcosmic)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Wayland](https://img.shields.io/badge/Wayland-Native-5277C3?style=for-the-badge&logo=wayland&logoColor=white)](https://wayland.freedesktop.org/)
[![Ollama](https://img.shields.io/badge/Ollama-Local_AI-white?style=for-the-badge&logo=ollama&logoColor=black)](https://ollama.com/)
[![Platform](https://img.shields.io/badge/Platform-Linux_(COSMIC_/_Wayland)-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://www.kernel.org/)
[![Vibe Coding](https://img.shields.io/badge/Built_with-AI_Vibe_Coding-8A2BE2?style=for-the-badge&logo=sparkles&logoColor=white)](#-このプロジェクトについて-ai-vibe-coding)
[![License: MIT](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

<p align="center">
  <strong>COSMIC / Wayland ネイティブ × ピースツリー Rope バッファ × Tree-sitter 構文解析 × 日本語 IME 完全対応 × 100% 完全ローカル AI (Ollama)</strong><br>
  外部クラウドにデータを一切送信しない、Pop!_OS COSMIC Desktop および Linux Wayland 向けの高速・軽量・高機能エディタ
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.ja.md">日本語</a>
</p>

</div>

---

## 💡 主な機能

- ⚡ **COSMIC & Wayland ネイティブコア**: 外部 LSP や重厚な子プロセスに依存しない軽量・高速起動。純粋な Rust `libcosmic` デスクトップ統合、専用マットアプリアイコン、クライアント装飾、ダーク/ライトテーマ自動同期を実現。
- 🏎️ **ビューポート仮想描画 & 折り返し行累積 Y モデル**: 画面内の可視行（+ 上下マージン数行）のみにレイアウト計算を限定。50MB / 185万行の巨大ファイルでもサブミリ秒（< 0.40 ms）でレイアウトを完了し、144+ FPS の超軽快な描画・スクロールを提供。疎な $O(\log W)$ 累積折り返しモデル（`LineWrapModel`）により、複数行に折り返された行の直下に後続行が重複せず厳密配置され、正確な Y 座標計算とサブピクセル精度のマウスヒットテストを実現。
- 🌳 **統一世代管理アーキテクチャ（Tree-sitter / Markdown / 検索パイプライン）**: 巨大ファイル（> 2MB〜50MB）の重い Tree-sitter AST 解析、Markdown プレビューパース（表示切替・本文編集・仕様切替時、非アクティブタブを含む全タブ）、および検索走査をワーカースレッド（`tokio::task::spawn_blocking`）へ完全移譲し、UI スレッドの描画フリーズ（144+ FPS）を完全に排除。すべての非同期パイプラインが決定論的な整数世代カウンター（`parse_generation`、`markdown_generation`、`search_generation`）で保護され、高速タイピング、ファイル切替、名前を付けて保存（言語変更）、Undo/Redo が競合しても古いスナップショットの解析結果をタイムスタンプ依存なく安全に破棄（Stale Protection 100%）。検索実行中であってもキーストロークレイテンシをリリースビルドで **約 3.9 µs**（デバッグ時でも < 0.1 ms）に維持。
- 🤖 **100% 完全ローカル AI**: Ollama を用いた完全オフライン動作。リアルタイムの Fill-in-the-Middle（FIM）インラインゴースト補完（`Ctrl + I` / `Alt + Enter`）と、トークン単位のストリーミング対話 AI チャットパネル（`Ctrl + Shift + A`）を搭載。
- 📜 **ピースツリー Rope バッファ**: `ropey` を採用し、メモリの連続再配置を抑えた効率的な文字・行編集を提供。最大行長追跡により折り返し判定を $O(1)$ 化。
- 🪟 **2分割ペイン編集 & リアルタイム Markdown プレビュー**: ツールバーやショートカット（`Ctrl + \`）で即座に左右分割。独立マルチタブ、同期編集、設定画面から **GFM**（GitHub Flavored Markdown: テーブル、タスクリスト、アラート、打ち消し線、自動リンク、`breaks: false`）と標準 **CommonMark** を動的に切り替え可能なライブ Markdown プレビュー（`Ctrl + M`）。
- 🧭 **快適な自動追従スクロール & 独立した右端スクロールバー**: カーソルが画面上端・下端に達した際に視界外へ隠れないようマージンを保って追従。2分割時も含め各ペイン右端に独立したスクロールバー（プロポーショナルつまみ、ドラッグ移動、トラッククリックジャンプ）を完備し、$O(1)$ のコンテンツ高計算で瞬時に応答。
- 🇯🇵 **Unicode Grapheme 単位のキャレット整合性 & 実グリフメトリクス連携**: Wayland text-input プロトコル（Fcitx5 / Mozc / IBus）をネイティブサポート。`unicode-segmentation` による書記素クラスタ境界認識により、結合文字（`é` 等）や絵文字 ZWJ シーケンス（`👨‍💻` 等）の不可分なキャレット移動・削除・選択を保証。さらに `cosmic-text` 実グリフレイアウト計測キャッシュ（`RwLock` 共有並行化・Tab / CJK 高速パス・`clear_glyph_cache`、U+2015 ダッシュ等の Ambiguous/CJK 完全一致）により、表示とカーソルのズレを根本解消。
- 📂 **リッチなファイルツリーエクスプローラー**: ワークスペースの走査、Nerd Font ファイルアイコン、右クリックコンテキストメニュー（新規ファイル、新規フォルダ、リネーム、安全削除）でのスクロール位置保持・ウィンドウ下限はみ出し防止クランプ、XDG Desktop Portal ネイティブファイルダイアログ連携。
- 🎨 **全20種の洗練された Classic & Neon テーマ**: Tokyo Night、Catppuccin、Gruvbox、Synthwave '84、Cyberpunk Neon などを網羅。エディタウィンドウ、ファイルツリー、タイトルバーの各透明度を独立調整でき、背景ディミングとともに `󰒓 Aesthetics & Preferences` モーダルで一元設定可能。
- 🔒 **安全性とファイル整合性**: 一時ファイル置換（Atomic Write）による書き込み破損リスクの低減、ファイル名パターン判定による機密ファイル除外ガード、50MB ファイルサイズ上限、パストラバーサル防止。

---

## 🚀 クイックスタート

### 1. 必要環境

- [Rust (Cargo)](https://rustup.rs/) (1.80 以上)
- Linux Wayland 環境 / Pop!_OS COSMIC Desktop
- システムビルド依存パッケージ (Debian / Pop!_OS / Ubuntu):
  ```bash
  sudo apt install build-essential libxkbcommon-dev libfontconfig1-dev
  ```
- [Ollama](https://ollama.com/) (ローカル AI 機能利用時)

### 2. Ollama のセットアップ

```bash
# Ollama デーモンを起動
ollama serve

# 推奨コーディングモデルをダウンロード
ollama pull qwen2.5-coder
# または汎用アシスタントモデル
ollama pull llama3.2
```

### 3. 開発モードでの起動

```bash
# リポジトリのクローン
git clone https://github.com/wammed/Rooney.git
cd Rooney

# 開発モードで実行
cargo run
```

### 4. プロダクションビルド & インストール

```bash
# 最適化リリースバイナリをビルドしてユーザーパスへ配置
cargo build --release
install -m 755 target/release/rooney ~/.local/bin/rooney
```
*スタンドアロン実行ファイル出力先: `~/.local/bin/rooney`。*

### 5. CLI 起動 & デスクトップファイル関連付け

Rooney はコマンドライン引数、デスクトップファイルマネージャ（COSMIC Files 等）のダブルクリック、および `.desktop` ファイル（`Exec=rooney %f`）からの直接オープンに完全対応しています：
```bash
# 指定ファイルを開いて起動
rooney document.txt /path/to/script.rs

# file:// URI 形式（ファイルマネージャからの起動）
rooney file:///home/user/notes.md

# 指定ディレクトリをファイルツリーで開く
rooney /path/to/project
```

### 6. 設定の永続化

Rooney は、選択したテーマ、フォント、フォントサイズ、各部独立透明度、ディミング、ローカル AI モデル、Markdown 仕様（GFM / CommonMark）、分割レイアウト、開いているタブのセッション状態を `~/.config/rooney/config.toml` に自動保存します。再起動時にも直前の作業状態がそのまま復元されます。

---

## ⌨️ 主なショートカットキー

| ショートカット | 動作 |
| :--- | :--- |
| `Ctrl + N` | **新規ファイル作成**（ファイル名入力モーダル） |
| `Ctrl + O` | **ファイルを開く**（XDG Desktop Portal ネイティブダイアログ） |
| `Ctrl + Shift + O` | **フォルダを開く**（ツリーのワークスペース変更） |
| `Ctrl + S` | **ファイルを保存**（アトミック安全保存） |
| `Ctrl + Shift + S` | **名前を付けて保存...** |
| `Ctrl + T` | **新規タブ作成** |
| `Ctrl + W` | **アクティブタブを閉じる** |
| `Ctrl + Tab` / `Ctrl + Shift + Tab` | 次のタブ / 前のタブへ切り替え |
| `Ctrl + \` または `Ctrl + E` | **左右2分割ペイン**の切り替え（Split / Single） |
| `Ctrl + M` | **Markdown プレビュー**の切り替え |
| `Ctrl + B` | **ファイルツリーサイドバー**の表示 / 非表示 |
| `Ctrl + F` | **ファイル内インクリメンタル検索バー**の表示 / 非表示 |
| `Ctrl + Shift + A` | **AI チャットパネル**の開閉トグル |
| `Ctrl + I` または `Alt + Enter` | **Local AI FIM インライン補完**の手動トリガー |
| `Ctrl + /` | **行コメントのトグル**（言語別自動判定） |
| `Ctrl + Shift + K` | **現在行の丸ごと削除** |
| `Ctrl + D` | **現在行の複製** |
| `Ctrl + ,` | **Aesthetics 設定**を開く（テーマ、フォント、各部透明度、AIモデル、Markdown仕様: GFM/CommonMark） |
| `Esc` | 開いているモーダル・検索バー・メニューを閉じる / 補完キャンセル |

---

## 🔒 セキュリティ & 堅牢性 (概要)

Rooney は **100% 完全オフライン・ローカルファースト** で動作します：
- **ゼロクラウド流出**: テレメトリ、外部クラウド API、トラッキングは一切ありません。プロンプトもコードストリームもすべて PC 内の Ollama で安全に処理されます。
- **パターン判定による機密ファイル除外ガード**: `.env*`、秘密鍵・クレデンシャル（`id_rsa`、`id_ed25519`、`credentials`、`*.pem`、`*.key`、`*.keystore`、`*.jks`、`.npmrc`、`.pypirc`、`kubeconfig`、`*.token`、`*secret*`、`.aws/credentials`、`.kube/config`）の編集中は、AI 自動補完リクエストの送信対象から除外し、偶発的な機密流出を防止。
- **一時ファイル置換によるアトミック保存（Atomic Safe Write）**: エディタ編集ファイルおよび `config.toml` を同一階層の一時ファイル（`.{filename}.tmp.{pid}`）への書き込み・ディスク同期（`sync_all`）・置換（`rename`）により保存し、OS クラッシュや電源断時の中断書き込みリスクを低減。
- **パストラバーサル & 境界保護**: ファイル・ディレクトリ作成やリネーム時のディレクトリ脱出（`..`、`/`）をサニタイズ。ルート（`/`）やホームディレクトリの誤った再帰削除をセーフガードで防止。システムアイコン配置時のホームディレクトリは `directories::BaseDirs` で安全に解決。
- **リソース保護 & 低フットプリント**: 50MB を超える巨大ファイルの誤オープン防止、1MB の AI ストリーミング受信バッファ制限、有界 $O(1)$ リングバッファ Undo/Redo スタック（`VecDeque`）、毎フレームの不要な一時アロケーションを抑えたイテレータ直接描画。

---

## 🗺️ ロードマップ

今後のマイルストーンで予定されている最適化および機能拡張項目です：

- [x] 🏎️ **ビューポート仮想スクロール（Viewport-Based Virtualization）**: 画面内の可視行のみにレイアウト計算を限定し、10KB〜50MBの巨大ファイルでもサブミリ秒（< 0.45 ms）でレイアウトを完了。
- [x] ⚡ **Tree-sitter 増分ハイライト・行キャッシュ**: `InputEdit` による構文木差分解析と行単位のトークンキャッシュにより、文字入力時の再解析・再トークナイズ負荷を極小化。
- [x] 📊 **マルチスケール性能ベンチマークスイート**: 10KB〜50MBのファイルサイズ別ベンチマークテスト（バッファ読み込み・初期構文解析・差分編集・ビューポートレイアウト・キャッシュ参照）を完備。
- [x] 📐 **実グリフメトリクス（`cosmic-text`）連携**: 近似計算ではなく、`cosmic-text` が算出する実際のグリフレイアウト幅を直接参照・キャッシュし、全角ダッシュ（`――`）や Ambiguous/CJK 記号のカーソル位置ズレを完全解消。
- 👁️ **ファイルシステム変更監視（File Watcher）**: ワークスペースディレクトリの変更イベントを非同期監視し、外部ツールによるファイル更新やツリー構造の自動同期を実現。
- 📝 **リッチなインライン Markdown AST 描画**: エディタキャンバス内で太字・斜体・インラインコード・ハイパーリンクなどの装飾をインライン直接描画。
- 💾 **Undo/Redo 差分メモリ最適化**: バッファ全体のクローンから、編集操作差分（Delta）の保持方式へ移行し、長時間の編集セッションでのメモリ消費を抑制。

---

## 🤖 このプロジェクトについて (AI Vibe Coding)

> [!IMPORTANT]
> ### 💡 AI Vibe Coding による開発
> **Rooney** は、**Google DeepMind の Antigravity (Gemini)** との対話的ペアプログラミング（AI Vibe Coding）によって作成されたプロジェクトです。
> 人間による設計方針・アイデアの提示と、AI による実装・デバッグ・最適化のフローを組み合わせ、低レイヤの Rust `libcosmic` Wayland text-input プロトコル、インプロセス Tree-sitter 構文解析、ピースツリー Ropey バッファ、CJK サブピクセルフォント計算、ローカル Ollama SSE ストリーミング、左右2分割デスクトップエディタ UI に至るまでフルスクラッチで実装されました。

---

## 📄 ライセンス

本プロジェクトは [MIT License](LICENSE) のもとで公開されています。

<p align="center">
  Crafted via <strong>AI Vibe Coding</strong> 🚀 · Built with ❤️ for Pop!_OS COSMIC & Linux Developers
</p>
