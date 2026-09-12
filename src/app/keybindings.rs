use crate::app::message::Message;
use crate::app::App;
use crate::editor::resolve_numpad_char;
use cosmic::app::Task;
use cosmic::iced::keyboard::{self, Key};
use cosmic::iced::{self};
use std::time::{Duration, Instant};

impl App {
    pub(crate) fn handle_key_event(&mut self, event: iced::Event) -> Task<Message> {
        match event {
            iced::Event::InputMethod(ime_event) => {
                match ime_event {
                    cosmic::iced::advanced::input_method::Event::Opened => {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.preedit = None;
                    }
                    cosmic::iced::advanced::input_method::Event::Commit(committed_text) => {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.preedit = None;
                        pane.buffer.insert_str(&committed_text);
                        pane.on_content_changed();
                        self.last_ime_commit = Instant::now();
                    }
                    cosmic::iced::advanced::input_method::Event::Preedit(preedit_text, selection) => {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        if preedit_text.is_empty() {
                            pane.preedit = None;
                        } else {
                            pane.preedit = Some((preedit_text, selection));
                        }
                    }
                    cosmic::iced::advanced::input_method::Event::Closed => {
                        let pane = self.current_pane_mut();
                        pane.preedit = None;
                    }
                }
                Task::none()
            }
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                physical_key,
                ..
            }) => {
                self.last_key_press = Instant::now();

                // Modal key handling: Enter (including Numpad Enter) confirms, Escape cancels
                if self.show_new_file_modal
                    || self.show_new_folder_modal
                    || self.show_rename_modal
                    || self.show_delete_modal
                {
                    let is_enter = matches!(&key, Key::Named(keyboard::key::Named::Enter))
                        || matches!(
                            &physical_key,
                            cosmic::iced::keyboard::key::Physical::Code(
                                cosmic::iced::keyboard::key::Code::NumpadEnter
                            )
                        )
                        || matches!(&key, Key::Character(c) if c == "\r" || c == "\n")
                        || text.as_deref() == Some("\r")
                        || text.as_deref() == Some("\n");

                    if is_enter {
                        if self.show_new_file_modal {
                            return self.handle_update(Message::ConfirmNewFile);
                        } else if self.show_new_folder_modal {
                            return self.handle_update(Message::ConfirmNewFolder);
                        } else if self.show_rename_modal {
                            return self.handle_update(Message::ConfirmRename);
                        } else if self.show_delete_modal {
                            return self.handle_update(Message::ConfirmDelete);
                        }
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::Escape)) {
                        if self.show_new_file_modal {
                            return self.handle_update(Message::CancelNewFile);
                        } else if self.show_new_folder_modal {
                            return self.handle_update(Message::CancelNewFolder);
                        } else if self.show_rename_modal {
                            return self.handle_update(Message::CancelRename);
                        } else if self.show_delete_modal {
                            return self.handle_update(Message::CancelDelete);
                        }
                    }
                    return Task::none();
                }

                // Tab Shortcuts
                // Shortcut: Ctrl + W (Close Active Tab)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("w"))
                {
                    let tab_idx = self.current_pane().active_tab_idx;
                    return self.handle_update(Message::CloseTab(self.active_pane, tab_idx));
                }

                // Shortcut: Ctrl + T (New Tab)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("t"))
                {
                    return self.handle_update(Message::NewTab(self.active_pane));
                }

                // Shortcut: Ctrl + Shift + Tab / Ctrl + PageUp (Previous Tab)
                if (modifiers.control()
                    && modifiers.shift()
                    && matches!(&key, Key::Named(keyboard::key::Named::Tab)))
                    || (modifiers.control()
                        && matches!(&key, Key::Named(keyboard::key::Named::PageUp)))
                {
                    return self.handle_update(Message::PrevTab(self.active_pane));
                }

                // Shortcut: Ctrl + Tab / Ctrl + PageDown (Next Tab)
                if (modifiers.control()
                    && !modifiers.shift()
                    && matches!(&key, Key::Named(keyboard::key::Named::Tab)))
                    || (modifiers.control()
                        && matches!(&key, Key::Named(keyboard::key::Named::PageDown)))
                {
                    return self.handle_update(Message::NextTab(self.active_pane));
                }

                // File Shortcuts
                // Shortcut: Ctrl + N (Create New File Prompt)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("n"))
                {
                    return self.handle_update(Message::PromptNewFile);
                }

                // Shortcut: Ctrl + O / Ctrl + Shift + O (Open File / Open Folder)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("o"))
                {
                    if modifiers.shift() {
                        return self.handle_update(Message::OpenFolderPrompt);
                    } else {
                        return self.handle_update(Message::OpenFilePrompt);
                    }
                }

                // Shortcut: Ctrl + S / Ctrl + Shift + S (Save / Save As)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("s"))
                {
                    if modifiers.shift() {
                        return self.handle_update(Message::SaveFileAsPrompt);
                    } else {
                        return self.handle_update(Message::SaveFile);
                    }
                }

