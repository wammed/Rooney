use crate::config::MarkdownSpec;
use pulldown_cmark::{
    Alignment as CmarkAlignment, BlockQuoteKind, Event, Options, Parser, Tag, TagEnd,
};

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
pub enum InlineSpan {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    Strikethrough(String),
    Link { text: String, url: String },
    ImageFallback { alt: String, url: String },
}

impl InlineSpan {
    pub fn plain_text(&self) -> &str {
        match self {
            InlineSpan::Text(s)
            | InlineSpan::Bold(s)
            | InlineSpan::Italic(s)
            | InlineSpan::Code(s)
            | InlineSpan::Strikethrough(s) => s.as_str(),
            InlineSpan::Link { text, .. } => text.as_str(),
            InlineSpan::ImageFallback { alt, .. } => alt.as_str(),
        }
    }
}

pub fn spans_plain_text(spans: &[InlineSpan]) -> String {
    let mut out = String::new();
    for span in spans {
        out.push_str(span.plain_text());
    }
    out
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

#[derive(Debug, Clone, Default)]
pub struct MarkdownDocument {
    pub blocks: Vec<MarkdownBlock>,
}

pub fn parse_html_fragment(html: &str) -> Vec<InlineSpan> {
    let mut spans = Vec::new();
    let mut i = 0;
    let bytes = html.as_bytes();
    let len = bytes.len();
    let mut text_buf = String::new();

    while i < len {
        if bytes[i] == b'<' {
            if let Some(close_pos) = html[i..].find('>') {
                let tag_end = i + close_pos;
                let tag_content = html[i + 1..tag_end].trim();
                let lower = tag_content.to_ascii_lowercase();

                if lower == "br" || lower.starts_with("br/") || lower.starts_with("br ") {
                    text_buf.push('\n');
                } else if lower.starts_with("img")
                    && (lower.len() == 3
                        || lower[3..].starts_with(char::is_whitespace)
                        || lower[3..].starts_with('/'))
                {
                    let alt = extract_html_attr(tag_content, "alt").unwrap_or_default();
                    let src = extract_html_attr(tag_content, "src").unwrap_or_default();
                    if !text_buf.is_empty() {
                        spans.push(InlineSpan::Text(std::mem::take(&mut text_buf)));
                    }
                    spans.push(InlineSpan::ImageFallback { alt, url: src });
                }
                i = tag_end + 1;
                continue;
            }
        }

        let ch = html[i..].chars().next().unwrap();
        text_buf.push(ch);
        i += ch.len_utf8();
    }

    if !text_buf.is_empty() {
        spans.push(InlineSpan::Text(text_buf));
    }

    spans
}

fn extract_html_attr(tag: &str, attr_name: &str) -> Option<String> {
    let lower_tag = tag.to_ascii_lowercase();
    let search_key = format!("{}=", attr_name);
    let pos = lower_tag.find(&search_key)?;
    let after = tag[pos + search_key.len()..].trim_start();
    if after.starts_with('"') {
        let end = after[1..].find('"')?;
        Some(after[1..1 + end].to_string())
    } else if after.starts_with('\'') {
        let end = after[1..].find('\'')?;
        Some(after[1..1 + end].to_string())
    } else {
        let end = after
            .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
            .unwrap_or(after.len());
        Some(after[..end].to_string())
    }
}

#[derive(Default)]
struct InlineCollector {
    spans: Vec<InlineSpan>,
    plain: String,
    strong: String,
    emphasis: String,
    strike: String,
    link: Option<(String, String)>,
    image: Option<(String, String)>,
    strong_depth: usize,
    emphasis_depth: usize,
    strike_depth: usize,
}

impl InlineCollector {
    fn reset(&mut self) {
        self.spans.clear();
        self.plain.clear();
        self.strong.clear();
        self.emphasis.clear();
        self.strike.clear();
        self.link = None;
        self.image = None;
        self.strong_depth = 0;
        self.emphasis_depth = 0;
        self.strike_depth = 0;
    }

    fn push_text(&mut self, text: &str) {
        if let Some((_, alt)) = &mut self.image {
            alt.push_str(text);
        } else if let Some((_, link_text)) = &mut self.link {
            link_text.push_str(text);
        } else if self.strong_depth > 0 {
            self.strong.push_str(text);
        } else if self.emphasis_depth > 0 {
            self.emphasis.push_str(text);
        } else if self.strike_depth > 0 {
            self.strike.push_str(text);
        } else {
            self.plain.push_str(text);
        }
    }

    fn push_code(&mut self, code: &str) {
        if let Some((_, alt)) = &mut self.image {
            alt.push_str(code);
        } else if let Some((_, link_text)) = &mut self.link {
            link_text.push_str(code);
        } else if self.strong_depth > 0 {
            self.flush_strong();
            self.spans.push(InlineSpan::Code(code.to_string()));
        } else if self.emphasis_depth > 0 {
            self.flush_emphasis();
            self.spans.push(InlineSpan::Code(code.to_string()));
        } else if self.strike_depth > 0 {
            self.flush_strike();
            self.spans.push(InlineSpan::Code(code.to_string()));
        } else {
            self.flush_plain();
            self.spans.push(InlineSpan::Code(code.to_string()));
        }
    }

    fn push_html(&mut self, html: &str) {
        let fragments = parse_html_fragment(html);
        for f in fragments {
            match f {
                InlineSpan::Text(s) => self.push_text(&s),
                InlineSpan::ImageFallback { alt, url } => {
                    if self.image.is_none()
                        && self.link.is_none()
                        && self.strong_depth == 0
                        && self.emphasis_depth == 0
                        && self.strike_depth == 0
                    {
                        self.flush_plain();
                        self.spans.push(InlineSpan::ImageFallback { alt, url });
                    } else {
                        let label = if alt.is_empty() {
                            format!(" [画像: {}] ", url)
                        } else {
                            format!(" [画像: {}] ", alt)
                        };
                        self.push_text(&label);
                    }
                }
                _ => {}
            }
        }
    }

    fn push_footnote_ref(&mut self, label: &str) {
        let ref_text = format!("[^{}]", label);
        if self.image.is_some()
            || self.link.is_some()
            || self.strong_depth > 0
            || self.emphasis_depth > 0
            || self.strike_depth > 0
        {
            self.push_text(&ref_text);
        } else {
            self.flush_plain();
            self.spans.push(InlineSpan::Link {
                text: ref_text,
                url: format!("#fn-{}", label),
            });
        }
    }

    fn start_strong(&mut self) {
        self.flush_plain();
        self.strong_depth += 1;
    }

    fn end_strong(&mut self) {
        self.strong_depth = self.strong_depth.saturating_sub(1);
        if self.strong_depth == 0 {
            self.flush_strong();
        }
    }

    fn start_emphasis(&mut self) {
        self.flush_plain();
        self.emphasis_depth += 1;
    }

    fn end_emphasis(&mut self) {
        self.emphasis_depth = self.emphasis_depth.saturating_sub(1);
        if self.emphasis_depth == 0 {
            self.flush_emphasis();
        }
    }

    fn start_strike(&mut self) {
        self.flush_plain();
        self.strike_depth += 1;
    }

    fn end_strike(&mut self) {
        self.strike_depth = self.strike_depth.saturating_sub(1);
        if self.strike_depth == 0 {
            self.flush_strike();
        }
    }

    fn start_link(&mut self, url: String) {
        self.flush_plain();
        self.link = Some((url, String::new()));
    }

    fn end_link(&mut self) {
        if let Some((url, text)) = self.link.take() {
            self.spans.push(InlineSpan::Link { text, url });
        }
    }

    fn start_image(&mut self, url: String) {
        self.flush_plain();
        self.image = Some((url, String::new()));
    }

    fn end_image(&mut self) {
        if let Some((url, alt)) = self.image.take() {
            self.spans.push(InlineSpan::ImageFallback { alt, url });
        }
    }

    fn flush_plain(&mut self) {
        if !self.plain.is_empty() {
            let s = std::mem::take(&mut self.plain);
            self.spans.push(InlineSpan::Text(s));
        }
    }

    fn flush_strong(&mut self) {
        if !self.strong.is_empty() {
            let s = std::mem::take(&mut self.strong);
            if let Some((_, alt)) = &mut self.image {
                alt.push_str(&s);
            } else if let Some((_, link_text)) = &mut self.link {
                link_text.push_str(&s);
            } else {
                self.spans.push(InlineSpan::Bold(s));
            }
        }
    }

    fn flush_emphasis(&mut self) {
        if !self.emphasis.is_empty() {
            let s = std::mem::take(&mut self.emphasis);
            if let Some((_, alt)) = &mut self.image {
                alt.push_str(&s);
            } else if let Some((_, link_text)) = &mut self.link {
                link_text.push_str(&s);
            } else {
                self.spans.push(InlineSpan::Italic(s));
            }
        }
    }

    fn flush_strike(&mut self) {
        if !self.strike.is_empty() {
            let s = std::mem::take(&mut self.strike);
            if let Some((_, alt)) = &mut self.image {
                alt.push_str(&s);
            } else if let Some((_, link_text)) = &mut self.link {
                link_text.push_str(&s);
            } else {
                self.spans.push(InlineSpan::Strikethrough(s));
            }
        }
    }

    fn finish(&mut self) -> Vec<InlineSpan> {
        self.flush_plain();
        self.flush_strong();
        self.flush_emphasis();
        self.flush_strike();
        if let Some((url, text)) = self.link.take() {
            self.spans.push(InlineSpan::Link { text, url });
        }
        if let Some((url, alt)) = self.image.take() {
            self.spans.push(InlineSpan::ImageFallback { alt, url });
        }
        self.strong_depth = 0;
        self.emphasis_depth = 0;
        self.strike_depth = 0;

        trim_spans(&mut self.spans);
        std::mem::take(&mut self.spans)
    }
}

fn trim_spans(spans: &mut Vec<InlineSpan>) {
    if let Some(first) = spans.first_mut() {
        if let InlineSpan::Text(s) = first {
            *s = s
                .trim_start_matches(|c: char| c == ' ' || c == '\t' || c == '\r' || c == '\n')
                .to_string();
        }
    }
    if let Some(last) = spans.last_mut() {
        if let InlineSpan::Text(s) = last {
            *s = s
                .trim_end_matches(|c: char| c == ' ' || c == '\t' || c == '\r' || c == '\n')
                .to_string();
        }
    }
    spans.retain(|s| match s {
        InlineSpan::Text(t) => !t.is_empty(),
        _ => true,
    });
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
        let mut in_heading = None;
        let mut in_code_block = None;
        let mut code_block_text = String::new();
        let mut in_list_item = false;
        let mut in_task_status = None;
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
                }
                Event::End(TagEnd::List(_)) => {
                    list_depth = list_depth.saturating_sub(1);
                }
                Event::Start(Tag::Item) => {
                    in_list_item = true;
                    in_task_status = None;
                    collector.reset();
                }
                Event::TaskListMarker(checked) => {
                    in_task_status = Some(checked);
                }
                Event::End(TagEnd::Item) => {
                    if in_list_item {
                        let spans = collector.finish();
                        blocks.push(MarkdownBlock::ListItem {
                            depth: list_depth.saturating_sub(1),
                            spans,
                            task_status: in_task_status,
                        });
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
                        && !in_list_item
                        && !in_blockquote
                        && in_alert.is_none()
                        && in_footnote.is_none()
                        && !in_table
                    {
                        in_paragraph = true;
                        collector.reset();
                    } else if in_blockquote || in_alert.is_some() || in_footnote.is_some() {
                        if !collector.spans.is_empty() || !collector.plain.is_empty() {
                            collector.push_text("\n\n");
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
                    collector.start_strong();
                }
                Event::End(TagEnd::Strong) => {
                    collector.end_strong();
                }
                Event::Start(Tag::Emphasis) => {
                    collector.start_emphasis();
                }
                Event::End(TagEnd::Emphasis) => {
                    collector.end_emphasis();
                }
                Event::Start(Tag::Strikethrough) => {
                    collector.start_strike();
                }
                Event::End(TagEnd::Strikethrough) => {
                    collector.end_strike();
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    collector.start_link(dest_url.to_string());
                }
                Event::End(TagEnd::Link) => {
                    collector.end_link();
                }
                Event::Start(Tag::Image { dest_url, .. }) => {
                    collector.start_image(dest_url.to_string());
                }
                Event::End(TagEnd::Image) => {
                    collector.end_image();
                }
                Event::Text(t) => {
                    if in_code_block.is_some() {
                        code_block_text.push_str(&t);
                    } else {
                        collector.push_text(&t);
                    }
                }
                Event::Code(c) => {
                    collector.push_code(&c);
                }
                Event::Html(h) => {
                    if in_html_block {
                        html_block_content.push_str(&h);
                    } else if in_heading.is_some()
                        || in_list_item
                        || in_blockquote
                        || in_alert.is_some()
                        || in_table
                        || in_footnote.is_some()
                        || in_paragraph
                    {
                        collector.push_html(&h);
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
                        collector.push_html(&h);
                    }
                }
                Event::FootnoteReference(label) => {
                    collector.push_footnote_ref(&label);
                }
                Event::Rule => {
                    blocks.push(MarkdownBlock::Rule);
                }
                Event::SoftBreak => {
                    collector.push_text(" ");
                }
                Event::HardBreak => {
                    collector.push_text("\n");
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

