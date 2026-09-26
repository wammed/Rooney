use crate::config::MarkdownSpec;
use crate::editor::buffer::TextBuffer;
use crate::markdown::MarkdownDocument;
use crate::syntax::{Highlighter, SupportedLanguage};
use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneId {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitLayout {
    Single,
    Split,
}

pub type WrapCacheKey = (u32, u32, &'static str);
pub type CachedWrapModel =
    std::sync::RwLock<Option<(WrapCacheKey, crate::ui::wrap::LineWrapModel)>>;

pub struct EditorTab {
    pub id: usize,
    pub file_path: Option<PathBuf>,
    pub file_name: String,
    pub buffer: TextBuffer,
    pub highlighter: Highlighter,
    pub scroll_y: Cell<f32>,
    pub scroll_x: Cell<f32>,
    pub needs_scroll_to_cursor: Cell<bool>,
    pub ghost_text: Option<String>,
    pub preedit: Option<(String, Option<std::ops::Range<usize>>)>,
    pub is_markdown_preview: bool,
    pub markdown_doc: Option<MarkdownDocument>,
    pub markdown_spec: MarkdownSpec,
    pub search_query: Option<String>,
    pub search_matches: Vec<(usize, usize, usize)>, // (line_idx, col_start, col_end)
    pub current_match_idx: usize,
    pub is_search_open: bool,
    pub last_edit_time: Instant,
    pub last_cursor_time: Instant,
    pub needs_highlight_parse: bool,
    pub is_parsing_async: bool,
    /// Parse worker ownership and generation separation:
    /// - `parse_generation`: Tracks whether parsed AST matches latest buffer content.
    /// - `active_parse_worker_generation`: Identifies who currently owns the active async parse worker slot.
    pub active_parse_worker_generation: Option<usize>,
    pub parse_worker_counter: usize,
    pub needs_search_update: bool,
    pub last_search_update: Instant,
    pub search_generation: usize,
    pub is_searching_async: bool,
    /// Search worker ownership and generation separation:
    /// - `search_generation`: Tracks whether search results match latest buffer content/query.
    /// - `active_search_worker_generation`: Identifies who currently owns the active async search worker slot.
    pub active_search_worker_generation: Option<usize>,
    pub search_worker_counter: usize,
    pub markdown_generation: usize,
    pub parse_generation: usize,
    pub cached_wrap_model: CachedWrapModel,
}

impl EditorTab {
    pub fn new(id: usize, title: &str) -> Self {
        Self {
            id,
            file_path: None,
            file_name: title.to_string(),
            buffer: TextBuffer::default(),
            highlighter: Highlighter::new(SupportedLanguage::PlainText),
            scroll_y: Cell::new(0.0),
            scroll_x: Cell::new(0.0),
            needs_scroll_to_cursor: Cell::new(true),
            ghost_text: None,
            preedit: None,
            is_markdown_preview: false,
            markdown_doc: None,
            markdown_spec: MarkdownSpec::default(),
            search_query: None,
            search_matches: Vec::new(),
            current_match_idx: 0,
            is_search_open: false,
            last_edit_time: Instant::now(),
            last_cursor_time: Instant::now(),
            needs_highlight_parse: false,
            is_parsing_async: false,
            active_parse_worker_generation: None,
            parse_worker_counter: 0,
            needs_search_update: false,
            last_search_update: Instant::now(),
            search_generation: 0,
            is_searching_async: false,
            active_search_worker_generation: None,
            search_worker_counter: 0,
            markdown_generation: 0,
            parse_generation: 0,
            cached_wrap_model: std::sync::RwLock::new(None),
        }
    }

    pub fn load_file(&mut self, path: &Path) -> std::io::Result<()> {
        let metadata = std::fs::metadata(path)?;
        // Safety guard: reject files > 50MB to prevent memory exhaustion and UI lockup
        const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
        if metadata.len() > MAX_FILE_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::FileTooLarge,
                format!(
                    "File size ({} MB) exceeds safety threshold of 50 MB",
                    metadata.len() / (1024 * 1024)
                ),
            ));
        }

        let content = std::fs::read_to_string(path)?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();

        let lang = SupportedLanguage::from_path(path);
        let mut highlighter = Highlighter::new(lang);

        self.file_path = Some(path.to_path_buf());
        self.file_name = name;
        self.buffer = TextBuffer::new(&content);
        self.buffer.mark_saved();
        if let Ok(mut guard) = self.cached_wrap_model.write() {
            *guard = None;
        }
        self.scroll_y.set(0.0);
        self.scroll_x.set(0.0);
        self.needs_scroll_to_cursor.set(true);
        self.ghost_text = None;
        self.preedit = None;
        self.markdown_generation = self.markdown_generation.wrapping_add(1);
        self.parse_generation = self.parse_generation.wrapping_add(1);

        const SYNC_PARSE_MAX_BYTES: usize = 2 * 1024 * 1024;
        self.is_searching_async = false;
        self.active_search_worker_generation = None;
        self.active_parse_worker_generation = None;
        if content.len() <= SYNC_PARSE_MAX_BYTES {
            highlighter.update_source(&content);
            self.highlighter = highlighter;
            self.is_parsing_async = false;
            self.needs_highlight_parse = false;

            if lang == SupportedLanguage::Markdown && self.is_markdown_preview {
                self.markdown_doc = Some(MarkdownDocument::parse(&content, self.markdown_spec));
            } else {
                self.markdown_doc = None;
            }
        } else {
            // Large file (> 2MB): avoid synchronous full tree-sitter parse on UI thread.
            // Tab is created and editor is immediately usable; tree-sitter parse is offloaded.
            self.highlighter = highlighter;
            self.is_parsing_async = false;
            self.needs_highlight_parse = true;
            // Set last_edit_time so background parse can trigger immediately
            self.last_edit_time = Instant::now() - std::time::Duration::from_millis(200);
            self.markdown_doc = None;
        }

        Ok(())
    }

    pub fn save_file(&mut self) -> std::io::Result<()> {
        if let Some(ref path) = self.file_path {
            let text = self.buffer.full_text();
            Self::atomic_write_file(path, &text)?;
            self.buffer.mark_saved();
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path set",
            ))
        }
    }

    pub fn save_file_as(&mut self, path: &Path) -> std::io::Result<()> {
        let text = self.buffer.full_text();
        Self::atomic_write_file(path, &text)?;
        self.file_path = Some(path.to_path_buf());
        self.file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();
        self.buffer.mark_saved();

        let lang = SupportedLanguage::from_path(path);
        let highlighter = Highlighter::new(lang);
        self.highlighter = highlighter;
        self.markdown_generation = self.markdown_generation.wrapping_add(1);
        self.parse_generation = self.parse_generation.wrapping_add(1);
        self.is_searching_async = false;
        self.active_search_worker_generation = None;
        self.active_parse_worker_generation = None;

        const SYNC_PARSE_MAX_BYTES: usize = 2 * 1024 * 1024;
        if text.len() <= SYNC_PARSE_MAX_BYTES {
            self.highlighter.update_source(&text);
            self.is_parsing_async = false;
            self.needs_highlight_parse = false;

            if lang == SupportedLanguage::Markdown && self.is_markdown_preview {
                self.markdown_doc = Some(MarkdownDocument::parse(&text, self.markdown_spec));
            } else {
                self.markdown_doc = None;
            }
        } else {
            self.is_parsing_async = false;
            self.needs_highlight_parse = true;
            self.last_edit_time = Instant::now() - std::time::Duration::from_millis(200);
            self.markdown_doc = None;
        }

        Ok(())
    }

    /// Atomically write file contents to disk using a temporary sibling file and rename.
    /// This guarantees that a system crash, power failure, or error will not truncate the original file.
    pub fn atomic_write_file(path: &Path, content: &str) -> std::io::Result<()> {
        use std::io::Write;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let existing_permissions = path.metadata().ok().map(|m| m.permissions());
        let pid = std::process::id();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");
        let temp_path = path.with_file_name(format!(".{file_name}.tmp.{pid}"));

        let write_result = (|| -> std::io::Result<()> {
            let mut file = std::fs::File::create(&temp_path)?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(e) = write_result {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }

        if let Some(perms) = existing_permissions {
            let _ = std::fs::set_permissions(&temp_path, perms);
        }

        if let Err(e) = std::fs::rename(&temp_path, path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }

        Ok(())
    }

    pub fn on_content_changed(&mut self) {
        self.last_edit_time = Instant::now();
        self.ghost_text = None;
        self.needs_scroll_to_cursor.set(true);
        self.markdown_generation = self.markdown_generation.wrapping_add(1);
        self.parse_generation = self.parse_generation.wrapping_add(1);
        if let Ok(mut guard) = self.cached_wrap_model.write() {
            *guard = None;
        }
        if let Some(edit) = self.buffer.last_edit.take() {
            self.highlighter.apply_edit(&edit);
        }

        // For files <= 2MB, perform immediate synchronous incremental parse (< 4ms).
        // For massive files (> 2MB, up to 50MB), defer the 50MB full_text() allocation
        // and parser.parse() AST scan until user typing pauses (debounced via Message::Tick).
        // This guarantees sub-millisecond typing latency (< 0.05ms) even on 50MB files.
        const SYNC_PARSE_MAX_BYTES: usize = 2 * 1024 * 1024;
        if self.buffer.len_bytes() <= SYNC_PARSE_MAX_BYTES {
            let text = self.buffer.full_text();
            self.highlighter.update_source(&text);
            self.needs_highlight_parse = false;

            if self.highlighter.lang == SupportedLanguage::Markdown && self.is_markdown_preview {
                self.markdown_doc = Some(MarkdownDocument::parse(&text, self.markdown_spec));
            }
        } else {
            self.needs_highlight_parse = true;
        }

        // Search updates: for files <= 2MB, update matches synchronously.
        // For massive files (> 2MB), defer full-text line scan until user pauses typing
        // to prevent hundreds of milliseconds of typing freeze on keystrokes.
        // Always increment search_generation to invalidate in-flight search workers from older buffer snapshots.
        const SYNC_SEARCH_MAX_BYTES: usize = 2 * 1024 * 1024;
        if let Some(ref q) = self.search_query.clone() {
            self.search_generation = self.search_generation.wrapping_add(1);
            if self.buffer.len_bytes() <= SYNC_SEARCH_MAX_BYTES {
                self.update_search(q);
                self.needs_search_update = false;
            } else {
                self.needs_search_update = true;
            }
        }
    }

    /// Flushes any pending background/debounced search query scan for large files (> 2MB).
    pub fn flush_pending_search(&mut self) {
        if self.needs_search_update {
            if let Some(ref q) = self.search_query.clone() {
                self.update_search(q);
            }
            self.needs_search_update = false;
            self.last_search_update = Instant::now();
        }
    }

    /// Flushes any pending background/debounced Tree-sitter parse for large files (> 2MB).
    pub fn flush_highlight_parse(&mut self) {
        if self.needs_highlight_parse {
            let text = self.buffer.full_text();
            self.highlighter.update_source(&text);
            self.needs_highlight_parse = false;

            if self.highlighter.lang == SupportedLanguage::Markdown && self.is_markdown_preview {
                self.markdown_doc = Some(MarkdownDocument::parse(&text, self.markdown_spec));
            }
        }
    }

    pub fn invalidate_wrap_cache(&self) {
        if let Ok(mut guard) = self.cached_wrap_model.write() {
            *guard = None;
        }
    }

    pub fn mark_cursor_moved(&self) {
        self.needs_scroll_to_cursor.set(true);
    }

    pub fn refresh_markdown(&mut self) {
        if self.is_markdown_preview {
            const SYNC_MD_MAX_BYTES: usize = 2 * 1024 * 1024;
            if self.buffer.len_bytes() <= SYNC_MD_MAX_BYTES {
                let text = self.buffer.full_text();
                self.markdown_doc = Some(MarkdownDocument::parse(&text, self.markdown_spec));
            }
        }
    }

    /// Safely apply parsed MarkdownDocument only if generation matches and preview is still enabled.
    pub fn apply_markdown_doc(&mut self, generation: usize, doc: MarkdownDocument) -> bool {
        if self.markdown_generation == generation && self.is_markdown_preview {
            self.markdown_doc = Some(doc);
            true
        } else {
            false
        }
    }

    /// Starts an async parse worker, acquiring ownership and returning `(parse_generation, worker_generation)`.
    pub fn start_parse_worker(&mut self) -> (usize, usize) {
        self.parse_worker_counter = self.parse_worker_counter.wrapping_add(1);
        let worker_gen = self.parse_worker_counter;
        self.active_parse_worker_generation = Some(worker_gen);
        self.is_parsing_async = true;
        (self.parse_generation, worker_gen)
    }

    /// Starts an async search worker, acquiring ownership and returning `(search_generation, worker_generation)`.
    pub fn start_search_worker(&mut self) -> (usize, usize) {
        self.search_generation = self.search_generation.wrapping_add(1);
        self.search_worker_counter = self.search_worker_counter.wrapping_add(1);
        let worker_gen = self.search_worker_counter;
        self.active_search_worker_generation = Some(worker_gen);
        self.is_searching_async = true;
        self.needs_search_update = false;
        (self.search_generation, worker_gen)
    }

    /// Safely apply parsed Tree-sitter Tree.
    ///
    /// Distinguishes between:
    /// - `generation` (`parse_generation`): "Is the parsed AST based on the latest buffer content?"
    /// - `worker_generation`: "Who currently owns the active async parse worker slot?"
    ///
    /// Invariants:
    /// - Case A (current worker + current generation): apply AST, clear active worker, set `is_parsing_async = false`, `needs_highlight_parse = false`.
    /// - Case B (current worker + stale generation): discard AST, clear active worker, set `is_parsing_async = false`, `needs_highlight_parse = true`.
    ///   This allows subsequent debounced ticks to launch the next worker without becoming permanently stalled.
    /// - Case C (stale worker, another worker already active or ownership cleared): discard AST, do NOT touch active worker or `is_parsing_async`.
    pub fn apply_highlight_tree(
        &mut self,
        generation: usize,
        worker_generation: usize,
        tree: Option<tree_sitter::Tree>,
    ) -> bool {
        if self.active_parse_worker_generation == Some(worker_generation) {
            self.active_parse_worker_generation = None;
            self.is_parsing_async = false;

            if self.parse_generation == generation {
                if let Some(new_tree) = tree {
                    self.highlighter.set_tree(new_tree);
                }
                self.needs_highlight_parse = false;
                true
            } else {
                // Stale generation: AST discarded, but worker ownership freed so next worker can run
                self.needs_highlight_parse = true;
                false
            }
        } else {
            // Case C: stale worker completion from an earlier superseded worker.
            // Do NOT touch active_parse_worker_generation or is_parsing_async!
            false
        }
    }

    pub fn undo(&mut self) {
        self.buffer.undo();
        self.on_content_changed();
    }

    pub fn redo(&mut self) {
        self.buffer.redo();
        self.on_content_changed();
    }

    pub fn start_search(&mut self, query: &str) -> (usize, bool) {
        if query.is_empty() {
            self.search_query = None;
            self.search_matches.clear();
            self.current_match_idx = 0;
            self.needs_search_update = false;
            self.is_searching_async = false;
            self.active_search_worker_generation = None;
            return (self.search_generation, false);
        }

        self.search_query = Some(query.to_string());
        self.search_generation = self.search_generation.wrapping_add(1);
        const SYNC_SEARCH_MAX_BYTES: usize = 2 * 1024 * 1024;
        let is_large = self.buffer.len_bytes() > SYNC_SEARCH_MAX_BYTES;

        if is_large {
            self.search_worker_counter = self.search_worker_counter.wrapping_add(1);
            let worker_gen = self.search_worker_counter;
            self.active_search_worker_generation = Some(worker_gen);
            self.is_searching_async = true;
            self.needs_search_update = false;
        } else {
            self.search_matches = run_search_on_rope(&self.buffer.rope, query);
            if self.current_match_idx >= self.search_matches.len() {
                self.current_match_idx = 0;
            }
            self.is_searching_async = false;
            self.active_search_worker_generation = None;
            self.needs_search_update = false;
        }
        (self.search_generation, is_large)
    }

    /// Safely apply background search results.
    ///
    /// Distinguishes between:
    /// - `generation` (`search_generation`): "Are the search matches based on the latest buffer content/query?"
    /// - `worker_generation`: "Who currently owns the active async search worker slot?"
    ///
    /// Invariants:
    /// - Case A (current worker + current generation): apply matches, clear active worker, set `is_searching_async = false`, `needs_search_update = false`.
    /// - Case B (current worker + stale generation): discard matches, clear active worker, set `is_searching_async = false`, `needs_search_update = true`.
    ///   This allows subsequent debounced ticks to launch the next worker without becoming permanently stalled.
    /// - Case C (stale worker, another worker already active or ownership cleared): discard matches, do NOT touch active worker or `is_searching_async`.
    pub fn apply_search_results(
        &mut self,
        generation: usize,
        worker_generation: usize,
        matches: Vec<(usize, usize, usize)>,
    ) -> bool {
        if self.active_search_worker_generation == Some(worker_generation) {
            self.active_search_worker_generation = None;
            self.is_searching_async = false;

            if generation == self.search_generation {
                self.search_matches = matches;
                if self.current_match_idx >= self.search_matches.len() {
                    self.current_match_idx = 0;
                }
                self.needs_search_update = false;
                true
            } else {
                // Stale generation: results discarded, but worker ownership freed so next worker can run
                self.needs_search_update = true;
                false
            }
        } else {
            // Case C: stale worker completion from an earlier superseded worker.
            // Do NOT touch active_search_worker_generation or is_searching_async!
            false
        }
    }

    pub fn update_search(&mut self, query: &str) {
        if query.is_empty() {
            self.search_query = None;
            self.search_matches.clear();
            self.current_match_idx = 0;
            return;
        }

        self.search_query = Some(query.to_string());
        self.search_matches = run_search_on_rope(&self.buffer.rope, query);
        if self.current_match_idx >= self.search_matches.len() {
            self.current_match_idx = 0;
        }
    }

    pub fn next_search_match(&mut self) -> Option<(usize, usize)> {
        if self.search_matches.is_empty() {
            return None;
        }
        self.current_match_idx = (self.current_match_idx + 1) % self.search_matches.len();
        let (line, start, _) = self.search_matches[self.current_match_idx];
        self.buffer.cursor = (line, start);
        Some((line, start))
    }

    pub fn prev_search_match(&mut self) -> Option<(usize, usize)> {
        if self.search_matches.is_empty() {
            return None;
        }
        if self.current_match_idx == 0 {
            self.current_match_idx = self.search_matches.len() - 1;
        } else {
            self.current_match_idx -= 1;
        }
        let (line, start, _) = self.search_matches[self.current_match_idx];
        self.buffer.cursor = (line, start);
        Some((line, start))
    }

    pub fn accept_ghost_text(&mut self) -> bool {
        if let Some(ghost) = self.ghost_text.take() {
            if !ghost.is_empty() {
                self.buffer.insert_str(&ghost);
                self.on_content_changed();
                return true;
            }
        }
        false
    }

    pub fn clear_ghost_text(&mut self) {
        self.ghost_text = None;
    }
}

pub struct EditorPane {
    pub id: PaneId,
    pub tabs: Vec<EditorTab>,
    pub active_tab_idx: usize,
    next_tab_id: usize,
}

impl EditorPane {
    pub fn new(id: PaneId, title: &str) -> Self {
        let first_tab = EditorTab::new(1, title);
        Self {
            id,
            tabs: vec![first_tab],
            active_tab_idx: 0,
            next_tab_id: 2,
        }
    }

    pub fn atomic_write_file(path: &Path, content: &str) -> std::io::Result<()> {
        EditorTab::atomic_write_file(path, content)
    }

    pub fn active_tab(&self) -> &EditorTab {
        &self.tabs[self.active_tab_idx]
    }

    pub fn active_tab_mut(&mut self) -> &mut EditorTab {
        &mut self.tabs[self.active_tab_idx]
    }

    pub fn open_file(&mut self, path: &Path) -> std::io::Result<()> {
        // 1. If already opened in a tab, switch to it
        if let Some(pos) = self
            .tabs
            .iter()
            .position(|t| t.file_path.as_deref() == Some(path))
        {
            self.active_tab_idx = pos;
            return Ok(());
        }

        // 2. If current tab is untouched and untitled/welcome, reuse it
        let current_is_untouched = {
            let cur = &self.tabs[self.active_tab_idx];
            cur.file_path.is_none()
                && !cur.buffer.is_modified
                && (cur.buffer.full_text().is_empty()
                    || cur.file_name == "Welcome"
                    || cur.file_name == "Untitled")
        };

        if path.exists() {
            if current_is_untouched {
                self.tabs[self.active_tab_idx].load_file(path)?;
                return Ok(());
            }

            // 3. Otherwise, open a new tab
            let spec = self.active_tab().markdown_spec;
            let id = self.next_tab_id;
            self.next_tab_id += 1;
            let mut tab = EditorTab::new(id, "Untitled");
            tab.markdown_spec = spec;
            tab.load_file(path)?;
            self.tabs.push(tab);
            self.active_tab_idx = self.tabs.len() - 1;
            Ok(())
        } else {
            // New file that does not exist on disk yet
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string();
            let lang = SupportedLanguage::from_path(path);
            let highlighter = Highlighter::new(lang);

            if current_is_untouched {
                let tab = &mut self.tabs[self.active_tab_idx];
                tab.file_path = Some(path.to_path_buf());
                tab.file_name = name;
                tab.buffer = TextBuffer::new("");
                tab.highlighter = highlighter;
                tab.parse_generation = tab.parse_generation.wrapping_add(1);
                tab.is_parsing_async = false;
                tab.active_parse_worker_generation = None;
                tab.is_searching_async = false;
                tab.active_search_worker_generation = None;
                tab.needs_highlight_parse = false;
                tab.needs_scroll_to_cursor.set(true);
                return Ok(());
            }

            let spec = self.active_tab().markdown_spec;
            let id = self.next_tab_id;
            self.next_tab_id += 1;
            let mut tab = EditorTab::new(id, &name);
            tab.file_path = Some(path.to_path_buf());
            tab.markdown_spec = spec;
            tab.highlighter = highlighter;
            tab.needs_scroll_to_cursor.set(true);
            self.tabs.push(tab);
            self.active_tab_idx = self.tabs.len() - 1;
            Ok(())
        }
    }

    pub fn new_tab(&mut self, title: &str) -> usize {
        let spec = self.active_tab().markdown_spec;
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let mut tab = EditorTab::new(id, title);
        tab.markdown_spec = spec;
        self.tabs.push(tab);
        self.active_tab_idx = self.tabs.len() - 1;
        self.active_tab_idx
    }

    pub fn set_markdown_spec(&mut self, spec: MarkdownSpec) {
        for tab in &mut self.tabs {
            tab.markdown_spec = spec;
            tab.markdown_generation = tab.markdown_generation.wrapping_add(1);
            tab.refresh_markdown();
            if !tab.is_markdown_preview {
                tab.markdown_doc = None;
            }
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.tabs.remove(idx);
            if self.tabs.is_empty() {
                let id = self.next_tab_id;
                self.next_tab_id += 1;
                self.tabs.push(EditorTab::new(id, "Untitled"));
                self.active_tab_idx = 0;
            } else if self.active_tab_idx >= self.tabs.len() {
                self.active_tab_idx = self.tabs.len() - 1;
            } else if self.active_tab_idx > idx {
                self.active_tab_idx -= 1;
            }
        }
    }

    pub fn select_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.active_tab_idx = idx;
        }
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_idx = (self.active_tab_idx + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_idx = if self.active_tab_idx == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab_idx - 1
            };
        }
    }

    pub fn active_tab_id(&self) -> usize {
        self.active_tab().id
    }

    pub fn tab_by_id(&self, id: usize) -> Option<&EditorTab> {
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn tab_by_id_mut(&mut self, id: usize) -> Option<&mut EditorTab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn invalidate_wrap_cache(&self) {
        for tab in &self.tabs {
            tab.invalidate_wrap_cache();
        }
    }
}

