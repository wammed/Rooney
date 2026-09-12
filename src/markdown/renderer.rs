use crate::config::MarkdownSpec;
use pulldown_cmark::{Alignment as CmarkAlignment, BlockQuoteKind, Event, Options, Parser, Tag, TagEnd};

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
    pub headers: Vec<String>,
    pub alignments: Vec<ColumnAlignment>,
    pub rows: Vec<Vec<String>>,
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
        text: String,
    },
    Paragraph(String),
    CodeBlock {
        lang: String,
        code: String,
    },
    ListItem {
        depth: usize,
        text: String,
        task_status: Option<bool>,
    },
    BlockQuote(String),
    Alert {
        kind: AlertKind,
        text: String,
    },
    Table(TableBlock),
    Rule,
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

        let mut current_text = String::new();
        let mut in_heading = None;
        let mut in_code_block = None;
        let mut in_list_item = false;
        let mut in_task_status = None;
        let mut in_blockquote = false;
        let mut in_alert = None;
        let mut in_strikethrough = false;
        let mut in_link: Option<String> = None;
        let mut list_depth: usize = 0;

        // Table state
        let mut in_table = false;
        let mut in_table_head = false;
        let mut table_alignments: Vec<ColumnAlignment> = Vec::new();
        let mut table_headers: Vec<String> = Vec::new();
        let mut table_rows: Vec<Vec<String>> = Vec::new();
        let mut table_current_row: Vec<String> = Vec::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    in_heading = Some(level as u32);
                    current_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(level) = in_heading.take() {
                        blocks.push(MarkdownBlock::Heading {
                            level,
                            text: current_text.trim().to_string(),
                        });
                        current_text.clear();
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(l) => l.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    in_code_block = Some(lang);
                    current_text.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(lang) = in_code_block.take() {
                        blocks.push(MarkdownBlock::CodeBlock {
                            lang,
                            code: current_text.clone(),
                        });
                        current_text.clear();
                    }
                }
                Event::Start(Tag::List(_)) => {
                    list_depth += 1;
                }
                Event::End(TagEnd::List(_)) => {
                    list_depth = list_depth.saturating_sub(1);
                }
                Event::Start(Tag::Item) => {
                    in_list_item = true;
                    in_task_status = None;
                    current_text.clear();
                }
                Event::TaskListMarker(checked) => {
                    in_task_status = Some(checked);
                }
                Event::End(TagEnd::Item) => {
                    if in_list_item {
                        blocks.push(MarkdownBlock::ListItem {
                            depth: list_depth.saturating_sub(1),
                            text: current_text.trim().to_string(),
                            task_status: in_task_status,
                        });
                        current_text.clear();
                        in_list_item = false;
                        in_task_status = None;
                    }
                }
                Event::Start(Tag::BlockQuote(alert_opt)) => {
                    if let Some(kind) = alert_opt {
                        in_alert = Some(kind.into());
                    } else {
                        in_blockquote = true;
                    }
                    current_text.clear();
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    if let Some(kind) = in_alert.take() {
                        blocks.push(MarkdownBlock::Alert {
                            kind,
                            text: current_text.trim().to_string(),
                        });
                        current_text.clear();
                    } else if in_blockquote {
                        blocks.push(MarkdownBlock::BlockQuote(current_text.trim().to_string()));
                        current_text.clear();
                        in_blockquote = false;
                    }
                }
                Event::Start(Tag::Paragraph) => {
                    current_text.clear();
                }
                Event::End(TagEnd::Paragraph) => {
                    let text = current_text.trim().to_string();
                    if !text.is_empty()
                        && in_heading.is_none()
                        && in_code_block.is_none()
                        && !in_list_item
                        && !in_blockquote
                        && in_alert.is_none()
                        && !in_table
                    {
                        blocks.push(MarkdownBlock::Paragraph(text));
                        current_text.clear();
                    }
                }
                Event::Start(Tag::Table(aligns)) => {
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
                    current_text.clear();
                }
                Event::End(TagEnd::TableCell) => {
                    let cell_str = current_text.trim().to_string();
                    current_text.clear();
                    if in_table_head {
                        table_headers.push(cell_str);
                    } else {
                        table_current_row.push(cell_str);
                    }
                }
                Event::Start(Tag::Strikethrough) => {
                    in_strikethrough = true;
                }
                Event::End(TagEnd::Strikethrough) => {
                    in_strikethrough = false;
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    in_link = Some(dest_url.to_string());
                }
                Event::End(TagEnd::Link) => {
                    in_link = None;
                }
                Event::Text(t) => {
                    if in_strikethrough {
                        for c in t.chars() {
                            current_text.push(c);
                            current_text.push('\u{0336}');
                        }
                    } else if let Some(ref url) = in_link {
                        if t.as_ref() == url.as_str() {
                            current_text.push_str("󰌹 ");
                            current_text.push_str(&t);
                        } else {
                            current_text.push_str(&t);
                        }
                    } else {
                        current_text.push_str(&t);
                    }
                }
                Event::Code(c) => {
                    current_text.push('`');
                    current_text.push_str(&c);
                    current_text.push('`');
                }
                Event::Rule => {
                    blocks.push(MarkdownBlock::Rule);
                }
                Event::SoftBreak => {
                    // breaks: false maintains CommonMark compliance (single newline -> space)
                    current_text.push(' ');
                }
                Event::HardBreak => {
                    current_text.push('\n');
                }
                _ => {}
            }
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
