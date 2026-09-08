pub mod ai_chat;
pub mod context_menu;
pub mod header;
pub mod modal;
pub mod search_bar;
pub mod tab_bar;

use crate::app::message::Message;
use crate::app::App;
use crate::editor::{PaneId, SplitLayout};
use crate::ui::canvas_editor::EditorCanvas;
use crate::ui::file_tree_view::view_file_tree;
use crate::ui::markdown_view::view_markdown;
use cosmic::iced::widget::{column, row};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{container, text, Space};

impl App {
    pub(crate) fn render_view(&self) -> Element<'_, Message> {
        let theme = &self.theme;
        let font_name = &self.font_manager.current_font;
        let font_size = self.font_manager.font_size;

        // Left Pane view
        let left_element: Element<'_, Message> = if self.left_pane.is_markdown_preview {
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

        // Tab header for left pane
        let left_tab_bar = self.render_tab_bar(&self.left_pane);

        let mut left_col = column::with_capacity(3).push(left_tab_bar);
        if let Some(sb) = self.render_search_bar(&self.left_pane) {
            left_col = left_col.push(sb);
        }
        let left_col = left_col
            .push(left_element)
            .width(Length::Fill)
            .height(Length::Fill);

        let left_pane_box = container(left_col)
            .width(Length::Fill)
            .height(Length::Fill);

        // Editor area (Single or Split)
        let editor_area: Element<'_, Message> = if self.split_layout == SplitLayout::Split {
            let right_element: Element<'_, Message> = if self.right_pane.is_markdown_preview {
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

            let right_tab_bar = self.render_tab_bar(&self.right_pane);

            let mut right_col = column::with_capacity(3).push(right_tab_bar);
            if let Some(sb) = self.render_search_bar(&self.right_pane) {
                right_col = right_col.push(sb);
            }
            let right_col = right_col
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
            let tree_view =
                view_file_tree(&self.file_tree, theme, font_name, Message::FileTreeMsg);
            main_row = main_row.push(tree_view);
            main_row = main_row.push(Space::new().width(Length::Fixed(2.0)));
        }

        main_row = main_row.push(editor_area);

        if self.show_ai_chat {
            main_row = main_row.push(Space::new().width(Length::Fixed(2.0)));
            main_row = main_row.push(self.render_ai_chat_panel());
        }

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

        let title_bar = self.render_title_bar();

        let base_view: Element<'_, Message> = column::with_capacity(3)
            .push(title_bar)
            .push(main_row)
            .push(status_container)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        let mut layers: Vec<Element<'_, Message>> = Vec::with_capacity(4);
        layers.push(base_view);

        if let Some((backdrop, menu)) = self.render_context_menu() {
            layers.push(backdrop);
            layers.push(menu);
        } else if let Some((backdrop, menu)) = self.render_file_tree_context_menu() {
            layers.push(backdrop);
            layers.push(menu);
        } else if let Some((backdrop, menu)) = self.render_header_menu() {
            layers.push(backdrop);
            layers.push(menu);
        }

        if let Some(modal_overlay) = self.render_active_modal() {
            layers.push(modal_overlay);
        }

        cosmic::iced::widget::stack(layers)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
