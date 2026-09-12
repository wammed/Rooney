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
    TogglePaneMode(PaneId),
    ToggleMarkdownPreview,
    SelectTheme(usize),
    SelectFont(usize),
    SelectMarkdownSpec(usize),
    IncreaseFontSize,
    DecreaseFontSize,
    ToggleAi,
    TriggerAiFim,
    AiFimResult(PaneId, Result<String, OllamaError>),
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
    AiChatChunk(String),
    AiChatDone,
    AiChatError(String),
    AiChatResult(Result<String, String>),
    ClearAiChat,
    AttachSelectionToAiChat,
    AttachFileToAiChat,
    InsertAiResponseAtCursor(String),
    CopyAiResponse(String),
}
