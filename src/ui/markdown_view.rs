use crate::markdown::renderer::{
    spans_plain_text, AlertKind, ColumnAlignment, InlineSpan, MarkdownBlock, MarkdownDocument,
};
use crate::theme::EditorTheme;
use cosmic::iced::widget::text::{Rich, Span};
use cosmic::iced::{Alignment, Border, Color, Font, Length, Padding};
use cosmic::prelude::*;
use cosmic::widget::{column, container, row, scrollable, text, Space};

fn render_spans_with_width<'a, Message: 'static + Clone>(
    spans: &'a [InlineSpan],
    base_size: f32,
    default_color: Color,
    theme: &'a EditorTheme,
    width: Length,
) -> Element<'a, Message> {
    if spans.is_empty() {
        return text("").size(base_size).into();
    }

    // Fast path: When it's a single plain text span, use standard cached text widget for high performance
    if spans.len() == 1 {
        if let InlineSpan::Text(s) = &spans[0] {
            return text(s.as_str())
                .size(base_size)
                .class(cosmic::theme::Text::Color(default_color))
                .into();
        }
    }

    let is_dimmed = default_color == theme.config.comment;

    let iced_spans: Vec<Span<'a, ()>> = spans
        .iter()
        .map(|span| match span {
            InlineSpan::Text(s) => Span::new(s.as_str())
                .size(base_size)
                .color(default_color),
            InlineSpan::Bold(s) => Span::new(s.as_str())
                .size(base_size)
                .color(default_color)
                .font(Font {
                    weight: cosmic::iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }),
            InlineSpan::Italic(s) => Span::new(s.as_str())
                .size(base_size)
                .color(default_color)
                .font(Font {
                    style: cosmic::iced::font::Style::Italic,
                    ..Font::DEFAULT
                }),
            InlineSpan::Code(s) => {
                let code_color = if is_dimmed {
                    theme.config.comment
                } else {
                    theme.config.function
                };
                Span::new(s.as_str())
                    .size(base_size * 0.9)
                    .font(Font::MONOSPACE)
                    .color(code_color)
                    .background(Color::from_rgba(1.0, 1.0, 1.0, 0.08))
                    .border(Border {
                        radius: 4.0.into(),
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.12),
                        width: 1.0,
                    })
                    .padding(Padding {
                        top: 1.5,
                        right: 5.0,
                        bottom: 1.5,
                        left: 5.0,
                    })
            }
            InlineSpan::Strikethrough(s) => Span::new(s.as_str())
                .size(base_size)
                .color(theme.config.comment)
                .strikethrough(true),
            InlineSpan::Link {
                text: link_text,
                url,
            } => {
                let display = if link_text.trim().is_empty() {
                    url.as_str()
                } else {
                    link_text.as_str()
                };
                let link_color = if is_dimmed {
                    theme.config.comment
                } else {
                    theme.config.accent
                };
                Span::new(display)
                    .size(base_size)
                    .color(link_color)
            }
            InlineSpan::ImageFallback { alt, url } => {
                let display = if alt.trim().is_empty() {
                    let filename = url.rsplit('/').next().unwrap_or("image");
                    format!("󰋩 [画像: {}]", filename)
                } else {
                    format!("󰋩 [画像: {}]", alt.trim())
                };
                Span::new(display)
                    .size(base_size * 0.9)
                    .font(Font::MONOSPACE)
                    .color(if is_dimmed {
                        theme.config.comment
                    } else {
                        theme.config.type_name
                    })
                    .background(Color::from_rgba(1.0, 1.0, 1.0, 0.08))
                    .border(Border {
                        radius: 3.0.into(),
                        color: theme.config.border,
                        width: 1.0,
                    })
                    .padding(Padding {
                        top: 1.0,
                        right: 4.0,
                        bottom: 1.0,
                        left: 4.0,
                    })
            }
        })
        .collect();

    Rich::with_spans(iced_spans)
        .size(base_size)
        .width(width)
        .into()
}