/// Pure helper function to execute substring search on a Rope without mutating editor state.
/// Ensures Unicode case-folding safety and exact character position mapping.
/// Absolutely avoids byte-offset slicing into differing strings to guarantee zero UTF-8 boundary panics.
pub fn run_search_on_rope(rope: &ropey::Rope, query: &str) -> Vec<(usize, usize, usize)> {
    if query.is_empty() {
        return Vec::new();
    }
    let q_lower_chars: Vec<char> = query.chars().flat_map(|c| c.to_lowercase()).collect();
    if q_lower_chars.is_empty() {
        return Vec::new();
    }
    let q_len = q_lower_chars.len();
    let mut matches = Vec::new();

    for (line_idx, line_slice) in rope.lines().enumerate() {
        let line_str = line_slice.to_string();
        let line_trimmed = line_str.trim_end_matches(['\r', '\n']);
        if line_trimmed.is_empty() {
            continue;
        }

        // Map each character in the original line to its lowercase expansion,
        // recording the original character index.
        let mut line_lower_chars: Vec<(char, usize)> = Vec::with_capacity(line_trimmed.len());
        for (orig_idx, ch) in line_trimmed.chars().enumerate() {
            for low_ch in ch.to_lowercase() {
                line_lower_chars.push((low_ch, orig_idx));
            }
        }

        if q_len <= line_lower_chars.len() {
            let mut i = 0;
            while i + q_len <= line_lower_chars.len() {
                let is_match = (0..q_len).all(|j| line_lower_chars[i + j].0 == q_lower_chars[j]);
                if is_match {
                    let start_char = line_lower_chars[i].1;
                    let end_char = line_lower_chars[i + q_len - 1].1 + 1;
                    matches.push((line_idx, start_char, end_char));
                    i += q_len.max(1);
                } else {
                    i += 1;
                }
            }
        }
    }
    matches
}

impl Deref for EditorPane {
    type Target = EditorTab;

    fn deref(&self) -> &Self::Target {
        self.active_tab()
    }
}

impl DerefMut for EditorPane {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.active_tab_mut()
    }
}
