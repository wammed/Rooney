pub mod keybindings;
pub mod message;
pub mod state;
pub mod ui;
pub mod update;

pub use message::Message;

use crate::ai::{AiStatus, OllamaClient};
use crate::config::AppConfig;
use crate::editor::{EditorPane, PaneId, SplitLayout};
use crate::font::FontManager;
use crate::fs::tree::FileTree;
use crate::theme::themes::{EditorTheme, ThemeId};

use cosmic::app::{Core, Task};
use cosmic::prelude::*;
use cosmic::{executor, iced};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct FileTreeContextMenuState {
    pub target: Option<PathBuf>,
    pub is_dir: bool,
    pub x: f32,
    pub y: f32,
}

pub struct App {
    pub(crate) core: Core,
    pub(crate) config: AppConfig,
    pub(crate) theme: EditorTheme,
    pub(crate) font_manager: FontManager,
    pub(crate) file_tree: FileTree,
    pub(crate) split_layout: SplitLayout,
    pub(crate) left_pane: EditorPane,
    pub(crate) right_pane: EditorPane,
    pub(crate) active_pane: PaneId,
    pub(crate) ollama: OllamaClient,
    pub(crate) ai_status: AiStatus,
    pub(crate) show_settings: bool,
    pub(crate) show_edit_menu: bool,
    pub(crate) active_header_menu: Option<crate::app::message::ActiveHeaderMenu>,
    pub(crate) context_menu: Option<(PaneId, f32, f32)>,
    pub(crate) mouse_pos: (f32, f32),
    pub(crate) file_tree_context_menu: Option<FileTreeContextMenuState>,
    pub(crate) status_msg: Option<String>,
    pub(crate) theme_names: Vec<String>,
    pub(crate) font_names: Vec<String>,
    pub(crate) ai_request_pending: bool,
    pub(crate) last_key_press: Instant,
    pub(crate) last_ime_commit: Instant,

    // New file modal state
    pub(crate) show_new_file_modal: bool,
    pub(crate) new_file_name_input: String,
    pub(crate) new_file_target_dir: PathBuf,

    // New folder modal state
    pub(crate) show_new_folder_modal: bool,
    pub(crate) new_folder_name_input: String,
    pub(crate) new_folder_target_dir: PathBuf,

    // Rename modal state
    pub(crate) show_rename_modal: bool,
    pub(crate) rename_target_path: Option<PathBuf>,
    pub(crate) rename_name_input: String,

    // Delete modal state
    pub(crate) show_delete_modal: bool,
    pub(crate) delete_target_path: Option<PathBuf>,

    // AI Chat panel state
    pub(crate) show_ai_chat: bool,
    pub(crate) ai_chat_messages: Vec<crate::ai::ChatMessage>,
    pub(crate) ai_chat_input: String,
    pub(crate) ai_chat_pending: bool,
    pub(crate) ai_chat_cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,

    // Search state
    pub(crate) search_input: String,
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
        let current_dir = if let Some(ref root) = config.session.root_dir {
            if root.exists() && root.is_dir() {
                root.clone()
            } else {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            }
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };
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
        // Restore left pane tabs from session if available
        if !config.session.left_pane.tabs.is_empty() {
            let mut restored_tabs = Vec::new();
            for (idx, tab_info) in config.session.left_pane.tabs.iter().enumerate() {
                let mut tab = crate::editor::EditorTab::new(idx + 1, &tab_info.file_name);
                if let Some(ref path) = tab_info.file_path {
                    if path.exists() {
                        let _ = tab.load_file(path);
                    }
                }
                tab.buffer.cursor = (tab_info.cursor_line, tab_info.cursor_col);
                tab.buffer.clamp_cursor();
                restored_tabs.push(tab);
            }
            if !restored_tabs.is_empty() {
                left_pane.tabs = restored_tabs;
                left_pane.active_tab_idx = config
                    .session
                    .left_pane
                    .active_tab_idx
                    .min(left_pane.tabs.len() - 1);
            }
        } else {
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
            left_pane.highlighter =
                crate::syntax::Highlighter::new(crate::syntax::SupportedLanguage::Rust);
            left_pane.highlighter.update_source(welcome_content);
        }

        let mut right_pane = EditorPane::new(PaneId::Right, "Preview / Editor 2");
        // Restore right pane tabs from session if available
        if let Some(ref right_info) = config.session.right_pane {
            if !right_info.tabs.is_empty() {
                let mut restored_tabs = Vec::new();
                for (idx, tab_info) in right_info.tabs.iter().enumerate() {
                    let mut tab = crate::editor::EditorTab::new(idx + 1, &tab_info.file_name);
                    if let Some(ref path) = tab_info.file_path {
                        if path.exists() {
                            let _ = tab.load_file(path);
                        }
                    }
                    tab.buffer.cursor = (tab_info.cursor_line, tab_info.cursor_col);
                    tab.buffer.clamp_cursor();
                    restored_tabs.push(tab);
                }
                if !restored_tabs.is_empty() {
                    right_pane.tabs = restored_tabs;
                    right_pane.active_tab_idx =
                        right_info.active_tab_idx.min(right_pane.tabs.len() - 1);
                }
            }
        } else {
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
            right_pane.highlighter =
                crate::syntax::Highlighter::new(crate::syntax::SupportedLanguage::Markdown);
            right_pane.highlighter.update_source(sample_md);
            right_pane.is_markdown_preview = false;
            right_pane.markdown_doc = Some(crate::markdown::MarkdownDocument::parse(sample_md));
        }

        let theme_names: Vec<String> = ThemeId::ALL
            .iter()
            .map(|t| t.display_name().to_string())
            .collect();
        let font_names = font_manager.available_fonts.clone();

        let mut ollama = OllamaClient::default();
        ollama.is_enabled = config.ai_enabled;
        ollama.active_model = config.ai_model.clone();

        let ai_status = if ollama.is_enabled {
            AiStatus::Ready(ollama.active_model.clone())
        } else {
            AiStatus::Disabled
        };

        let app = Self {
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
            active_header_menu: None,
            context_menu: None,
            mouse_pos: (0.0, 0.0),
            file_tree_context_menu: None,
            status_msg: Some("CosmicCode Ready".to_string()),
            theme_names,
            font_names,
            ai_request_pending: false,
            last_key_press: Instant::now(),
            last_ime_commit: Instant::now(),
            show_new_file_modal: false,
            new_file_name_input: String::new(),
            new_file_target_dir: current_dir.clone(),
            show_new_folder_modal: false,
            new_folder_name_input: String::new(),
            new_folder_target_dir: current_dir.clone(),
            show_rename_modal: false,
            rename_target_path: None,
            rename_name_input: String::new(),
            show_delete_modal: false,
            delete_target_path: None,
            show_ai_chat: false,
            ai_chat_messages: Vec::new(),
            ai_chat_input: String::new(),
            ai_chat_pending: false,
            ai_chat_cancel: None,
            search_input: String::new(),
        };

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
        self.handle_update(message)
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        self.render_header_start()
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        self.render_header_end()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.render_view()
    }
}
