use super::html::parse_html_fragment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub strike: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineSpan {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    Strikethrough(String),
    Styled { text: String, style: InlineStyle },
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
            | InlineSpan::Strikethrough(s)
            | InlineSpan::Styled { text: s, .. } => s.as_str(),
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

pub fn find_next_autolink(text: &str) -> Option<(usize, usize)> {
    let mut search_from = 0;
    while let Some(rel_pos) = text[search_from..]
        .find("http://")
        .or_else(|| text[search_from..].find("https://"))
    {
        let start = search_from + rel_pos;
        if start > 0 {
            let prev_char = text[..start].chars().next_back().unwrap();
            if prev_char.is_alphanumeric() || prev_char == '_' {
                search_from = start + 7;
                continue;
            }
        }
        let after = &text[start..];
        let mut len = after
            .find(|c: char| c.is_whitespace() || c == '<' || c == '>' || c == '"' || c == '`')
            .unwrap_or(after.len());

        while len > 0 {
            let last_char = after[..len].chars().next_back().unwrap();
            if matches!(last_char, '.' | ',' | ')' | '!' | '?' | ':' | ';' | '\'' | ']') {
                len -= last_char.len_utf8();
            } else {
                break;
            }
        }

        if len > 7 {
            return Some((start, start + len));
        }

        search_from = start + 7;
    }
    None
}

#[derive(Default)]
pub struct InlineCollector {
    pub spans: Vec<InlineSpan>,
    pub current_text: String,
    pub link: Option<(String, String)>,
    pub image: Option<(String, String)>,
    pub strong_depth: usize,
    pub emphasis_depth: usize,
    pub strike_depth: usize,
}

impl InlineCollector {
    pub fn reset(&mut self) {
        self.spans.clear();
        self.current_text.clear();
        self.link = None;
        self.image = None;
        self.strong_depth = 0;
        self.emphasis_depth = 0;
        self.strike_depth = 0;
    }

    pub fn flush_current(&mut self) {
        if self.current_text.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.current_text);
        let style = InlineStyle {
            bold: self.strong_depth > 0,
            italic: self.emphasis_depth > 0,
            strike: self.strike_depth > 0,
        };
        let span = match (style.bold, style.italic, style.strike) {
            (false, false, false) => InlineSpan::Text(text),
            (true, false, false) => InlineSpan::Bold(text),
            (false, true, false) => InlineSpan::Italic(text),
            (false, false, true) => InlineSpan::Strikethrough(text),
            _ => InlineSpan::Styled { text, style },
        };
        self.spans.push(span);
    }

    pub fn push_text(&mut self, mut text: &str) {
        if let Some((_, alt)) = &mut self.image {
            alt.push_str(text);
            return;
        }
        if let Some((_, link_text)) = &mut self.link {
            link_text.push_str(text);
            return;
        }

        while let Some((start, end)) = find_next_autolink(text) {
            if start > 0 {
                self.current_text.push_str(&text[..start]);
            }
            self.flush_current();
            let url = text[start..end].to_string();
            self.spans.push(InlineSpan::Link {
                text: url.clone(),
                url,
            });
            text = &text[end..];
        }

        if !text.is_empty() {
            self.current_text.push_str(text);
        }
    }

    pub fn push_code(&mut self, code: &str) {
        if let Some((_, alt)) = &mut self.image {
            alt.push_str(code);
        } else if let Some((_, link_text)) = &mut self.link {
            link_text.push_str(code);
        } else {
            self.flush_current();
            self.spans.push(InlineSpan::Code(code.to_string()));
        }
    }

    pub fn push_html(&mut self, html: &str) {
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
                        self.flush_current();
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

    pub fn push_footnote_ref(&mut self, label: &str) {
        let ref_text = format!("[^{}]", label);
        if self.image.is_some()
            || self.link.is_some()
            || self.strong_depth > 0
            || self.emphasis_depth > 0
            || self.strike_depth > 0
        {
            self.push_text(&ref_text);
        } else {
            self.flush_current();
            self.spans.push(InlineSpan::Link {
                text: ref_text,
                url: format!("#fn-{}", label),
            });
        }
    }

    pub fn start_strong(&mut self) {
        self.flush_current();
        self.strong_depth += 1;
    }

    pub fn end_strong(&mut self) {
        self.flush_current();
        self.strong_depth = self.strong_depth.saturating_sub(1);
    }

    pub fn start_emphasis(&mut self) {
        self.flush_current();
        self.emphasis_depth += 1;
    }

    pub fn end_emphasis(&mut self) {
        self.flush_current();
        self.emphasis_depth = self.emphasis_depth.saturating_sub(1);
    }

    pub fn start_strike(&mut self) {
        self.flush_current();
        self.strike_depth += 1;
    }

    pub fn end_strike(&mut self) {
        self.flush_current();
        self.strike_depth = self.strike_depth.saturating_sub(1);
    }

    pub fn start_link(&mut self, url: String) {
        self.flush_current();
        self.link = Some((url, String::new()));
    }

    pub fn end_link(&mut self) {
        if let Some((url, text)) = self.link.take() {
            self.spans.push(InlineSpan::Link { text, url });
        }
    }

    pub fn start_image(&mut self, url: String) {
        self.flush_current();
        self.image = Some((url, String::new()));
    }

    pub fn end_image(&mut self) {
        if let Some((url, alt)) = self.image.take() {
            self.spans.push(InlineSpan::ImageFallback { alt, url });
        }
    }

    pub fn finish(&mut self) -> Vec<InlineSpan> {
        self.flush_current();
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

pub fn trim_spans(spans: &mut Vec<InlineSpan>) {
    if let Some(InlineSpan::Text(s) | InlineSpan::Styled { text: s, .. }) = spans.first_mut() {
        *s = s
            .trim_start_matches([' ', '\t', '\r', '\n'])
            .to_string();
    }
    if let Some(InlineSpan::Text(s) | InlineSpan::Styled { text: s, .. }) = spans.last_mut() {
        *s = s
            .trim_end_matches([' ', '\t', '\r', '\n'])
            .to_string();
    }
    spans.retain(|s| match s {
        InlineSpan::Text(t) | InlineSpan::Styled { text: t, .. } => !t.is_empty(),
        _ => true,
    });
}
