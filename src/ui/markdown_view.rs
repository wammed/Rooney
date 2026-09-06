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
            MarkdownBlock::ListItem { depth, text: t } => {
                let indent = (*depth as f32) * 16.0 + 8.0;
                let list_row = row::with_capacity(3)
                    .push(Space::new().width(Length::Fixed(indent)))
                    .push(text("• ").class(cosmic::theme::Text::Color(theme.config.accent)))
                    .push(text(t).class(cosmic::theme::Text::Color(theme.config.fg)))
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
