use crate::ai::OllamaError;
use crate::editor::PaneId;
use crate::ui::file_tree_view::FileTreeMessage;
use cosmic::iced;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveHeaderMenu {
    File,
    Edit,
    View,
    Ai,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Aesthetics,
    About,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FimRequestToken {
    pub pane_id: PaneId,
    pub tab_id: usize,
    pub request_id: usize,
    pub buffer_revision: usize,
    pub cursor: (usize, usize),
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Message {
    Tick,
    Event(iced::Event),
    FileTreeMsg(FileTreeMessage),
    ToggleSplit,
    SetActivePane(PaneId),
    ClickPane(PaneId, usize, usize),
    DragSelect(PaneId, usize, usize),
    ScrollPane(PaneId, f32),
    SetScrollY(PaneId, f32),
    TogglePaneMode(PaneId),
    ToggleMarkdownPreview,
    SelectTheme(usize),
    SelectFont(usize),
    SelectMarkdownSpec(usize),
    IncreaseFontSize,
    DecreaseFontSize,
    ToggleAi,
    TriggerAiFim,
    AiFimResult(FimRequestToken, Result<String, OllamaError>),
    AiModelsFetched(Result<Vec<String>, OllamaError>),
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
    CloseFileTreeContextMenu,
    ToggleHeaderMenu(ActiveHeaderMenu),
    CloseHeaderMenu,
    ToggleEditMenu,
    CloseEditMenu,
    ToggleSettings,
    ChangeOpacity(f32),
    ChangeFileTreeOpacity(f32),
    ChangeTitleBarOpacity(f32),
    ChangeDimming(f32),
    CloseSettings,
    SelectSettingsTab(SettingsTab),
    ToggleLicenseDetail(usize),
    DragWindow,
    MaximizeWindow,
    MinimizeWindow,
    CloseWindow,

    // Search & Replace
    ToggleSearch,
    SearchQueryChanged(String),
    NextSearchMatch,
    PrevSearchMatch,
    CloseSearch,

    // Advanced editing
    DeleteLine,
    DuplicateLine,
    ToggleComment,

    // Tab operations
    SelectTab(PaneId, usize),
    CloseTab(PaneId, usize),
    NewTab(PaneId),
    NextTab(PaneId),
    PrevTab(PaneId),

    // File tree operations & modals
    PromptNewFolder(Option<PathBuf>),
    NewFolderNameChanged(String),
    ConfirmNewFolder,
    CancelNewFolder,
    PromptRename(PathBuf),
    RenameInputChanged(String),
    ConfirmRename,
    CancelRename,
    PromptDelete(PathBuf),
    ConfirmDelete,
    CancelDelete,

    // AI Chat & Code Generation Panel
    ToggleAiChat,
    AiChatInputChanged(String),
    SendAiChatMessage,
    StopAiChat,
    AiChatChunk {
        request_id: usize,
        chunk: String,
    },
    AiChatDone {
        request_id: usize,
    },
    AiChatError {
        request_id: usize,
        error: String,
    },
    AiChatResult(Result<String, String>),
    ClearAiChat,
    AttachSelectionToAiChat,
    AttachFileToAiChat,
    InsertAiResponseAtCursor(String),
    CopyAiResponse(String),

    // Background Async Tree-sitter Syntax Parsing
    HighlightParseCompleted {
        pane_id: PaneId,
        tab_id: usize,
        generation: usize,
        worker_generation: usize,
        tree: Option<tree_sitter::Tree>,
    },

    // Background Async Search Query Scanning
    SearchCompleted {
        pane_id: PaneId,
        tab_id: usize,
        generation: usize,
        worker_generation: usize,
        matches: Vec<(usize, usize, usize)>,
    },

    // Background Async Markdown Preview Parsing (for large files on Toggle/Spec change)
    MarkdownParseCompleted {
        pane_id: PaneId,
        tab_id: usize,
        generation: usize,
        doc: crate::markdown::MarkdownDocument,
    },
}
