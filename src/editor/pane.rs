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
        highlighter.update_source(&content);

        self.file_path = Some(path.to_path_buf());
        self.file_name = name;
        self.buffer = TextBuffer::new(&content);
        self.highlighter = highlighter;
        self.scroll_y.set(0.0);
        self.scroll_x.set(0.0);
        self.needs_scroll_to_cursor.set(true);
        self.ghost_text = None;
        self.preedit = None;

        if lang == SupportedLanguage::Markdown && self.is_markdown_preview {
            self.markdown_doc = Some(MarkdownDocument::parse(&content, self.markdown_spec));
        } else {
            self.markdown_doc = None;
        }

        Ok(())
    }

    pub fn save_file(&mut self) -> std::io::Result<()> {
        if let Some(ref path) = self.file_path {
            let text = self.buffer.full_text();
            Self::atomic_write_file(path, &text)?;
            self.buffer.is_modified = false;
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
        self.buffer.is_modified = false;

        let lang = SupportedLanguage::from_path(path);
        self.highlighter = Highlighter::new(lang);
        self.highlighter.update_source(&self.buffer.full_text());

        if lang == SupportedLanguage::Markdown && self.is_markdown_preview {
            self.markdown_doc = Some(MarkdownDocument::parse(&self.buffer.full_text(), self.markdown_spec));
        } else {
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
        let text = self.buffer.full_text();
        self.highlighter.update_source(&text);

        if self.highlighter.lang == SupportedLanguage::Markdown && self.is_markdown_preview {
            self.markdown_doc = Some(MarkdownDocument::parse(&text, self.markdown_spec));
        }

        if let Some(ref q) = self.search_query.clone() {
            self.update_search(q);
        }
    }

    pub fn mark_cursor_moved(&self) {
        self.needs_scroll_to_cursor.set(true);
    }

    pub fn refresh_markdown(&mut self) {
        if self.is_markdown_preview {
            let text = self.buffer.full_text();
            self.markdown_doc = Some(MarkdownDocument::parse(&text, self.markdown_spec));
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
        let mut matches = Vec::new();
        let q_lower = query.to_lowercase();
        let q_len = query.chars().count();

        for line_idx in 0..self.buffer.line_count() {
            if let Some(line) = self.buffer.line_text(line_idx) {
                let line_lower = line.to_lowercase();
                let mut start_byte = 0;
                while let Some(byte_pos) = line_lower[start_byte..].find(&q_lower) {
                    let actual_byte = start_byte + byte_pos;
                    let char_start = line[..actual_byte].chars().count();
                    matches.push((line_idx, char_start, char_start + q_len));
                    start_byte = actual_byte + q_lower.len().max(1);
                }
            }
        }

        self.search_matches = matches;
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

        // 2. If current tab is untouched and untitled, reuse it
        let current_is_untouched = {
            let cur = &self.tabs[self.active_tab_idx];
            cur.file_path.is_none() && !cur.buffer.is_modified && cur.buffer.full_text().is_empty()
        };
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
            tab.refresh_markdown();
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
