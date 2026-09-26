# ライセンスおよびサードパーティ通知 (Licensing & Third-Party Notice)

<p align="center">
  <a href="LICENSES.md">English</a> | <strong>日本語</strong>
</p>

---

本書は、**Rooney** プロジェクトにおけるライセンス方針、ソースコード配布モデル、サードパーティ製依存関係の管理方針、およびライセンス境界について説明する公式ドキュメントです。

---

## 1. プロジェクトライセンス (Rooney)

**Rooney** 本体のソースコードは、**MIT License** のもとで公開されています。

- ライセンス全文はリポジトリルートの [LICENSE](LICENSE) ファイルに記載されています。
- Copyright (c) 2026 wammed
- 著作権表示および許諾表示を保持する限り、商用・非商用を問わず、複製、改変、再配布、サブライセンス付与、および派生著作物の作成を自由に行うことができます。

---

## 2. 配布およびビルドモデル (Distribution & Build Model)

### ソースコード配布モデル (Source-Only Distribution)

Rooney プロジェクトでは、**GitHub や公式チャンネルを通じてコンパイル済み実行可能バイナリやインストーラーパッケージを直接配布しない方針**を採用しています。

- **ソースコードでの提供**:
  利用者は公式 Git リポジトリよりソースコードを取得し、標準の Rust ツールチェーン（Cargo）を用いて自身のローカル環境でビルドします：
  ```bash
  cargo build --release
  ```
- **リポジトリ内の独立性**:
  本リポジトリには Rooney 自身のソースコードおよび独自アセットのみが含まれており、外部クレートのソースコードをリポジトリ内に直接内包（ベンダー同梱 / vendoring）していません。ビルド時に Cargo が `Cargo.lock` に基づいて必要な依存関係を crates.io または指定された公式 Git リポジトリから直接ダウンロードします。
- **バイナリ再配布時の留意点**:
  ソースコード配布において適用されるのは Rooney 自身の MIT License です。しかし、ユーザーや各ディストリビューションのパッケージメンテナーがローカル環境でビルドしたバイナリを第三者へ再配布する場合、バイナリには静的リンクされた依存ライブラリのコードが含まれます。その際には、各依存関係のライセンス条件を確認・遵守する必要があります。

---

## 3. サードパーティ製依存関係とライセンス構成

Rooney が依存している外部クレートは、パーミッシブ（寛容型）ライセンス、ウィークコピーレフトライセンス、および強力なコピーレフトライセンス（GPL）で構成されています。

### 主な依存コンポーネント一覧

`Cargo.lock` に固定されている主要な依存関係のメタデータおよびライセンスは以下のとおりです。