                // Shortcut: Ctrl + B (Toggle Sidebar File Tree)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("b"))
                {
                    self.file_tree.is_visible = !self.file_tree.is_visible;
                    self.save_config();
                    return Task::none();
                }

                // Shortcut: Ctrl + \ or Ctrl + E (Toggle Split Layout)
                if modifiers.control()
                    && (matches!(&key, Key::Character(c) if c == "\\" || c.eq_ignore_ascii_case("e")))
                {
                    return self.handle_update(Message::ToggleSplit);
                }

                // Shortcut: Ctrl + M (Toggle Markdown Preview)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("m"))
                {
                    return self.handle_update(Message::ToggleMarkdownPreview);
                }

                // Shortcut: Ctrl + I or Alt + Enter (Trigger AI FIM manually)
                if (modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("i")))
                    || (modifiers.alt() && matches!(&key, Key::Named(keyboard::key::Named::Enter)))
                    || (modifiers.alt() && matches!(&key, Key::Character(c) if c == " "))
                {
                    return self.handle_update(Message::TriggerAiFim);
                }

                // Shortcut: Ctrl + Z (Undo)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("z"))
                {
                    if modifiers.shift() {
                        return self.handle_update(Message::Redo);
                    } else {
                        return self.handle_update(Message::Undo);
                    }
                }

                // Shortcut: Ctrl + Y (Redo)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("y"))
                {
                    return self.handle_update(Message::Redo);
                }

                // Shortcut: Ctrl + Shift + A (Toggle AI Chat Panel)
                if modifiers.control()
                    && modifiers.shift()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("a"))
                {
                    return self.handle_update(Message::ToggleAiChat);
                }

                // Shortcut: Ctrl + A (Select All)
                if modifiers.control()
                    && !modifiers.shift()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("a"))
                {
                    return self.handle_update(Message::SelectAll);
                }

                // Shortcut: Ctrl + C (Copy)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("c"))
                {
                    return self.handle_update(Message::Copy);
                }

                // Shortcut: Ctrl + X (Cut)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("x"))
                {
                    return self.handle_update(Message::Cut);
                }

                // Shortcut: Ctrl + F (Find in File)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("f"))
                {
                    return self.handle_update(Message::ToggleSearch);
                }

                // Shortcut: Ctrl + / (Toggle Comment)
                if modifiers.control() && matches!(&key, Key::Character(c) if c == "/") {
                    return self.handle_update(Message::ToggleComment);
                }

                // Shortcut: Ctrl + Shift + K (Delete Line)
                if modifiers.control()
                    && modifiers.shift()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("k"))
                {
                    return self.handle_update(Message::DeleteLine);
                }

                // Shortcut: Ctrl + D (Duplicate Line)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("d"))
                {
                    return self.handle_update(Message::DuplicateLine);
                }

                // Shortcut: Ctrl + V (Paste)
                if modifiers.control()
                    && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("v"))
                {
                    return self.handle_update(Message::Paste);
                }

                // Tab key (Indentation / unindentation)
                if !modifiers.control() && matches!(&key, Key::Named(keyboard::key::Named::Tab)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    if modifiers.shift() {
                        pane.buffer.unindent_selection();
                        pane.on_content_changed();
                    } else if pane.buffer.selection_anchor.is_some() {
                        pane.buffer.indent_selection();
                        pane.on_content_changed();
                    } else if !pane.accept_ghost_text() {
                        pane.buffer.insert_str("    ");
                        pane.on_content_changed();
                    }
                    return Task::none();
                }

                // Escape key
                if matches!(&key, Key::Named(keyboard::key::Named::Escape)) {
                    if self.current_pane().is_search_open {
                        return self.handle_update(Message::CloseSearch);
                    }
                    if self.context_menu.is_some() {
                        self.context_menu = None;
                        return Task::none();
                    }
                    if self.file_tree_context_menu.is_some() {
                        self.file_tree_context_menu = None;
                        return Task::none();
                    }
                    if self.active_header_menu.is_some() {
                        self.active_header_menu = None;
                        return Task::none();
                    }
                    if self.show_edit_menu {
                        self.show_edit_menu = false;
                        return Task::none();
                    }
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        pane.preedit = None;
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.selection_anchor = None;
                    return Task::none();
                }

                // Backspace
                if matches!(&key, Key::Named(keyboard::key::Named::Backspace)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.delete_backspace();
                    pane.on_content_changed();
                    return Task::none();
                }

                // Numpad input: intercept numpad physical keys before Delete and Navigation.
                if !modifiers.control() && !modifiers.alt() {
                    if let Some(num_str) = resolve_numpad_char(&physical_key) {
                        let pane = self.current_pane_mut();
                        if pane.preedit.is_none() {
                            pane.clear_ghost_text();
                            pane.buffer.insert_str(num_str);
                            pane.on_content_changed();
                            return Task::none();
                        }
                    }
                }

                // Delete
                if matches!(&key, Key::Named(keyboard::key::Named::Delete)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.delete_forward();
                    pane.on_content_changed();
                    return Task::none();
                }

                // Enter (including Numpad Enter)
                let is_enter = matches!(&key, Key::Named(keyboard::key::Named::Enter))
                    || matches!(
                        &physical_key,
                        cosmic::iced::keyboard::key::Physical::Code(
                            cosmic::iced::keyboard::key::Code::NumpadEnter
                        )
                    )
                    || matches!(&key, Key::Character(c) if c == "\r" || c == "\n")
                    || text.as_deref() == Some("\r")
                    || text.as_deref() == Some("\n")
                    || text.as_deref() == Some("\r\n");

                if is_enter {
                    if self.current_pane().is_search_open {
                        if modifiers.shift() {
                            return self.handle_update(Message::PrevSearchMatch);
                        } else {
                            return self.handle_update(Message::NextSearchMatch);
                        }
                    }

                    if self.last_ime_commit.elapsed() < Duration::from_millis(100) {
                        return Task::none();
                    }
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.insert_char('\n');
                    pane.on_content_changed();
                    return Task::none();
                }

                let shift = modifiers.shift();

                // Word Navigation: Ctrl + Left / Ctrl + Right
                if modifiers.control()
                    && matches!(&key, Key::Named(keyboard::key::Named::ArrowLeft))
                {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.move_word_left(shift);
                    pane.mark_cursor_moved();
                    return Task::none();
                }
                if modifiers.control()
                    && matches!(&key, Key::Named(keyboard::key::Named::ArrowRight))
                {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.move_word_right(shift);
                    pane.mark_cursor_moved();
                    return Task::none();
                }

                // Cursor navigation
                if matches!(&key, Key::Named(keyboard::key::Named::ArrowLeft)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.move_left(shift);
                    pane.mark_cursor_moved();
                    return Task::none();
                }
                if matches!(&key, Key::Named(keyboard::key::Named::ArrowRight)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.move_right(shift);
                    pane.mark_cursor_moved();
                    return Task::none();
                }
                if matches!(&key, Key::Named(keyboard::key::Named::ArrowUp)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.move_up(shift);
                    pane.mark_cursor_moved();
                    return Task::none();
                }
                if matches!(&key, Key::Named(keyboard::key::Named::ArrowDown)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.move_down(shift);
                    pane.mark_cursor_moved();
                    return Task::none();
                }
                if matches!(&key, Key::Named(keyboard::key::Named::Home)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.move_line_start(shift);
                    pane.mark_cursor_moved();
                    return Task::none();
                }
                if matches!(&key, Key::Named(keyboard::key::Named::End)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    pane.buffer.move_line_end(shift);
                    pane.mark_cursor_moved();
                    return Task::none();
                }
                if matches!(&key, Key::Named(keyboard::key::Named::PageUp)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    for _ in 0..20 {
                        pane.buffer.move_up(shift);
                    }
                    pane.mark_cursor_moved();
                    return Task::none();
                }
                if matches!(&key, Key::Named(keyboard::key::Named::PageDown)) {
                    let pane = self.current_pane_mut();
                    if pane.preedit.is_some() {
                        return Task::none();
                    }
                    pane.clear_ghost_text();
                    for _ in 0..20 {
                        pane.buffer.move_down(shift);
                    }
                    pane.mark_cursor_moved();
                    return Task::none();
                }

                // Regular Character Input & Numpad Numbers/Operators fallback
                if !modifiers.control() && !modifiers.alt() {
                    let input_str: Option<String> = text
                        .as_deref()
                        .filter(|s| !s.is_empty() && !s.chars().all(|c| c.is_control()))
                        .map(|s| s.to_string())
                        .or_else(|| {
                            if let Key::Character(ref c) = key {
                                let s = c.as_str();
                                if !s.is_empty() && !s.chars().all(|c| c.is_control()) {
                                    return Some(s.to_string());
                                }
                            }
                            resolve_numpad_char(&physical_key).map(|s| s.to_string())
                        });

                    if let Some(s) = input_str {
                        let pane = self.current_pane_mut();
                        if pane.preedit.is_none() {
                            pane.clear_ghost_text();
                            pane.buffer.insert_str(&s);
                            pane.on_content_changed();
                            return Task::none();
                        }
                    }
                }
                Task::none()
            }
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                self.mouse_pos = (position.x, position.y);
                if position.x > self.window_size.0 {
                    self.window_size.0 = position.x + 20.0;
                }
                if position.y > self.window_size.1 {
                    self.window_size.1 = position.y + 20.0;
                }
                Task::none()
            }
            iced::Event::Window(iced::window::Event::Resized(size)) => {
                self.window_size = (size.width, size.height);
                Task::none()
            }
            iced::Event::Window(iced::window::Event::Opened { size, .. }) => {
                self.window_size = (size.width, size.height);
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
