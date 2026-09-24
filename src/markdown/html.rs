use crate::markdown::inline::InlineSpan;

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

pub fn extract_html_attr(tag: &str, attr_name: &str) -> Option<String> {
    let lower_tag = tag.to_ascii_lowercase();
    let search_key = format!("{}=", attr_name);
    let pos = lower_tag.find(&search_key)?;
    let after = tag[pos + search_key.len()..].trim_start();
    if let Some(stripped) = after.strip_prefix('"') {
        let end = stripped.find('"')?;
        Some(stripped[..end].to_string())
    } else if let Some(stripped) = after.strip_prefix('\'') {
        let end = stripped.find('\'')?;
        Some(stripped[..end].to_string())
    } else {
        let end = after
            .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
            .unwrap_or(after.len());
        Some(after[..end].to_string())
    }
}
