use crate::app::message::Message;
use crate::app::App;
use crate::editor::EditorPane;
use cosmic::iced::widget::row;
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, container, text, text_input};

impl App {
    pub(crate) fn render_search_bar<'a>(
        &'a self,
        pane: &'a EditorPane,
    ) -> Option<Element<'a, Message>> {
        if !pane.is_search_open {
            return None;
        }
        let theme = &self.theme;
        let matches_info = if pane.search_matches.is_empty() {
            if self.search_input.is_empty() {
                "".to_string()
            } else {
                "0/0".to_string()
            }
        } else {
            format!("{}/{}", pane.current_match_idx + 1, pane.search_matches.len())
        };

        let bar = row::with_capacity(6)
            .push(
                text("  ")
                    .size(14.0)
                    .class(cosmic::theme::Text::Color(theme.config.accent)),
            )
            .push(
                text_input("Find...", &self.search_input)
                    .on_input(Message::SearchQueryChanged)
                    .width(Length::Fixed(220.0))
                    .padding([4, 8]),
            )
            .push(
                text(matches_info)
                    .size(12.0)
                    .class(cosmic::theme::Text::Color(theme.config.comment)),
            )
            .push(
                button::text(" ▲ ")
                    .on_press(Message::PrevSearchMatch)
                    .padding([2, 6]),
            )
            .push(
                button::text(" ▼ ")
                    .on_press(Message::NextSearchMatch)
                    .padding([2, 6]),
            )
            .push(
                button::text(" ✕ ")
                    .on_press(Message::CloseSearch)
                    .padding([2, 6]),
            )
            .spacing(6)
            .align_y(Alignment::Center)
            .padding([4, 8]);

        Some(
            container(bar)
                .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                    container::Style {
                        background: Some(theme.config.gutter_bg.into()),
                        border: cosmic::iced::border::rounded(6)
                            .color(theme.config.border)
                            .width(1.0),
                        ..Default::default()
                    }
                })))
                .into(),
        )
    }
}
