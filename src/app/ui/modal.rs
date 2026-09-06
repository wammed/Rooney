use crate::app::message::Message;
use crate::app::App;
use cosmic::iced::widget::{column, row};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, container, dropdown, slider, text, text_input};

impl App {
    pub(crate) fn render_active_modal<'a>(&'a self) -> Option<Element<'a, Message>> {
        if let Some(modal) = self.render_new_file_modal() {
            return Some(modal);
        }
        if let Some(modal) = self.render_new_folder_modal() {
            return Some(modal);
        }
        if let Some(modal) = self.render_rename_modal() {
            return Some(modal);
        }
        if let Some(modal) = self.render_delete_modal() {
            return Some(modal);
        }
        None
    }

    pub(crate) fn render_new_file_modal<'a>(&'a self) -> Option<Element<'a, Message>> {
        if !self.show_new_file_modal {
            return None;
        }

        let theme = &self.theme;
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

        Some(
            container(modal_box)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
        )
    }

    pub(crate) fn render_new_folder_modal<'a>(&'a self) -> Option<Element<'a, Message>> {
        if !self.show_new_folder_modal {
            return None;
        }

        let theme = &self.theme;
        let dir_display = self.new_folder_target_dir.to_string_lossy().to_string();

        let modal_box = container(
            column::with_capacity(6)
                .spacing(14)
                .padding(24)
                .push(
                    row::with_capacity(2)
                        .push(
                            text::title3(" Create New Folder")
                                .class(cosmic::theme::Text::Color(theme.config.accent)),
                        )
                        .align_y(Alignment::Center),
                )
                .push(
                    text("Enter folder name:")
                        .size(13.0)
                        .class(cosmic::theme::Text::Color(theme.config.fg)),
                )
                .push(
                    text_input("e.g. components", &self.new_folder_name_input)
                        .on_input(Message::NewFolderNameChanged)
                        .on_submit(|_| Message::ConfirmNewFolder)
                        .padding([8, 12])
                        .size(14.0),
                )
                .push(
                    text(format!(" Location: {}", dir_display))
                        .size(12.0)
                        .class(cosmic::theme::Text::Color(theme.config.comment)),
                )
                .push(
                    row::with_capacity(3)
                        .spacing(12)
                        .push(cosmic::iced::widget::space::horizontal())
                        .push(
                            button::text("Cancel")
                                .on_press(Message::CancelNewFolder)
                                .padding([6, 16]),
                        )
                        .push(
                            button::suggested("Create Folder")
                                .on_press(Message::ConfirmNewFolder)
                                .padding([6, 16]),
                        ),
                ),
        )
        .padding(16)
        .width(Length::Fixed(480.0));

        Some(
            container(modal_box)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
        )
    }

    pub(crate) fn render_rename_modal<'a>(&'a self) -> Option<Element<'a, Message>> {
        if !self.show_rename_modal {
            return None;
        }

        let theme = &self.theme;
        let old_name = self
            .rename_target_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let modal_box = container(
            column::with_capacity(6)
                .spacing(14)
                .padding(24)
                .push(
                    row::with_capacity(2)
                        .push(
                            text::title3(" Rename Item")
                                .class(cosmic::theme::Text::Color(theme.config.accent)),
                        )
                        .align_y(Alignment::Center),
                )
                .push(
                    text(format!("Current: {old_name}"))
                        .size(12.0)
                        .class(cosmic::theme::Text::Color(theme.config.comment)),
                )
                .push(
                    text_input("New name...", &self.rename_name_input)
                        .on_input(Message::RenameInputChanged)
                        .on_submit(|_| Message::ConfirmRename)
                        .padding([8, 12])
                        .size(14.0),
                )
                .push(
                    row::with_capacity(3)
                        .spacing(12)
                        .push(cosmic::iced::widget::space::horizontal())
                        .push(
                            button::text("Cancel")
                                .on_press(Message::CancelRename)
                                .padding([6, 16]),
                        )
                        .push(
                            button::suggested("Rename")
                                .on_press(Message::ConfirmRename)
                                .padding([6, 16]),
                        ),
                ),
        )
        .padding(16)
        .width(Length::Fixed(460.0));

        Some(
            container(modal_box)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
        )
    }

    pub(crate) fn render_delete_modal<'a>(&'a self) -> Option<Element<'a, Message>> {
        if !self.show_delete_modal {
            return None;
        }

        let theme = &self.theme;
        let item_name = self
            .delete_target_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("item");

        let modal_box = container(
            column::with_capacity(5)
                .spacing(14)
                .padding(24)
                .push(
                    row::with_capacity(2)
                        .push(
                            text::title3(" Delete Item")
                                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.95, 0.35, 0.35))),
                        )
                        .align_y(Alignment::Center),
                )
                .push(
                    text(format!("Are you sure you want to permanently delete '{}'?", item_name))
                        .size(13.0)
                        .class(cosmic::theme::Text::Color(theme.config.fg)),
                )
                .push(
                    text("This operation cannot be undone.")
                        .size(11.0)
                        .class(cosmic::theme::Text::Color(theme.config.comment)),
                )
                .push(
                    row::with_capacity(3)
                        .spacing(12)
                        .push(cosmic::iced::widget::space::horizontal())
                        .push(
                            button::text("Cancel")
                                .on_press(Message::CancelDelete)
                                .padding([6, 16]),
                        )
                        .push(
                            button::text("Delete Permanently")
                                .on_press(Message::ConfirmDelete)
                                .class(cosmic::theme::Button::Destructive)
                                .padding([6, 16]),
                        ),
                ),
        )
        .padding(16)
        .width(Length::Fixed(460.0));

        Some(
            container(modal_box)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
        )
    }

    pub(crate) fn render_settings_box<'a>(&'a self) -> Element<'a, Message> {
        let theme = &self.theme;
        let cur_model_idx = self
            .ollama
            .available_models
            .iter()
            .position(|m| m == &self.ollama.active_model)
            .unwrap_or(0);

        container(
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
                .push(button::suggested("Close").on_press(Message::CloseSettings)),
        )
        .padding(16)
        .width(Length::Fixed(360.0))
        .into()
    }
}
