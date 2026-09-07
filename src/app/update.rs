use crate::ai::AiStatus;
use crate::app::message::Message;
use crate::app::App;
use crate::editor::SplitLayout;
use crate::ui::file_tree_view::FileTreeMessage;
use crate::theme::themes::ThemeId;
use cosmic::app::Task;
use cosmic::ApplicationExt;
use std::path::Path;
use std::time::Duration;

/// Validate whether a file or folder name is safe and valid (no path separators, no directory traversal).
pub fn is_valid_file_or_folder_name(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed == "." || trimmed == ".." {
        return false;
    }
    if trimmed.contains('\0') {
        return false;
    }
    true
}

/// Determine whether a file contains credentials or sensitive keys that should not be auto-sent to AI.
pub fn is_sensitive_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if file_name.starts_with(".env")
        || file_name == ".git-credentials"
        || file_name == ".netrc"
        || file_name == "id_rsa"
        || file_name == "id_ed25519"
        || file_name == "id_ecdsa"
        || file_name == "id_dsa"
        || file_name == "credentials"
        || file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".pfx")
        || file_name.ends_with(".p12")
    {
        return true;
    }

    false
}

impl App {
    pub(crate) fn handle_update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let pane = self.current_pane();
                let elapsed = pane.last_edit_time.elapsed();

