# 📚 Rooney ドキュメントポータル (Documentation Portal)

Rooney の公式ドキュメントポータルへようこそ。  
Rooney は、COSMIC Desktop / Linux Wayland 環境向けにフルスクラッチで開発された、100% 完全ローカル AI 統合型コード & Markdown エディタです。

本ポータルでは、利用目的に応じて必要な詳細仕様やガイドへ迷わずアクセスできるように各ドキュメントを整理しています。

---

## 🧭 ドキュメント総合ナビゲーション

| ドキュメント | 主な内容 | 推奨読者 |
| :--- | :--- | :--- |
| **[クイックスタート](../README.ja.md#-クイックスタート)** | 必要環境、Ollama セットアップ、ビルド、起動、初期設定 | 初めて利用する方、インストールしたい方 |
| **[⌨️ ショートカット & 操作ガイド](SHORTCUTS.ja.md)** | 全キーバインド一覧、マルチペイン分割、エディタ編集、AI操作 | 日常的に操作をマスターしたい方 |
| **[💡 詳細機能仕様 (FEATURES)](FEATURES.ja.md)** | 全機能の詳細仕様、GFM Markdown レンダリング、テーマ・外観 | 機能の詳細やMarkdown仕様を知りたい方 |
| **[📐 アーキテクチャ設計 (ARCHITECTURE)](ARCHITECTURE.ja.md)** | 世代管理非同期設計、仮想描画、Rope バッファ、CJK 実グリフ連携 | 内部構造やパフォーマンス設計を知りたい方 |
| **[🛡️ セキュリティモデル (SECURITY)](SECURITY.ja.md)** | ゼロクラウド流出、機密ファイル除外、アトミック保存、Zero Remote I/O | セキュリティやデータ整合性を確認したい方 |
| **[📄 ライセンス & 依存通知](../LICENSES.ja.md)** | MIT ライセンス、サードパーティ OSS ライセンス、再配布留意事項 | 法的要件やライセンスを確認したい方 |

---

## 🎯 目的別ガイド

### 1. まずは動かしてみたい
- [README.ja.md: クイックスタート](../README.ja.md#-クイックスタート) を参照し、`cargo run` または `cargo build --release` でバイナリを生成してください。
- ローカル AI 補完やチャットを利用する場合は、Ollama をインストールして `ollama serve` および `ollama pull qwen2.5-coder` を実行します。

### 2. キーボード操作を効率化したい
- [SHORTCUTS.ja.md](SHORTCUTS.ja.md) をご覧ください。タブ移動、左右2分割（`Ctrl + \` / `Ctrl + E`）、Markdown プレビュー切替（`Ctrl + M`）、AI チャット（`Ctrl + Shift + A`）、FIM インライン補完（`Ctrl + I`）などを網羅しています。

### 3. Markdown やエディタの詳細機能を知りたい
- [FEATURES.ja.md](FEATURES.ja.md) では、GFM アラート（Callout）、テーブル、タスクリスト、外部通信を遮断した Zero Remote I/O 設計、20 種類のカラーテーマ、各部独立透明度設定などの詳細仕様を解説しています。

### 4. 内部設計や超高速描画の仕組みを知りたい
- [ARCHITECTURE.ja.md](ARCHITECTURE.ja.md) では、巨大ファイル（50MB / 185万行）でも 144+ FPS を維持するビューポート仮想化、整数世代カウンターによる非同期レースコンディション防止（Stale Protection）、実グリフメトリクス（`cosmic-text`）キャッシュ機構を詳しく解説しています。

### 5. セキュリティや機密保護の方針を確認したい
- [SECURITY.ja.md](SECURITY.ja.md) では、外部クラウドへの一切の通信を行わないローカルファースト原則、`.env` や秘密鍵の自動 AI 除外ガード、アトミックファイル保存、パストラバーサル防止を解説しています。

---

<p align="center">
  <a href="../README.ja.md">← ルート README (日本語) に戻る</a> | <a href="PORTAL.md">English Portal →</a>
</p>
