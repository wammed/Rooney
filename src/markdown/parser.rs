use crate::config::MarkdownSpec;
use pulldown_cmark::{
    Alignment as CmarkAlignment, BlockQuoteKind, Event, Options, Parser, Tag, TagEnd,
};

use super::html::parse_html_fragment;
use super::inline::{spans_plain_text, trim_spans, InlineCollector, InlineSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlignment {
    None,
    Left,
    Center,
    Right,
}

impl From<CmarkAlignment> for ColumnAlignment {
    fn from(a: CmarkAlignment) -> Self {
        match a {
            CmarkAlignment::None => Self::None,
            CmarkAlignment::Left => Self::Left,
            CmarkAlignment::Center => Self::Center,
            CmarkAlignment::Right => Self::Right,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableBlock {
    pub headers: Vec<Vec<InlineSpan>>,
    pub alignments: Vec<ColumnAlignment>,
    pub rows: Vec<Vec<Vec<InlineSpan>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl From<BlockQuoteKind> for AlertKind {
    fn from(k: BlockQuoteKind) -> Self {
        match k {
            BlockQuoteKind::Note => Self::Note,
            BlockQuoteKind::Tip => Self::Tip,
            BlockQuoteKind::Important => Self::Important,
            BlockQuoteKind::Warning => Self::Warning,
            BlockQuoteKind::Caution => Self::Caution,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownBlock {
    Heading {
        level: u32,
        spans: Vec<InlineSpan>,
    },
    Paragraph(Vec<InlineSpan>),
    CodeBlock {
        lang: String,
        code: String,
    },
    ListItem {
        depth: usize,
        spans: Vec<InlineSpan>,
        task_status: Option<bool>,
    },
    BlockQuote(Vec<InlineSpan>),
    Alert {
        kind: AlertKind,
        spans: Vec<InlineSpan>,
    },
    Table(TableBlock),
    Footnote {
        label: String,
        spans: Vec<InlineSpan>,
    },
    Rule,
}

impl MarkdownBlock {
    pub fn plain_text(&self) -> String {
        match self {
            MarkdownBlock::Heading { spans, .. }
            | MarkdownBlock::Paragraph(spans)
            | MarkdownBlock::ListItem { spans, .. }
            | MarkdownBlock::BlockQuote(spans)
            | MarkdownBlock::Alert { spans, .. }
            | MarkdownBlock::Footnote { spans, .. } => spans_plain_text(spans),
            MarkdownBlock::CodeBlock { code, .. } => code.clone(),
            MarkdownBlock::Table(_) | MarkdownBlock::Rule => String::new(),
        }
    }
}

#[derive(Default)]
pub struct ListItemState {
    pub depth: usize,
    pub collector: InlineCollector,
    pub task_status: Option<bool>,
    pub emitted: bool,
    pub has_children: bool,
}

#[inline]
pub fn active_collector<'a>(
    list_stack: &'a mut [ListItemState],
    global: &'a mut InlineCollector,
    in_table: bool,
    in_blockquote: bool,
    in_alert: bool,
    in_footnote: bool,
) -> &'a mut InlineCollector {
    if in_table || in_blockquote || in_alert || in_footnote {
        global
    } else if let Some(item) = list_stack.last_mut() {
        &mut item.collector
    } else {
        global
    }
}

#[inline]
pub fn emit_parent_list_item(
    list_stack: &mut [ListItemState],
    blocks: &mut Vec<MarkdownBlock>,
) {
    if let Some(parent) = list_stack.last_mut() {
        if !parent.emitted {
            let spans = parent.collector.finish();
            if !spans.is_empty() || parent.task_status.is_some() {
                blocks.push(MarkdownBlock::ListItem {
                    depth: parent.depth,
                    spans,
                    task_status: parent.task_status,
                });
                parent.emitted = true;
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MarkdownDocument {
    pub blocks: Vec<MarkdownBlock>,
}

impl MarkdownDocument {
    pub fn parse(source: &str, spec: MarkdownSpec) -> Self {
        let opts = match spec {
            MarkdownSpec::CommonMark => Options::empty(),
            MarkdownSpec::Gfm => {
                let mut o = Options::empty();
                o.insert(Options::ENABLE_TABLES);
                o.insert(Options::ENABLE_TASKLISTS);
                o.insert(Options::ENABLE_STRIKETHROUGH);
                o.insert(Options::ENABLE_GFM);
                o.insert(Options::ENABLE_FOOTNOTES);
                o
            }
        };

        let parser = Parser::new_ext(source, opts);
        let mut blocks = Vec::new();
        let mut footnotes = Vec::new();

        let mut collector = InlineCollector::default();
        let mut list_item_stack: Vec<ListItemState> = Vec::new();
        let mut in_heading = None;
        let mut in_code_block = None;
        let mut code_block_text = String::new();
        let mut in_blockquote = false;
        let mut in_alert = None;
        let mut in_footnote: Option<String> = None;
        let mut in_paragraph = false;
        let mut list_depth: usize = 0;

        // HTML block state
        let mut in_html_block = false;
        let mut html_block_content = String::new();

        // Table state
        let mut in_table = false;
        let mut in_table_head = false;
        let mut table_alignments: Vec<ColumnAlignment> = Vec::new();
        let mut table_headers: Vec<Vec<InlineSpan>> = Vec::new();
        let mut table_rows: Vec<Vec<Vec<InlineSpan>>> = Vec::new();
        let mut table_current_row: Vec<Vec<InlineSpan>> = Vec::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    in_heading = Some(level as u32);
                    collector.reset();
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(level) = in_heading.take() {
                        let spans = collector.finish();
                        if !spans.is_empty() {
                            blocks.push(MarkdownBlock::Heading { level, spans });
                        }
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    emit_parent_list_item(&mut list_item_stack, &mut blocks);
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(l) => l.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    in_code_block = Some(lang);
                    code_block_text.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(lang) = in_code_block.take() {
                        blocks.push(MarkdownBlock::CodeBlock {
                            lang,
                            code: std::mem::take(&mut code_block_text),
                        });
                    }
                }
                Event::Start(Tag::List(_)) => {
                    list_depth += 1;
                    if let Some(parent) = list_item_stack.last_mut() {
                        parent.has_children = true;
                    }
                    emit_parent_list_item(&mut list_item_stack, &mut blocks);
                }
                Event::End(TagEnd::List(_)) => {
                    list_depth = list_depth.saturating_sub(1);
                }
                Event::Start(Tag::Item) => {
                    let depth = list_depth.saturating_sub(1);
                    list_item_stack.push(ListItemState {
                        depth,
                        collector: InlineCollector::default(),
                        task_status: None,
                        emitted: false,
                        has_children: false,
                    });
                }
                Event::TaskListMarker(checked) => {
                    if let Some(item) = list_item_stack.last_mut() {
                        item.task_status = Some(checked);
                    }
                }
                Event::End(TagEnd::Item) => {
                    if let Some(mut item) = list_item_stack.pop() {
                        let spans = item.collector.finish();
                        if !spans.is_empty() {
                            blocks.push(MarkdownBlock::ListItem {
                                depth: item.depth,
                                spans,
                                task_status: if item.emitted { None } else { item.task_status },
                            });
                        } else if !item.emitted && !item.has_children {
                            blocks.push(MarkdownBlock::ListItem {
                                depth: item.depth,
                                spans: Vec::new(),
                                task_status: item.task_status,
                            });
                        }
                    }
                }
                Event::Start(Tag::BlockQuote(alert_opt)) => {
                    emit_parent_list_item(&mut list_item_stack, &mut blocks);
                    if let Some(kind) = alert_opt {
                        in_alert = Some(kind.into());
                    } else {
                        in_blockquote = true;
                    }
                    collector.reset();
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    if let Some(kind) = in_alert.take() {
                        let spans = collector.finish();
                        blocks.push(MarkdownBlock::Alert { kind, spans });
                    } else if in_blockquote {
                        let spans = collector.finish();
                        blocks.push(MarkdownBlock::BlockQuote(spans));
                        in_blockquote = false;
                    }
                }
                Event::Start(Tag::FootnoteDefinition(label)) => {
                    emit_parent_list_item(&mut list_item_stack, &mut blocks);
                    in_footnote = Some(label.to_string());
                    collector.reset();
                }
                Event::End(TagEnd::FootnoteDefinition) => {
                    if let Some(label) = in_footnote.take() {
                        let spans = collector.finish();
                        footnotes.push(MarkdownBlock::Footnote { label, spans });
                    }
                }
                Event::Start(Tag::Paragraph) => {
                    if in_heading.is_none()
                        && in_code_block.is_none()
                        && list_item_stack.is_empty()
                        && !in_blockquote
                        && in_alert.is_none()
                        && in_footnote.is_none()
                        && !in_table
                    {
                        in_paragraph = true;
                        collector.reset();
                    } else if in_blockquote || in_alert.is_some() || in_footnote.is_some() {
                        if !collector.spans.is_empty() || !collector.current_text.is_empty() {
                            collector.push_text("\n\n");
                        }
                    } else if let Some(item) = list_item_stack.last_mut() {
                        if !item.collector.spans.is_empty() || !item.collector.current_text.is_empty() {
                            item.collector.push_text("\n\n");
                        }
                    }
                }
                Event::End(TagEnd::Paragraph) => {
                    if in_paragraph {
                        in_paragraph = false;
                        let spans = collector.finish();
                        if !spans.is_empty() {
                            blocks.push(MarkdownBlock::Paragraph(spans));
                        }
                    }
                }
                Event::Start(Tag::HtmlBlock) => {
                    in_html_block = true;
                    html_block_content.clear();
                }
                Event::End(TagEnd::HtmlBlock) => {
                    in_html_block = false;
                    let mut spans = parse_html_fragment(&html_block_content);
                    trim_spans(&mut spans);
                    if !spans.is_empty()
                        && spans.iter().any(|s| !s.plain_text().trim().is_empty())
                    {
                        blocks.push(MarkdownBlock::Paragraph(spans));
                    }
                    html_block_content.clear();
                }
                Event::Start(Tag::Table(aligns)) => {
                    emit_parent_list_item(&mut list_item_stack, &mut blocks);
                    in_table = true;
                    table_alignments = aligns.into_iter().map(ColumnAlignment::from).collect();
                    table_headers.clear();
                    table_rows.clear();
                }
                Event::End(TagEnd::Table) => {
                    if in_table {
                        blocks.push(MarkdownBlock::Table(TableBlock {
                            headers: std::mem::take(&mut table_headers),
                            alignments: std::mem::take(&mut table_alignments),
                            rows: std::mem::take(&mut table_rows),
                        }));
                        in_table = false;
                    }
                }
                Event::Start(Tag::TableHead) => {
                    in_table_head = true;
                }
                Event::End(TagEnd::TableHead) => {
                    in_table_head = false;
                }
                Event::Start(Tag::TableRow) => {
                    table_current_row.clear();
                }
                Event::End(TagEnd::TableRow) => {
                    if !in_table_head {
                        table_rows.push(std::mem::take(&mut table_current_row));
                    }
                }
                Event::Start(Tag::TableCell) => {
                    collector.reset();
                }
                Event::End(TagEnd::TableCell) => {
                    let spans = collector.finish();
                    if in_table_head {
                        table_headers.push(spans);
                    } else {
                        table_current_row.push(spans);
                    }
                }
                Event::Start(Tag::Strong) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .start_strong();
                }
                Event::End(TagEnd::Strong) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .end_strong();
                }
                Event::Start(Tag::Emphasis) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .start_emphasis();
                }
                Event::End(TagEnd::Emphasis) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .end_emphasis();
                }
                Event::Start(Tag::Strikethrough) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .start_strike();
                }
                Event::End(TagEnd::Strikethrough) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .end_strike();
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .start_link(dest_url.to_string());
                }
                Event::End(TagEnd::Link) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .end_link();
                }
                Event::Start(Tag::Image { dest_url, .. }) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .start_image(dest_url.to_string());
                }
                Event::End(TagEnd::Image) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .end_image();
                }
                Event::Text(t) => {
                    if in_code_block.is_some() {
                        code_block_text.push_str(&t);
                    } else {
                        active_collector(
                            &mut list_item_stack,
                            &mut collector,
                            in_table,
                            in_blockquote,
                            in_alert.is_some(),
                            in_footnote.is_some(),
                        )
                        .push_text(&t);
                    }
                }
                Event::Code(c) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .push_code(&c);
                }
                Event::Html(h) => {
                    if in_html_block {
                        html_block_content.push_str(&h);
                    } else if in_heading.is_some()
                        || !list_item_stack.is_empty()
                        || in_blockquote
                        || in_alert.is_some()
                        || in_table
                        || in_footnote.is_some()
                        || in_paragraph
                    {
                        active_collector(
                            &mut list_item_stack,
                            &mut collector,
                            in_table,
                            in_blockquote,
                            in_alert.is_some(),
                            in_footnote.is_some(),
                        )
                        .push_html(&h);
                    } else {
                        let mut spans = parse_html_fragment(&h);
                        trim_spans(&mut spans);
                        if !spans.is_empty()
                            && spans.iter().any(|s| !s.plain_text().trim().is_empty())
                        {
                            blocks.push(MarkdownBlock::Paragraph(spans));
                        }
                    }
                }
                Event::InlineHtml(h) => {
                    if in_html_block {
                        html_block_content.push_str(&h);
                    } else {
                        active_collector(
                            &mut list_item_stack,
                            &mut collector,
                            in_table,
                            in_blockquote,
                            in_alert.is_some(),
                            in_footnote.is_some(),
                        )
                        .push_html(&h);
                    }
                }
                Event::FootnoteReference(label) => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .push_footnote_ref(&label);
                }
                Event::Rule => {
                    emit_parent_list_item(&mut list_item_stack, &mut blocks);
                    blocks.push(MarkdownBlock::Rule);
                }
                Event::SoftBreak => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .push_text(" ");
                }
                Event::HardBreak => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        in_blockquote,
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .push_text("\n");
                }
                _ => {}
            }
        }

        if !footnotes.is_empty() {
            if !blocks.is_empty() && !matches!(blocks.last(), Some(MarkdownBlock::Rule)) {
                blocks.push(MarkdownBlock::Rule);
            }
            blocks.extend(footnotes);
        }

        Self { blocks }
    }

    pub fn parse_gfm(source: &str) -> Self {
        Self::parse(source, MarkdownSpec::Gfm)
    }

    pub fn parse_commonmark(source: &str) -> Self {
        Self::parse(source, MarkdownSpec::CommonMark)
    }
}
