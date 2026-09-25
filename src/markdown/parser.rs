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
        children: Vec<MarkdownBlock>,
    },
    BlockQuote(Vec<MarkdownBlock>),
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
            | MarkdownBlock::Alert { spans, .. }
            | MarkdownBlock::Footnote { spans, .. } => spans_plain_text(spans),
            MarkdownBlock::ListItem {
                spans, children, ..
            } => {
                let mut text = spans_plain_text(spans);
                for child in children {
                    let child_text = child.plain_text();
                    if !child_text.is_empty() {
                        if !text.is_empty() {
                            text.push('\n');
                        }
                        text.push_str(&child_text);
                    }
                }
                text
            }
            MarkdownBlock::BlockQuote(blocks) => blocks
                .iter()
                .map(|b| b.plain_text())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
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
    pub children: Vec<MarkdownBlock>,
    pub block_index: Option<usize>,
    pub in_blockquote_depth: usize,
}

#[inline]
pub fn active_collector<'a>(
    list_stack: &'a mut [ListItemState],
    global: &'a mut InlineCollector,
    in_table: bool,
    _in_blockquote: bool,
    _in_alert: bool,
    in_footnote: bool,
) -> &'a mut InlineCollector {
    if in_table || in_footnote {
        global
    } else if let Some(item) = list_stack.last_mut() {
        &mut item.collector
    } else {
        global
    }
}

#[inline]
pub fn push_to_current_container(
    blocks: &mut Vec<MarkdownBlock>,
    blockquote_stack: &mut [Vec<MarkdownBlock>],
    block: MarkdownBlock,
) -> usize {
    if let Some(bq) = blockquote_stack.last_mut() {
        bq.push(block);
        bq.len() - 1
    } else {
        blocks.push(block);
        blocks.len() - 1
    }
}

#[inline]
pub fn emit_parent_list_item(list_stack: &mut [ListItemState], blocks: &mut Vec<MarkdownBlock>) {
    emit_parent_list_item_in_scope(list_stack, blocks, &mut []);
}

