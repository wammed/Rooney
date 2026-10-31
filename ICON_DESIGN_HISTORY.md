# Icon Design & IP Review History

<p align="center">
  <a href="docs/PORTAL.md">Documentation Portal</a> | <strong>English</strong> | <a href="ICON_DESIGN_HISTORY.ja.md">日本語</a>
</p>

> **Public Repository Record**
>
> This document records the development history, AI-assisted similarity reviews, design changes, and provenance
> of the icons used by the four applications Fluffy, Waddle, Rooney, and Toodle.
>
> **Scope and Limitations:** This document is a project development-history and due-diligence record.
> It is not a legal opinion, trademark clearance, copyright clearance, design-rights clearance, or a guarantee
> that no third-party rights are implicated. Definitive expressions appearing in the historical record, such as
> “Zero Risk,” “Approved,” and “Fully Compliant,” are outputs of AI-generated reviews preserved as historical records
> and **are not adopted as legal conclusions in this document**.
>
> The purpose of this document is to record the process of icon design and provenance in a transparent
> and traceable manner.

## How to Read This Public Record

- **Past AI outputs** are preserved as historical records to the extent necessary for provenance. They should not be interpreted as legal advice or as indicating that the project has obtained legal clearance.
- **Project actions** are recorded in terms of which design elements were removed, retained, or redesigned when similarity concerns were identified.
- **Current repository-facing records** are the individual `IP_COMPLIANCE.md` files placed in the Fluffy, Waddle, Rooney, and Toodle projects.

# AI Generation and Intellectual Property (IP) Audit History for the Four Applications Under Development

This document is an integrated chronological record covering the process from the initial conversations with Gemini, through icon similarity reviews and preparation of summary documents, to the final due-diligence records.

## 1. Common Design Policy and Audit Criteria

- **Unified design language:**
  - A **matte, flat, and geometric design** that harmonizes with the COSMIC Desktop Environment (libcosmic) and Linux Wayland environments, eliminating excessive gloss, synthetic glow, and fake 3D shadows (embossing).
  - Depending on the utility of the application, either solid flat charcoal backgrounds or transparent backgrounds with floating monolithic slates are utilized.
- **IP compliance (provenance) management:**
  - To prevent rights-related issues specific to AI-assisted assets and rejection during store reviews (such as Flathub and COSMIC Store) due to confusion with third-party assets, an `IP_COMPLIANCE.md` documenting provenance is deployed in each project's repository.

## 2. History, Similarity Findings, and Regeneration Evaluation for Each Application

### ① Fluffy (Video Wallpaper Manager for Linux / COSMIC DE)

#### 1. Initial Generation and Similarity Findings
- **Initial design:** A modern icon with a play symbol (▶) placed inside a flowing, cloud- or liquid-like form.
- **Similarity findings by Gemini:**
  - **Fluid Video Player / Video Cloud / IRIS player / QNAP cloud surveillance:** Placing a play symbol within liquid or cloud silhouettes was noted as common visual conventions in the video/cloud application category.

#### 2. Regeneration Process
1. **Direction shift:** Combined the initial letter “F” with video timeline/frame geometry.
2. **Separation from official COSMIC branding:** Removed multi-ring orange elements from an intermediate draft that resembled System76 COSMIC orbital lines.
3. **Final composition:** An F-shaped frame rendered in matte-white lines enclosing a solid cyan-blue “display + play” symbol.

---

### ② Waddle (Fully Local AI-Integrated Terminal Emulator)

#### 1. Initial Design and Similarity Findings
- **Initial design:** A Linux penguin (Tux) with outspread arms and a lightning bolt against a blue/green “W” background.
- **Similarity findings by Gemini:**
  - **WSL (Windows Subsystem for Linux):** The “W + Tux” composition was evaluated as strongly similar to WSL visual identity.
  - **WezTerm:** Shared terminal category and initial letter “W”.

#### 2. Regeneration Process
- **Complete removal of Tux:** Eliminated the penguin mascot causing potential confusion with WSL.
- **CLI prompt integration:** Replaced font glyphs with geometric segments forming a “W” derived from terminal prompts (`>`) and code lines, centered with a minimal lightning glyph.

---

### ③ Rooney (AI-Integrated Translucent 2-Pane Markdown Editor for COSMIC / Wayland)

#### 1. Initial Generation to Second Generation
- **Initial design:** Large gear in the background, `</>` code screen in the center, and a bold “R” at the base.
  - **Similarity finding:** Identified as strongly resembling the official Rust programming language logo. The gear was promptly removed.
