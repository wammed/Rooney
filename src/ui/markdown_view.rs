use crate::markdown::renderer::{MarkdownBlock, MarkdownDocument};
use crate::theme::EditorTheme;
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{column, container, row, scrollable, text, Space};

pub fn view_markdown<'a, Message: 'static + Clone>(
    doc: &'a MarkdownDocument,
    theme: &'a EditorTheme,
    _font_name: &'a str,
) -> Element<'a, Message> {
    let mut col = column::with_capacity(doc.blocks.len() + 2)
        .spacing(12)
        .padding(20);

    for block in &doc.blocks {
        match block {
            MarkdownBlock::Heading { level, text: t } => {
                let (size, color) = match level {
                    1 => (24.0, theme.config.accent),
                    2 => (20.0, theme.config.function),
                    3 => (16.0, theme.config.type_name),
                    _ => (14.0, theme.config.fg),
                };
                col = col.push(
                    text(t)
                        .size(size)
                        .class(cosmic::theme::Text::Color(color)),
                );
            }
            MarkdownBlock::Paragraph(t) => {
                col = col.push(
                    text(t)
                        .size(13.5)
                        .class(cosmic::theme::Text::Color(theme.config.fg)),
                );
            }
            MarkdownBlock::CodeBlock { lang, code } => {
                let code_widget = text(code)
                    .size(12.5)
                    .class(cosmic::theme::Text::Color(theme.config.fg));

                let header = text(if lang.is_empty() { "code" } else { lang })
                    .size(10.0)
                    .class(cosmic::theme::Text::Color(theme.config.comment));

                let block_col = column::with_capacity(2)
                    .spacing(4)
                    .push(header)
                    .push(code_widget);

                let code_box = container(block_col)
                    .padding(10)
                    .width(Length::Fill);

                col = col.push(code_box);
            }
            MarkdownBlock::ListItem {
                depth,
                text: t,
                task_status,
            } => {
                let indent = (*depth as f32) * 16.0 + 8.0;
                let (bullet_str, bullet_color, text_color) = match task_status {
                    Some(true) => ("󰱒 ", theme.config.comment, theme.config.comment),
                    Some(false) => ("󰄱 ", theme.config.fg, theme.config.fg),
                    None => ("• ", theme.config.accent, theme.config.fg),
                };
                let list_row = row::with_capacity(3)
                    .push(Space::new().width(Length::Fixed(indent)))
                    .push(text(bullet_str).class(cosmic::theme::Text::Color(bullet_color)))
                    .push(text(t).class(cosmic::theme::Text::Color(text_color)))
                    .align_y(Alignment::Center);

                col = col.push(list_row);
            }
            MarkdownBlock::BlockQuote(t) => {
                let quote_row = row::with_capacity(3)
                    .push(text("▍ ").class(cosmic::theme::Text::Color(theme.config.comment)))
                    .push(text(t).class(cosmic::theme::Text::Color(theme.config.comment)))
                    .align_y(Alignment::Center);

                col = col.push(quote_row);
            }
            MarkdownBlock::Alert { kind, text: t } => {
                let (label, icon, color) = match kind {
                    crate::markdown::renderer::AlertKind::Note => {
                        ("Note", "󰋽", theme.config.function)
                    }
                    crate::markdown::renderer::AlertKind::Tip => ("Tip", "󰌵", theme.config.string),
                    crate::markdown::renderer::AlertKind::Important => {
                        ("Important", "󰅒", theme.config.keyword)
                    }
                    crate::markdown::renderer::AlertKind::Warning => {
                        ("Warning", "󰀦", theme.config.number)
                    }
                    crate::markdown::renderer::AlertKind::Caution => {
                        ("Caution", "󰳦", cosmic::iced::Color::from_rgb(0.95, 0.35, 0.35))
                    }
                };

                let alert_header = row::with_capacity(2)
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .push(text(icon).size(14.0).class(cosmic::theme::Text::Color(color)))
                    .push(text(label).size(13.0).class(cosmic::theme::Text::Color(color)));

                let alert_body = text(t)
                    .size(12.5)
                    .class(cosmic::theme::Text::Color(theme.config.fg));

                let alert_col = column::with_capacity(2)
                    .spacing(6)
                    .push(alert_header)
                    .push(alert_body);

                let alert_box = container(alert_col)
                    .padding([8, 12])
                    .width(Length::Fill);

                let alert_row = row::with_capacity(2)
                    .spacing(4)
                    .push(text("▍").size(24.0).class(cosmic::theme::Text::Color(color)))
                    .push(alert_box)
                    .align_y(Alignment::Start);

                col = col.push(alert_row);
            }
            MarkdownBlock::Table(table) => {
                let num_cols = table
                    .headers
                    .len()
                    .max(table.rows.iter().map(|r| r.len()).max().unwrap_or(0));

                if num_cols > 0 {
                    let mut col_widths: Vec<f32> = vec![80.0; num_cols];
                    for (c, header) in table.headers.iter().enumerate() {
                        let char_len = header.chars().count() as f32;
                        col_widths[c] = col_widths[c].max(char_len * 9.5 + 24.0);
                    }
                    for row_data in &table.rows {
                        for (c, cell) in row_data.iter().enumerate() {
                            if c < num_cols {
                                let char_len = cell.chars().count() as f32;
                                col_widths[c] = col_widths[c].max(char_len * 9.0 + 24.0);
                            }
                        }
                    }
                    for w in &mut col_widths {
                        *w = w.clamp(90.0, 320.0);
                    }
                    let total_width: f32 = col_widths.iter().sum::<f32>() + (num_cols as f32 * 4.0);

                    let mut table_col =
                        column::with_capacity(table.rows.len() + 3).spacing(4);

                    // Header row
                    let mut header_row_widget = row::with_capacity(num_cols).spacing(4);
                    for (c, width) in col_widths.iter().enumerate() {
                        let header_str = table.headers.get(c).map(|s| s.as_str()).unwrap_or("");
                        let align = table
                            .alignments
                            .get(c)
                            .copied()
                            .unwrap_or(crate::markdown::renderer::ColumnAlignment::None);

                        let header_txt = text(header_str)
                            .size(12.5)
                            .class(cosmic::theme::Text::Color(theme.config.accent));

                        let mut cell_row = row::with_capacity(3).align_y(Alignment::Center);
                        match align {
                            crate::markdown::renderer::ColumnAlignment::Right => {
                                cell_row = cell_row
                                    .push(Space::new().width(Length::Fill))
                                    .push(header_txt);
                            }
                            crate::markdown::renderer::ColumnAlignment::Center => {
                                cell_row = cell_row
                                    .push(Space::new().width(Length::Fill))
                                    .push(header_txt)
                                    .push(Space::new().width(Length::Fill));
                            }
                            crate::markdown::renderer::ColumnAlignment::Left
                            | crate::markdown::renderer::ColumnAlignment::None => {
                                cell_row = cell_row
                                    .push(header_txt)
                                    .push(Space::new().width(Length::Fill));
                            }
                        }

                        let cell_box = container(cell_row)
                            .width(Length::Fixed(*width))
                            .padding([6, 8]);
                        header_row_widget = header_row_widget.push(cell_box);
                    }
                    table_col = table_col.push(header_row_widget);

                    // Separator
                    table_col = table_col.push(
                        container(Space::new().height(Length::Fixed(1.0)))
                            .width(Length::Fixed(total_width)),
                    );

                    // Data rows
                    for row_data in &table.rows {
                        let mut data_row_widget = row::with_capacity(num_cols).spacing(4);
                        for (c, width) in col_widths.iter().enumerate() {
                            let cell_str = row_data.get(c).map(|s| s.as_str()).unwrap_or("");
                            let align = table
                                .alignments
                                .get(c)
                                .copied()
                                .unwrap_or(crate::markdown::renderer::ColumnAlignment::None);

                            let cell_txt = text(cell_str)
                                .size(12.0)
                                .class(cosmic::theme::Text::Color(theme.config.fg));

                            let mut cell_row = row::with_capacity(3).align_y(Alignment::Center);
                            match align {
                                crate::markdown::renderer::ColumnAlignment::Right => {
                                    cell_row = cell_row
                                        .push(Space::new().width(Length::Fill))
                                        .push(cell_txt);
                                }
                                crate::markdown::renderer::ColumnAlignment::Center => {
                                    cell_row = cell_row
                                        .push(Space::new().width(Length::Fill))
                                        .push(cell_txt)
                                        .push(Space::new().width(Length::Fill));
                                }
                                crate::markdown::renderer::ColumnAlignment::Left
                                | crate::markdown::renderer::ColumnAlignment::None => {
                                    cell_row = cell_row
                                        .push(cell_txt)
                                        .push(Space::new().width(Length::Fill));
                                }
                            }

                            let cell_box = container(cell_row)
                                .width(Length::Fixed(*width))
                                .padding([4, 8]);
                            data_row_widget = data_row_widget.push(cell_box);
                        }
                        table_col = table_col.push(data_row_widget);
                    }

                    let table_box = container(scrollable(table_col).direction(
                        cosmic::iced::widget::scrollable::Direction::Horizontal(
                            cosmic::iced::widget::scrollable::Scrollbar::default(),
                        ),
                    ))
                    .padding(6)
                    .width(Length::Fill);

                    col = col.push(table_box);
                }
            }
            MarkdownBlock::Rule => {
                let rule = container(Space::new().height(Length::Fixed(1.0)))
                    .width(Length::Fill);
                col = col.push(rule);
            }
        }
    }

    let scroll = scrollable(col)
        .width(Length::Fill)
        .height(Length::Fill);

    container(scroll)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
