# Icon Design & IP Compliance Due Diligence Record

<p align="center">
  <a href="docs/PORTAL.md">Documentation Portal</a> | <strong>English</strong> | <a href="IP_COMPLIANCE.ja.md">日本語</a>
</p>

This document records the provenance, design review history, and intellectual property (IP) due diligence process for the **Rooney** application icon.

> **Scope:** This document is a record of project provenance and due diligence. It does not constitute legal opinion, formal trademark clearance, or a guarantee against third-party claims.

## 1. Asset & Provenance

- **Application:** Rooney
- **Function:** AI-integrated Linux code and translucent split-view Markdown editor for COSMIC / Wayland.
- **Generation Method:** AI-assisted iterative geometric SVG design and refinement via Google Gemini.
- **Current Design Language:** Matte industrial aesthetic aligned with COSMIC DE (Pop!_OS) design conventions, featuring a transparent background.
- **Current Visual Concept:**
  - **Transparent Background + Dual Slate Plates:** Physical representation of the editor's translucency and side-by-side 2-pane editing (source & preview).
  - **Central Amber Emissive Slit:** A high-intensity laser/backlight beam leaking through the vertical gap in the Pop!_OS / COSMIC signature color (amber orange).
  - **Geometric Ultrawide "R":** A custom low-profile monogram with a 2:1 ratio (extended bowl vs. short stubby legs), ultra-thick stem, and bold right leg bridging both plates across the luminous slit.

## 2. Design Review History

### Initial Concepts & Past Concerns

Early draft iterations of the Rooney icon included a prominent gear motif combined with the letter "R", raising visual similarity concerns with the official Rust programming language logo. Since Rooney is implemented in Rust, the project determined it was critical to eliminate any gear motif to avoid brand confusion or trademark overlap.

### Revision Process

The project abandoned generic container boxes and textures in favor of an industrial, custom-constructed geometry directly reflecting the editor's core behavior:

1. **Transparent Background & Dual Plates:** Outer boundaries were made fully transparent, leaving two floating matte-slate plates (`#1b2028` / `#1e2530`). The right plate incorporates a frosted-glass translucent bleed layer.
2. **Central Slit Backlight:** A vibrant multi-layered amber neon glow (`#ff7900` to `#ffb300`) illuminates the vertical chasm between the plates, symbolizing translucency.
3. **Custom "R" Monogram:** Rather than adopting pre-existing typography, a completely custom SVG path was engineered—featuring an ultrawide span (>80px width), a heavy 2:1 vertical ratio with compressed legs, and monolithic weight.

## 3. Similarity Review

| Subject | Observations | Project Response |
| :--- | :--- | :--- |
| **Rust Programming Language Branding** | Shares the letter "R", developer tool category, and warm amber/orange accents. | The gear motif was completely eliminated. The icon relies on a dual-plate split, an emissive vertical slit, and an un-serifed, ultrawide custom geometric monogram, eliminating visual confusion and infringement risks. |
| **RStudio / Posit** | Shares the "rounded R in a developer editor/IDE" category. | RStudio utilizes a circular blue badge with a thin white R. Rooney employs a transparent background, dual matte-slate plates, an amber neon slit, and an ultra-bold, low-profile custom R, distinguishing it clearly in color, shape, and structure. |
| **Raycast** | Dark desktop/developer tool featuring red/orange geometric accents. | Raycast uses an angled red polygon prism and does not feature the letter "R" or a split-view plate layout, presenting zero similarity. |
| **COSMIC / System76 Native App Icons** | Conforms to COSMIC DE color palettes (dark slate + amber) and corner radii. | While respecting the OS palette, Rooney departs from System76's single-color flat line-art icons, providing a distinct third-party silhouette with dimensional slit lighting and heavy monolithic styling. |

## 4. Current Design Rationale

The final monogram directly embodies the application's unique identity rather than its underlying implementation language:

- **Dual Split Slate Plates** → Markdown split-view editing (live preview side-by-side)
- **Slit Backlight & Frosted Bleed** → Translucency and integration with the desktop wallpaper
- **Amber Orange Glow** → Alignment with the COSMIC / Pop!_OS ecosystem
- **Ultrawide Low-Profile "R"** → Monogram for "Rooney", evoking architectural stability and editor buffer resilience (Rope structure)

## 5. Limitations of AI-Assisted Review

The design iterations and comparative reviews involved AI assistance via Google Gemini. Colloquial phrases from conversational logs (such as "zero risk" or "complete compliance") reflect informal AI output and are not adopted as formal legal determinations in this document.

This record does not constitute official trademark or design patent registration clearance, but serves as an objective internal provenance record.

## 6. Retention of Records

Full conversation transcripts with Google Gemini and raw SVG iteration data are archived separately as foundational development records. For comprehensive historical details covering earlier multi-app reviews, see [ICON_DESIGN_HISTORY.ja.md](ICON_DESIGN_HISTORY.ja.md) / [ICON_DESIGN_HISTORY.md](ICON_DESIGN_HISTORY.md).

This document serves as the concise summary for repository inclusion.

## 7. Current Status

- **Status:** The current icon utilizes a transparent background, dual matte-slate plates, a central amber emissive slit, and an original ultrawide short-legged "R", deliberately avoiding all previous gear motifs or third-party proprietary designs.
- **Review Date:** 2026-10-31