| コンポーネント | バージョン | ソース / リポジトリ | Pinned Commit / Tag | 宣言ライセンス | 役割 |
| :--- | :---: | :--- | :--- | :---: | :--- |
| **`libcosmic`** | 1.0.0 | [pop-os/libcosmic](https://github.com/pop-os/libcosmic) | `d4d71fd53e5ed6bd3a430089114dffa2da3cd498` | MPL-2.0 | GUI / Wayland ウィンドウ統合 |
| **`cosmic-protocols`** | 0.2.0 | [pop-os/cosmic-protocols](https://github.com/pop-os/cosmic-protocols) | `32283d76a8d0342da74c4cc022a533c52dcf378f` | GPL-3.0-only | Wayland プロトコルバインディング |
| **`cosmic-client-toolkit`** | 0.2.0 | [pop-os/cosmic-protocols](https://github.com/pop-os/cosmic-protocols) | `32283d76a8d0342da74c4cc022a533c52dcf378f` | GPL-3.0-only | COSMIC クライアント支援ライブラリ |
| **`window_clipboard`** | 0.4.1 | [pop-os/window_clipboard](https://github.com/pop-os/window_clipboard.git) | `tag=sctk-0.20` (`f68595ee0e62f...`) | MIT | クリップボード連携 |
| **`dnd` / `mime`** | 0.1.0 | [pop-os/window_clipboard](https://github.com/pop-os/window_clipboard.git) | `tag=sctk-0.20` (`f68595ee0e62f...`) | MIT | ドラッグ＆ドロップ / MIME 処理 |
| **`winit`** | 0.31.0-beta.2 | [pop-os/winit](https://github.com/pop-os/winit.git) | `tag=cosmic-0.14` (`71ce08c0438...`) | Apache-2.0 OR MIT | イベントループ・ウィンドウ管理 |
| **`ropey`** | 1.6.1 | crates.io | — | MIT | テキスト編集バッファ（Rope） |
| **`tree-sitter`** | 0.24.7 | crates.io | — | MIT | 構文解析エンジン |
| **`cosmic-text`** | 0.12.1 | crates.io | — | Apache-2.0 OR MIT | テキストレイアウト・グリフ計測 |
| **`tokio`** | 1.40.0 | crates.io | — | MIT | 非同期ランタイム |
| **`reqwest`** | 0.12.12 | crates.io | — | Apache-2.0 OR MIT | ローカル Ollama 通信 |

---

## 4. GPL-3.0-only 依存関係について

本プロジェクトの依存関係には、マニフェスト（`Cargo.toml`）上で `license = "GPL-3.0-only"` と明記されているクレートが含まれています：

- **対象クレート**:
  - `cosmic-protocols` (v0.2.0)
  - `cosmic-client-toolkit` (v0.2.0)
- **ソース**: `https://github.com/pop-os/cosmic-protocols`（コミット `32283d76a8d0342da74c4cc022a533c52dcf378f`）
- **ライセンス本文**: 上流リポジトリのライセンス全文を [`THIRD_PARTY_LICENSES/GPL-3.0-only.txt`](THIRD_PARTY_LICENSES/GPL-3.0-only.txt) に複製・保管しています。

### 事実ベースの境界説明
1. **Rooney 自身のソースコード**:
   Rooney のリポジトリ内に含まれるソースコードは、一貫して MIT License でライセンスされています。リポジトリ内に GPL-3.0-only のコードを直接改変・混入している箇所はありません。
2. **コンパイル時の結合**:
   `cargo build` を実行して生成されるバイナリには、Rust の通常のコンパイル挙動としてこれらの依存関係が静的にリンクされます。
3. **下流再配布者への案内**:
   生成されたバイナリを第三者へ配布・提供するパッケージメンテナーや利用者は、リンクされた `GPL-3.0-only` クレートに起因する義務や条件（ソースコード開示要件など）について、各自の配布形態や目的に照らしてライセンス要件を確認・遵守する必要があります。

---

## 5. アセットおよび同梱ファイル方針

- **ビジュアルアセット**:
  - `images/Rooney-matte-icon.svg`
  - `images/Rooney-icon.svg`
  - `images/Rooney-banner.svg`
  これらのアセットは Rooney プロジェクト用に独自に制作されたものであり、Rooney 本体と同じ **MIT License** のもとで提供されます。
- **フォント**:
  Rooney ではサードパーティ製フォントファイルをリポジトリ内やバイナリ内に直接同梱していません。ホスト OS のフォント環境（Fontconfig 等）を通じてシステムフォントを利用します。

---

## 6. 自動ライセンス監査および一覧生成ツール

プロジェクトでは、依存関係の追跡と検証のために以下のツールを利用しています：

```bash
# 依存関係のライセンス・ソース・禁止クレート検査 (cargo-deny)
cargo deny check licenses bans sources

# 包括的な第三者ライセンス HTML ドキュメントの生成 (cargo-about)
cargo install cargo-about
cargo about generate about.hbs --offline > THIRD_PARTY_LICENSES.html
```

- 詳細な調査記録やメタデータ未記載クレートの監査結果については、[`THIRD_PARTY_LICENSES/README.md`](THIRD_PARTY_LICENSES/README.md) を参照してください。

---

## 7. 留意事項および下流パッケージャー向けガイドライン (Downstream Considerations)

1. **下流ディストリビューターによるバイナリ再配布時の法的評価**:
   - Rooney プロジェクト自身はソースコード配布形式をとっているため、リポジトリ自体の配布には MIT License が適用されます。
   - 一方で、Linux 各ディストリビューション向けパッケージ（Arch Linux AUR、Debian、Fedora、Flatpak、Snap 等）を作成し、コンパイル済みバイナリを第三者へ再配布するパッケージメンテナーや組織は、バイナリに静的リンクされる `cosmic-protocols` および `cosmic-client-toolkit`（`GPL-3.0-only`）の義務（ソースコード開示要件、ライセンス通知等）を、各ディストリビューションの再配布ポリシーや適用法令に照らして独自に確認・評価・履行する必要があります。
2. **法的助言の非提供**:
   - 本文書および関連する監査記録は、客観的な事実およびメタデータの調査結果をまとめた技術資料であり、法的な保証や法的助言（リーガルオピニオン）を構成するものではありません。
3. **将来の第三者アセット追加時の管理**:
   - 現行バージョンでは外部フォントやサードパーティ製アイコンをバイナリ埋め込みしていませんが、将来的にカスタムフォント（SIL OFL-1.1 等）や第三者ビジュアルアセットを追加する場合は、[`THIRD_PARTY_LICENSES/`](THIRD_PARTY_LICENSES/) に正式な上流ライセンス本文を追加し、本書および README を更新するものとします。

