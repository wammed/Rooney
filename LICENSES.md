# Licensing & Third-Party Notice

<p align="center">
  <strong>English</strong> | <a href="LICENSES.ja.md">日本語</a>
</p>

---

This document outlines the licensing policies, source-only distribution model, third-party dependency governance, and licensing boundaries for the **Rooney** project.

---

## 1. Project License (Rooney)

The source code of **Rooney** is licensed under the **MIT License**.

- The full license text is located in the repository root at [LICENSE](LICENSE).
- Copyright (c) 2026 wammed
- You are free to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, subject to retaining the copyright notice and permission notice.

---

## 2. Distribution & Build Model

### Source-Only Distribution Model

Rooney maintains a policy of **not distributing pre-compiled executable binaries, static libraries, or installer packages through GitHub or official project channels**.

- **Source Code Availability**:
  Users and distribution maintainers obtain the source code from the official Git repository and build it in their local environment using standard Rust tooling (Cargo):
  ```bash
  cargo build --release
  ```
- **Repository Independence**:
  This repository contains strictly Rooney's own source code and original assets. Third-party crates are not vendored directly inside the source tree. All dependencies are fetched during build time by Cargo from official registries ([crates.io](https://crates.io/)) or designated official Git repositories according to `Cargo.lock`.
- **Downstream Binary Redistribution**:
  When redistributing the source code, the terms of the MIT License apply. However, if users or distribution packagers compile Rooney and redistribute the resulting binary to third parties, that binary incorporates statically linked third-party dependency code. Downstream distributors are responsible for reviewing and complying with the respective third-party license obligations.

---

## 3. Third-Party Dependencies & Licensing Composition

Rooney depends on a set of external crates comprising permissive licenses, weak-copyleft licenses, and strong-copyleft licenses (GPL).

### Key Audited Dependencies

The following table summarizes the primary dependencies pinned in `Cargo.lock`:

| Component | Version | Source / Repository | Pinned Commit / Tag | Declared License | Purpose |
| :--- | :---: | :--- | :--- | :---: | :--- |
| **`libcosmic`** | 1.0.0 | [pop-os/libcosmic](https://github.com/pop-os/libcosmic) | `d4d71fd53e5ed6bd3a430089114dffa2da3cd498` | MPL-2.0 | GUI & Wayland integration |
| **`cosmic-protocols`** | 0.2.0 | [pop-os/cosmic-protocols](https://github.com/pop-os/cosmic-protocols) | `32283d76a8d0342da74c4cc022a533c52dcf378f` | GPL-3.0-only | Wayland protocol bindings |
| **`cosmic-client-toolkit`** | 0.2.0 | [pop-os/cosmic-protocols](https://github.com/pop-os/cosmic-protocols) | `32283d76a8d0342da74c4cc022a533c52dcf378f` | GPL-3.0-only | COSMIC client helper library |
| **`window_clipboard`** | 0.4.1 | [pop-os/window_clipboard](https://github.com/pop-os/window_clipboard.git) | `tag=sctk-0.20` (`f68595ee0e62f...`) | MIT | Clipboard handling |
| **`dnd` / `mime`** | 0.1.0 | [pop-os/window_clipboard](https://github.com/pop-os/window_clipboard.git) | `tag=sctk-0.20` (`f68595ee0e62f...`) | MIT | Drag-and-drop & MIME handling |
| **`winit`** | 0.31.0-beta.2 | [pop-os/winit](https://github.com/pop-os/winit.git) | `tag=cosmic-0.14` (`71ce08c0438...`) | Apache-2.0 OR MIT | Event loop & windowing |
| **`ropey`** | 1.6.1 | crates.io | — | MIT | Text buffer (Rope structure) |
| **`tree-sitter`** | 0.24.7 | crates.io | — | MIT | Incremental parsing engine |
| **`cosmic-text`** | 0.12.1 | crates.io | — | Apache-2.0 OR MIT | Font layout & glyph shaping |
| **`tokio`** | 1.40.0 | crates.io | — | MIT | Async runtime |
| **`reqwest`** | 0.12.12 | crates.io | — | Apache-2.0 OR MIT | Local Ollama HTTP client |

---

## 4. GPL-3.0-only Dependencies

The dependency tree includes crates whose package manifests explicitly declare `license = "GPL-3.0-only"`:

- **Affected Crates**:
  - `cosmic-protocols` (v0.2.0)
  - `cosmic-client-toolkit` (v0.2.0)
- **Source**: `https://github.com/pop-os/cosmic-protocols` (pinned at commit `32283d76a8d0342da74c4cc022a533c52dcf378f`)
- **License Text**: A verbatim copy of the upstream GPL-3.0 license is retained at [`THIRD_PARTY_LICENSES/GPL-3.0-only.txt`](THIRD_PARTY_LICENSES/GPL-3.0-only.txt).

### Fact-Based Boundaries
1. **Rooney's Own Source Code**:
   Rooney's source code in this repository is licensed exclusively under the MIT License. No third-party GPL code is vendored or modified directly within this repository.
2. **Compilation & Static Linking**:
   When building the project via `cargo build`, these crates are compiled and statically linked into the target binary per standard Rust compilation behavior.
3. **Guidance for Downstream Redistributors**:
   Entities or individuals redistributing compiled Rooney binaries must evaluate and satisfy the license obligations of the linked dependencies (including `GPL-3.0-only` terms such as source disclosure requirements) according to their specific distribution model and context.

---

## 5. Assets & Bundled Content Policy

- **Visual Assets**:
  - `images/Rooney-matte-icon.svg`
  - `images/Rooney-icon.svg`
  - `images/Rooney-banner.svg`
  These visual assets were created specifically for Rooney and are provided under the same **MIT License** as the project source.
- **Fonts**:
  Rooney does not bundle third-party font files within its repository or binary. System fonts are discovered and rendered dynamically at runtime via Fontconfig and `cosmic-text`.

---

## 6. Automated Audit & License Extraction Tools

The following commands are used to audit and extract dependency licensing metadata:

```bash
# Verify license compliance, bans, and sources via cargo-deny
cargo deny check licenses bans sources

# Generate comprehensive third-party license HTML report via cargo-about
cargo install cargo-about
cargo about generate about.hbs --offline > THIRD_PARTY_LICENSES.html
```

- For detailed audit notes regarding crates with missing crate-level metadata, refer to [`THIRD_PARTY_LICENSES/README.md`](THIRD_PARTY_LICENSES/README.md).

---

## 7. Downstream Packaging & Legal Considerations

1. **Downstream Binary Redistribution & Legal Assessment**:
   - Rooney is maintained and distributed as a source-only repository under the MIT License.
   - Package maintainers or organizations packaging and redistributing compiled Rooney binaries (e.g. for Arch Linux AUR, Debian, Fedora, Flatpak, or Snap) must independently review and comply with the obligations imposed by statically linked dependencies, specifically `cosmic-protocols` and `cosmic-client-toolkit` (`GPL-3.0-only`), according to their own distribution channels and applicable legal standards.
2. **Disclaimer of Legal Counsel**:
   - This document and related audit records compile technical findings and upstream metadata. They do not constitute formal legal advice or legal guarantees.
3. **Future Asset Ingestion Governance**:
   - While the current release avoids embedding third-party fonts or external icons, any future inclusion of bundled assets (e.g. SIL OFL-1.1 fonts) must include verbatim upstream licenses in [`THIRD_PARTY_LICENSES/`](THIRD_PARTY_LICENSES/) and corresponding notices in this file.

