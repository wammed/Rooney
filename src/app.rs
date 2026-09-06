use crate::ai::{AiStatus, OllamaClient};
use crate::editor::pane::{EditorPane, PaneId, SplitLayout};
use crate::font::FontManager;
use crate::fs::tree::FileTree;
use crate::theme::themes::{EditorTheme, ThemeId};
use crate::ui::canvas_editor::EditorCanvas;
use crate::ui::file_tree_view::{view_file_tree, FileTreeMessage};
use crate::ui::markdown_view::view_markdown;

use cosmic::app::{Core, Task};
use cosmic::iced::keyboard::{self, Key};
use cosmic::iced::widget::canvas::Canvas;
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, column, container, dropdown, row, slider, text, Space};
use cosmic::{executor, iced};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Event(iced::Event),
    FileTreeMsg(FileTreeMessage),
    ToggleSplit,
    SetActivePane(PaneId),
    ClickPane(PaneId, usize, usize),
    ScrollPane(PaneId, f32),
    TogglePaneMode(PaneId),
    ToggleMarkdownPreview,
    SelectTheme(usize),
    SelectFont(usize),
    IncreaseFontSize,
    DecreaseFontSize,
    ToggleAi,
    TriggerAiFim,
    AiFimResult(PaneId, Result<String, String>),
    AiModelsFetched(Result<Vec<String>, String>),
    SelectAiModel(usize),
    SaveFile,
    NewFile,
    ToggleSettings,
    ChangeOpacity(f32),
    ChangeDimming(f32),
    CloseSettings,
}

pub struct App {
    core: Core,
    theme: EditorTheme,
    font_manager: FontManager,
    file_tree: FileTree,
    split_layout: SplitLayout,
    left_pane: EditorPane,
    right_pane: EditorPane,
    active_pane: PaneId,
    ollama: OllamaClient,
    ai_status: AiStatus,
    show_settings: bool,
    status_msg: Option<String>,
    theme_names: Vec<String>,
    font_names: Vec<String>,
    ai_request_pending: bool,
    _last_key_press: Instant,
}

impl App {
    fn current_pane(&self) -> &EditorPane {
        match self.active_pane {
            PaneId::Left => &self.left_pane,
            PaneId::Right => &self.right_pane,
        }
    }

    fn current_pane_mut(&mut self) -> &mut EditorPane {
        match self.active_pane {
            PaneId::Left => &mut self.left_pane,
            PaneId::Right => &mut self.right_pane,
        }
    }

    fn open_file_in_active_pane(&mut self, path: &Path) {
        let pane = self.current_pane_mut();
        if let Err(e) = pane.load_file(path) {
            self.status_msg = Some(format!("Failed to open file: {e}"));
        } else {
            self.status_msg = Some(format!("Opened {}", pane.file_name));
        }
    }
}

impl cosmic::Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "org.pop_os.CosmicCode";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let file_tree = FileTree::new(&current_dir);
        let theme = EditorTheme::default();
        let font_manager = FontManager::new();

        let mut left_pane = EditorPane::new(PaneId::Left, "Welcome");
        let welcome_content = r#"// CosmicCode (Rooney) - Cosmic-Native Lightweight Code & Markdown Editor
// Fast in-process editing with Ropey buffer & Tree-sitter highlighting
// Built-in Local AI Fill-in-the-Middle (FIM) powered by Ollama

fn main() {
    println!("Welcome to CosmicCode!");
    println!("Press Tab to accept inline AI suggestions.");
    println!("Use the Split button in the top bar to toggle side-by-side editing.");
}
"#;
        left_pane.buffer = crate::editor::buffer::TextBuffer::new(welcome_content);
        left_pane.highlighter = crate::syntax::Highlighter::new(crate::syntax::SupportedLanguage::Rust);
        left_pane.highlighter.update_source(welcome_content);

        let mut right_pane = EditorPane::new(PaneId::Right, "Preview / Editor 2");
        let sample_md = r#"# CosmicCode Preview
A lightweight, fast, in-process code and markdown editor.