- **Second generation:** A flat geometric monogram on solid charcoal representing the internal “Rope” buffer and 2-pane split-view.

#### 2. Latest Redesign (Transparent Background, Dual Slates, Amber Slit Backlight)
To directly embody the editor's core attributes—translucency and 2-pane editing (source & preview)—the icon was completely re-architected into a custom geometric monolith:

1. **Transparent Background & Dual Slate Plates:**
   - Outer container bounding boxes were eliminated in favor of full background transparency. Two independent floating matte-slate plates (`#1b2028` / `#1e2530`) represent the 2-pane split view, with the right plate featuring a frosted-glass translucent bleed layer.
2. **Pop!_OS Amber Emissive Slit:**
   - A high-intensity laser/backlight beam in Pop!_OS / COSMIC signature amber orange (`#ff7900` to `#ffb300`) leaks through the vertical chasm between the plates, visually demonstrating translucency.
3. **Ultrawide Low-Profile Heavy-Bold “R”:**
   - Spanning both plates and bridging the luminous slit, a custom geometric monogram was constructed with an extreme 2:1 vertical ratio (elongated bowl over stubby legs), an ultra-thick vertical stem (~26px), and a heavily weighted right foot.

#### 3. Similarity Review for the New Icon
- **Rust Programming Language Logo:** No gear motifs exist. The monogram is an ultrawide flat geometric monolith, presenting zero visual or trademark confusion.
- **RStudio / Posit:** RStudio uses a circular blue badge with a thin white R. Rooney employs a transparent background, dual matte-slate plates, an amber neon slit, and an ultra-bold custom R, distinct in every visual aspect.
- **Raycast:** Raycast features a red polygon prism with no letter “R” or split-plate layout.
- **COSMIC Native App Icons:** Respects the system color palette while providing a clear third-party silhouette distinct from System76’s single-color flat line-art icons.

---

### ④ Toodle (Modern Digital Clock & Weather Widget for COSMIC Desktop)

#### 1. Initial Design and Similarity Findings
- **Initial design:** White monogram combining “T” and clock dial on charcoal, with a glowing sun in the upper right.
- **Similarity findings by Gemini:**
  - **Toodledo / Todoist:** Prefix and category overlap with time-management tools.
  - **Design issues:** 3D embossing and outer glow deviated from flat/matte guidelines.

#### 2. Adjustments
- Replaced glowing sun gradients with a flat solid yellow disc and removed pseudo-3D drop shadows.
- Retained the compact “T” × clock dial × solid sun minimalist structure.

---

## 3. Revision Process Through External Review

External review from legal and IP due-diligence perspectives established key guidelines:
- **Clarification of AI overstatements:** Replaced terms like “Zero Risk” or “Approved” with factual descriptions, clarifying that conversational AI remarks do not constitute formal legal opinions.
- **Clear Scope & Limitations:** Stated explicitly that `IP_COMPLIANCE.md` records project provenance and diligence rather than formal worldwide trademark registration.

## 4. Division of Roles Between Records

```text
ICON_DESIGN_HISTORY.md / .ja.md
    ↓
Integrated history of design changes and provenance across all four applications

IP_COMPLIANCE.md / .ja.md
    ↓
Current repository-facing Due Diligence Record based on current asset specifications
```

---

## 5. Repository-Deployed IP_COMPLIANCE.md Status

### ① Waddle
- **Concept:** Terminal prompt (`>`) + code line geometric “W” with lightning glyph.
- **Review Date:** 2026-09-28

### ② Rooney (Latest Revision)
- **Concept:** Transparent background, dual matte-slate plates, central Pop!_OS amber emissive slit, ultrawide low-profile heavy-bold “R” (2:1 ratio).
- **Review Date:** 2026-10-31

### ③ Toodle
- **Concept:** Minimalist “T” × clock dial × solid yellow sun.
- **Review Date:** 2026-09-28

### ④ Fluffy
- **Concept:** Flowing “F” frame + solid cyan display/play symbol.
- **Review Date:** 2026-09-28

---

## 6. Update Policy

This document and the relevant `IP_COMPLIANCE.md` files shall be updated when:
- Major structural or design changes are made to an icon;
- New third-party assets are incorporated;
- New similarity or branding concerns are identified;
- Existing provenance details require correction or supplementation.

**Review date:** 2026-10-31