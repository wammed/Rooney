use crate::ai::ChatRole;
use crate::app::message::Message;
use crate::app::App;
use cosmic::iced::widget::{column, row, scrollable};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, container, text, text_input, Space};

impl App {
    pub(crate) fn render_ai_chat_panel(&self) -> Element<'_, Message> {
        let theme = &self.theme;

        // Header
        let header = row::with_capacity(5)
            .spacing(6)
            .align_y(Alignment::Center)
            .push(
                text("󰚩 AI Assistant")
                    .size(13.0)
                    .class(cosmic::theme::Text::Color(theme.config.accent)),
            )
            .push(
                text(format!("({})", self.ollama.active_model))
                    .size(11.0)
                    .class(cosmic::theme::Text::Color(theme.config.comment)),
            )
            .push(cosmic::iced::widget::space::horizontal())
            .push(
                button::text("󰃢 Clear")
                    .on_press(Message::ClearAiChat)
                    .padding([2, 6])
                    .class(cosmic::theme::Button::Text),
            )
            .push(
                button::text("✕")
                    .on_press(Message::ToggleAiChat)
                    .padding([2, 6])
                    .class(cosmic::theme::Button::Text),
            );

        // Context Attachment Bar
        let context_bar = row::with_capacity(3)
            .spacing(6)
            .align_y(Alignment::Center)
            .push(
                button::text("󰈔 Attach Selection")
                    .on_press(Message::AttachSelectionToAiChat)
                    .padding([2, 6]),
            )
            .push(
                button::text("󰈙 Attach File")
                    .on_press(Message::AttachFileToAiChat)
                    .padding([2, 6]),
            );

        // Message List
        let mut messages_col = column::with_capacity(self.ai_chat_messages.len() + 2).spacing(8);

        if self.ai_chat_messages.is_empty() {
            let empty_prompt = text("Ask AI anything, generate code, or attach file/selection context.")
                .size(12.0)
                .class(cosmic::theme::Text::Color(theme.config.comment));
            messages_col = messages_col.push(empty_prompt);
        } else {
            let total_msgs = self.ai_chat_messages.len();
            for (i, msg) in self.ai_chat_messages.iter().enumerate() {
                let is_latest_assistant = i == total_msgs - 1 && msg.role == ChatRole::Assistant;
                let is_streaming_now = self.ai_chat_pending && is_latest_assistant;

                let role_title = match msg.role {
                    ChatRole::User => "󰭹 You",
                    ChatRole::Assistant => {
                        if is_streaming_now && msg.content.is_empty() {
                            "󰚩 AI (thinking...)"
                        } else if is_streaming_now {
                            "󰚩 AI (streaming...)"
                        } else {
                            "󰚩 AI"
                        }
                    }
                    ChatRole::Error => "󰚩 Error",
                };

                let role_color = match msg.role {
                    ChatRole::User => theme.config.accent,
                    ChatRole::Assistant => theme.config.function,
                    ChatRole::Error => cosmic::iced::Color::from_rgb(0.95, 0.3, 0.3),
                };

                let role_header = text(role_title)
                    .size(11.0)
                    .class(cosmic::theme::Text::Color(role_color));

                let content_str = if is_streaming_now {
                    if msg.content.is_empty() {
                        "Thinking...".to_string()
                    } else {
                        format!("{} ▋", msg.content)
                    }
                } else {
                    msg.content.clone()
                };

                let content_body = text(content_str)
                    .size(12.0)
                    .class(cosmic::theme::Text::Color(
                        if is_streaming_now && msg.content.is_empty() {
                            theme.config.comment
                        } else {
                            theme.config.fg
                        },
                    ));

                let mut msg_box = column::with_capacity(3)
                    .spacing(4)
                    .padding(6)
                    .push(role_header)
                    .push(content_body);

                if msg.role == ChatRole::Assistant && !msg.content.is_empty() {
                    let actions = row::with_capacity(3)
                        .spacing(6)
                        .push(
                            button::text("󰆒 Insert at Cursor")
                                .on_press(Message::InsertAiResponseAtCursor(msg.content.clone()))
                                .padding([1, 6])
                                .class(cosmic::theme::Button::Text),
                        )
                        .push(
                            button::text("󰆏 Copy")
                                .on_press(Message::CopyAiResponse(msg.content.clone()))
                                .padding([1, 6])
                                .class(cosmic::theme::Button::Text),
                        );
                    msg_box = msg_box.push(actions);
                }

                let card = container(msg_box)
                    .width(Length::Fill)
                    .class(cosmic::theme::Container::Card);

                messages_col = messages_col.push(card);
            }
        }

        let scrollable_messages = scrollable(messages_col)
            .height(Length::Fill)
            .width(Length::Fill);

        // Input row with dynamic Send / Stop button
        let action_btn = if self.ai_chat_pending {
            button::text("󰓛 Stop")
                .on_press(Message::StopAiChat)
                .class(cosmic::theme::Button::Destructive)
                .padding([6, 10])
        } else {
            button::suggested("󰒭 Send")
                .on_press(Message::SendAiChatMessage)
                .padding([6, 10])
        };

        let input_row = row::with_capacity(3)
            .spacing(6)
            .align_y(Alignment::Center)
            .push(
                text_input("Ask AI or generate code...", &self.ai_chat_input)
                    .on_input(Message::AiChatInputChanged)
                    .on_submit(|_| {
                        if self.ai_chat_pending {
                            Message::StopAiChat
                        } else {
                            Message::SendAiChatMessage
                        }
                    })
                    .padding([6, 8])
                    .size(13.0)
                    .width(Length::Fill),
            )
            .push(action_btn);

        let full_panel = column::with_capacity(5)
            .spacing(8)
            .padding(8)
            .push(header)
            .push(context_bar)
            .push(scrollable_messages)
            .push(Space::new().height(Length::Fixed(2.0)))
            .push(input_row)
            .width(Length::Fixed(360.0))
            .height(Length::Fill);

        container(full_panel)
            .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                container::Style {
                    background: Some(theme.config.bg.into()),
                    border: cosmic::iced::border::rounded(0)
                        .color(theme.config.border)
                        .width(1.0),
                    ..Default::default()
                }
            })))
            .into()
    }
}