#[inline]
pub fn emit_parent_list_item_in_scope(
    list_stack: &mut [ListItemState],
    blocks: &mut Vec<MarkdownBlock>,
    blockquote_stack: &mut [Vec<MarkdownBlock>],
) {
    if let Some(parent) = list_stack.last_mut() {
        if !parent.emitted {
            let spans = parent.collector.finish();
            if !spans.is_empty() || parent.task_status.is_some() || !parent.children.is_empty() {
                let children = std::mem::take(&mut parent.children);
                let idx = push_to_current_container(
                    blocks,
                    blockquote_stack,
                    MarkdownBlock::ListItem {
                        depth: parent.depth,
                        spans,
                        task_status: parent.task_status,
                        children,
                    },
                );
                parent.block_index = Some(idx);
                parent.in_blockquote_depth = blockquote_stack.len();
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
        let mut blockquote_stack: Vec<Vec<MarkdownBlock>> = Vec::new();
        let mut footnotes = Vec::new();

        let mut collector = InlineCollector::default();
        let mut list_item_stack: Vec<ListItemState> = Vec::new();
        let mut in_heading = None;
        let mut in_code_block = None;
        let mut code_block_text = String::new();
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
                            push_to_current_container(
                                &mut blocks,
                                &mut blockquote_stack,
                                MarkdownBlock::Heading { level, spans },
                            );
                        }
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(l) => l.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    in_code_block = Some(lang);
                    code_block_text.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(lang) = in_code_block.take() {
                        let code_block = MarkdownBlock::CodeBlock {
                            lang,
                            code: std::mem::take(&mut code_block_text),
                        };
                        if let Some(item) = list_item_stack.last_mut() {
                            item.children.push(code_block);
                        } else {
                            push_to_current_container(
                                &mut blocks,
                                &mut blockquote_stack,
                                code_block,
                            );
                        }
                    }
                }
                Event::Start(Tag::List(_)) => {
                    list_depth += 1;
                    if let Some(parent) = list_item_stack.last_mut() {
                        parent.has_children = true;
                    }
                    emit_parent_list_item_in_scope(
                        &mut list_item_stack,
                        &mut blocks,
                        &mut blockquote_stack,
                    );
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
                        children: Vec::new(),
                        block_index: None,
                        in_blockquote_depth: blockquote_stack.len(),
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
                        if item.emitted {
                            let target = if item.in_blockquote_depth == blockquote_stack.len() {
                                if let Some(bq) = blockquote_stack.last_mut() {
                                    item.block_index.and_then(|idx| bq.get_mut(idx))
                                } else {
                                    item.block_index.and_then(|idx| blocks.get_mut(idx))
                                }
                            } else {
                                None
                            };

                            if let Some(MarkdownBlock::ListItem {
                                spans: target_spans,
                                children: target_children,
                                ..
                            }) = target
                            {
                                if !spans.is_empty() {
                                    if !target_spans.is_empty() {
                                        target_spans.push(InlineSpan::Text("\n".to_string()));
                                    }
                                    target_spans.extend(spans);
                                }
                                target_children.extend(item.children);
                            }
                        } else {
                            if !spans.is_empty()
                                || item.task_status.is_some()
                                || !item.children.is_empty()
                            {
                                push_to_current_container(
                                    &mut blocks,
                                    &mut blockquote_stack,
                                    MarkdownBlock::ListItem {
                                        depth: item.depth,
                                        spans,
                                        task_status: item.task_status,
                                        children: item.children,
                                    },
                                );
                            } else if !item.has_children {
                                push_to_current_container(
                                    &mut blocks,
                                    &mut blockquote_stack,
                                    MarkdownBlock::ListItem {
                                        depth: item.depth,
                                        spans: Vec::new(),
                                        task_status: item.task_status,
                                        children: item.children,
                                    },
                                );
                            }
                        }
                    }
                }
                Event::Start(Tag::BlockQuote(alert_opt)) => {
                    if let Some(kind) = alert_opt {
                        in_alert = Some(kind.into());
                        collector.reset();
                    } else {
                        blockquote_stack.push(Vec::new());
                    }
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    if let Some(kind) = in_alert.take() {
                        let spans = collector.finish();
                        push_to_current_container(
                            &mut blocks,
                            &mut blockquote_stack,
                            MarkdownBlock::Alert { kind, spans },
                        );
                    } else if let Some(inner_blocks) = blockquote_stack.pop() {
                        push_to_current_container(
                            &mut blocks,
                            &mut blockquote_stack,
                            MarkdownBlock::BlockQuote(inner_blocks),
                        );
                    }
                }
                Event::Start(Tag::FootnoteDefinition(label)) => {
                    emit_parent_list_item_in_scope(
                        &mut list_item_stack,
                        &mut blocks,
                        &mut blockquote_stack,
                    );
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
                        && in_alert.is_none()
                        && in_footnote.is_none()
                        && !in_table
                    {
                        in_paragraph = true;
                        collector.reset();
                    } else if in_alert.is_some() || in_footnote.is_some() {
                        if !collector.spans.is_empty() || !collector.current_text.is_empty() {
                            collector.push_text("\n\n");
                        }
                    } else if let Some(item) = list_item_stack.last_mut() {
                        if !item.collector.spans.is_empty()
                            || !item.collector.current_text.is_empty()
                        {
                            item.collector.push_text("\n\n");
                        }
                    }
                }
                Event::End(TagEnd::Paragraph) => {
                    if in_paragraph {
                        in_paragraph = false;
                        let spans = collector.finish();
                        if !spans.is_empty() {
                            push_to_current_container(
                                &mut blocks,
                                &mut blockquote_stack,
                                MarkdownBlock::Paragraph(spans),
                            );
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
                    if !spans.is_empty() && spans.iter().any(|s| !s.plain_text().trim().is_empty())
                    {
                        push_to_current_container(
                            &mut blocks,
                            &mut blockquote_stack,
                            MarkdownBlock::Paragraph(spans),
                        );
                    }
                    html_block_content.clear();
                }
                Event::Start(Tag::Table(aligns)) => {
                    emit_parent_list_item_in_scope(
                        &mut list_item_stack,
                        &mut blocks,
                        &mut blockquote_stack,
                    );
                    in_table = true;
                    table_alignments = aligns.into_iter().map(ColumnAlignment::from).collect();
                    table_headers.clear();
                    table_rows.clear();
                }
                Event::End(TagEnd::Table) => {
                    if in_table {
                        let table_block = MarkdownBlock::Table(TableBlock {
                            headers: std::mem::take(&mut table_headers),
                            alignments: std::mem::take(&mut table_alignments),
                            rows: std::mem::take(&mut table_rows),
                        });
                        if let Some(item) = list_item_stack.last_mut() {
                            item.children.push(table_block);
                        } else {
                            push_to_current_container(
                                &mut blocks,
                                &mut blockquote_stack,
                                table_block,
                            );
                        }
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
                        !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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
                            !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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
                        || !blockquote_stack.is_empty()
                        || in_alert.is_some()
                        || in_table
                        || in_footnote.is_some()
                        || in_paragraph
                    {
                        active_collector(
                            &mut list_item_stack,
                            &mut collector,
                            in_table,
                            !blockquote_stack.is_empty(),
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
                            push_to_current_container(
                                &mut blocks,
                                &mut blockquote_stack,
                                MarkdownBlock::Paragraph(spans),
                            );
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
                            !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
                        in_alert.is_some(),
                        in_footnote.is_some(),
                    )
                    .push_footnote_ref(&label);
                }
                Event::Rule => {
                    emit_parent_list_item_in_scope(
                        &mut list_item_stack,
                        &mut blocks,
                        &mut blockquote_stack,
                    );
                    push_to_current_container(
                        &mut blocks,
                        &mut blockquote_stack,
                        MarkdownBlock::Rule,
                    );
                }
                Event::SoftBreak => {
                    active_collector(
                        &mut list_item_stack,
                        &mut collector,
                        in_table,
                        !blockquote_stack.is_empty(),
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
                        !blockquote_stack.is_empty(),
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

        let doc = Self { blocks };
        #[cfg(any(debug_assertions, test))]
        doc.validate_list_depth_invariants()
            .expect("List depth invariant violated");
        doc
    }

    /// Verifies depth invariants of the depth-based List AST:
    /// 1. Top-level list items start at depth 0.
    /// 2. Subsequent list items can increase depth by at most 1 (d <= prev + 1).
    /// 3. Nested non-list blocks (CodeBlock, etc.) reside in `children`.
    /// 4. `ListItem` itself is never recursively nested in `children`; list hierarchy is represented via `depth + document order`.
    pub fn validate_list_depth_invariants(&self) -> Result<(), String> {
        fn check_blocks(blocks: &[MarkdownBlock]) -> Result<(), String> {
            let mut prev_list_depth: Option<usize> = None;
            for block in blocks {
                match block {
                    MarkdownBlock::ListItem {
                        depth, children, ..
                    } => {
                        let d = *depth;
                        if let Some(prev) = prev_list_depth {
                            if d > prev + 1 {
                                return Err(format!(
                                    "List depth jump exceeds 1: prev={}, curr={}",
                                    prev, d
                                ));
                            }
                        } else if d != 0 {
                            return Err(format!(
                                "Initial list item in context must start at depth 0, got {}",
                                d
                            ));
                        }
                        prev_list_depth = Some(d);

                        for child in children {
                            if matches!(child, MarkdownBlock::ListItem { .. }) {
                                return Err(
                                    "ListItem must not be recursively nested in children; list hierarchy is depth-based"
                                        .to_string(),
                                );
                            }
                        }
                        check_blocks(children)?;
                    }
                    MarkdownBlock::BlockQuote(inner) => {
                        prev_list_depth = None;
                        check_blocks(inner)?;
                    }
                    _ => {
                        prev_list_depth = None;
                    }
                }
            }
            Ok(())
        }

        check_blocks(&self.blocks)
    }

    pub fn parse_gfm(source: &str) -> Self {
        Self::parse(source, MarkdownSpec::Gfm)
    }

    pub fn parse_commonmark(source: &str) -> Self {
        Self::parse(source, MarkdownSpec::CommonMark)
    }
}
