use crate::ai::{AiStatus, OllamaClient};
use crate::config::AppConfig;
use crate::editor::{resolve_numpad_char, EditorPane, PaneId, SplitLayout};
use crate::font::FontManager;
use crate::fs::tree::FileTree;
use crate::theme::themes::{EditorTheme, ThemeId};
use crate::ui::canvas_editor::EditorCanvas;
use crate::ui::file_tree_view::{view_file_tree, FileTreeMessage};
use crate::ui::markdown_view::view_markdown;

use cosmic::app::{Core, Task};
use cosmic::iced::keyboard::{self, Key};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::iced::widget::stack;
use cosmic::widget::{button, column, container, dropdown, row, slider, text, text_input, Space};
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
    DragSelect(PaneId, usize, usize),
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
    OpenFilePrompt,
    FileOpened(Option<PathBuf>),
    OpenFolderPrompt,
    FolderOpened(Option<PathBuf>),
    SaveFile,
    SaveFileAsPrompt,
    FileSavedAs(PaneId, Option<PathBuf>),
    PromptNewFile,
    NewFileNameChanged(String),
    ConfirmNewFile,
    CancelNewFile,
    BrowseNewFileFolder,
    NewFileFolderSelected(Option<PathBuf>),
    Copy,
    Cut,
    Paste,
    ClipboardPasted(Option<String>),
    SelectAll,
    Undo,
    Redo,
    OpenContextMenu(PaneId, f32, f32),
    CloseContextMenu,
    ToggleEditMenu,
    CloseEditMenu,
    ToggleSettings,
    ChangeOpacity(f32),
    ChangeDimming(f32),
    CloseSettings,
}

pub struct App {
    core: Core,
    config: AppConfig,
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
    show_edit_menu: bool,
    context_menu: Option<(PaneId, f32, f32)>,
    status_msg: Option<String>,
    theme_names: Vec<String>,
    font_names: Vec<String>,
    ai_request_pending: bool,
    _last_key_press: Instant,
    last_ime_commit: Instant,

    // New file modal state
    show_new_file_modal: bool,
    new_file_name_input: String,
    new_file_target_dir: PathBuf,
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
        let _ = self.config.save();
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
        let config = AppConfig::load();
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut file_tree = FileTree::new(&current_dir);
        file_tree.is_visible = config.file_tree_visible;

        let mut theme = EditorTheme::default();
        theme.set_theme(config.theme);
        theme.opacity = config.opacity;
        theme.dimming = config.dimming;

        let mut font_manager = FontManager::new();
        if font_manager.available_fonts.contains(&config.font) {
            font_manager.set_font(config.font.clone());
        }
        font_manager.set_font_size(config.font_size);

        let split_layout = if config.split_layout == "Single" {
            SplitLayout::Single
        } else {
            SplitLayout::Split
        };

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

        let mut ollama = OllamaClient::default();
        ollama.is_enabled = config.ai_enabled;
        ollama.active_model = config.ai_model.clone();

        let ai_status = if ollama.is_enabled {
            AiStatus::Ready(ollama.active_model.clone())
        } else {
            AiStatus::Disabled
        };

