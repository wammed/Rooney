use crate::app::message::{Message, SettingsTab};
use crate::app::App;
use cosmic::iced::widget::{column, row, scrollable};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, container, dropdown, slider, text, text_input};

const ROONEY_ICON_BYTES: &[u8] = include_bytes!("../../../images/Rooney-matte-icon.svg");

fn wrap_modal<'a>(
    content: impl Into<Element<'a, Message>>,
    theme: &crate::theme::EditorTheme,
    width: f32,
) -> Element<'a, Message> {
    let modal_bg = cosmic::iced::Color {
        a: 1.0,
        ..theme.config.gutter_bg
    };
    let modal_border = theme.config.border;
    let backdrop_color = cosmic::iced::Color::from_rgba(0.0, 0.0, 0.0, 0.60);

    let modal_box = container(content)
        .class(cosmic::theme::Container::Custom(Box::new(move |_| {
            container::Style {
                background: Some(modal_bg.into()),
                border: cosmic::iced::border::rounded(8)
                    .color(modal_border)
                    .width(1.0),
                ..Default::default()
            }
        })))
        .padding(16)
        .width(Length::Fixed(width));

    container(modal_box)
        .class(cosmic::theme::Container::Custom(Box::new(move |_| {
            container::Style {
                background: Some(backdrop_color.into()),
                ..Default::default()
            }
        })))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

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
        if let Some(modal) = self.render_settings_modal() {
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

        let modal_col = column::with_capacity(7)
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
            );

        Some(wrap_modal(modal_col, theme, 520.0))
    }

    pub(crate) fn render_new_folder_modal<'a>(&'a self) -> Option<Element<'a, Message>> {
        if !self.show_new_folder_modal {
            return None;
        }

        let theme = &self.theme;
        let dir_display = self.new_folder_target_dir.to_string_lossy().to_string();

        let modal_col = column::with_capacity(6)
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
            );

        Some(wrap_modal(modal_col, theme, 480.0))
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

        let modal_col = column::with_capacity(6)
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
            );

        Some(wrap_modal(modal_col, theme, 460.0))
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

        let modal_col = column::with_capacity(5)
            .spacing(14)
            .padding(24)
            .push(
                row::with_capacity(2)
                    .push(
                        text::title3(" Delete Item").class(cosmic::theme::Text::Color(
                            cosmic::iced::Color::from_rgb(0.95, 0.35, 0.35),
                        )),
                    )
                    .align_y(Alignment::Center),
            )
            .push(
                text(format!(
                    "Are you sure you want to permanently delete '{}'?",
                    item_name
                ))
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
            );

        Some(wrap_modal(modal_col, theme, 460.0))
    }

    pub(crate) fn render_settings_modal<'a>(&'a self) -> Option<Element<'a, Message>> {
        if !self.show_settings {
            return None;
        }

        let theme = &self.theme;
        let is_about = self.settings_tab == SettingsTab::About;

        // Top App Header: Icon + Title + Version + License badge
        let icon_handle = cosmic::iced::widget::svg::Handle::from_memory(ROONEY_ICON_BYTES);
        let app_icon: Element<'_, Message> = container(
            cosmic::widget::svg(icon_handle)
                .width(Length::Fixed(28.0))
                .height(Length::Fixed(28.0)),
        )
        .align_y(Alignment::Center)
        .into();

        let header_row = row::with_capacity(5)
            .spacing(10)
            .align_y(Alignment::Center)
            .push(app_icon)
            .push(
                text::title3("Rooney Settings")
                    .class(cosmic::theme::Text::Color(theme.config.accent)),
            )
            .push(
                container(
                    text("v0.1.0")
                        .size(11.0)
                        .class(cosmic::theme::Text::Color(theme.config.comment)),
                )
                .padding([2, 6])
                .class(cosmic::theme::Container::Custom(Box::new(|_| {
                    container::Style {
                        border: cosmic::iced::border::rounded(4).width(1.0),
                        ..Default::default()
                    }
                }))),
            )
            .push(
                container(
                    text("MIT License")
                        .size(11.0)
                        .class(cosmic::theme::Text::Color(theme.config.accent)),
                )
                .padding([2, 6])
                .class(cosmic::theme::Container::Custom(Box::new(|_| {
                    container::Style {
                        border: cosmic::iced::border::rounded(4).width(1.0),
                        ..Default::default()
                    }
                }))),
            );

        // Tab selection row: [ Appearance / Aesthetics ] [ About / Licenses ]
        let btn_aesthetics = if !is_about {
            button::suggested("󰒓 Aesthetics")
                .on_press(Message::SelectSettingsTab(SettingsTab::Aesthetics))
                .padding([5, 14])
        } else {
            button::text("󰒓 Aesthetics")
                .on_press(Message::SelectSettingsTab(SettingsTab::Aesthetics))
                .padding([5, 14])
        };

        let btn_about = if is_about {
            button::suggested("󰌆 About / Licenses")
                .on_press(Message::SelectSettingsTab(SettingsTab::About))
                .padding([5, 14])
        } else {
            button::text("󰌆 About / Licenses")
                .on_press(Message::SelectSettingsTab(SettingsTab::About))
                .padding([5, 14])
        };

        let tab_bar = row::with_capacity(3)
            .spacing(8)
            .align_y(Alignment::Center)
            .push(btn_aesthetics)
            .push(btn_about);

        let content_column = if is_about {
            self.render_about_settings_tab(theme)
        } else {
            self.render_aesthetics_settings_tab(theme)
        };

        let modal_col = column::with_capacity(5)
            .spacing(14)
            .padding(20)
            .push(header_row)
            .push(tab_bar)
            .push(content_column)
            .push(
                row::with_capacity(2)
                    .push(cosmic::iced::widget::space::horizontal())
                    .push(
                        button::suggested(" Close ")
                            .on_press(Message::CloseSettings)
                            .padding([6, 20]),
                    ),
            );

        let modal_width = if is_about { 620.0 } else { 500.0 };
        Some(wrap_modal(modal_col, theme, modal_width))
    }

    fn render_aesthetics_settings_tab<'a>(
        &'a self,
        theme: &'a crate::theme::EditorTheme,
    ) -> Element<'a, Message> {
        let cur_model_idx = self
            .ollama
            .available_models
            .iter()
            .position(|m| m == &self.ollama.active_model)
            .unwrap_or(0);

        let cur_theme_idx = crate::theme::themes::ThemeId::ALL
            .iter()
            .position(|&t| t == self.theme.config.id)
            .unwrap_or(0);

        let cur_font_idx = self
            .font_names
            .iter()
            .position(|f| f == &self.font_manager.current_font)
            .unwrap_or(0);

        column::with_capacity(20)
            .spacing(11)
            .push(
                text("Editor Theme (20 Themes):")
                    .size(12.5)
                    .class(cosmic::theme::Text::Color(theme.config.fg)),
            )
            .push(
                dropdown(&self.theme_names, Some(cur_theme_idx), Message::SelectTheme)
                    .width(Length::Fill),
            )
            .push(
                text("Editor Font (Nerd Font):")
                    .size(12.5)
                    .class(cosmic::theme::Text::Color(theme.config.fg)),
            )
            .push(
                dropdown(&self.font_names, Some(cur_font_idx), Message::SelectFont)
                    .width(Length::Fill),
            )
            .push(
                row::with_capacity(3)
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(
                        text(format!("Font Size: {:.0} px", self.font_manager.font_size))
                            .size(12.5)
                            .class(cosmic::theme::Text::Color(theme.config.fg)),
                    )
                    .push(cosmic::iced::widget::space::horizontal())
                    .push(
                        button::text(" A- (Smaller) ")
                            .on_press(Message::DecreaseFontSize)
                            .padding([4, 10]),
                    )
                    .push(
                        button::text(" A+ (Larger) ")
                            .on_press(Message::IncreaseFontSize)
                            .padding([4, 10]),
                    ),
            )
            .push(
                text(format!(
                    "Editor Window Opacity: {:.0}%",
                    self.theme.opacity * 100.0
                ))
                .size(12.5)
                .class(cosmic::theme::Text::Color(theme.config.fg)),
            )
            .push(slider(0.1..=1.0_f32, self.theme.opacity, Message::ChangeOpacity).step(0.05_f32))
            .push(
                text(format!(
                    "File Tree Opacity: {:.0}%",
                    self.theme.file_tree_opacity * 100.0
                ))
                .size(12.5)
                .class(cosmic::theme::Text::Color(theme.config.fg)),
            )
            .push(
                slider(
                    0.0..=1.0_f32,
                    self.theme.file_tree_opacity,
                    Message::ChangeFileTreeOpacity,
                )
                .step(0.05_f32),
            )
            .push(
                text(format!(
                    "Title Bar Opacity: {:.0}%",
                    self.theme.title_bar_opacity * 100.0
                ))
                .size(12.5)
                .class(cosmic::theme::Text::Color(theme.config.fg)),
            )
            .push(
                slider(
                    0.0..=1.0_f32,
                    self.theme.title_bar_opacity,
                    Message::ChangeTitleBarOpacity,
                )
                .step(0.05_f32),
            )
            .push(
                text(format!(
                    "Background Dimming Overlay: {:.0}%",
                    self.theme.dimming * 100.0
                ))
                .size(12.5)
                .class(cosmic::theme::Text::Color(theme.config.fg)),
            )
            .push(slider(0.0..=1.0_f32, self.theme.dimming, Message::ChangeDimming).step(0.05_f32))
            .push(
                text("Local AI Model (Ollama):")
                    .size(12.5)
                    .class(cosmic::theme::Text::Color(theme.config.fg)),
            )
            .push(
                dropdown(
                    &self.ollama.available_models,
                    Some(cur_model_idx),
                    Message::SelectAiModel,
                )
                .width(Length::Fill),
            )
            .push(
                text("Markdown Specification:")
                    .size(12.5)
                    .class(cosmic::theme::Text::Color(theme.config.fg)),
            )
            .push(
                dropdown(
                    &[
                        "GFM (GitHub Flavored - Recommended)",
                        "CommonMark (Standard)",
                    ],
                    Some(
                        if self.config.markdown_spec == crate::config::MarkdownSpec::Gfm {
                            0
                        } else {
                            1
                        },
                    ),
                    Message::SelectMarkdownSpec,
                )
                .width(Length::Fill),
            )
            .into()
    }

    fn render_about_settings_tab<'a>(
        &'a self,
        theme: &'a crate::theme::EditorTheme,
    ) -> Element<'a, Message> {
        let card_bg = cosmic::iced::Color {
            a: 0.5,
            ..theme.config.bg
        };
        let border_col = theme.config.border;

        let make_card = move |content: Element<'a, Message>| {
            container(content).padding(12).width(Length::Fill).class(
                cosmic::theme::Container::Custom(Box::new(move |_| container::Style {
                    background: Some(card_bg.into()),
                    border: cosmic::iced::border::rounded(6)
                        .color(border_col)
                        .width(1.0),
                    ..Default::default()
                })),
            )
        };

        // Project Overview Card
        let project_card = make_card(
            column::with_capacity(4)
                .spacing(6)
                .push(
                    row::with_capacity(3)
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .push(
                            text("Rooney Code & Markdown Editor")
                                .size(14.0)
                                .class(cosmic::theme::Text::Color(theme.config.fg)),
                        )
                        .push(
                            text("(MIT License)")
                                .size(12.0)
                                .class(cosmic::theme::Text::Color(theme.config.accent)),
                        ),
                )
                .push(
                    text("AI-Integrated Next-Generation Linux Code & Markdown Editor for COSMIC / Wayland")
                        .size(11.5)
                        .class(cosmic::theme::Text::Color(theme.config.comment)),
                )
                .push(
                    text("Copyright (c) 2026 wammed · Distributed as source code under MIT License")
                        .size(11.0)
                        .class(cosmic::theme::Text::Color(theme.config.comment)),
                )
                .push(
                    text("Repository: https://github.com/wammed/Rooney")
                        .size(11.0)
                        .class(cosmic::theme::Text::Color(theme.config.fg)),
                )
                .into(),
        );

        // MIT License text accordion (expandable)
        let mit_expanded = self.expanded_license_idx == Some(0);
        let mit_header = row::with_capacity(3)
            .spacing(8)
            .align_y(Alignment::Center)
            .push(
                text(if mit_expanded {
                    "▾ Project License: MIT"
                } else {
                    "▸ Project License: MIT"
                })
                .size(13.0)
                .class(cosmic::theme::Text::Color(theme.config.accent)),
            )
            .push(cosmic::iced::widget::space::horizontal())
            .push(
                button::text(if mit_expanded {
                    "Hide Text"
                } else {
                    "View Full Text"
                })
                .on_press(Message::ToggleLicenseDetail(0))
                .padding([2, 8]),
            );

        let mut mit_card_col = column::with_capacity(3).spacing(6).push(mit_header);
        if mit_expanded {
            let mit_text =
                "Permission is hereby granted, free of charge, to any person obtaining a copy\n\
                of this software and associated documentation files (the \"Software\"), to deal\n\
                in the Software without restriction, including without limitation the rights\n\
                to use, copy, modify, merge, publish, distribute, sublicense, and/or sell\n\
                copies of the Software, and to permit persons to whom the Software is\n\
                furnished to do so, subject to the following conditions:\n\n\
                The above copyright notice and this permission notice shall be included in all\n\
                copies or substantial portions of the Software.\n\n\
                THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR\n\
                IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,\n\
                FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.";
            mit_card_col = mit_card_col.push(
                container(
                    text(mit_text)
                        .size(10.5)
                        .class(cosmic::theme::Text::Color(theme.config.fg)),
                )
                .padding(8)
                .class(cosmic::theme::Container::Custom(Box::new(|_| {
                    container::Style {
                        border: cosmic::iced::border::rounded(4).width(1.0),
                        ..Default::default()
                    }
                }))),
            );
        }
        let mit_card = make_card(mit_card_col.into());

        // GPL-3.0-only Notice Card
        let gpl_expanded = self.expanded_license_idx == Some(1);
        let gpl_header = row::with_capacity(3)
            .spacing(8)
            .align_y(Alignment::Center)
            .push(
                text(if gpl_expanded {
                    "▾ GPL-3.0-only Dependencies"
                } else {
                    "▸ GPL-3.0-only Dependencies"
                })
                .size(13.0)
                .class(cosmic::theme::Text::Color(theme.config.accent)),
            )
            .push(cosmic::iced::widget::space::horizontal())
            .push(
                button::text(if gpl_expanded {
                    "Hide Details"
                } else {
                    "View Details"
                })
                .on_press(Message::ToggleLicenseDetail(1))
                .padding([2, 8]),
            );

        let mut gpl_card_col = column::with_capacity(4).spacing(6).push(gpl_header).push(
            text(
                "Crates: cosmic-protocols (0.2.0), cosmic-client-toolkit (0.2.0)\n\
                      Source: https://github.com/pop-os/cosmic-protocols (Pinned: 32283d7)\n\
                      License declaration: GPL-3.0-only",
            )
            .size(11.0)
            .class(cosmic::theme::Text::Color(theme.config.comment)),
        );

        if gpl_expanded {
            gpl_card_col = gpl_card_col.push(
                container(
                    text("Fact-Based Boundary:\n\
                          1. Rooney source code is licensed under MIT (no vendored GPL code in repo).\n\
                          2. When compiled via cargo build, these crates are statically linked into the target binary.\n\
                          3. Downstream packagers who redistribute compiled binaries to third parties must evaluate and comply with applicable GPL-3.0 obligations.\n\
                          See THIRD_PARTY_LICENSES/GPL-3.0-only.txt and LICENSES.ja.md for details.")
                        .size(10.5)
                        .class(cosmic::theme::Text::Color(theme.config.fg)),
                )
                .padding(8)
                .class(cosmic::theme::Container::Custom(Box::new(|_| {
                    container::Style {
                        border: cosmic::iced::border::rounded(4).width(1.0),
                        ..Default::default()
                    }
                }))),
            );
        }
        let gpl_card = make_card(gpl_card_col.into());

        // Other Major Components Card
        let other_card = make_card(
            column::with_capacity(6)
                .spacing(5)
                .push(
                    text("Other Third-Party Dependencies (Audited via Cargo.lock):")
                        .size(12.5)
                        .class(cosmic::theme::Text::Color(theme.config.fg)),
                )
                .push(
                    text("• libcosmic family (MPL-2.0) - Pinned git rev d4d71fd\n\
                          • ropey 1.6, tree-sitter 0.24, tokio 1.40 (MIT)\n\
                          • cosmic-text 0.12, reqwest 0.12, winit 0.31 (Apache-2.0 OR MIT)\n\
                          • window_clipboard / dnd / mime (MIT) - Tag sctk-0.20")
                        .size(11.0)
                        .class(cosmic::theme::Text::Color(theme.config.comment)),
                )
                .push(
                    text("Documentation & Audit Records:\n\
                          Refer to LICENSES.md, LICENSES.ja.md, and THIRD_PARTY_LICENSES/README.md.")
                        .size(10.5)
                        .class(cosmic::theme::Text::Color(theme.config.fg)),
                )
                .into(),
        );

        let list_col = column::with_capacity(5)
            .spacing(10)
            .push(project_card)
            .push(mit_card)
            .push(gpl_card)
            .push(other_card);

        scrollable(list_col).height(Length::Fixed(360.0)).into()
    }
}
