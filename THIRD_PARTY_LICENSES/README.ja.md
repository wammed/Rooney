# 第三者ライセンス監査記録 (Third-Party License Audit Records)

最終更新日: **2026-09-26**

<p align="center">
  <a href="README.md">English</a> | <strong>日本語</strong>
</p>

---

> **注意**:
> 本ドキュメントは監査および調査の記録であり、各上流プロジェクトが配布する正式なライセンス本文の代替物ではありません。各依存関係の正式なライセンスファイルおよび著作権表示は、それぞれの上流ソースツリーまたは同梱ファイルに帰属します。

---

## 1. 監査方針 (Audit Policy)

Rooney では、依存関係の健全性とライセンス適合性を担保するために [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) を導入し、SPDX 識別子による明示的な許可リスト方式（[`deny.toml`](../deny.toml)）を採用しています。

- **メタデータ不備とライセンスの区別**:
  上流クレートの `Cargo.toml` に `license` または `license-file` フィールドが欠落している場合、それを無差別にプロジェクト全体で無条件許可（allow）するのではなく、リポジトリ単位のライセンスファイルおよびソースヘッダーを人間およびツールで精査し、その事実を本ドキュメントおよび `deny.toml` の `[[licenses.clarify]]` に明示的に記録します。
- **実ファイル・実コミットの検証**:
  Git 依存関係については、`Cargo.lock` に固定されている一意のコミットハッシュ（Pinned Commit SHA）を基準に、チェックアウトされた実ソースコードおよびマニフェストを直接監査します。
- **ソース配布モデルとの関係**:
  Rooney はソースコードのみを配布するモデル（source-only distribution）を採用しており、公式のバイナリ配布を行っていません。しかし、ユーザーやパッケージメンテナーがローカル環境でビルドして生成したバイナリを第三者へ再配布するケースを想定し、推測を排除した正確な依存関係ライセンスの監査記録を提供します。

---

## 2. 重要な依存関係の調査・監査記録

### A. GPL-3.0-only 依存関係

Rooney が利用する COSMIC デスクトップ基盤の一部クレートには、`GPL-3.0-only` のライセンス宣言が含まれています。

#### 1. `cosmic-protocols`
- **クレート名**: `cosmic-protocols` (v0.2.0)
- **ソースリポジトリ**: `https://github.com/pop-os/cosmic-protocols`
- **Pinned Commit**: `32283d76a8d0342da74c4cc022a533c52dcf378f` (`rev = "32283d7"`)
- **確認証跡**:
  チェックアウト先の実ファイル `Cargo.toml` において以下を確認済み：
  ```toml
  [package]
  name = "cosmic-protocols"
  version = "0.2.0"
  license = "GPL-3.0-only"
  ```
- **ライセンス本文**:
  リポジトリ内 `src/LICENSE` に GNU General Public License Version 3 の全文が収録されています。本ディレクトリにその同一複製を [`GPL-3.0-only.txt`](GPL-3.0-only.txt) として保存しています。

#### 2. `cosmic-client-toolkit`
- **クレート名**: `cosmic-client-toolkit` (v0.2.0)
- **ソースリポジトリ**: `https://github.com/pop-os/cosmic-protocols` (`client-toolkit` ワークスペースメンバー)
- **Pinned Commit**: `32283d76a8d0342da74c4cc022a533c52dcf378f`
- **確認証跡**:
  実ファイル `client-toolkit/Cargo.toml` において以下を確認済み：
  ```toml
  [package]
  name = "cosmic-client-toolkit"
  version = "0.2.0"
  license = "GPL-3.0-only"
  ```

---

### B. 上流 Cargo メタデータが未記載のクレート（Clarification 対象）

一部の Git 依存クレートでは、個別の `Cargo.toml` に `license` フィールドが記載されていませんが、リポジトリルートのライセンスファイルおよび SPDX ヘッダーからライセンスが特定されています。これらは `deny.toml` の `[[licenses.clarify]]` にて補足定義されています。

#### 1. `libcosmic` ファミリ
- **ソースリポジトリ**: `https://github.com/pop-os/libcosmic`
- **Pinned Commit**: `d4d71fd53e5ed6bd3a430089114dffa2da3cd498`
- **対象クレート**:
  - `libcosmic` (v1.0.0)
  - `cosmic-config` (v1.0.0)
  - `cosmic-config-derive` (v1.0.0)
  - `cosmic-theme` (v1.0.0)
- **調査結果**:
  リポジトリルートに Mozilla Public License Version 2.0 (`MPL-2.0`) が配置されており、ソースファイルヘッダーに `SPDX-License-Identifier: MPL-2.0` が記載されています。

#### 2. `cosmic-settings-daemon`
- **ソースリポジトリ**: `dbus-settings-bindings`
- **対象クレート**: `cosmic-settings-daemon` (v0.1.0)
- **調査結果**:
  リポジトリルートの `LICENSE.md` に基づき `MPL-2.0` として判定。

#### 3. `window_clipboard` ファミリ (`dnd`, `mime`)
- **ソースリポジトリ**: `https://github.com/pop-os/window_clipboard.git`
- **Pinned Tag / Commit**: `tag = "sctk-0.20"` (`f68595ee0e62fbd6589f4709b5aaa5c3c7ea5f6c`)
- **対象クレート**:
  - `dnd` (v0.1.0)
  - `mime` (v0.1.0)
- **調査結果**:
  リポジトリルートの `LICENSE` およびルート `Cargo.toml` にて `license = "MIT"` が明記されています。

#### 4. `iced_accessibility`
- **対象クレート**: `iced_accessibility` (v0.1.0)
- **調査結果**:
  上流 Iced エコシステムのライセンス表記に基づき `MIT` として判定。

---

## 3. 配布物に含まれるアセット (Assets & Bundled Content)

Rooney では、外部フォントやサードパーティ製アイコンパックのバイナリ埋め込み（`include_bytes!` 等）は行っていません。

- **独自作成アセット**:
  - `images/Rooney-matte-icon.svg`
  - `images/Rooney-icon.svg`
  - `images/Rooney-banner.svg`
  これらは Rooney プロジェクトのために独自に作成されたオリジナルアセットであり、本体と同じ **MIT License** が適用されます。
- **フォント**:
  システムのフォント設定（Fontconfig）および `cosmic-text` を介してホスト環境のシステムフォントを動的に参照します。サードパーティ製フォントファイルの同梱はありません。

---

## 4. 再監査およびライセンス抽出コマンド

依存関係の更新時やパッケージ作成時には、以下のコマンドで監査を再現できます：

```bash
# cargo-deny によるライセンス・禁止クレート・ソース検査
cargo deny check licenses bans sources

# cargo-about による HTML ライセンスドキュメント生成
cargo about generate about.hbs --offline > THIRD_PARTY_LICENSES.html
```