fn render_spans<'a, Message: 'static + Clone>(
    spans: &'a [InlineSpan],
    base_size: f32,
    default_color: Color,
    theme: &'a EditorTheme,
) -> Element<'a, Message> {
    render_spans_with_width(spans, base_size, default_color, theme, Length::Fill)
}

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
            MarkdownBlock::Heading { level, spans } => {
                let (size, color) = match level {
                    1 => (24.0, theme.config.accent),
                    2 => (20.0, theme.config.function),
                    3 => (16.0, theme.config.type_name),
                    _ => (14.0, theme.config.fg),
                };
                let heading_widget = render_spans(spans, size, color, theme);
                if *level <= 2 {
                    let border_color = theme.config.border;
                    let border_line = container(Space::new().height(Length::Fixed(1.0)))
                        .width(Length::Fill)
                        .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                            container::Style {
                                background: Some(border_color.into()),
                                ..Default::default()
                            }
                        })));

                    let h_col = column::with_capacity(2)
                        .spacing(6)
                        .push(heading_widget)
                        .push(border_line);

                    col = col.push(h_col);
                } else {
                    col = col.push(heading_widget);
                }
            }
            MarkdownBlock::Paragraph(spans) => {
                col = col.push(render_spans(spans, 13.5, theme.config.fg, theme));
            }
            MarkdownBlock::CodeBlock { lang, code } => {
                let header = text(if lang.is_empty() { "text" } else { lang })
                    .size(10.5)
                    .font(Font::MONOSPACE)
                    .class(cosmic::theme::Text::Color(theme.config.comment));

                let code_widget = text(code)
                    .size(12.5)
                    .font(Font::MONOSPACE)
                    .class(cosmic::theme::Text::Color(theme.config.fg));

                let block_col = column::with_capacity(2)
                    .spacing(6)
                    .push(header)
                    .push(code_widget);

                let border_color = theme.config.border;
                let bg_color = theme.config.gutter_bg;
                let code_box = container(block_col)
                    .padding([10, 14])
                    .width(Length::Fill)
                    .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                        container::Style {
                            background: Some(bg_color.into()),
                            border: cosmic::iced::border::rounded(6)
                                .color(border_color)
                                .width(1.0),
                            ..Default::default()
                        }
                    })));

                col = col.push(code_box);
            }
            MarkdownBlock::ListItem {
                depth,
                spans,
                task_status,
            } => {
                let indent = (*depth as f32) * 16.0 + 8.0;
                let (bullet_str, bullet_color, text_color) = match task_status {
                    Some(true) => ("󰱒 ", theme.config.comment, theme.config.comment),
                    Some(false) => ("󰄱 ", theme.config.fg, theme.config.fg),
                    None => ("• ", theme.config.accent, theme.config.fg),
                };

                let bullet_widget =
                    text(bullet_str).class(cosmic::theme::Text::Color(bullet_color));
                let content_widget = render_spans(spans, 13.0, text_color, theme);

                let list_row = row::with_capacity(3)
                    .spacing(6)
                    .push(Space::new().width(Length::Fixed(indent)))
                    .push(bullet_widget)
                    .push(content_widget)
                    .width(Length::Fill)
                    .align_y(Alignment::Start);

                col = col.push(list_row);
            }
            MarkdownBlock::BlockQuote(spans) => {
                let accent_color = theme.config.comment;
                let left_bar =
                    container(Space::new().width(Length::Fixed(3.5)).height(Length::Fill)).class(
                        cosmic::theme::Container::Custom(Box::new(move |_| container::Style {
                            background: Some(accent_color.into()),
                            border: cosmic::iced::border::rounded(2),
                            ..Default::default()
                        })),
                    );

                let quote_content = render_spans(spans, 13.0, theme.config.comment, theme);
                let content_box = container(quote_content)
                    .padding([2, 8])
                    .width(Length::Fill);

                let quote_row = row::with_capacity(2)
                    .spacing(8)
                    .push(left_bar)
                    .push(content_box)
                    .width(Length::Fill);

                col = col.push(quote_row);
            }
            MarkdownBlock::Alert { kind, spans } => {
                let (label, icon, color) = match kind {
                    AlertKind::Note => ("Note", "󰋽", theme.config.function),
                    AlertKind::Tip => ("Tip", "󰌵", theme.config.string),
                    AlertKind::Important => ("Important", "󰅒", theme.config.keyword),
                    AlertKind::Warning => ("Warning", "󰀦", theme.config.number),
                    AlertKind::Caution => (
                        "Caution",
                        "󰳦",
                        cosmic::iced::Color::from_rgb(0.95, 0.35, 0.35),
                    ),
                };

                let left_bar =
                    container(Space::new().width(Length::Fixed(4.0)).height(Length::Fill)).class(
                        cosmic::theme::Container::Custom(Box::new(move |_| container::Style {
                            background: Some(color.into()),
                            border: cosmic::iced::border::rounded(2),
                            ..Default::default()
                        })),
                    );

                let alert_header = row::with_capacity(2)
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .push(text(icon).size(14.0).class(cosmic::theme::Text::Color(color)))
                    .push(text(label).size(13.0).class(cosmic::theme::Text::Color(color)));

                let alert_body = render_spans(spans, 12.5, theme.config.fg, theme);

                let alert_col = column::with_capacity(2)
                    .spacing(6)
                    .push(alert_header)
                    .push(alert_body);

                let border_color = theme.config.border;
                let alert_box = container(alert_col)
                    .padding([8, 12])
                    .width(Length::Fill)
                    .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                        container::Style {
                            background: Some(
                                cosmic::iced::Color::from_rgba(1.0, 1.0, 1.0, 0.02).into(),
                            ),
                            border: cosmic::iced::border::rounded(4)
                                .color(border_color)
                                .width(0.5),
                            ..Default::default()
                        }
                    })));

                let alert_row = row::with_capacity(2)
                    .spacing(8)
                    .push(left_bar)
                    .push(alert_box)
                    .width(Length::Fill)
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
                        let char_len = spans_plain_text(header).chars().count() as f32;
                        col_widths[c] = col_widths[c].max(char_len * 9.5 + 24.0);
                    }
                    for row_data in &table.rows {
                        for (c, cell) in row_data.iter().enumerate() {
                            if c < num_cols {
                                let char_len = spans_plain_text(cell).chars().count() as f32;
                                col_widths[c] = col_widths[c].max(char_len * 9.0 + 24.0);
                            }
                        }
                    }
                    for w in &mut col_widths {
                        *w = w.clamp(90.0, 360.0);
                    }

                    let border_color = theme.config.border;
                    let mut table_col = column::with_capacity(table.rows.len() + 1).spacing(0);

                    // Header row with background and grid borders
                    let mut header_row_widget = row::with_capacity(num_cols).spacing(0);
                    for (c, width) in col_widths.iter().enumerate() {
                        let header_spans =
                            table.headers.get(c).map(|s| s.as_slice()).unwrap_or(&[]);
                        let align = table
                            .alignments
                            .get(c)
                            .copied()
                            .unwrap_or(ColumnAlignment::None);

                        let header_txt = render_spans_with_width(
                            header_spans,
                            12.5,
                            theme.config.accent,
                            theme,
                            Length::Shrink,
                        );

                        let mut cell_row = row::with_capacity(3).align_y(Alignment::Center);
                        match align {
                            ColumnAlignment::Right => {
                                cell_row = cell_row
                                    .push(Space::new().width(Length::Fill))
                                    .push(header_txt);
                            }
                            ColumnAlignment::Center => {
                                cell_row = cell_row
                                    .push(Space::new().width(Length::Fill))
                                    .push(header_txt)
                                    .push(Space::new().width(Length::Fill));
                            }
                            ColumnAlignment::Left | ColumnAlignment::None => {
                                cell_row = cell_row
                                    .push(header_txt)
                                    .push(Space::new().width(Length::Fill));
                            }
                        }

                        let cell_box = container(cell_row)
                            .width(Length::Fixed(*width))
                            .padding([6, 10])
                            .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                                container::Style {
                                    background: Some(
                                        cosmic::iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08).into(),
                                    ),
                                    border: cosmic::iced::Border {
                                        color: border_color,
                                        width: 1.0,
                                        radius: 0.0.into(),
                                    },
                                    ..Default::default()
                                }
                            })));
                        header_row_widget = header_row_widget.push(cell_box);
                    }
                    table_col = table_col.push(header_row_widget);

                    // Data rows with grid borders and zebra striping: rgba(255, 255, 255, 0.04)
                    for (r, row_data) in table.rows.iter().enumerate() {
                        let is_even_row = r % 2 == 1;
                        let mut data_row_widget = row::with_capacity(num_cols).spacing(0);
                        for (c, width) in col_widths.iter().enumerate() {
                            let cell_spans =
                                row_data.get(c).map(|s| s.as_slice()).unwrap_or(&[]);
                            let align = table
                                .alignments
                                .get(c)
                                .copied()
                                .unwrap_or(ColumnAlignment::None);

                            let cell_txt = render_spans_with_width(
                                cell_spans,
                                12.0,
                                theme.config.fg,
                                theme,
                                Length::Shrink,
                            );

                            let mut cell_row = row::with_capacity(3).align_y(Alignment::Center);
                            match align {
                                ColumnAlignment::Right => {
                                    cell_row = cell_row
                                        .push(Space::new().width(Length::Fill))
                                        .push(cell_txt);
                                }
                                ColumnAlignment::Center => {
                                    cell_row = cell_row
                                        .push(Space::new().width(Length::Fill))
                                        .push(cell_txt)
                                        .push(Space::new().width(Length::Fill));
                                }
                                ColumnAlignment::Left | ColumnAlignment::None => {
                                    cell_row = cell_row
                                        .push(cell_txt)
                                        .push(Space::new().width(Length::Fill));
                                }
                            }

                            let cell_box = container(cell_row)
                                .width(Length::Fixed(*width))
                                .padding([5, 10])
                                .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                                    container::Style {
                                        background: if is_even_row {
                                            Some(
                                                cosmic::iced::Color::from_rgba(
                                                    1.0, 1.0, 1.0, 0.04,
                                                )
                                                .into(),
                                            )
                                        } else {
                                            None
                                        },
                                        border: cosmic::iced::Border {
                                            color: border_color,
                                            width: 1.0,
                                            radius: 0.0.into(),
                                        },
                                        ..Default::default()
                                    }
                                })));
                            data_row_widget = data_row_widget.push(cell_box);
                        }
                        table_col = table_col.push(data_row_widget);
                    }

                    let table_box = container(table_col)
                        .padding(6)
                        .width(Length::Fill);

                    col = col.push(table_box);
                }
            }
            MarkdownBlock::Footnote { label, spans } => {
                let fn_prefix = text(format!("[^{}]: ", label))
                    .size(11.5)
                    .class(cosmic::theme::Text::Color(theme.config.accent));
                let fn_body = render_spans(spans, 11.5, theme.config.comment, theme);
                let fn_row = row::with_capacity(2)
                    .spacing(6)
                    .push(fn_prefix)
                    .push(fn_body)
                    .width(Length::Fill)
                    .align_y(Alignment::Start);
                col = col.push(fn_row);
            }
            MarkdownBlock::Rule => {
                let border_color = theme.config.border;
                let rule = container(Space::new().height(Length::Fixed(1.0)))
                    .width(Length::Fill)
                    .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                        container::Style {
                            background: Some(border_color.into()),
                            ..Default::default()
                        }
                    })));
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