                if self.ollama.is_enabled
                    && !self.ai_request_pending
                    && elapsed > Duration::from_millis(450)
                    && elapsed < Duration::from_millis(600)
                    && pane.ghost_text.is_none()
                {
                    return self.handle_update(Message::TriggerAiFim);
                }
                Task::none()
            }

            Message::Event(event) => self.handle_key_event(event),

            Message::ClickPane(pane_id, line, col) => {
                self.active_pane = pane_id;
                self.context_menu = None;
                self.file_tree_context_menu = None;
                self.active_header_menu = None;
                self.show_edit_menu = false;
                let pane = self.current_pane_mut();
                pane.clear_ghost_text();
                pane.preedit = None;
                pane.buffer.cursor = (line, col);
                pane.buffer.clamp_cursor();
                pane.buffer.selection_anchor = None;
                Task::none()
            }

            Message::DragSelect(pane_id, line, col) => {
                self.active_pane = pane_id;
                let pane = self.current_pane_mut();
                if pane.buffer.selection_anchor.is_none() {
                    pane.buffer.selection_anchor = Some(pane.buffer.cursor);
                }
                pane.buffer.cursor = (line, col);
                pane.buffer.clamp_cursor();
                Task::none()
            }

            Message::Copy => self.execute_copy(),

            Message::Cut => self.execute_cut(),

            Message::Paste => self.execute_paste(),

            Message::ClipboardPasted(text_opt) => {
                if let Some(text) = text_opt {
                    if !text.is_empty() {
                        let count = text.chars().count();
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.delete_selection();
                        pane.buffer.insert_str(&text);
                        pane.on_content_changed();
                        self.status_msg = Some(format!("Pasted {} chars", count));
                    }
                }
                Task::none()
            }

            Message::SelectAll => {
                self.context_menu = None;
                self.show_edit_menu = false;
                self.current_pane_mut().buffer.select_all();
                Task::none()
            }

            Message::Undo => {
                self.context_menu = None;
                self.show_edit_menu = false;
                self.current_pane_mut().buffer.undo();
                self.current_pane_mut().on_content_changed();
                Task::none()
            }

            Message::Redo => {
                self.context_menu = None;
                self.show_edit_menu = false;
                self.current_pane_mut().buffer.redo();
                self.current_pane_mut().on_content_changed();
                Task::none()
            }

            Message::OpenContextMenu(pane_id, x, y) => {
                self.active_pane = pane_id;
                self.context_menu = Some((pane_id, x, y));
                Task::none()
            }

            Message::CloseContextMenu => {
                self.context_menu = None;
                Task::none()
            }

            Message::CloseFileTreeContextMenu => {
                self.file_tree_context_menu = None;
                Task::none()
            }

            Message::ToggleHeaderMenu(menu) => {
                if self.active_header_menu == Some(menu) {
                    self.active_header_menu = None;
                } else {
                    self.active_header_menu = Some(menu);
                    self.context_menu = None;
                    self.file_tree_context_menu = None;
                }
                Task::none()
            }

            Message::CloseHeaderMenu => {
                self.active_header_menu = None;
                Task::none()
            }

            Message::ToggleEditMenu => {
                self.handle_update(Message::ToggleHeaderMenu(crate::app::message::ActiveHeaderMenu::Edit))
            }

            Message::CloseEditMenu => {
                self.active_header_menu = None;
                self.show_edit_menu = false;
                Task::none()
            }

            Message::ScrollPane(pane_id, delta_y) => {
                let target_pane = self.pane_mut(pane_id);
                target_pane.scroll_y = (target_pane.scroll_y + delta_y).max(0.0);
                Task::none()
            }

            Message::TogglePaneMode(pane_id) => {
                let target_pane = self.pane_mut(pane_id);
                target_pane.is_markdown_preview = !target_pane.is_markdown_preview;
                if target_pane.is_markdown_preview && target_pane.markdown_doc.is_none() {
                    let text = target_pane.buffer.full_text();
                    target_pane.markdown_doc =
                        Some(crate::markdown::MarkdownDocument::parse(&text));
                }
                Task::none()
            }

            Message::FileTreeMsg(msg) => {
                self.file_tree_context_menu = None;
                match msg {
                    FileTreeMessage::ToggleDir(path) => {
                        self.file_tree.toggle_dir(&path);
                        Task::none()
                    }
                    FileTreeMessage::OpenFile(path) => {
                        self.file_tree.select(path.clone());
                        self.open_file_in_active_pane(&path);
                        Task::none()
                    }
                    FileTreeMessage::Refresh => {
                        self.file_tree.refresh();
                        self.status_msg = Some("File tree refreshed".into());
                        Task::none()
                    }
                    FileTreeMessage::ToggleVisibility => {
                        self.file_tree.is_visible = !self.file_tree.is_visible;
                        self.save_config();
                        Task::none()
                    }
                    FileTreeMessage::OpenFolder => self.handle_update(Message::OpenFolderPrompt),
                    FileTreeMessage::GoToParent => {
                        if self.file_tree.go_to_parent() {
                            let name = self
                                .file_tree
                                .root
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("ROOT")
                                .to_string();
                            self.new_file_target_dir = self.file_tree.root.clone();
                            self.status_msg = Some(format!("Directory: {name}"));
                        }
                        Task::none()
                    }
                    FileTreeMessage::NewFileIn(parent_opt) => {
                        self.new_file_target_dir =
                            parent_opt.unwrap_or_else(|| self.file_tree.root.clone());
                        self.show_new_file_modal = true;
                        self.new_file_name_input.clear();
                        Task::none()
                    }
                    FileTreeMessage::NewFolderIn(parent_opt) => {
                        self.handle_update(Message::PromptNewFolder(parent_opt))
                    }
                    FileTreeMessage::Rename(path) => {
                        self.handle_update(Message::PromptRename(path))
                    }
                    FileTreeMessage::Delete(path) => {
                        self.handle_update(Message::PromptDelete(path))
                    }
                    FileTreeMessage::RightClick(path, is_dir) => {
                        self.file_tree.select(path.clone());
                        self.file_tree_context_menu = Some(crate::app::FileTreeContextMenuState {
                            target: Some(path),
                            is_dir,
                            x: self.mouse_pos.0,
                            y: self.mouse_pos.1,
                        });
                        Task::none()
                    }
                    FileTreeMessage::RightClickRoot => {
                        self.file_tree.selected_path = None;
                        self.file_tree_context_menu = Some(crate::app::FileTreeContextMenuState {
                            target: None,
                            is_dir: true,
                            x: self.mouse_pos.0,
                            y: self.mouse_pos.1,
                        });
                        Task::none()
                    }
                }
            },

            Message::ToggleSplit => {
                self.split_layout = match self.split_layout {
                    SplitLayout::Single => SplitLayout::Split,
                    SplitLayout::Split => SplitLayout::Single,
                };
                self.status_msg = Some(format!(
                    "Layout: {}",
                    if self.split_layout == SplitLayout::Split {
                        "Side-by-Side Split"
                    } else {
                        "Single Pane"
                    }
                ));
                self.save_config();
                Task::none()
            }

            Message::SetActivePane(pane_id) => {
                self.active_pane = pane_id;
                Task::none()
            }

            Message::ToggleMarkdownPreview => {
                let pane = self.current_pane_mut();
                pane.is_markdown_preview = !pane.is_markdown_preview;
                if pane.is_markdown_preview && pane.markdown_doc.is_none() {
                    let text = pane.buffer.full_text();
                    pane.markdown_doc = Some(crate::markdown::MarkdownDocument::parse(&text));
                }
                Task::none()
            }

            Message::SelectTheme(index) => {
                if let Some(&id) = ThemeId::ALL.get(index) {
                    self.theme.set_theme(id);
                    self.status_msg = Some(format!("Theme: {}", id.display_name()));
                    self.save_config();
                }
                Task::none()
            }

            Message::SelectFont(index) => {
                if let Some(font) = self.font_names.get(index) {
                    self.font_manager.set_font(font.clone());
                    self.status_msg = Some(format!("Font: {}", font));
                    self.save_config();
                }
                Task::none()
            }

            Message::IncreaseFontSize => {
                let s = self.font_manager.font_size + 1.0;
                self.font_manager.set_font_size(s);
                self.save_config();
                Task::none()
            }

            Message::DecreaseFontSize => {
                let s = self.font_manager.font_size - 1.0;
                self.font_manager.set_font_size(s);
                self.save_config();
                Task::none()
            }

            Message::ToggleAi => {
                self.ollama.is_enabled = !self.ollama.is_enabled;
                self.ai_status = if self.ollama.is_enabled {
                    AiStatus::Ready(self.ollama.active_model.clone())
                } else {
                    AiStatus::Disabled
                };
                self.status_msg = Some(if self.ollama.is_enabled {
                    "Local AI FIM enabled".into()
                } else {
                    "Local AI FIM disabled".into()
                });
                self.save_config();
                Task::none()
            }

            Message::TriggerAiFim => {
                if !self.ollama.is_enabled || self.ai_request_pending {
                    return Task::none();
                }

                // Privacy guard: skip automatic FIM generation for sensitive files (.env, keys, credentials)
                if let Some(ref path) = self.current_pane().file_path {
                    if is_sensitive_file(path) {
                        return Task::none();
                    }
                }

                let active_pane = self.active_pane;
                let (prefix, suffix) = self.current_pane().buffer.get_fim_prefix_suffix(1500);

                if prefix.trim().is_empty() {
                    return Task::none();
                }

                self.ai_request_pending = true;
                self.ai_status = AiStatus::Generating;
                let client = self.ollama.clone();

                Task::perform(
                    async move {
                        let res = client.generate_fim(&prefix, &suffix).await;
                        (active_pane, res)
                    },
                    |(pane, res)| cosmic::Action::App(Message::AiFimResult(pane, res)),
                )
            }

            Message::AiFimResult(pane_id, result) => {
                self.ai_request_pending = false;
                match result {
                    Ok(text) => {
                        self.ai_status = AiStatus::Ready(self.ollama.active_model.clone());
                        let target_pane = self.pane_mut(pane_id);
                        if !text.is_empty() {
                            target_pane.ghost_text = Some(text);
                            self.status_msg =
                                Some("AI: Suggestion ready (Press Tab to accept)".into());
                        }
                    }
                    Err(e) => {
                        self.ai_status = AiStatus::Error(e.clone());
                        self.status_msg = Some(format!("AI Error: {e}"));
                    }
                }
                Task::none()
            }

            Message::AiModelsFetched(result) => {
                match result {
                    Ok(models) => {
                        if !models.is_empty() {
                            self.ollama.available_models = models.clone();
                            self.ai_status = AiStatus::Ready(self.ollama.active_model.clone());
                            self.status_msg =
                                Some(format!("Ollama: {} models found", models.len()));
                        }
                    }
                    Err(e) => {
                        self.status_msg = Some(format!("Ollama: {e}"));
                    }
                }
                Task::none()
            }

            Message::SelectAiModel(idx) => {
                if let Some(m) = self.ollama.available_models.get(idx) {
                    self.ollama.active_model = m.clone();
                    self.ai_status = AiStatus::Ready(m.clone());
                    self.status_msg = Some(format!("AI Model: {}", m));
                    self.save_config();
                }
                Task::none()
            }

            Message::OpenFilePrompt => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Open File")
                        .pick_file()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                |path| cosmic::Action::App(Message::FileOpened(path)),
            ),

            Message::FileOpened(path) => {
                if let Some(path) = path {
                    self.open_file_in_active_pane(&path);
                    if path.starts_with(&self.file_tree.root) {
                        self.file_tree.select(path);
                    }
                }
                Task::none()
            }

            Message::OpenFolderPrompt => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Open Folder")
                        .pick_folder()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                |path| cosmic::Action::App(Message::FolderOpened(path)),
            ),

            Message::FolderOpened(path) => {
                if let Some(dir) = path {
                    let name = dir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("ROOT")
                        .to_string();
                    self.new_file_target_dir = dir.clone();
                    self.file_tree.set_root(dir);
                    self.status_msg = Some(format!("Opened folder: {name}"));
                }
                Task::none()
            }

            Message::SaveFile => {
                let pane = self.current_pane();
                if pane.file_path.is_none() {
                    return self.handle_update(Message::SaveFileAsPrompt);
                }
                let pane = self.current_pane_mut();
                if let Err(e) = pane.save_file() {
                    self.status_msg = Some(format!("Save failed: {e}"));
                } else {
                    self.status_msg = Some(format!("Saved {}", pane.file_name));
                    self.file_tree.refresh();
                }
                Task::none()
            }

            Message::SaveFileAsPrompt => {
                let active_pane = self.active_pane;
                let default_name = self.current_pane().file_name.clone();
                let starting_dir = self.file_tree.root.clone();
                Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title("Save File As")
                            .set_directory(&starting_dir)
                            .set_file_name(&default_name)
                            .save_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    move |path| cosmic::Action::App(Message::FileSavedAs(active_pane, path)),
                )
            }

            Message::FileSavedAs(pane_id, path) => {
                if let Some(path) = path {
                    let pane = self.pane_mut(pane_id);
                    if let Err(e) = pane.save_file_as(&path) {
                        self.status_msg = Some(format!("Save failed: {e}"));
                    } else {
                        self.status_msg = Some(format!("Saved as {}", pane.file_name));
                        self.file_tree.refresh();
                        if path.starts_with(&self.file_tree.root) {
                            self.file_tree.select(path);
                        }
                    }
                }
                Task::none()
            }

            Message::PromptNewFile => {
                self.show_new_file_modal = true;
                self.new_file_name_input = String::new();
                self.new_file_target_dir = self.file_tree.root.clone();
                Task::none()
            }

            Message::NewFileNameChanged(val) => {
                self.new_file_name_input = val;
                Task::none()
            }

            Message::BrowseNewFileFolder => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Select Folder for New File")
                        .pick_folder()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                |path| cosmic::Action::App(Message::NewFileFolderSelected(path)),
            ),

            Message::NewFileFolderSelected(path) => {
                if let Some(dir) = path {
                    self.new_file_target_dir = dir;
                }
                Task::none()
            }

            Message::ConfirmNewFile => {
                let raw_name = self.new_file_name_input.trim();
                if !is_valid_file_or_folder_name(raw_name) {
                    self.status_msg = Some("Invalid file name: path separators and '..' are not allowed".into());
                    return Task::none();
                }

                let target_path = self.new_file_target_dir.join(raw_name);
                if let Some(parent) = target_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                if !target_path.exists() {
                    if let Err(e) = std::fs::write(&target_path, "") {
                        self.status_msg = Some(format!("Failed to create file: {e}"));
                        return Task::none();
                    }
                }

                self.open_file_in_active_pane(&target_path);
                self.file_tree.refresh();
                if target_path.starts_with(&self.file_tree.root) {
                    self.file_tree.select(target_path.clone());
                }
                self.show_new_file_modal = false;
                self.new_file_name_input.clear();
                self.status_msg = Some(format!(
                    "Created and opened {}",
                    self.current_pane().file_name
                ));
                Task::none()
            }

            Message::CancelNewFile => {
                self.show_new_file_modal = false;
                self.new_file_name_input.clear();
                Task::none()
            }

            Message::ToggleSettings => {
                self.show_settings = !self.show_settings;
                Task::none()
            }

            Message::ChangeOpacity(val) => {
                self.theme.opacity = val;
                self.save_config();
                Task::none()
            }

            Message::ChangeFileTreeOpacity(val) => {
                self.theme.file_tree_opacity = val;
                self.save_config();
                Task::none()
            }

            Message::ChangeTitleBarOpacity(val) => {
                self.theme.title_bar_opacity = val;
                self.save_config();
                Task::none()
            }

            Message::ChangeDimming(val) => {
                self.theme.dimming = val;
                self.save_config();
                Task::none()
            }

            Message::DragWindow => self.drag(),
            Message::MaximizeWindow => self.maximize(),
            Message::MinimizeWindow => self.minimize(),
            Message::CloseWindow => {
                cosmic::iced::Task::done(cosmic::app::Action::Close).map(cosmic::Action::Cosmic)
            }

            Message::CloseSettings => {
                self.show_settings = false;
                Task::none()
            }

            Message::ToggleSearch => {
                let pane = self.current_pane_mut();
                pane.is_search_open = !pane.is_search_open;
                if pane.is_search_open {
                    if let Some(ref q) = pane.search_query.clone() {
                        self.search_input = q.clone();
                    }
                }
                Task::none()
            }

            Message::SearchQueryChanged(q) => {
                self.search_input = q.clone();
                let pane = self.current_pane_mut();
                pane.update_search(&q);
                Task::none()
            }

            Message::NextSearchMatch => {
                let pane = self.current_pane_mut();
                pane.next_search_match();
                Task::none()
            }

            Message::PrevSearchMatch => {
                let pane = self.current_pane_mut();
                pane.prev_search_match();
                Task::none()
            }

            Message::CloseSearch => {
                let pane = self.current_pane_mut();
                pane.is_search_open = false;
                pane.search_query = None;
                pane.search_matches.clear();
                self.search_input.clear();
                Task::none()
            }

            Message::DeleteLine => {
                self.execute_delete_line();
                Task::none()
            }

            Message::DuplicateLine => {
                self.execute_duplicate_line();
                Task::none()
            }

            Message::ToggleComment => {
                self.execute_toggle_comment();
                Task::none()
            }

            // Tab operations
            Message::SelectTab(pane_id, tab_idx) => {
                self.active_pane = pane_id;
                self.pane_mut(pane_id).select_tab(tab_idx);
                Task::none()
            }

            Message::CloseTab(pane_id, tab_idx) => {
                self.active_pane = pane_id;
                self.pane_mut(pane_id).close_tab(tab_idx);
                Task::none()
            }

            Message::NewTab(pane_id) => {
                self.active_pane = pane_id;
                self.pane_mut(pane_id).new_tab("Untitled");
                Task::none()
            }

            Message::NextTab(pane_id) => {
                self.active_pane = pane_id;
                self.pane_mut(pane_id).next_tab();
                Task::none()
            }

            Message::PrevTab(pane_id) => {
                self.active_pane = pane_id;
                self.pane_mut(pane_id).prev_tab();
                Task::none()
            }

            // File tree operations & modals
            Message::PromptNewFolder(parent_opt) => {
                self.show_new_folder_modal = true;
                self.new_folder_name_input.clear();
                self.new_folder_target_dir =
                    parent_opt.unwrap_or_else(|| self.file_tree.root.clone());
                Task::none()
            }

            Message::NewFolderNameChanged(val) => {
                self.new_folder_name_input = val;
                Task::none()
            }

            Message::ConfirmNewFolder => {
                let raw_name = self.new_folder_name_input.trim();
                if !is_valid_file_or_folder_name(raw_name) {
                    self.status_msg = Some("Invalid folder name: path separators and '..' are not allowed".into());
                    return Task::none();
                }

                let target_path = self.new_folder_target_dir.join(raw_name);
                if let Err(e) = std::fs::create_dir_all(&target_path) {
                    self.status_msg = Some(format!("Failed to create folder: {e}"));
                    return Task::none();
                }

                self.file_tree.refresh();
                self.show_new_folder_modal = false;
                self.new_folder_name_input.clear();
                self.status_msg = Some(format!("Created folder {}", target_path.display()));
                Task::none()
            }

            Message::CancelNewFolder => {
                self.show_new_folder_modal = false;
                self.new_folder_name_input.clear();
                Task::none()
            }

            Message::PromptRename(path) => {
                self.show_rename_modal = true;
                let default_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                self.rename_target_path = Some(path);
                self.rename_name_input = default_name;
                Task::none()
            }

            Message::RenameInputChanged(val) => {
                self.rename_name_input = val;
                Task::none()
            }

            Message::ConfirmRename => {
                let new_name = self.rename_name_input.trim().to_string();
                if !is_valid_file_or_folder_name(&new_name) {
                    self.status_msg = Some("Invalid name: path separators and '..' are not allowed".into());
                    return Task::none();
                }

                if let Some(old_path) = self.rename_target_path.take() {
                    if let Some(parent) = old_path.parent() {
                        let new_path = parent.join(&new_name);
                        if new_path != old_path {
                            if let Err(e) = std::fs::rename(&old_path, &new_path) {
                                self.status_msg = Some(format!("Failed to rename: {e}"));
                                self.rename_target_path = Some(old_path);
                                return Task::none();
                            }

                            // Synchronize open tabs in both panes
                            for pane in [&mut self.left_pane, &mut self.right_pane] {
                                for tab in &mut pane.tabs {
                                    if let Some(ref mut tab_path) = tab.file_path {
                                        if *tab_path == old_path {
                                            *tab_path = new_path.clone();
                                            tab.file_name = new_name.clone();
                                        } else if tab_path.starts_with(&old_path) {
                                            if let Ok(rel) = tab_path.strip_prefix(&old_path) {
                                                let updated = new_path.join(rel);
                                                tab.file_name = updated
                                                    .file_name()
                                                    .and_then(|n| n.to_str())
                                                    .unwrap_or("Untitled")
                                                    .to_string();
                                                *tab_path = updated;
                                            }
                                        }
                                    }
                                }
                            }

                            self.file_tree.refresh();
                            self.status_msg = Some(format!("Renamed to {new_name}"));
                        }
                    }
                }

                self.show_rename_modal = false;
                self.rename_name_input.clear();
                self.save_config();
                Task::none()
            }

            Message::CancelRename => {
                self.show_rename_modal = false;
                self.rename_target_path = None;
                self.rename_name_input.clear();
                Task::none()
            }

            Message::PromptDelete(path) => {
                self.delete_target_path = Some(path);
                self.show_delete_modal = true;
                Task::none()
            }

            Message::ConfirmDelete => {
                if let Some(path) = self.delete_target_path.take() {
                    let is_root = path == Path::new("/") || path.parent().is_none();
                    let is_home = directories::BaseDirs::new()
                        .map(|b| path == b.home_dir())
                        .unwrap_or(false);
                    let is_tree_root = path == self.file_tree.root;

                    if is_root || is_home || is_tree_root {
                        self.status_msg = Some(
                            "Safety guard: Root, home, or workspace root cannot be deleted".into(),
                        );
                        self.show_delete_modal = false;
                        return Task::none();
                    }

                    let is_symlink = path
                        .symlink_metadata()
                        .map(|m| m.file_type().is_symlink())
                        .unwrap_or(false);

                    let result = if is_symlink {
                        std::fs::remove_file(&path)
                    } else if path.is_dir() {
                        std::fs::remove_dir_all(&path)
                    } else {
                        std::fs::remove_file(&path)
                    };

                    match result {
                        Ok(()) => {
                            for pane in [&mut self.left_pane, &mut self.right_pane] {
                                let mut idx = 0;
                                while idx < pane.tabs.len() {
                                    let matches = pane.tabs[idx]
                                        .file_path
                                        .as_ref()
                                        .map(|p| p == &path || p.starts_with(&path))
                                        .unwrap_or(false);
                                    if matches {
                                        pane.close_tab(idx);
                                    } else {
                                        idx += 1;
                                    }
                                }
                            }
                            self.file_tree.refresh();
                            self.status_msg = Some(format!("Deleted {}", path.display()));
                        }
                        Err(e) => {
                            self.status_msg = Some(format!("Failed to delete: {e}"));
                        }
                    }
                }
                self.show_delete_modal = false;
                self.save_config();
                Task::none()
            }

            Message::CancelDelete => {
                self.show_delete_modal = false;
                self.delete_target_path = None;
                Task::none()
            }

            // AI Chat & Code Generation Panel
            Message::ToggleAiChat => {
                self.show_ai_chat = !self.show_ai_chat;
                self.save_config();
                Task::none()
            }

            Message::AiChatInputChanged(val) => {
                self.ai_chat_input = val;
                Task::none()
            }

            Message::SendAiChatMessage => {
                let prompt = self.ai_chat_input.trim().to_string();
                if prompt.is_empty() || self.ai_chat_pending {
                    return Task::none();
                }

                // Append user prompt
                self.ai_chat_messages.push(crate::ai::ChatMessage {
                    role: crate::ai::ChatRole::User,
                    content: prompt.clone(),
                });
                // Append initial empty assistant message for streaming
                self.ai_chat_messages.push(crate::ai::ChatMessage {
                    role: crate::ai::ChatRole::Assistant,
                    content: String::new(),
                });
                self.ai_chat_input.clear();
                self.ai_chat_pending = true;

                let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                self.ai_chat_cancel = Some(cancel_flag.clone());

                let client = self.ollama.clone();
                let (tx, rx) = futures_channel::mpsc::unbounded::<crate::ai::ChatStreamEvent>();

                tokio::spawn(async move {
                    client
                        .chat_generate_stream(None, &prompt, tx, cancel_flag)
                        .await;
                });

                use futures_util::StreamExt;
                let action_stream = rx.map(|event| match event {
                    crate::ai::ChatStreamEvent::Chunk(chunk) => {
                        cosmic::Action::App(Message::AiChatChunk(chunk))
                    }
                    crate::ai::ChatStreamEvent::Done => cosmic::Action::App(Message::AiChatDone),
                    crate::ai::ChatStreamEvent::Error(err) => {
                        cosmic::Action::App(Message::AiChatError(err))
                    }
                });

                cosmic::task::stream(action_stream)
            }

            Message::StopAiChat => {
                if let Some(flag) = self.ai_chat_cancel.take() {
                    flag.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                self.ai_chat_pending = false;
                self.status_msg = Some("AI generation stopped".into());
                Task::none()
            }

            Message::AiChatChunk(chunk) => {
                if let Some(last) = self.ai_chat_messages.last_mut() {
                    if last.role == crate::ai::ChatRole::Assistant {
                        last.content.push_str(&chunk);
                    }
                }
                Task::none()
            }

            Message::AiChatDone => {
                self.ai_chat_pending = false;
                self.ai_chat_cancel = None;
                self.status_msg = Some("AI response complete".into());
                Task::none()
            }

            Message::AiChatError(err) => {
                self.ai_chat_pending = false;
                self.ai_chat_cancel = None;
                if let Some(last) = self.ai_chat_messages.last_mut() {
                    if last.role == crate::ai::ChatRole::Assistant && last.content.is_empty() {
                        last.role = crate::ai::ChatRole::Error;
                        last.content = format!("Error: {err}");
                    } else {
                        self.ai_chat_messages.push(crate::ai::ChatMessage {
                            role: crate::ai::ChatRole::Error,
                            content: format!("Error: {err}"),
                        });
                    }
                }
                self.status_msg = Some(format!("AI Chat Error: {err}"));
                Task::none()
            }

            Message::AiChatResult(res) => {
                self.ai_chat_pending = false;
                self.ai_chat_cancel = None;
                match res {
                    Ok(text) => {
                        self.ai_chat_messages.push(crate::ai::ChatMessage {
                            role: crate::ai::ChatRole::Assistant,
                            content: text,
                        });
                    }
                    Err(e) => {
                        self.ai_chat_messages.push(crate::ai::ChatMessage {
                            role: crate::ai::ChatRole::Error,
                            content: format!("Error: {e}"),
                        });
                        self.status_msg = Some(format!("AI Chat Error: {e}"));
                    }
                }
                Task::none()
            }

            Message::ClearAiChat => {
                self.ai_chat_messages.clear();
                Task::none()
            }

            Message::AttachSelectionToAiChat => {
                let snippet_opt = {
                    let pane = self.current_pane();
                    pane.buffer.selected_text().map(|selected_text| {
                        let ext = pane
                            .file_path
                            .as_ref()
                            .and_then(|p| p.extension())
                            .and_then(|e| e.to_str())
                            .unwrap_or("");
                        format!("\n```{ext}\n{selected_text}\n```\n")
                    })
                };
                if let Some(snippet) = snippet_opt {
                    self.ai_chat_input.push_str(&snippet);
                    self.status_msg = Some("Attached selection to AI Chat".into());
                } else {
                    self.status_msg = Some("No text selected in editor".into());
                }
                Task::none()
            }

            Message::AttachFileToAiChat => {
                let (file_name, snippet) = {
                    let pane = self.current_pane();
                    let name = pane.file_name.clone();
                    let full_text = pane.buffer.full_text();
                    let ext = pane
                        .file_path
                        .as_ref()
                        .and_then(|p| p.extension())
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    let snippet = format!("\nFile: {name}\n```{ext}\n{full_text}\n```\n");
                    (name, snippet)
                };
                self.ai_chat_input.push_str(&snippet);
                self.status_msg = Some(format!("Attached {file_name} to AI Chat"));
                Task::none()
            }

            Message::InsertAiResponseAtCursor(text) => {
                let to_insert = extract_code_block_or_text(&text);
                let pane = self.current_pane_mut();
                pane.clear_ghost_text();
                pane.buffer.delete_selection();
                pane.buffer.insert_str(&to_insert);
                pane.on_content_changed();
                self.status_msg = Some("Inserted AI response at cursor".into());
                Task::none()
            }

            Message::CopyAiResponse(text) => {
                let to_copy = extract_code_block_or_text(&text);
                self.status_msg = Some("Copied AI response to clipboard".into());
                cosmic::iced::clipboard::write(to_copy)
            }
        }
    }
}

fn extract_code_block_or_text(text: &str) -> String {
    if let Some(start) = text.find("```") {
        let after_start = &text[start + 3..];
        if let Some(newline_pos) = after_start.find('\n') {
            let code_start = &after_start[newline_pos + 1..];
            if let Some(end_fence) = code_start.rfind("```") {
                return code_start[..end_fence].trim_end().to_string();
            }
        }
    }
    text.to_string()
}
