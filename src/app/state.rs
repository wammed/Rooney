use crate::app::message::Message;
use crate::app::App;
use crate::editor::{EditorPane, PaneId, SplitLayout};
use cosmic::app::Task;
use std::path::Path;

impl App {
    pub fn current_pane(&self) -> &EditorPane {
        match self.active_pane {
            PaneId::Left => &self.left_pane,
            PaneId::Right => &self.right_pane,
        }
    }

    pub fn current_pane_mut(&mut self) -> &mut EditorPane {
        match self.active_pane {
            PaneId::Left => &mut self.left_pane,
            PaneId::Right => &mut self.right_pane,
        }
    }

    pub fn pane(&self, id: PaneId) -> &EditorPane {
        match id {
            PaneId::Left => &self.left_pane,
            PaneId::Right => &self.right_pane,
        }
    }

    pub fn pane_mut(&mut self, id: PaneId) -> &mut EditorPane {
        match id {
            PaneId::Left => &mut self.left_pane,
            PaneId::Right => &mut self.right_pane,
        }
    }

    pub fn open_file_in_active_pane(&mut self, path: &Path) {
        let pane = self.current_pane_mut();
        if let Err(e) = pane.open_file(path) {
            self.status_msg = Some(format!("Failed to open file: {e}"));
        } else {
            self.status_msg = Some(format!("Opened {}", pane.file_name));
        }
    }

    pub fn save_config(&mut self) {
        self.config.theme = self.theme.config.id;
        self.config.font = self.font_manager.current_font.clone();
        self.config.font_size = self.font_manager.font_size;
        self.config.opacity = self.theme.opacity;
        self.config.dimming = self.theme.dimming;
        self.config.split_layout = match self.split_layout {
            SplitLayout::Single => "Single",
            SplitLayout::Split => "Split",
        }
        .to_string();
        self.config.file_tree_visible = self.file_tree.is_visible;
        self.config.ai_enabled = self.ollama.is_enabled;
        self.config.ai_model = self.ollama.active_model.clone();
        self.config.ai_chat_visible = self.show_ai_chat;

        let left_tabs = self
            .left_pane
            .tabs
            .iter()
            .map(|t| crate::config::TabSessionInfo {
                file_path: t.file_path.clone(),
                file_name: t.file_name.clone(),
                cursor_line: t.buffer.cursor.0,
                cursor_col: t.buffer.cursor.1,
            })
            .collect();

        let right_tabs = if self.split_layout == SplitLayout::Split {
            Some(crate::config::PaneSessionInfo {
                tabs: self
                    .right_pane
                    .tabs
                    .iter()
                    .map(|t| crate::config::TabSessionInfo {
                        file_path: t.file_path.clone(),
                        file_name: t.file_name.clone(),
                        cursor_line: t.buffer.cursor.0,
                        cursor_col: t.buffer.cursor.1,
                    })
                    .collect(),
                active_tab_idx: self.right_pane.active_tab_idx,
            })
        } else {
            None
        };

        self.config.session = crate::config::SessionConfig {
            root_dir: Some(self.file_tree.root.clone()),
            left_pane: crate::config::PaneSessionInfo {
                tabs: left_tabs,
                active_tab_idx: self.left_pane.active_tab_idx,
            },
            right_pane: right_tabs,
        };

        let _ = self.config.save();
    }

    pub fn execute_copy(&mut self) -> Task<Message> {
        self.context_menu = None;
        self.file_tree_context_menu = None;
        self.show_edit_menu = false;
        if let Some(selected) = self.current_pane().buffer.selected_text() {
            let count = selected.chars().count();
            self.status_msg = Some(format!("Copied {} chars", count));
            return cosmic::iced::clipboard::write(selected);
        }
        Task::none()
    }

    pub fn execute_cut(&mut self) -> Task<Message> {
        self.context_menu = None;
        self.file_tree_context_menu = None;
        self.show_edit_menu = false;
        if let Some(selected) = self.current_pane().buffer.selected_text() {
            let pane = self.current_pane_mut();
            let count = selected.chars().count();
            pane.buffer.delete_selection();
            pane.on_content_changed();
            self.status_msg = Some(format!("Cut {} chars", count));
            return cosmic::iced::clipboard::write(selected);
        }
        Task::none()
    }

    pub fn execute_paste(&mut self) -> Task<Message> {
        self.context_menu = None;
        self.file_tree_context_menu = None;
        self.show_edit_menu = false;
        cosmic::iced::clipboard::read()
            .map(|opt| cosmic::Action::App(Message::ClipboardPasted(opt)))
    }

    pub fn execute_delete_line(&mut self) {
        let pane = self.current_pane_mut();
        pane.clear_ghost_text();
        pane.buffer.delete_line();
        pane.on_content_changed();
    }

    pub fn execute_duplicate_line(&mut self) {
        let pane = self.current_pane_mut();
        pane.clear_ghost_text();
        pane.buffer.duplicate_line();
        pane.on_content_changed();
    }

    pub fn execute_toggle_comment(&mut self) {
        let pane = self.current_pane_mut();
        pane.clear_ghost_text();
        let comment_prefix = pane.highlighter.lang.line_comment_prefix().unwrap_or("//");
        pane.buffer.toggle_comment(comment_prefix);
        pane.on_content_changed();
    }
}
