use crate::fs::tree::FileTree;
use crate::theme::EditorTheme;
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, column, container, mouse_area, row, scrollable, text, Space};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileTreeMessage {
    ToggleDir(PathBuf),
    OpenFile(PathBuf),
    Refresh,
    ToggleVisibility,
    OpenFolder,
    GoToParent,
    NewFileIn(Option<PathBuf>),
    NewFolderIn(Option<PathBuf>),
    Rename(PathBuf),
    Delete(PathBuf),
    RightClick(PathBuf, bool),
    RightClickRoot,
}

pub fn view_file_tree<'a, Message: 'static + Clone>(
    tree: &'a FileTree,
    theme: &'a EditorTheme,
    _font_name: &'a str,
    on_msg: impl Fn(FileTreeMessage) -> Message + Copy + 'static,
) -> Element<'a, Message> {
    let mut col = column::with_capacity(tree.items.len() + 3).spacing(1);

    let root_name = tree
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("ROOT");

    let header_row = row::with_capacity(8)
        .push(
            text::title4(format!("  {}", root_name.to_uppercase()))
                .size(11.0)
                .class(cosmic::theme::Text::Color(theme.config.sidebar_fg)),
        )
        .push(cosmic::iced::widget::space::horizontal())
        .push(
            button::text("")
                .on_press(on_msg(FileTreeMessage::NewFileIn(None)))
                .padding([2, 5]),
        )
        .push(
            button::text("")
                .on_press(on_msg(FileTreeMessage::NewFolderIn(None)))
                .padding([2, 5]),
        )
        .push(
            button::text("")
                .on_press(on_msg(FileTreeMessage::GoToParent))
                .padding([2, 5]),
        )
        .push(
            button::text("")
                .on_press(on_msg(FileTreeMessage::OpenFolder))
                .padding([2, 5]),
        )
        .push(
            button::text("")
                .on_press(on_msg(FileTreeMessage::Refresh))
                .padding([2, 5]),
        )
        .push(Space::new().width(Length::Fixed(14.0)))
        .align_y(Alignment::Center)
        .spacing(3)
        .padding([4, 6]);

    let header_area = mouse_area(header_row)
        .on_right_press(on_msg(FileTreeMessage::RightClickRoot));

    col = col.push(header_area);

    for item in &tree.items {
        let is_selected = tree.selected_path.as_ref() == Some(&item.path);
        let indent = (item.depth as f32) * 12.0 + 4.0;

        let icon_widget = text(item.icon.glyph)
            .size(13.0)
            .class(cosmic::theme::Text::Color(item.icon.color));

        let label_color = if is_selected {
            theme.config.accent
        } else {
            theme.config.sidebar_fg
        };

        let label_widget = text(&item.name)
            .size(12.0)
            .class(cosmic::theme::Text::Color(label_color));

        let path = item.path.clone();
        let is_dir = item.is_dir;

        // Item body button
        let main_click_content = row::with_capacity(4)
            .push(Space::new().width(Length::Fixed(indent)))
            .push(icon_widget)
            .push(Space::new().width(Length::Fixed(6.0)))
            .push(label_widget)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        let main_btn = button::custom(main_click_content)
            .on_press(if is_dir {
                on_msg(FileTreeMessage::ToggleDir(path.clone()))
            } else {
                on_msg(FileTreeMessage::OpenFile(path.clone()))
            })
            .width(Length::Fill)
            .padding([2, 4])
            .class(cosmic::theme::Button::Transparent);

        // Action buttons on the right
        let mut actions = row::with_capacity(3).spacing(1).align_y(Alignment::Center);

        if is_dir {
            actions = actions.push(
                button::text("")
                    .on_press(on_msg(FileTreeMessage::NewFileIn(Some(path.clone()))))
                    .padding([1, 3])
                    .class(cosmic::theme::Button::Text),
            );
        }

        actions = actions.push(
            button::text("")
                .on_press(on_msg(FileTreeMessage::Rename(path.clone())))
                .padding([1, 3])
                .class(cosmic::theme::Button::Text),
        );

        actions = actions.push(
            button::text("")
                .on_press(on_msg(FileTreeMessage::Delete(path.clone())))
                .padding([1, 3])
                .class(cosmic::theme::Button::Text),
        );

        let row_container = row::with_capacity(3)
            .push(main_btn)
            .push(actions)
            .push(Space::new().width(Length::Fixed(16.0)))
            .align_y(Alignment::Center)
            .width(Length::Fill);

        let row_area = mouse_area(row_container)
            .on_right_press(on_msg(FileTreeMessage::RightClick(path.clone(), is_dir)));

        col = col.push(row_area);
    }

    let empty_bottom = mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
        .on_right_press(on_msg(FileTreeMessage::RightClickRoot));
    col = col.push(empty_bottom);

    let sidebar_w = tree.width.max(280.0);

    let scroll = scrollable(col)
        .width(Length::Fixed(sidebar_w))
        .height(Length::Fill);

    let sidebar_bg = theme.sidebar_with_alpha();
    container(scroll)
        .class(cosmic::theme::Container::Custom(Box::new(move |_| {
            container::Style {
                background: Some(sidebar_bg.into()),
                ..Default::default()
            }
        })))
        .width(Length::Fixed(sidebar_w))
        .height(Length::Fill)
        .into()
}
