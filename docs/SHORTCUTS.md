# ⌨️ Rooney Shortcuts & Operations Guide

A comprehensive reference for keyboard shortcuts and interaction models in Rooney.  
Rooney is engineered around a fluid, keyboard-driven development experience across file management, dual-pane splitting, local AI assistance, and precision editing.

---

## 📑 Contents
1. [File & Tab Operations](#1-file--tab-operations)
2. [Window & Pane Layouts](#2-window--pane-layouts)
3. [Editing & Caret Navigation](#3-editing--caret-navigation)
4. [Search & Navigation](#4-search--navigation)
5. [Markdown Preview Controls](#5-markdown-preview-controls)
6. [Offline Local AI Operations](#6-offline-local-ai-operations)
7. [Preferences & Dialog Interactions](#7-preferences--dialog-interactions)

---

## 1. File & Tab Operations

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + N` | **New File** | Prompt modal to create a new file in the active workspace |
| `Ctrl + O` | **Open File** | Open XDG Desktop Portal native file picker |
| `Ctrl + Shift + O` | **Open Folder** | Select workspace directory and reload file tree |
| `Ctrl + S` | **Save File** | Atomic safe write via temporary file replacement |
| `Ctrl + Shift + S` | **Save As...** | Save buffer to a new path and reinitialize language parser |
| `Ctrl + T` | **New Tab** | Create a new untitled buffer |
| `Ctrl + W` | **Close Tab** | Close currently active buffer tab |
| `Ctrl + Tab` | **Next Tab** | Cycle forward through open tabs |
| `Ctrl + Shift + Tab` | **Previous Tab** | Cycle backward through open tabs |

---

## 2. Window & Pane Layouts

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + \` or `Ctrl + E` | **Toggle Dual Split** | Toggle between side-by-side 2-pane split and single-pane view |
| `Ctrl + B` | **Toggle Sidebar** | Show / hide left file tree explorer |
| `Ctrl + ,` | **Aesthetics & Preferences**| Open modal to adjust themes, opacities, fonts, AI models, and GFM |
| `Esc` | **Dismiss / Cancel** | Close open modals, search bar, context menus, or ghost suggestions |

---

## 3. Editing & Caret Navigation

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + Z` | **Undo** | Undo the most recent edit action |
| `Ctrl + Y` / `Ctrl + Shift + Z` | **Redo** | Replay undone edit actions |
| `Ctrl + A` | **Select All** | Select all content across the entire buffer |
| `Ctrl + C` | **Copy** | Copy selected text (or entire current line if no selection) |
| `Ctrl + X` | **Cut** | Cut selected text (or entire current line if no selection) |
| `Ctrl + V` | **Paste** | Insert clipboard text at caret |
| `Ctrl + /` | **Toggle Comment** | Language-aware line commenting / uncommenting |
| `Ctrl + D` | **Duplicate Line** | Duplicate the current line directly below |
| `Ctrl + Shift + K` | **Delete Line** | Delete current line and pull up subsequent lines |
| `Tab` | **Indent** | Insert spaces or indent selected block of lines |
| `Shift + Tab` | **Unindent** | Remove leading indentation |
| `Home` / `End` | **Line Start / End** | Move caret to start or end of current line |
| `Ctrl + Home` / `Ctrl + End` | **Buffer Top / Bottom** | Move caret to beginning or end of entire file |
| `PageUp` / `PageDown` | **Page Scroll** | Rapid scroll by viewport page heights |

---

## 4. Search & Navigation

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + F` | **Toggle In-Buffer Search** | Open incremental search bar at editor bottom |
| `Enter` (within search bar) | **Next Match** | Focus jump to next search result |
| `Shift + Enter` (within search bar)| **Previous Match** | Focus jump to previous search result |
| `Esc` (within search bar) | **Close Search** | Dismiss search bar and restore focus to canvas |

---

## 5. Markdown Preview Controls

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + M` | **Toggle Markdown Preview**| Open live GFM / CommonMark native preview in right pane |
| Preferences Modal | **Markdown Spec Switch** | Dynamically switch between **GFM** and standard **CommonMark** |

*Note: Markdown rendering strictly enforces a **Zero Remote I/O** policy with no external image downloading, webview overhead, or remote network calls.*

---

## 6. Offline Local AI Operations

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + I` or `Alt + Enter` | **Trigger FIM Completion** | Request inline Fill-in-the-Middle ghost suggestions via Ollama |
| `Tab` (when suggestion visible) | **Accept Suggestion** | Insert ghost text into document |
| `Esc` (when suggestion visible) | **Dismiss Suggestion** | Clear active ghost suggestion |
| `Ctrl + Shift + A` | **Toggle AI Chat Panel** | Open interactive streaming chat assistant on the right pane |

---

## 7. Preferences & Dialog Interactions

- **Themes & Window Opacities**: Press `Ctrl + ,` to choose from 20 bundled color schemes, tune independent opacities for the editor canvas, file tree, and title bar, set background dimming, and choose Ollama models.
- **File Explorer Context Menus**: Right-click any file or directory in the tree to create new files/folders, rename items, or safely delete files.

---

<p align="center">
  <a href="PORTAL.md">← Back to Portal</a> | <a href="../README.md">Back to Root README</a>
</p>
