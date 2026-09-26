# Third-Party License Audit Records

Last reviewed: **2026-09-26**

<p align="center">
  <strong>English</strong> | <a href="README.ja.md">日本語</a>
</p>

---

This directory serves as the official record for third-party dependency licensing investigations, documentation of upstream Cargo metadata deficiencies, and audit evidence for the **Rooney** project.

> **Notice**:
> This document is an audit and investigation record, not a replacement for authoritative upstream license texts. Authoritative license files and copyright notices reside within the respective upstream source trees or accompanying files.

---

## 1. Audit Policy

To ensure dependency health and license compliance, Rooney utilizes [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) with an explicit SPDX allowlist policy ([`deny.toml`](../deny.toml)).

- **Distinction Between Missing Metadata and License Expression**:
  When an upstream crate's `Cargo.toml` lacks a `license` or `license-file` field, Rooney does not convert this deficiency into an unconditional global allow. Instead, repository-level license files and source headers are examined manually and through tools, with findings explicitly documented here and in `deny.toml` under `[[licenses.clarify]]`.
- **Verification of Concrete Files and Commits**:
  For Git dependencies, the exact commit hash pinned in `Cargo.lock` (Pinned Commit SHA) serves as the ground truth. Checked-out sources and manifests are inspected directly.
- **Relationship to Source-Only Distribution**:
  Rooney adopts a source-only distribution model and does not distribute official pre-compiled binaries. However, to support users and package maintainers building or packaging binaries for third-party redistribution, factual and verifiable dependency licensing audit records are provided without conjecture.

---

## 2. Key Audited Dependencies & Findings

### A. GPL-3.0-only Dependencies

Certain COSMIC desktop platform crates utilized by Rooney explicitly declare `license = "GPL-3.0-only"`.

#### 1. `cosmic-protocols`
- **Crate**: `cosmic-protocols` (v0.2.0)
- **Source Repository**: `https://github.com/pop-os/cosmic-protocols`
- **Pinned Commit**: `32283d76a8d0342da74c4cc022a533c52dcf378f` (`rev = "32283d7"`)
- **Verification Evidence**:
  Confirmed directly within the checked-out manifest `Cargo.toml`:
  ```toml
  [package]
  name = "cosmic-protocols"
  version = "0.2.0"
  license = "GPL-3.0-only"
  ```
- **License Text**:
  GNU General Public License Version 3 full text is located in `src/LICENSE` of the upstream repository. An exact verbatim copy is stored in this directory as [`GPL-3.0-only.txt`](GPL-3.0-only.txt).

#### 2. `cosmic-client-toolkit`
- **Crate**: `cosmic-client-toolkit` (v0.2.0)
- **Source Repository**: `https://github.com/pop-os/cosmic-protocols` (`client-toolkit` workspace member)
- **Pinned Commit**: `32283d76a8d0342da74c4cc022a533c52dcf378f`
- **Verification Evidence**:
  Confirmed directly within the checked-out `client-toolkit/Cargo.toml`:
  ```toml
  [package]
  name = "cosmic-client-toolkit"
  version = "0.2.0"
  license = "GPL-3.0-only"
  ```

---

### B. Upstream Crates with Missing Cargo Metadata (Clarifications)

Several Git dependencies lack an explicit `license` field in their package-level `Cargo.toml`. Their licenses have been established via repository-level license files and source SPDX headers, and clarified in `deny.toml` under `[[licenses.clarify]]`.

#### 1. `libcosmic` Family
- **Source Repository**: `https://github.com/pop-os/libcosmic`
- **Pinned Commit**: `d4d71fd53e5ed6bd3a430089114dffa2da3cd498`
- **Affected Packages**:
  - `libcosmic` (v1.0.0)
  - `cosmic-config` (v1.0.0)
  - `cosmic-config-derive` (v1.0.0)
  - `cosmic-theme` (v1.0.0)
- **Audit Findings**:
  The repository root contains Mozilla Public License Version 2.0 (`MPL-2.0`), and source file headers declare `SPDX-License-Identifier: MPL-2.0`.

#### 2. `cosmic-settings-daemon`
- **Source Repository**: `dbus-settings-bindings`
- **Affected Package**: `cosmic-settings-daemon` (v0.1.0)
- **Audit Findings**:
  Classified as `MPL-2.0` based on the repository root `LICENSE.md`.

#### 3. `window_clipboard` Family (`dnd`, `mime`)
- **Source Repository**: `https://github.com/pop-os/window_clipboard.git`
- **Pinned Tag / Commit**: `tag = "sctk-0.20"` (`f68595ee0e62fbd6589f4709b5aaa5c3c7ea5f6c`)
- **Affected Packages**:
  - `dnd` (v0.1.0)
  - `mime` (v0.1.0)
- **Audit Findings**:
  The repository root `LICENSE` and root `Cargo.toml` specify `license = "MIT"`.

#### 4. `iced_accessibility`
- **Affected Package**: `iced_accessibility` (v0.1.0)
- **Audit Findings**:
  Classified as `MIT` based on upstream Iced ecosystem licensing.

---

## 3. Assets & Bundled Content Policy

Rooney does not bundle third-party fonts or external vector icon packs into the compiled binary via `include_bytes!`.

- **Original Visual Assets**:
  - `images/Rooney-matte-icon.svg`
  - `images/Rooney-icon.svg`
  - `images/Rooney-banner.svg`
  These assets were designed exclusively for Rooney and are distributed under the same **MIT License** as the project itself.
- **Fonts**:
  System fonts are discovered and loaded dynamically at runtime via Fontconfig and `cosmic-text`. No third-party font binaries are embedded.

---

## 4. Re-Audit & Extraction Commands

To revalidate dependencies or generate third-party license notices for downstream packaging:

```bash
# Check licenses, bans, and sources via cargo-deny
cargo deny check licenses bans sources

# Generate comprehensive third-party license HTML report via cargo-about
cargo about generate about.hbs --offline > THIRD_PARTY_LICENSES.html
```
