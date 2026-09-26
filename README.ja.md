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
  <a href="README.md">English</a> | <a href="README.ja.md">日本語</a> | <a href="docs/PORTAL.ja.md">📚 ドキュメントポータル</a> | <a href="LICENSES.ja.md">ライセンス通知</a>
</p>

</div>

---

## 🚀 クイックスタート

### 1. 必要環境

- [Rust (Cargo)](https://rustup.rs/) (1.80 以上、最新 Stable 推奨 / 実機検証: 1.98.1)
- Linux Wayland 環境 / Pop!_OS COSMIC Desktop
- システムビルド依存パッケージ:
  - **Debian / Pop!_OS / Ubuntu**:
    ```bash
    sudo apt install build-essential libxkbcommon-dev libfontconfig1-dev wayland-protocols
    ```
  - **Arch Linux / CachyOS**:
    ```bash
    sudo pacman -S base-devel libxkbcommon fontconfig wayland
    ```
- [Ollama](https://ollama.com/) (ローカル AI 機能利用時)

### 2. Ollama のセットアップ (ローカル AI)

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

選択したテーマ、フォント、フォントサイズ、各部独立透明度、ディミング、AI モデル、Markdown 仕様（GFM / CommonMark）、分割状態、開いているタブセッションは `~/.config/rooney/config.toml` に自動保存・復元されます。

---

## 💡 主な特徴

- ⚡ **COSMIC & Wayland ネイティブ**: 純粋な Rust `libcosmic` 統合。外部 LSP や重厚な Electron ランタイムに依存しない高速起動と自動テーマ同期。
- 🏎️ **ビューポート仮想描画 (< 0.40 ms)**: 50MB / 185万行の巨大ファイルでも画面内可視行のみを即時計算し、144+ FPS の滑らかなスクロールと $O(\log W)$ 累積折り返しモデルを実現。
- 🌳 **統一世代管理アーキテクチャ**: Tree-sitter 構文解析・Markdown パース・検索を非同期ワーカースレッドへ移譲。整数世代カウンターにより、UI フリーズと競合による古い描画の混入（Stale 結果）を 100% 排除。
- 🤖 **100% 完全ローカル AI (Ollama)**: 外部クラウド通信ゼロ。リアルタイム FIM（Fill-in-the-Middle）ゴースト補完（`Ctrl + I`）とストリーミング対話 AI チャット（`Ctrl + Shift + A`）を搭載。
- 📜 **ピースツリー Rope バッファ (`ropey`)**: 大規模ファイル編集時のメモリ連続再配置を抑え、$O(1)$ 行長追跡で瞬時に折り返し判定。
- 🪟 **2分割ペイン & GFM 志向 Markdown プレビュー**: 左右分割編集（`Ctrl + \`）と、WebView を排除した純粋なネイティブ Rust レンダラーによる高速プレビュー。外部通信を行わない **Zero Remote I/O** 設計。
- 🇯🇵 **Unicode Grapheme & 実グリフ計測連携**: 日本語 IME（Fcitx5 / Mozc / IBus）ネイティブ対応。結合文字・絵文字 ZWJ の不可分移動、および `cosmic-text` 実グリフ幅キャッシュによる全角ダッシュ等のカーソルズレ完全解消。
- 🎨 **全20種のテーマ & 各部独立透明度**: Tokyo Night、Catppuccin、Gruvbox など豊富なテーマと、エディタ・ツリー・タイトルバーの独立透明度調整。

> 📖 **詳細な仕様や設計解説**:
> 各機能の完全な技術仕様は [docs/FEATURES.ja.md](docs/FEATURES.ja.md) を、内部設計・非同期世代管理の詳細は [docs/ARCHITECTURE.ja.md](docs/ARCHITECTURE.ja.md) をご覧ください。

---

## ⌨️ 主なショートカットキー

最頻出の主要キー一覧です。

| ショートカット | 動作 |
| :--- | :--- |
| `Ctrl + N` / `Ctrl + O` | **新規ファイル作成** / **ファイルを開く** |
| `Ctrl + S` / `Ctrl + Shift + S` | **ファイルを保存**（アトミック安全保存）/ **名前を付けて保存** |
| `Ctrl + T` / `Ctrl + W` | **新規タブ作成** / **タブを閉じる** |
| `Ctrl + Tab` / `Ctrl + Shift + Tab` | 次のタブ / 前のタブへ移動 |
| `Ctrl + \` または `Ctrl + E` | **左右 2 分割ペインの切り替え**（Split / Single） |
| `Ctrl + M` | **Markdown プレビューの切り替え**（GFM / CommonMark） |
| `Ctrl + B` | **ファイルツリーサイドバーの表示 / 非表示** |
| `Ctrl + F` | **ファイル内インクリメンタル検索バーの開閉** |
| `Ctrl + Shift + A` | **AI チャットパネルの開閉トグル** |
| `Ctrl + I` または `Alt + Enter` | **Local AI FIM インライン補完の手動トリガー** |
| `Ctrl + /` | **行コメントのトグル**（言語別自動判定） |
| `Ctrl + ,` | **Aesthetics 設定**（テーマ、フォント、各部透明度、AIモデル） |
| `Esc` | 開いているモーダル・検索バー・補完提案を閉じる |

> ⌨️ **完全版リファレンス**:
> 編集操作（Undo/Redo、行複製・削除、インデント）や詳細なキーバインド一覧は **[docs/SHORTCUTS.ja.md](docs/SHORTCUTS.ja.md)** を参照してください。

---

## 📚 ドキュメントポータル

Rooney のより詳しいドキュメントは、以下の専門ドキュメントに体系化されています：

| ドキュメント | 内容 |
| :--- | :--- |
| **[📚 ドキュメントポータル](docs/PORTAL.ja.md)** | 目的別・読者別の総合案内ハブ |
| **[⌨️ ショートカット & 操作ガイド](docs/SHORTCUTS.ja.md)** | 全キーバインド、ペイン分割、AI 操作の完全リファレンス |
| **[💡 詳細機能仕様書 (FEATURES)](docs/FEATURES.ja.md)** | 全機能の仕様、GFM レンダリング、テーマ・透明度設定の詳細 |
| **[📐 アーキテクチャ設計書 (ARCHITECTURE)](docs/ARCHITECTURE.ja.md)** | 世代管理非同期設計、仮想レンダリング、Rope、CJK 実グリフ計測 |
| **[🛡️ セキュリティ & 堅牢性仕様 (SECURITY)](docs/SECURITY.ja.md)** | ゼロクラウド流出、機密ファイル除外、アトミック保存、Zero Remote I/O |

---

## 🔒 セキュリティ & プライバシー (概要)

- **100% 完全オフライン・ゼロクラウド流出**: テレメトリ、クラッシュレポート、クラウド API 通信は一切行いません。AI 処理はすべてマシン内のローカル Ollama デーモンで完結します。
- **機密ファイル除外ガード**: `.env*`、秘密鍵（`id_rsa`、`*.pem`、`credentials` 等）の編集中は AI 補完リクエストを自動遮断。
- **一時ファイル置換によるアトミック保存**: 一時ファイル（`.{file}.tmp.{pid}`）への書き込み・`sync_all`・置換により、ファイル破損リスクを極小化。
- **Zero Remote I/O Markdown**: Markdown プレビューはリモート画像取得や外部通信を行わず、軽量テキストバッジへ安全にフォールバックします。

> 🛡️ **セキュリティ詳細**:
> より詳しいセキュリティ境界や防御仕様については **[docs/SECURITY.ja.md](docs/SECURITY.ja.md)** をご覧ください。

---

## 🗺️ ロードマップ

- [x] 🏎️ **ビューポート仮想スクロール**: 画面内の可視行のみ計算し、50MB でもサブミリ秒（< 0.40 ms）レイアウト。
- [x] ⚡ **Tree-sitter 増分ハイライト・行キャッシュ**: 入力時の再解析負荷を極小化。
- [x] 📊 **マルチスケール性能ベンチマークスイート**: 10KB〜50MB の各種パフォーマンステストを完備。
- [x] 📐 **実グリフメトリクス（`cosmic-text`）連携**: 全角ダッシュ（`――`）や CJK 記号のカーソルズレを完全解消。
- [ ] 👁️ **ファイルシステム変更監視（File Watcher）**: ワークスペースディレクトリの外部変更を非同期検知・同期。
- [ ] 📝 **リッチなインライン Markdown AST 描画**: エディタキャンバス内でのインライン Markdown 構造装飾。
- [ ] 💾 **Undo/Redo 差分メモリ最適化**: 操作差分（Delta）保持方式による長時間編集時のメモリ抑制。

---

## 🤖 このプロジェクトについて (AI Vibe Coding)

> [!IMPORTANT]
> ### 💡 AI Vibe Coding による開発
> **Rooney** は、**Google DeepMind の Antigravity (Gemini)** との対話的ペアプログラミング（AI Vibe Coding）によって作成されたプロジェクトです。
> 人間による設計方針・アイデアの提示と、AI による実装・デバッグ・最適化のフローを組み合わせ、低レイヤの Rust `libcosmic` Wayland text-input プロトコル、インプロセス Tree-sitter 構文解析、ピースツリー Ropey バッファ、CJK サブピクセルフォント計算、ローカル Ollama SSE ストリーミング、左右2分割デスクトップエディタ UI に至るまでフルスクラッチで実装されました。

---

## 📄 ライセンス

Rooney 本体のソースコードは [MIT License](LICENSE) のもとで公開されています。

Rooney が利用するサードパーティ製依存関係には、各上流のライセンス（`MPL-2.0`、`Apache-2.0`、`MIT`、`GPL-3.0-only` 等）が適用されます。ライセンス体系、ソースコード配布方針、Git 依存関係のコミット固定情報、およびバイナリ再配布時の留意事項に関する詳細は、**[LICENSES.ja.md](LICENSES.ja.md)**（[英語版: LICENSES.md](LICENSES.md)）および [THIRD_PARTY_LICENSES/README.ja.md](THIRD_PARTY_LICENSES/README.ja.md) を参照してください。

<p align="center">
  Crafted via <strong>AI Vibe Coding</strong> 🚀 · Built with ❤️ for Pop!_OS COSMIC & Linux Developers
</p>
