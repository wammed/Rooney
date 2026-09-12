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
  <strong>ピースツリー Rope バッファ × Tree-sitter インプロセス構文解析 × 100% 完全ローカル AI (Ollama) × Wayland IME 完全対応</strong><br>
  外部クラウドにデータを一切送信しない、Pop!_OS COSMIC Desktop および Linux Wayland 向けの超高速・軽量・高機能エディタ
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.ja.md">日本語</a>
</p>

</div>

---

## 💡 主な特徴

- ⚡ **COSMIC & Wayland ネイティブコア**: 外部 LSP や重厚な子プロセスを完全排除。純粋な Rust `libcosmic` デスクトップ統合、専用マットアプリアイコン、クライアント装飾、0ms 瞬時起動、ダーク/ライトテーマ自動同期を実現。
- 🤖 **100% 完全ローカル AI**: Ollama を用いた完全オフライン動作。リアルタイムの Fill-in-the-Middle（FIM）インラインゴースト補完（`Ctrl + I` / `Alt + Enter`）と、トークン単位のストリーミング対話 AI チャットパネル（`Ctrl + Shift + A`）を搭載。
- 📜 **超高速ピースツリー Rope バッファ**: `ropey` を採用し、巨大ファイルでもゼロコピーで軽快に処理。インプロセスでの安全かつ瞬時なバッファ編集を提供。
- 🌳 **インプロセス Tree-sitter 構文ハイライト**: 12以上の言語（Rust, Python, JS, TS, C, C++, Bash, Fish, JSON, TOML, YAML, Markdown）を高精度かつ低負荷でリアルタイムハイライト。
- 🪟 **2分割ペイン編集 & リアルタイム Markdown プレビュー**: ツールバーやショートカット（`Ctrl + \`）で即座に左右分割。独立マルチタブ、同期編集、設定画面から **GFM**（GitHub Flavored Markdown: テーブル、タスクリスト、アラート、打ち消し線、自動リンク、`breaks: false`）と標準 **CommonMark** を動的に切り替え可能なライブ Markdown プレビュー（`Ctrl + M`）。
- 🇯🇵 **サブピクセル高精度な日本語 IME 完全対応**: Wayland text-input プロトコル（Fcitx5 / Mozc / IBus）をネイティブサポート。インライン変換下線プレビューと、半角・全角実寸幅の厳密計算により、文字入力時のカーソル位置ズレを完全解消。
- 📂 **リッチなファイルツリーエクスプローラー**: ワークスペースのリアルタイム走査、Nerd Font ファイルアイコン、右クリックコンテキストメニュー（新規ファイル、新規フォルダ、リネーム、安全削除）でのスクロール位置保持・ウィンドウ下限はみ出し防止クランプ、XDG Desktop Portal ネイティブファイルダイアログ連携。
- 🎨 **全20種の洗練された Classic & Neon テーマ**: Tokyo Night、Catppuccin、Gruvbox、Synthwave '84、Cyberpunk Neon などを網羅。エディタウィンドウ、ファイルツリー、タイトルバーの各透明度を独立調整でき、背景ディミングとともに `󰒓 Aesthetics & Preferences` モーダルで一元設定可能。
- 🔒 **鉄壁のローカルセキュリティ保護**: 一時ファイル置換によるアトミック安全保存（0バイト破損防止）、50MB ファイルサイズ上限ガード、パストラバーサル防止、機密ファイル（`.env*`, 秘密鍵等）の AI 自動シールド。

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

### 5. 設定の永続化

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
- **機密ファイル自動シールド**: `.env*`、秘密鍵・クレデンシャル（`id_rsa`、`id_ed25519`、`credentials`、`*.pem`、`*.key`、`*.keystore`、`*.jks`、`.npmrc`、`.pypirc`、`kubeconfig`、`*.token`、`*secret*`、`.aws/credentials`、`.kube/config`）の編集中は、AI 自動補完リクエストの送信を自動的にバイパスして情報漏洩を遮断。
- **原子的ファイル保存（Atomic Crash-Safe Write）**: エディタ編集ファイルおよび `config.toml` を同一階層の一時ファイル（`.{filename}.tmp.{pid}`）への書き込み・ディスク同期（`sync_all`）・置換（`rename`）により保存し、OS クラッシュや電源断時の 0 バイト破損を根絶。
- **パストラバーサル & 境界保護**: ファイル・ディレクトリ作成やリネーム時のディレクトリ脱出（`..`、`/`）をサニタイズ。ルート（`/`）やホームディレクトリの誤った再帰削除をセーフガードで防止。システムアイコン配置時のホームディレクトリは `directories::BaseDirs` で安全に解決。
- **リソース保護 & 高パフォーマンス**: 50MB を超える巨大ファイルの誤オープン防止、1MB の AI ストリーミング受信バッファ制限、有界 $O(1)$ リングバッファ Undo/Redo スタック（`VecDeque`）、Tree-sitter 増分構文解析（Incremental Parsing）を完備。

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