        let mut app = Self {
            core,
            config,
            theme,
            font_manager,
            file_tree,
            split_layout,
            left_pane,
            right_pane,
            active_pane: PaneId::Left,
            ollama,
            ai_status,
            show_settings: false,
            show_edit_menu: false,
            context_menu: None,
            status_msg: Some("CosmicCode Ready".to_string()),
            theme_names,
            font_names,
            ai_request_pending: false,
            _last_key_press: Instant::now(),
            last_ime_commit: Instant::now(),
            show_new_file_modal: false,
            new_file_name_input: String::new(),
            new_file_target_dir: current_dir.clone(),
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
                        return Task::none();
                    }
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        text,
                        physical_key,
                        ..
                    }) => {
                        self._last_key_press = Instant::now();

                        // Modal key handling: Enter (including Numpad Enter) confirms, Escape cancels
                        if self.show_new_file_modal {
                            let is_enter = matches!(&key, Key::Named(keyboard::key::Named::Enter))
                                || matches!(&physical_key, cosmic::iced::keyboard::key::Physical::Code(cosmic::iced::keyboard::key::Code::NumpadEnter))
                                || matches!(&key, Key::Character(c) if c == "\r" || c == "\n")
                                || text.as_deref() == Some("\r")
                                || text.as_deref() == Some("\n");

                            if is_enter {
                                return self.update(Message::ConfirmNewFile);
                            }
                            if matches!(&key, Key::Named(keyboard::key::Named::Escape)) {
                                return self.update(Message::CancelNewFile);
                            }
                            return Task::none();
                        }

                        // Shortcut: Ctrl + N (Create New File Prompt)
                        if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("n")) {
                            return self.update(Message::PromptNewFile);
                        }

                        // Shortcut: Ctrl + O / Ctrl + Shift + O (Open File / Open Folder)
                        if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("o")) {
                            if modifiers.shift() {
                                return self.update(Message::OpenFolderPrompt);
                            } else {
                                return self.update(Message::OpenFilePrompt);
                            }
                        }

                        // Shortcut: Ctrl + S / Ctrl + Shift + S (Save / Save As)
                        if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("s")) {
                            if modifiers.shift() {
                                return self.update(Message::SaveFileAsPrompt);
                            } else {
                                return self.update(Message::SaveFile);
                            }
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

                    // Shortcut: Ctrl + I or Alt + Enter (Trigger AI FIM manually, freeing Ctrl + Space for IME)
                    if (modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("i")))
                        || (modifiers.alt() && matches!(&key, Key::Named(keyboard::key::Named::Enter)))
                        || (modifiers.alt() && matches!(&key, Key::Character(c) if c == " "))
                    {
                        return self.update(Message::TriggerAiFim);
                    }

                    // Shortcut: Ctrl + Z (Undo)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("z")) {
                        if modifiers.shift() {
                            return self.update(Message::Redo);
                        } else {
                            return self.update(Message::Undo);
                        }
                    }

                    // Shortcut: Ctrl + Y (Redo)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("y")) {
                        return self.update(Message::Redo);
                    }

                    // Shortcut: Ctrl + A (Select All)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("a")) {
                        return self.update(Message::SelectAll);
                    }

                    // Shortcut: Ctrl + C (Copy)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("c")) {
                        return self.update(Message::Copy);
                    }

                    // Shortcut: Ctrl + X (Cut)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("x")) {
                        return self.update(Message::Cut);
                    }

                    // Shortcut: Ctrl + V (Paste)
                    if modifiers.control() && matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("v")) {
                        return self.update(Message::Paste);
                    }

                    // Tab key
                    if matches!(&key, Key::Named(keyboard::key::Named::Tab)) {
                        let pane = self.current_pane_mut();
                        if pane.preedit.is_some() {
                            return Task::none();
                        }
                        if !pane.accept_ghost_text() {
                            pane.buffer.insert_str("    ");
                            pane.on_content_changed();
                        }
                        return Task::none();
                    }

                    // Escape key
                    if matches!(&key, Key::Named(keyboard::key::Named::Escape)) {
                        if self.context_menu.is_some() {
                            self.context_menu = None;
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
                    // On Wayland/Linux, when NumLock is not tracked by the client compositor session,
                    // numpad physical keys emit keysyms like Delete (NumpadDecimal), ArrowLeft (Numpad4), End (Numpad1), etc.
                    // Intercepting here ensures 0-9 and operators always type characters in direct English input mode.
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
                        || matches!(&physical_key, cosmic::iced::keyboard::key::Physical::Code(cosmic::iced::keyboard::key::Code::NumpadEnter))
                        || matches!(&key, Key::Character(c) if c == "\r" || c == "\n")
                        || text.as_deref() == Some("\r")
                        || text.as_deref() == Some("\n")
                        || text.as_deref() == Some("\r\n");

                    if is_enter {
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

                    // Navigation & Numpad navigation
                    let shift = modifiers.shift();
                    if matches!(&key, Key::Named(keyboard::key::Named::ArrowLeft)) {
                        let pane = self.current_pane_mut();
                        if pane.preedit.is_some() {
                            return Task::none();
                        }
                        pane.clear_ghost_text();
                        pane.buffer.move_left(shift);
                        return Task::none();
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::ArrowRight)) {
                        let pane = self.current_pane_mut();
                        if pane.preedit.is_some() {
                            return Task::none();
                        }
                        pane.clear_ghost_text();
                        pane.buffer.move_right(shift);
                        return Task::none();
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::ArrowUp)) {
                        let pane = self.current_pane_mut();
                        if pane.preedit.is_some() {
                            return Task::none();
                        }
                        pane.clear_ghost_text();
                        pane.buffer.move_up(shift);
                        return Task::none();
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::ArrowDown)) {
                        let pane = self.current_pane_mut();
                        if pane.preedit.is_some() {
                            return Task::none();
                        }
                        pane.clear_ghost_text();
                        pane.buffer.move_down(shift);
                        return Task::none();
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::Home)) {
                        let pane = self.current_pane_mut();
                        if pane.preedit.is_some() {
                            return Task::none();
                        }
                        pane.clear_ghost_text();
                        pane.buffer.move_line_start(shift);
                        return Task::none();
                    }
                    if matches!(&key, Key::Named(keyboard::key::Named::End)) {
                        let pane = self.current_pane_mut();
                        if pane.preedit.is_some() {
                            return Task::none();
                        }
                        pane.clear_ghost_text();
                        pane.buffer.move_line_end(shift);
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
                    _ => Task::none(),
                }
            }

            Message::ClickPane(pane_id, line, col) => {
                self.active_pane = pane_id;
                self.context_menu = None;
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

            Message::Copy => {
                self.context_menu = None;
                self.show_edit_menu = false;
                if let Some(selected) = self.current_pane().buffer.selected_text() {
                    let count = selected.chars().count();
                    self.status_msg = Some(format!("Copied {} chars", count));
                    return cosmic::iced::clipboard::write(selected);
                }
                Task::none()
            }

            Message::Cut => {
                self.context_menu = None;
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

            Message::Paste => {
                self.context_menu = None;
                self.show_edit_menu = false;
                cosmic::iced::clipboard::read()
                    .map(|opt| cosmic::Action::App(Message::ClipboardPasted(opt)))
            }

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

            Message::ToggleEditMenu => {
                self.show_edit_menu = !self.show_edit_menu;
                if self.show_edit_menu {
                    self.context_menu = None;
                }
                Task::none()
            }

            Message::CloseEditMenu => {
                self.show_edit_menu = false;
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
                    self.save_config();
                    Task::none()
                }
                FileTreeMessage::OpenFolder => self.update(Message::OpenFolderPrompt),
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
                    return self.update(Message::SaveFileAsPrompt);
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
                    let pane = match pane_id {
                        PaneId::Left => &mut self.left_pane,
                        PaneId::Right => &mut self.right_pane,
                    };
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
                if raw_name.is_empty() {
                    self.status_msg = Some("File name cannot be empty".into());
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

            Message::ChangeDimming(val) => {
                self.theme.dimming = val;
                self.save_config();
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

            button::text("  New ").on_press(Message::PromptNewFile).into(),
            button::text(" 󰈔 Open ").on_press(Message::OpenFilePrompt).into(),
            button::text(" 󰆓 Save ").on_press(Message::SaveFile).into(),

            button::text(if self.show_edit_menu {
                " 󰧑 Edit ▴"
            } else {
                " 󰧑 Edit ▾"
            })
            .on_press(Message::ToggleEditMenu)
            .into(),

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
            let editor = EditorCanvas::new(
                &self.left_pane,
                theme,
                self.active_pane == PaneId::Left,
                font_name,
                font_size,
            );

            container(editor)
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
                let editor = EditorCanvas::new(
                    &self.right_pane,
                    theme,
                    self.active_pane == PaneId::Right,
                    font_name,
                    font_size,
                );

                container(editor)
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

        // New File Modal / Dialog
        if self.show_new_file_modal {
            let dir_display = self.new_file_target_dir.to_string_lossy().to_string();
            let modal_box = container(
                column::with_capacity(7)
                    .spacing(14)
                    .padding(24)
                    .push(
                        row::with_capacity(2)
                            .push(
                                text::title3(" Create New File")
                                    .class(cosmic::theme::Text::Color(theme.config.accent)),
                            )
                            .align_y(Alignment::Center),
                    )
                    .push(
                        text("Enter file name (e.g. main.rs, notes.md, src/lib.rs):")
                            .size(13.0)
                            .class(cosmic::theme::Text::Color(theme.config.fg)),
                    )
                    .push(
                        text_input("e.g. main.rs", &self.new_file_name_input)
                            .on_input(Message::NewFileNameChanged)
                            .on_submit(|_| Message::ConfirmNewFile)
                            .padding([8, 12])
                            .size(14.0),
                    )
                    .push(
                        row::with_capacity(3)
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(
                                text(format!(" Target Folder: {}", dir_display))
                                    .size(12.0)
                                    .class(cosmic::theme::Text::Color(theme.config.comment)),
                            )
                            .push(cosmic::iced::widget::space::horizontal())
                            .push(
                                button::text("Change...")
                                    .on_press(Message::BrowseNewFileFolder)
                                    .padding([3, 10]),
                            ),
                    )
                    .push(
                        row::with_capacity(3)
                            .spacing(12)
                            .push(cosmic::iced::widget::space::horizontal())
                            .push(
                                button::text("Cancel")
                                    .on_press(Message::CancelNewFile)
                                    .padding([6, 16]),
                            )
                            .push(
                                button::suggested("Create File")
                                    .on_press(Message::ConfirmNewFile)
                                    .padding([6, 16]),
                            ),
                    ),
            )
            .padding(16)
            .width(Length::Fixed(520.0));

            let centered_overlay = container(modal_box)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill);

            return column::with_capacity(2)
                .push(centered_overlay)
                .push(status_container)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        let mut main_col = column::with_capacity(3).width(Length::Fill).height(Length::Fill);

        if self.show_edit_menu {
            let edit_bar = container(
                row::with_capacity(9)
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .padding([4, 12])
                    .push(
                        text(" 󰧑 Edit: ")
                            .size(13.0)
                            .class(cosmic::theme::Text::Color(theme.config.accent)),
                    )
                    .push(button::text(" 󰕌 Undo (Ctrl+Z) ").on_press(Message::Undo).padding([4, 8]))
                    .push(button::text(" 󰑎 Redo (Ctrl+Y) ").on_press(Message::Redo).padding([4, 8]))
                    .push(button::text(" 󰆐 Cut (Ctrl+X) ").on_press(Message::Cut).padding([4, 8]))
                    .push(button::text(" 󰆏 Copy (Ctrl+C) ").on_press(Message::Copy).padding([4, 8]))
                    .push(button::text(" 󰆒 Paste (Ctrl+V) ").on_press(Message::Paste).padding([4, 8]))
                    .push(button::text(" 󰒅 Select All (Ctrl+A) ").on_press(Message::SelectAll).padding([4, 8]))
                    .push(cosmic::iced::widget::space::horizontal())
                    .push(button::text(" ✕ Close ").on_press(Message::CloseEditMenu).padding([3, 8])),
            )
            .width(Length::Fill);
            main_col = main_col.push(edit_bar);
        }

        main_col = main_col.push(main_row);

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

            main_col = main_col.push(settings_box);
        }

        let base_view: Element<'_, Self::Message> = column::with_capacity(2)
            .push(main_col)
            .push(status_container)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if let Some((_pane_id, cx, cy)) = self.context_menu {
            let menu_w = 230.0;
            let menu_h = 280.0;
            let menu_x = if cx + menu_w > 1750.0 {
                (cx - menu_w).max(10.0)
            } else {
                cx.max(10.0)
            };
            let menu_y = if cy + menu_h > 1700.0 {
                (cy - menu_h).max(10.0)
            } else {
                cy.max(10.0)
            };

            let make_item = |icon: &'static str, label: &'static str, shortcut: &'static str, msg: Message| {
                let content = row::with_capacity(3)
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .padding([5, 8])
                    .push(
                        text(format!("{icon}  {label}"))
                            .size(13.0)
                            .class(cosmic::theme::Text::Color(theme.config.fg)),
                    )
                    .push(cosmic::iced::widget::space::horizontal())
                    .push(
                        text(shortcut)
                            .size(11.0)
                            .class(cosmic::theme::Text::Color(theme.config.comment)),
                    );

                button::custom(content)
                    .on_press(msg)
                    .class(cosmic::theme::Button::Transparent)
                    .width(Length::Fill)
            };

            let menu_items = column::with_capacity(7)
                .spacing(2)
                .padding(4)
                .push(make_item("󰆏", "Copy", "Ctrl+C", Message::Copy))
                .push(make_item("󰆐", "Cut", "Ctrl+X", Message::Cut))
                .push(make_item("󰆒", "Paste", "Ctrl+V", Message::Paste))
                .push(make_item("󰒅", "Select All", "Ctrl+A", Message::SelectAll))
                .push(make_item("󰕌", "Undo", "Ctrl+Z", Message::Undo))
                .push(make_item("󰑎", "Redo", "Ctrl+Y", Message::Redo))
                .push(make_item("✕", "Close", "Esc", Message::CloseContextMenu));

            let context_menu_box = container(menu_items)
                .class(cosmic::theme::Container::Card)
                .padding(4)
                .width(Length::Fixed(menu_w));

            let positioned_menu = row::with_capacity(2)
                .push(Space::new().width(Length::Fixed(menu_x)))
                .push(
                    column::with_capacity(2)
                        .push(Space::new().height(Length::Fixed(menu_y)))
                        .push(context_menu_box),
                )
                .width(Length::Fill)
                .height(Length::Fill);

            let backdrop = button::custom(Space::new().width(Length::Fill).height(Length::Fill))
                .on_press(Message::CloseContextMenu)
                .class(cosmic::theme::Button::Transparent);

            stack(vec![base_view, backdrop.into(), positioned_menu.into()]).into()
        } else {
            base_view
        }
    }
}
