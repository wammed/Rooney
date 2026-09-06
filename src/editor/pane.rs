use crate::editor::buffer::TextBuffer;
use crate::markdown::MarkdownDocument;
use crate::syntax::{Highlighter, SupportedLanguage};
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

pub struct EditorPane {
    pub id: PaneId,
    pub file_path: Option<PathBuf>,
    pub file_name: String,
    pub buffer: TextBuffer,
    pub highlighter: Highlighter,
    pub scroll_y: f32,
    pub scroll_x: f32,
    pub ghost_text: Option<String>,
    pub preedit: Option<(String, Option<std::ops::Range<usize>>)>,
    pub is_markdown_preview: bool,
    pub markdown_doc: Option<MarkdownDocument>,
    pub last_edit_time: Instant,
    pub last_cursor_time: Instant,
}

impl EditorPane {
    pub fn new(id: PaneId, title: &str) -> Self {
        Self {
            id,
            file_path: None,
            file_name: title.to_string(),
            buffer: TextBuffer::default(),
            highlighter: Highlighter::new(SupportedLanguage::PlainText),
            scroll_y: 0.0,
            scroll_x: 0.0,
            ghost_text: None,
            preedit: None,
            is_markdown_preview: false,
            markdown_doc: None,
            last_edit_time: Instant::now(),
            last_cursor_time: Instant::now(),
        }
    }

    pub fn load_file(&mut self, path: &Path) -> std::io::Result<()> {
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
        self.scroll_y = 0.0;
        self.scroll_x = 0.0;
        self.ghost_text = None;
        self.preedit = None;

        if lang == SupportedLanguage::Markdown {
            self.markdown_doc = Some(MarkdownDocument::parse(&content));
        } else {
            self.markdown_doc = None;
        }

        Ok(())
    }

    pub fn save_file(&mut self) -> std::io::Result<()> {
        if let Some(ref path) = self.file_path {
            let text = self.buffer.full_text();
            std::fs::write(path, text)?;
            self.buffer.is_modified = false;
        }
        Ok(())
    }

    pub fn save_file_as(&mut self, path: &Path) -> std::io::Result<()> {
        let text = self.buffer.full_text();
        std::fs::write(path, text)?;
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

        Ok(())
    }

    pub fn on_content_changed(&mut self) {
        self.last_edit_time = Instant::now();
        self.ghost_text = None;
        let text = self.buffer.full_text();
        self.highlighter.update_source(&text);

        if self.highlighter.lang == SupportedLanguage::Markdown {
            self.markdown_doc = Some(MarkdownDocument::parse(&text));
        }
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
