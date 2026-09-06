use crate::app::message::Message;
use crate::app::App;
use crate::ui::file_tree_view::FileTreeMessage;
use cosmic::iced::widget::{column, row};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, container, text, Space};

impl App {
    pub(crate) fn render_context_menu_overlay<'a>(
        &'a self,
        base_view: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let Some((_pane_id, cx, cy)) = self.context_menu else {
            return base_view;
        };

        let theme = &self.theme;
        let menu_w = 230.0;
        let menu_h = 280.0;
        let menu_x = if cx + menu_w > 1550.0 {
            (cx - menu_w).max(10.0)
        } else {
            cx.max(10.0)
        };
        let menu_y = if cy + menu_h > 1500.0 {
            (cy - menu_h).max(10.0)
        } else {
            cy.max(10.0)
        };

        let make_item =
            |icon: &'static str, label: &'static str, shortcut: &'static str, msg: Message| {
                let content = row::with_capacity(3)
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .padding([5, 8])
                    .push(
                        text(format!("{icon}  {label}"))
                            .size(13.0)
                            .class(cosmic::theme::Text::Color(theme.config.fg)),
                    )
                    .push(cosmic::iced::widget::space::horizontal())
                    .push(
                        text(shortcut)
                            .size(11.0)
                            .class(cosmic::theme::Text::Color(theme.config.comment)),
                    );

