use crate::app::message::Message;
use crate::app::App;
use crate::editor::{EditorPane, PaneId, SplitLayout};
use crate::fs::tree::FileTypeIcon;
use crate::syntax::SupportedLanguage;
use cosmic::iced::widget::row;
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, container, text};
use std::path::Path;

impl App {
    pub(crate) fn render_tab_bar<'a>(&'a self, pane: &'a EditorPane) -> Element<'a, Message> {
        let theme = &self.theme;
        let is_pane_active = self.active_pane == pane.id;

        let mut tabs_row = row::with_capacity(pane.tabs.len() + 2).spacing(4);

        for (idx, tab) in pane.tabs.iter().enumerate() {
            let is_active = idx == pane.active_tab_idx;

            // Resolve file icon from path or filename
            let file_path = tab.file_path.as_deref().unwrap_or(Path::new(&tab.file_name));
            let file_icon = FileTypeIcon::for_path(file_path, false, false);

            let dirty_mark = if tab.buffer.is_modified { " ●" } else { "" };
            let title_text = format!("{} {}{dirty_mark}", file_icon.glyph, tab.file_name);

            // Tab button (click to select)
            let tab_label = text(title_text)
                .size(12.0)
                .class(cosmic::theme::Text::Color(if is_active && is_pane_active {
                    theme.config.accent
                } else if is_active {
                    theme.config.fg
                } else {
                    theme.config.comment
                }));

            let close_btn = button::text("✕")
                .on_press(Message::CloseTab(pane.id, idx))
                .padding([1, 4])
                .class(cosmic::theme::Button::Text);

            let tab_content = row::with_capacity(2)
                .push(
                    button::custom(tab_label)
                        .on_press(Message::SelectTab(pane.id, idx))
                        .padding([3, 6])
                        .class(cosmic::theme::Button::Text),
                )
                .push(close_btn)
                .align_y(Alignment::Center);

            let tab_container = container(tab_content)
                .padding([1, 2])
                .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                    if is_active {
                        container::Style {
                            background: Some(theme.config.gutter_bg.into()),
                            border: cosmic::iced::border::rounded(4)
                                .color(if is_pane_active {
                                    theme.config.accent
                                } else {
                                    theme.config.border
                                })
                                .width(1.0),
                            ..Default::default()
                        }
                    } else {
                        container::Style {
                            background: Some(theme.config.bg.into()),
                            border: cosmic::iced::border::rounded(4)
                                .color(theme.config.border)
                                .width(0.5),
                            ..Default::default()
                        }
                    }
                })));

            tabs_row = tabs_row.push(tab_container);
        }

        // New tab button "+"
        let add_tab_btn = button::text(" ＋ ")
            .on_press(Message::NewTab(pane.id))
            .padding([2, 6])
            .class(cosmic::theme::Button::Text);
        tabs_row = tabs_row.push(add_tab_btn);

        let mut right_controls = row::with_capacity(3).spacing(6).align_y(Alignment::Center);

        // Markdown preview toggle button if current active tab is Markdown
        if pane.active_tab().highlighter.lang == SupportedLanguage::Markdown {
            let preview_btn = button::text(if pane.active_tab().is_markdown_preview {
                "  Edit "
            } else {
                "  Preview "
            })
            .on_press(Message::TogglePaneMode(pane.id))
            .padding([2, 6]);
            right_controls = right_controls.push(preview_btn);
        }

        // Split close button if in split mode and right pane
        if self.split_layout == SplitLayout::Split && pane.id == PaneId::Right {
            let close_split_btn = button::text(" ✕ Close Split ")
                .on_press(Message::ToggleSplit)
                .padding([2, 6])
                .class(cosmic::theme::Button::Destructive);
            right_controls = right_controls.push(close_split_btn);
        }

        let full_bar = row::with_capacity(3)
            .push(tabs_row)
            .push(cosmic::iced::widget::space::horizontal())
            .push(right_controls)
            .align_y(Alignment::Center)
            .padding([3, 6])
            .width(Length::Fill);

        container(full_bar)
            .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                container::Style {
                    background: Some(theme.config.bg.into()),
                    border: cosmic::iced::border::rounded(0)
                        .color(theme.config.border)
                        .width(0.5),
                    ..Default::default()
                }
            })))
            .into()
    }
}
