use crate::app::message::Message;
use crate::app::App;
use crate::editor::SplitLayout;
use crate::ui::file_tree_view::FileTreeMessage;
use crate::theme::themes::ThemeId;
use cosmic::prelude::*;
use cosmic::widget::{button, dropdown};

impl App {
    pub(crate) fn render_header_start(&self) -> Vec<Element<'_, Message>> {
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
            button::text(if self.show_ai_chat {
                " 󰭹 Chat: ON "
            } else {
                " 󰭹 Chat "
            })
            .on_press(Message::ToggleAiChat)
            .into(),
            button::text(" 󰒓 Aesthetics ")
                .on_press(Message::ToggleSettings)
                .into(),
        ]
    }

    pub(crate) fn render_header_end(&self) -> Vec<Element<'_, Message>> {
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
            button::text(" A- ")
                .on_press(Message::DecreaseFontSize)
                .into(),
            button::text(" A+ ")
                .on_press(Message::IncreaseFontSize)
                .into(),
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
}