                button::custom(content)
                    .on_press(msg)
                    .class(cosmic::theme::Button::Transparent)
                    .width(Length::Fill)
            };

        let menu_items = column::with_capacity(7)
            .spacing(2)
            .padding(4)
            .push(make_item("󰆏", "Copy", "Ctrl+C", Message::Copy))
            .push(make_item("󰆐", "Cut", "Ctrl+X", Message::Cut))
            .push(make_item("󰆒", "Paste", "Ctrl+V", Message::Paste))
            .push(make_item("󰒅", "Select All", "Ctrl+A", Message::SelectAll))
            .push(make_item("󰕌", "Undo", "Ctrl+Z", Message::Undo))
            .push(make_item("󰑎", "Redo", "Ctrl+Y", Message::Redo))
            .push(make_item("✕", "Close", "Esc", Message::CloseContextMenu));

        let context_menu_box = container(menu_items)
            .class(cosmic::theme::Container::Card)
            .padding(4)
            .width(Length::Fixed(menu_w));

        let positioned_menu = row::with_capacity(2)
            .push(Space::new().width(Length::Fixed(menu_x)))
            .push(
                column::with_capacity(2)
                    .push(Space::new().height(Length::Fixed(menu_y)))
                    .push(context_menu_box),
            )
            .width(Length::Fill)
            .height(Length::Fill);

        let backdrop = button::custom(Space::new().width(Length::Fill).height(Length::Fill))
            .on_press(Message::CloseContextMenu)
            .class(cosmic::theme::Button::Transparent);

        cosmic::iced::widget::stack(vec![base_view, backdrop.into(), positioned_menu.into()]).into()
    }

    pub(crate) fn render_file_tree_context_menu_overlay<'a>(
        &'a self,
        base_view: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let Some(ref cm) = self.file_tree_context_menu else {
            return base_view;
        };

        let theme = &self.theme;
        let menu_w = 220.0;
        let menu_h = 240.0;
        let menu_x = if cm.x + menu_w > 1550.0 {
            (cm.x - menu_w).max(10.0)
        } else {
            cm.x.max(10.0)
        };
        let menu_y = if cm.y + menu_h > 1500.0 {
            (cm.y - menu_h).max(10.0)
        } else {
            cm.y.max(10.0)
        };

        let make_item = |icon: &'static str, label: &'static str, msg: Message| {
            let content = row::with_capacity(2)
                .spacing(8)
                .align_y(Alignment::Center)
                .padding([5, 8])
                .push(
                    text(format!("{icon}  {label}"))
                        .size(12.5)
                        .class(cosmic::theme::Text::Color(theme.config.fg)),
                )
                .width(Length::Fill);

            button::custom(content)
                .on_press(msg)
                .class(cosmic::theme::Button::Transparent)
                .width(Length::Fill)
        };

        let mut menu_items = column::with_capacity(8).spacing(2).padding(4);

        if let Some(ref target_path) = cm.target {
            let display_name = target_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            let truncated_name = if display_name.len() > 22 {
                format!("{}…", &display_name[..20])
            } else {
                display_name.to_string()
            };

            let header_icon = if cm.is_dir { "" } else { "󰈙" };
            let header = container(
                text(format!("{header_icon}  {truncated_name}"))
                    .size(11.0)
                    .class(cosmic::theme::Text::Color(theme.config.accent)),
            )
            .padding([4, 8]);

            menu_items = menu_items.push(header);

            if cm.is_dir {
                menu_items = menu_items.push(make_item(
                    "",
                    "New File Here...",
                    Message::FileTreeMsg(FileTreeMessage::NewFileIn(Some(target_path.clone()))),
                ));
                menu_items = menu_items.push(make_item(
                    "",
                    "New Folder Here...",
                    Message::FileTreeMsg(FileTreeMessage::NewFolderIn(Some(target_path.clone()))),
                ));
                menu_items = menu_items.push(make_item(
                    "",
                    "Rename Folder...",
                    Message::FileTreeMsg(FileTreeMessage::Rename(target_path.clone())),
                ));
                menu_items = menu_items.push(make_item(
                    "",
                    "Delete Folder...",
                    Message::FileTreeMsg(FileTreeMessage::Delete(target_path.clone())),
                ));
            } else {
                menu_items = menu_items.push(make_item(
                    "󰈙",
                    "Open File",
                    Message::FileTreeMsg(FileTreeMessage::OpenFile(target_path.clone())),
                ));
                menu_items = menu_items.push(make_item(
                    "",
                    "Rename File...",
                    Message::FileTreeMsg(FileTreeMessage::Rename(target_path.clone())),
                ));
                menu_items = menu_items.push(make_item(
                    "",
                    "Delete File...",
                    Message::FileTreeMsg(FileTreeMessage::Delete(target_path.clone())),
                ));
            }
        } else {
            let header = container(
                text("  WORKSPACE ROOT")
                    .size(11.0)
                    .class(cosmic::theme::Text::Color(theme.config.accent)),
            )
            .padding([4, 8]);

            menu_items = menu_items.push(header);

            menu_items = menu_items.push(make_item(
                "",
                "New File in Root...",
                Message::FileTreeMsg(FileTreeMessage::NewFileIn(None)),
            ));
            menu_items = menu_items.push(make_item(
                "",
                "New Folder in Root...",
                Message::FileTreeMsg(FileTreeMessage::NewFolderIn(None)),
            ));
            menu_items = menu_items.push(make_item(
                "",
                "Open Folder...",
                Message::FileTreeMsg(FileTreeMessage::OpenFolder),
            ));
        }

        menu_items = menu_items.push(make_item(
            "",
            "Refresh Tree",
            Message::FileTreeMsg(FileTreeMessage::Refresh),
        ));
        menu_items = menu_items.push(make_item(
            "✕",
            "Close Menu",
            Message::CloseFileTreeContextMenu,
        ));

        let context_menu_box = container(menu_items)
            .class(cosmic::theme::Container::Card)
            .padding(4)
            .width(Length::Fixed(menu_w));

        let positioned_menu = row::with_capacity(2)
            .push(Space::new().width(Length::Fixed(menu_x)))
            .push(
                column::with_capacity(2)
                    .push(Space::new().height(Length::Fixed(menu_y)))
                    .push(context_menu_box),
            )
            .width(Length::Fill)
            .height(Length::Fill);

        let backdrop = button::custom(Space::new().width(Length::Fill).height(Length::Fill))
            .on_press(Message::CloseFileTreeContextMenu)
            .class(cosmic::theme::Button::Transparent);

        cosmic::iced::widget::stack(vec![base_view, backdrop.into(), positioned_menu.into()]).into()
    }
}