## Key Features
- **20 Classic & Neon Themes**: Tokyo Night, Cyberpunk Neon, Matrix Green, etc.
- **Side-by-Side Split View**: Dual pane editing with independent files.
- **Rich File Tree**: Hierarchical directory browsing with colored Nerd Font icons.
- **Local AI FIM**: High speed code completion using local Ollama models.
- **Wayland Native Transparency**: Custom opacity & dimming overlays.
"#;
        right_pane.buffer = crate::editor::buffer::TextBuffer::new(sample_md);
        right_pane.highlighter = crate::syntax::Highlighter::new(crate::syntax::SupportedLanguage::Markdown);
        right_pane.highlighter.update_source(sample_md);
        right_pane.is_markdown_preview = false;
        right_pane.markdown_doc = Some(crate::markdown::MarkdownDocument::parse(sample_md));

        let theme_names: Vec<String> = ThemeId::ALL.iter().map(|t| t.display_name().to_string()).collect();
        let font_names = font_manager.available_fonts.clone();

        let ollama = OllamaClient::default();
        let ai_status = AiStatus::Ready(ollama.active_model.clone());

        let mut app = Self {
            core,
            theme,
            font_manager,
            file_tree,
            split_layout: SplitLayout::Split,
            left_pane,
            right_pane,
            active_pane: PaneId::Left,
            ollama,
            ai_status,
            show_settings: false,
            status_msg: Some("CosmicCode Ready".to_string()),
            theme_names,
            font_names,
            ai_request_pending: false,
            _last_key_press: Instant::now(),
        };

        let readme_path = current_dir.join("README.md");
        if readme_path.exists() {
            let _ = app.right_pane.load_file(&readme_path);
            app.right_pane.is_markdown_preview = false;
        }

        let client_clone = app.ollama.clone();
        let fetch_models_task = Task::perform(
            async move {
                let mut c = client_clone;
                c.fetch_models().await
            },
            |res| cosmic::Action::App(Message::AiModelsFetched(res)),
        );

        (app, fetch_models_task)
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let timer = iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick);
        let events = iced::event::listen().map(Message::Event);
        iced::Subscription::batch(vec![timer, events])
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
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
                    return self.update(Message::TriggerAiFim);
                }
                Task::none()
            }

            Message::Event(event) => {
                match event {
                    iced::Event::InputMethod(ime_event) => {
                        match ime_event {
                            cosmic::iced::advanced::input_method::Event::Commit(committed_text) => {
                                let pane = self.current_pane_mut();
                                pane.clear_ghost_text();
                                pane.preedit = None;
                                pane.buffer.insert_str(&committed_text);
                                pane.on_content_changed();
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
                            _ => {}
                        }
                        return Task::none();
                    }
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        text,
                        ..
                    }) => {
                        self._last_key_press = Instant::now();

                    // Shortcut: Ctrl + S (Save)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("s")) {
                        return self.update(Message::SaveFile);
                    }

                    // Shortcut: Ctrl + B (Toggle Sidebar File Tree)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("b")) {
                        self.file_tree.is_visible = !self.file_tree.is_visible;
                        return Task::none();
                    }

                    // Shortcut: Ctrl + \ or Ctrl + E (Toggle Split Layout)
                    if modifiers.control()
                        && (matches!(&key, Key::Character(c) if c == "\\" || c.eq_ignore_ascii_case("e")))
                    {
                        return self.update(Message::ToggleSplit);
                    }

                    // Shortcut: Ctrl + M (Toggle Markdown Preview)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("m")) {
                        return self.update(Message::ToggleMarkdownPreview);
                    }

                    // Shortcut: Ctrl + Space (Trigger AI FIM manually)
                    if modifiers.control()
                        && (matches!(&key, Key::Character(c) if c == " ")
                            || matches!(&key, Key::Named(k) if format!("{k:?}") == "Space"))
                    {
                        return self.update(Message::TriggerAiFim);
                    }

                    // Shortcut: Ctrl + Z (Undo)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("z")) {
                        if modifiers.shift() {
                            self.current_pane_mut().buffer.redo();
                        } else {
                            self.current_pane_mut().buffer.undo();
                        }
                        self.current_pane_mut().on_content_changed();
                        return Task::none();
                    }

                    // Shortcut: Ctrl + Y (Redo)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("y")) {
                        self.current_pane_mut().buffer.redo();
                        self.current_pane_mut().on_content_changed();
                        return Task::none();
                    }

                    // Shortcut: Ctrl + A (Select All)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("a")) {
                        self.current_pane_mut().buffer.select_all();
                        return Task::none();
                    }

                    // Tab key
                    if matches!(&key, Key::Named(keyboard::key::Named::Tab)) {
                        let pane = self.current_pane_mut();
                        if !pane.accept_ghost_text() {
                            pane.buffer.insert_str("    ");
                            pane.on_content_changed();
                        }
                        return Task::none();
                    }

                    // Escape key
                    if matches!(&key, Key::Named(keyboard::key::Named::Escape)) {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.selection_anchor = None;
                        return Task::none();
                    }

                    // Backspace
                    if matches!(&key, Key::Named(keyboard::key::Named::Backspace)) {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.delete_backspace();
                        pane.on_content_changed();
                        return Task::none();
                    }

                    // Delete
                    if matches!(&key, Key::Named(keyboard::key::Named::Delete)) {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.delete_forward();
                        pane.on_content_changed();
                        return Task::none();
                    }

                    // Enter
                    if matches!(&key, Key::Named(keyboard::key::Named::Enter)) {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.insert_char('\n');
                        pane.on_content_changed();
                        return Task::none();
                    }

                    // Arrow navigation
                    let shift = modifiers.shift();
                    if matches!(&key, Key::Named(keyboard::key::Named::ArrowLeft)) {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.move_left(shift);
                        return Task::none();
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::ArrowRight)) {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.move_right(shift);
                        return Task::none();
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::ArrowUp)) {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.move_up(shift);
                        return Task::none();
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::ArrowDown)) {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.move_down(shift);
                        return Task::none();
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::Home)) {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.move_line_start(shift);
                        return Task::none();
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::End)) {
                        let pane = self.current_pane_mut();
                        pane.clear_ghost_text();
                        pane.buffer.move_line_end(shift);
                        return Task::none();
                    }

                    // Regular Character Input
                    if !modifiers.control() && !modifiers.alt() {
                        if let Some(t) = text {
                            let s = t.as_str();
                            if !s.is_empty() && !s.chars().all(|c| c.is_control()) {
                                let pane = self.current_pane_mut();
                                if pane.preedit.is_none() {
                                    pane.clear_ghost_text();
                                    pane.buffer.insert_str(s);
                                    pane.on_content_changed();
                                    return Task::none();
                                }
                            }
                        }
                    }
                        Task::none()
                    }
                    _ => Task::none(),
                }
            }

            Message::ClickPane(pane_id, line, col) => {
                self.active_pane = pane_id;
                let pane = self.current_pane_mut();
                pane.clear_ghost_text();
                pane.preedit = None;
                pane.buffer.cursor = (line, col);
                pane.buffer.clamp_cursor();
                pane.buffer.selection_anchor = None;
                Task::none()
            }

            Message::ScrollPane(pane_id, delta_y) => {
                let target_pane = match pane_id {
                    PaneId::Left => &mut self.left_pane,
                    PaneId::Right => &mut self.right_pane,
                };
                target_pane.scroll_y = (target_pane.scroll_y + delta_y).max(0.0);
                Task::none()
            }

            Message::TogglePaneMode(pane_id) => {
                let target_pane = match pane_id {
                    PaneId::Left => &mut self.left_pane,
                    PaneId::Right => &mut self.right_pane,
                };
                target_pane.is_markdown_preview = !target_pane.is_markdown_preview;
                if target_pane.is_markdown_preview && target_pane.markdown_doc.is_none() {
                    let text = target_pane.buffer.full_text();
                    target_pane.markdown_doc = Some(crate::markdown::MarkdownDocument::parse(&text));
                }
                Task::none()
            }

            Message::FileTreeMsg(msg) => match msg {
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
                    Task::none()
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
                }
                Task::none()
            }

            Message::SelectFont(index) => {
                if let Some(font) = self.font_names.get(index) {
                    self.font_manager.set_font(font.clone());
                    self.status_msg = Some(format!("Font: {}", font));
                }
                Task::none()
            }

            Message::IncreaseFontSize => {
                let s = self.font_manager.font_size + 1.0;
                self.font_manager.set_font_size(s);
                Task::none()
            }

            Message::DecreaseFontSize => {
                let s = self.font_manager.font_size - 1.0;
                self.font_manager.set_font_size(s);
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
                Task::none()
            }

            Message::TriggerAiFim => {
                if !self.ollama.is_enabled || self.ai_request_pending {
                    return Task::none();
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
                        let target_pane = match pane_id {
                            PaneId::Left => &mut self.left_pane,
                            PaneId::Right => &mut self.right_pane,
                        };
                        if !text.is_empty() {
                            target_pane.ghost_text = Some(text);
                            self.status_msg = Some("AI: Suggestion ready (Press Tab to accept)".into());
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
                            self.status_msg = Some(format!("Ollama: {} models found", models.len()));
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
                }
                Task::none()
            }

            Message::SaveFile => {
                let pane = self.current_pane_mut();
                if let Err(e) = pane.save_file() {
                    self.status_msg = Some(format!("Save failed: {e}"));
                } else {
                    self.status_msg = Some(format!("Saved {}", pane.file_name));
                    self.file_tree.refresh();
                }
                Task::none()
            }

            Message::NewFile => {
                let pane = self.current_pane_mut();
                pane.file_path = None;
                pane.file_name = "Untitled".into();
                pane.buffer = crate::editor::buffer::TextBuffer::default();
                pane.on_content_changed();
                self.status_msg = Some("New file created".into());
                Task::none()
            }

            Message::ToggleSettings => {
                self.show_settings = !self.show_settings;
                Task::none()
            }

            Message::ChangeOpacity(val) => {
                self.theme.opacity = val;
                Task::none()
            }

            Message::ChangeDimming(val) => {
                self.theme.dimming = val;
                Task::none()
            }

            Message::CloseSettings => {
                self.show_settings = false;
                Task::none()
            }
        }
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let is_split = self.split_layout == SplitLayout::Split;
        let is_preview = self.current_pane().is_markdown_preview;
        let ai_on = self.ollama.is_enabled;

        vec![
            button::text(if self.file_tree.is_visible {
                "  Files "
            } else {
                "  Files "
            })
            .on_press(Message::FileTreeMsg(FileTreeMessage::ToggleVisibility))
            .into(),

            button::text("  New ").on_press(Message::NewFile).into(),
            button::text(" 󰆓 Save ").on_press(Message::SaveFile).into(),

            button::text(if is_split {
                "  Single "
            } else {
                "  Split "
            })
            .on_press(Message::ToggleSplit)
            .into(),

            button::text(if is_preview {
                "  Edit "
            } else {
                "  Preview "
            })
            .on_press(Message::ToggleMarkdownPreview)
            .into(),

            button::text(if ai_on {
                " 󰚩 AI: ON "
            } else {
                " 󰚩 AI: OFF "
            })
            .on_press(Message::ToggleAi)
            .into(),

            button::text(" 󰒓 Aesthetics ").on_press(Message::ToggleSettings).into(),
        ]
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        let cur_theme_idx = ThemeId::ALL
            .iter()
            .position(|&t| t == self.theme.config.id)
            .unwrap_or(0);

        let cur_font_idx = self
            .font_names
            .iter()
            .position(|f| f == &self.font_manager.current_font)
            .unwrap_or(0);

        vec![
            button::text(" A- ").on_press(Message::DecreaseFontSize).into(),
            button::text(" A+ ").on_press(Message::IncreaseFontSize).into(),

            dropdown(
                &self.theme_names,
                Some(cur_theme_idx),
                Message::SelectTheme,
            )
            .into(),

            dropdown(
                &self.font_names,
                Some(cur_font_idx),
                Message::SelectFont,
            )
            .into(),
        ]
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let theme = &self.theme;
        let font_name = &self.font_manager.current_font;
        let font_size = self.font_manager.font_size;

        // Left Pane view
        let left_element: Element<'_, Self::Message> = if self.left_pane.is_markdown_preview {
            if let Some(ref doc) = self.left_pane.markdown_doc {
                view_markdown(doc, theme, font_name)
            } else {
                container(
                    text("Markdown document empty")
                        .class(cosmic::theme::Text::Color(theme.config.fg)),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        } else {
            let canvas = Canvas::new(EditorCanvas::new(
                &self.left_pane,
                theme,
                self.active_pane == PaneId::Left,
                font_name,
                font_size,
            ))
            .width(Length::Fill)
            .height(Length::Fill);

            container(canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        // Tab header for left pane (with independent Edit/Preview toggle)
        let left_tab = row::with_capacity(4)
            .push(
                button::custom(
                    row::with_capacity(2)
                        .push(
                            text(format!(
                                "{} {}{}",
                                if self.active_pane == PaneId::Left { "●" } else { "○" },
                                self.left_pane.file_name,
                                if self.left_pane.buffer.is_modified { " *" } else { "" }
                            ))
                            .size(12.0)
                            .class(cosmic::theme::Text::Color(if self.active_pane == PaneId::Left {
                                theme.config.accent
                            } else {
                                theme.config.fg
                            })),
                        )
                        .align_y(Alignment::Center),
                )
                .on_press(Message::SetActivePane(PaneId::Left))
                .padding([4, 8]),
            )
            .push(cosmic::iced::widget::space::horizontal())
            .push(
                button::text(if self.left_pane.is_markdown_preview {
                    "  Edit "
                } else {
                    "  Preview "
                })
                .on_press(Message::TogglePaneMode(PaneId::Left))
                .padding([2, 6]),
            )
            .align_y(Alignment::Center)
            .padding([2, 8])
            .width(Length::Fill);

        let left_col = column::with_capacity(2)
            .push(left_tab)
            .push(left_element)
            .width(Length::Fill)
            .height(Length::Fill);

        let left_pane_box = container(left_col)
            .width(Length::Fill)
            .height(Length::Fill);

        // Editor area (Single or Split)
        let editor_area: Element<'_, Self::Message> = if self.split_layout == SplitLayout::Split {
            let right_element: Element<'_, Self::Message> = if self.right_pane.is_markdown_preview {
                if let Some(ref doc) = self.right_pane.markdown_doc {
                    view_markdown(doc, theme, font_name)
                } else {
                    container(
                        text("Markdown document empty")
                            .class(cosmic::theme::Text::Color(theme.config.fg)),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                }
            } else {
                let canvas = Canvas::new(EditorCanvas::new(
                    &self.right_pane,
                    theme,
                    self.active_pane == PaneId::Right,
                    font_name,
                    font_size,
                ))
                .width(Length::Fill)
                .height(Length::Fill);

                container(canvas)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            };

            // Tab header for right pane (with independent Edit/Preview toggle)
            let right_tab = row::with_capacity(4)
                .push(
                    button::custom(
                        row::with_capacity(2)
                            .push(
                                text(format!(
                                    "{} {}{}",
                                    if self.active_pane == PaneId::Right { "●" } else { "○" },
                                    self.right_pane.file_name,
                                    if self.right_pane.buffer.is_modified { " *" } else { "" }
                                ))
                                .size(12.0)
                                .class(cosmic::theme::Text::Color(if self.active_pane == PaneId::Right {
                                    theme.config.accent
                                } else {
                                    theme.config.fg
                                })),
                            )
                            .align_y(Alignment::Center),
                    )
                    .on_press(Message::SetActivePane(PaneId::Right))
                    .padding([4, 8]),
                )
                .push(cosmic::iced::widget::space::horizontal())
                .push(
                    button::text(if self.right_pane.is_markdown_preview {
                        "  Edit "
                    } else {
                        "  Preview "
                    })
                    .on_press(Message::TogglePaneMode(PaneId::Right))
                    .padding([2, 6]),
                )
                .align_y(Alignment::Center)
                .padding([2, 8])
                .width(Length::Fill);

            let right_col = column::with_capacity(2)
                .push(right_tab)
                .push(right_element)
                .width(Length::Fill)
                .height(Length::Fill);

            let right_pane_box = container(right_col)
                .width(Length::Fill)
                .height(Length::Fill);

            row::with_capacity(3)
                .push(left_pane_box)
                .push(Space::new().width(Length::Fixed(4.0)))
                .push(right_pane_box)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            left_pane_box.into()
        };

        let mut main_row = row::with_capacity(3).width(Length::Fill).height(Length::Fill);

        if self.file_tree.is_visible {
            let tree_view = view_file_tree(&self.file_tree, theme, font_name, Message::FileTreeMsg);
            main_row = main_row.push(tree_view);
            main_row = main_row.push(Space::new().width(Length::Fixed(1.0)));
        }

        main_row = main_row.push(editor_area);

        // Status Bar
        let current_pane = self.current_pane();
        let cursor = current_pane.buffer.cursor;
        let total_lines = current_pane.buffer.line_count();
        let total_chars = current_pane.buffer.rope.len_chars();
        let lang_name = current_pane.highlighter.lang.display_name();

        let status_left = row::with_capacity(4)
            .spacing(12)
            .push(
                text(format!(
                    "[{}] {}",
                    match self.active_pane {
                        PaneId::Left => "LEFT",
                        PaneId::Right => "RIGHT",
                    },
                    current_pane.file_name
                ))
                .size(11.0)
                .class(cosmic::theme::Text::Color(theme.config.accent)),
            )
            .push(
                text(lang_name)
                    .size(11.0)
                    .class(cosmic::theme::Text::Color(theme.config.function)),
            )
            .push(
                text(self.status_msg.as_deref().unwrap_or("UTF-8"))
                    .size(11.0)
                    .class(cosmic::theme::Text::Color(theme.config.comment)),
            )
            .align_y(Alignment::Center);

        let status_right = row::with_capacity(4)
            .spacing(14)
            .push(
                text(format!("Ln {}, Col {}", cursor.0 + 1, cursor.1 + 1))
                    .size(11.0)
                    .class(cosmic::theme::Text::Color(theme.config.fg)),
            )
            .push(
                text(format!("{} lines, {} chars", total_lines, total_chars))
                    .size(11.0)
                    .class(cosmic::theme::Text::Color(theme.config.comment)),
            )
            .push(
                text(self.ai_status.display_text())
                    .size(11.0)
                    .class(cosmic::theme::Text::Color(if self.ollama.is_enabled {
                        theme.config.accent
                    } else {
                        theme.config.comment
                    })),
            )
            .align_y(Alignment::Center);

        let status_bar = row::with_capacity(3)
            .push(status_left)
            .push(cosmic::iced::widget::space::horizontal())
            .push(status_right)
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .padding([4, 12]);

        let status_container = container(status_bar).width(Length::Fill);

        // Settings Modal / Overlay (Transparency & Aesthetics)
        if self.show_settings {
            let cur_model_idx = self
                .ollama
                .available_models
                .iter()
                .position(|m| m == &self.ollama.active_model)
                .unwrap_or(0);

            let settings_box = container(
                column::with_capacity(8)
                    .spacing(12)
                    .padding(16)
                    .push(
                        text::title4("󰒓 Aesthetics & AI Configuration")
                            .class(cosmic::theme::Text::Color(theme.config.accent)),
                    )
                    .push(text(format!(
                        "Wayland Alpha Opacity: {:.0}%",
                        self.theme.opacity * 100.0
                    )))
                    .push(
                        slider(0.1..=1.0_f32, self.theme.opacity, Message::ChangeOpacity)
                            .step(0.05_f32),
                    )
                    .push(text(format!(
                        "Background Dimming Overlay: {:.0}%",
                        self.theme.dimming * 100.0
                    )))
                    .push(
                        slider(0.0..=1.0_f32, self.theme.dimming, Message::ChangeDimming)
                            .step(0.05_f32),
                    )
                    .push(text("Local AI Model (Ollama):"))
                    .push(
                        dropdown(
                            &self.ollama.available_models,
                            Some(cur_model_idx),
                            Message::SelectAiModel,
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        button::suggested("Close")
                            .on_press(Message::CloseSettings),
                    ),
            )
            .padding(16)
            .width(Length::Fixed(360.0));

            return column::with_capacity(3)
                .push(main_row)
                .push(settings_box)
                .push(status_container)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        column::with_capacity(2)
            .push(main_row)
            .push(status_container)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
