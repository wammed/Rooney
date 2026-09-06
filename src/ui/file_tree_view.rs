use crate::fs::tree::FileTree;
use crate::theme::EditorTheme;
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, column, container, row, scrollable, text, Space};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileTreeMessage {
    ToggleDir(PathBuf),
    OpenFile(PathBuf),
    Refresh,
    ToggleVisibility,
}

pub fn view_file_tree<'a, Message: 'static + Clone>(
    tree: &'a FileTree,
    theme: &'a EditorTheme,
    _font_name: &'a str,
    on_msg: impl Fn(FileTreeMessage) -> Message + Copy + 'static,
) -> Element<'a, Message> {
    let mut col = column::with_capacity(tree.items.len() + 2).spacing(1);

    let root_name = tree
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("WORKSPACE");

    let header_row = row::with_capacity(3)
        .push(
            text::title4(format!("   {}", root_name.to_uppercase()))
                .size(11.0)
                .class(cosmic::theme::Text::Color(theme.config.sidebar_fg)),
        )
        .push(cosmic::iced::widget::space::horizontal())
        .push(
            button::text("")
                .on_press(on_msg(FileTreeMessage::Refresh))
                .padding([2, 6]),
        )
        .align_y(Alignment::Center)
        .padding([4, 8]);

    col = col.push(header_row);

    for item in &tree.items {
        let is_selected = tree.selected_path.as_ref() == Some(&item.path);
        let indent = (item.depth as f32) * 14.0 + 8.0;

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

        let item_row = row::with_capacity(4)
            .push(Space::new().width(Length::Fixed(indent)))
            .push(icon_widget)
            .push(Space::new().width(Length::Fixed(6.0)))
            .push(label_widget)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        let path = item.path.clone();
        let is_dir = item.is_dir;

        let btn = button::custom(item_row)
            .on_press(if is_dir {
                on_msg(FileTreeMessage::ToggleDir(path))
            } else {
                on_msg(FileTreeMessage::OpenFile(path))
            })
            .width(Length::Fill)
            .padding([3, 4]);

        col = col.push(btn);
    }

    let scroll = scrollable(col)
        .width(Length::Fixed(tree.width))
        .height(Length::Fill);

    container(scroll)
        .width(Length::Fixed(tree.width))
        .height(Length::Fill)
        .into()
}
