use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourceRange {
    pub start_byte: usize,
    pub end_byte: usize,
}

impl SourceRange {
    #[inline]
    pub fn new(start_byte: usize, end_byte: usize) -> Self {
        Self {
            start_byte,
            end_byte,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.end_byte.saturating_sub(self.start_byte)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start_byte >= self.end_byte
    }

    #[inline]
    pub fn contains_byte(&self, byte_offset: usize) -> bool {
        byte_offset >= self.start_byte && byte_offset < self.end_byte
    }

    /// Converts this byte range into character range on the given `rope`.
    pub fn to_char_range(&self, rope: &Rope) -> std::ops::Range<usize> {
        let max_bytes = rope.len_bytes();
        let start_b = self.start_byte.min(max_bytes);
        let end_b = self.end_byte.min(max_bytes);
        let start_char = rope.byte_to_char(start_b);
        let end_char = rope.byte_to_char(end_b);
        start_char..end_char
    }
}

/// Visual styling properties for a rendered text segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SegmentStyle {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub link: bool,
    pub heading_level: Option<u32>,
    pub task_status: Option<bool>,
    pub is_quote: bool,
}

/// Represents a segment in a rendered line, mapping displayed characters back to source Rope characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplaySegment {
    /// Range of character columns in the visually rendered line.
    pub visual_char_range: std::ops::Range<usize>,
    /// Range of character columns in the original source line.
    pub source_char_range: std::ops::Range<usize>,
    /// Range of byte offsets in the source text.
    pub source_byte_range: std::ops::Range<usize>,
    /// The text content of this segment.
    pub text: String,
    /// Style applied to this segment.
    pub style: SegmentStyle,
    /// Whether this segment is a syntax marker (e.g. "**", "#", "`") that can be hidden.
    pub is_syntax_marker: bool,
    /// Whether this segment is currently hidden (suppressed) from visual display.
    pub is_hidden: bool,
}

/// Line-level bidirectional mapping between visual columns and source columns.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineSourceMap {
    pub segments: Vec<DisplaySegment>,
}

impl LineSourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts a visual character column on this line to the corresponding source character column.
    pub fn visual_to_source_col(&self, visual_col: usize) -> usize {
        if self.segments.is_empty() {
            return visual_col;
        }

        for seg in &self.segments {
            if seg.is_hidden {
                continue;
            }
            if visual_col >= seg.visual_char_range.start && visual_col < seg.visual_char_range.end {
                let offset = visual_col - seg.visual_char_range.start;
                return seg.source_char_range.start + offset;
            }
        }

        // If beyond last visible segment, map to end of last source segment (including any trailing hidden markers)
        if let Some(last_visible) = self.segments.iter().rev().find(|s| !s.is_hidden) {
            if visual_col >= last_visible.visual_char_range.end {
                let offset = visual_col - last_visible.visual_char_range.end;
                let final_source_end = self
                    .segments
                    .last()
                    .map(|s| s.source_char_range.end)
                    .unwrap_or(last_visible.source_char_range.end);
                return final_source_end + offset;
            }
        }

        visual_col
    }

    /// Converts a source character column on this line to the corresponding visual character column.
    pub fn source_to_visual_col(&self, source_col: usize) -> usize {
        if self.segments.is_empty() {
            return source_col;
        }

        for seg in &self.segments {
            if source_col >= seg.source_char_range.start && source_col < seg.source_char_range.end {
                if seg.is_hidden {
                    // Hidden marker collapses to start of next visible segment or end of previous
                    return seg.visual_char_range.start;
                }
                let offset = source_col - seg.source_char_range.start;
                return seg.visual_char_range.start + offset;
            }
        }

        // If beyond last segment
        if let Some(last) = self.segments.last() {
            if source_col >= last.source_char_range.end {
                let offset = source_col - last.source_char_range.end;
                let last_vis = self
                    .segments
                    .iter()
                    .rev()
                    .find(|s| !s.is_hidden)
                    .map(|s| s.visual_char_range.end)
                    .unwrap_or(0);
                return last_vis + offset;
            }
        }

        source_col
    }
}

/// Result of parsing a line for Live Preview rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LivePreviewLine {
    pub visual_text: String,
    pub source_map: LineSourceMap,
    pub segments: Vec<DisplaySegment>,
}

/// Parses a line of Markdown for Live Preview display.
/// When `is_cursor_on_line` is true, syntax markers are kept visible (expanded).
/// When `is_cursor_on_line` is false, syntax markers are hidden and styles are applied.
pub fn parse_line_live_preview(line_text: &str, is_cursor_on_line: bool) -> LivePreviewLine {
    if line_text.is_empty() {
        return LivePreviewLine {
            visual_text: String::new(),
            source_map: LineSourceMap::new(),
            segments: Vec::new(),
        };
    }

    let mut raw_segments = Vec::new();
    let mut current_pos_bytes = 0;
    let mut current_pos_chars = 0;

    let trimmed = line_text.trim_start();
    let indent_chars = line_text.chars().count() - trimmed.chars().count();
    let indent_bytes = line_text.len() - trimmed.len();

    // Check for Heading (# Heading)
    let mut heading_level = None;
    let mut content_start_bytes = 0;

    if trimmed.starts_with('#') {
        let hashes = trimmed.chars().take_while(|&c| c == '#').count();
        if hashes <= 6 && trimmed[hashes..].starts_with(' ') {
            heading_level = Some(hashes as u32);
            let marker_len_bytes = indent_bytes + hashes + 1; // including trailing space
            let marker_chars = indent_chars + hashes + 1;

            raw_segments.push(DisplaySegment {
                visual_char_range: 0..0,
                source_char_range: 0..marker_chars,
                source_byte_range: 0..marker_len_bytes,
                text: line_text[0..marker_len_bytes].to_string(),
                style: SegmentStyle {
                    heading_level,
                    ..Default::default()
                },
                is_syntax_marker: true,
                is_hidden: !is_cursor_on_line,
            });

            current_pos_bytes = marker_len_bytes;
            current_pos_chars = marker_chars;
            content_start_bytes = marker_len_bytes;
        }
    } else if trimmed.starts_with("- [ ] ")
        || trimmed.starts_with("* [ ] ")
        || trimmed.starts_with("- [x] ")
        || trimmed.starts_with("* [x] ")
    {
        // Task list
        let is_checked = trimmed.chars().nth(3) == Some('x') || trimmed.chars().nth(3) == Some('X');
        let marker_len_bytes = indent_bytes + 6;
        let marker_chars = indent_chars + 6;

        raw_segments.push(DisplaySegment {
            visual_char_range: 0..0,
            source_char_range: 0..marker_chars,
            source_byte_range: 0..marker_len_bytes,
            text: line_text[0..marker_len_bytes].to_string(),
            style: SegmentStyle {
                task_status: Some(is_checked),
                ..Default::default()
            },
            is_syntax_marker: true,
            is_hidden: !is_cursor_on_line,
        });

        current_pos_bytes = marker_len_bytes;
        current_pos_chars = marker_chars;
        content_start_bytes = marker_len_bytes;
    } else if trimmed.starts_with("> ") {
        // Blockquote
        let marker_len_bytes = indent_bytes + 2;
        let marker_chars = indent_chars + 2;

        raw_segments.push(DisplaySegment {
            visual_char_range: 0..0,
            source_char_range: 0..marker_chars,
            source_byte_range: 0..marker_len_bytes,
            text: line_text[0..marker_len_bytes].to_string(),
            style: SegmentStyle {
                is_quote: true,
                ..Default::default()
            },
            is_syntax_marker: true,
            is_hidden: !is_cursor_on_line,
        });

        current_pos_bytes = marker_len_bytes;
        current_pos_chars = marker_chars;
        content_start_bytes = marker_len_bytes;
    }

    // Parse inline formatting on remainder of line
    let content_text = &line_text[content_start_bytes..];
    parse_inlines_into_segments(
        content_text,
        current_pos_bytes,
        current_pos_chars,
        is_cursor_on_line,
        heading_level,
        &mut raw_segments,
    );

    // Compute visual character ranges and compile visual text
    let mut visual_text = String::new();
    let mut vis_char_offset = 0;

    for seg in &mut raw_segments {
        if is_cursor_on_line {
            seg.is_hidden = false;
        }

        if seg.is_hidden {
            seg.visual_char_range = vis_char_offset..vis_char_offset;
        } else {
            let seg_chars = seg.text.chars().count();
            seg.visual_char_range = vis_char_offset..vis_char_offset + seg_chars;
            visual_text.push_str(&seg.text);
            vis_char_offset += seg_chars;
        }
    }

    let source_map = LineSourceMap {
        segments: raw_segments.clone(),
    };

    LivePreviewLine {
        visual_text,
        source_map,
        segments: raw_segments,
    }
}

fn parse_inlines_into_segments(
    text: &str,
    base_bytes: usize,
    base_chars: usize,
    is_cursor_on_line: bool,
    heading_level: Option<u32>,
    out: &mut Vec<DisplaySegment>,
) {
    if text.is_empty() {
        return;
    }

    let mut cur_byte = 0;
    let mut cur_char = 0;
    let bytes = text.as_bytes();
    let len = bytes.len();

    let mut plain_start_b = 0;
    let mut plain_start_c = 0;

    let base_style = SegmentStyle {
        heading_level,
        ..Default::default()
    };

    macro_rules! flush_plain {
        ($end_b:expr, $end_c:expr) => {
            if $end_b > plain_start_b {
                let slice = &text[plain_start_b..$end_b];
                out.push(DisplaySegment {
                    visual_char_range: 0..0,
                    source_char_range: (base_chars + plain_start_c)..(base_chars + $end_c),
                    source_byte_range: (base_bytes + plain_start_b)..(base_bytes + $end_b),
                    text: slice.to_string(),
                    style: base_style,
                    is_syntax_marker: false,
                    is_hidden: false,
                });
            }
        };
    }

    while cur_byte < len {
        // 1. Inline code: `code`
        if bytes[cur_byte] == b'`' {
            if let Some(close_rel) = text[cur_byte + 1..].find('`') {
                let close_b = cur_byte + 1 + close_rel;
                flush_plain!(cur_byte, cur_char);

                // Opening `
                out.push(DisplaySegment {
                    visual_char_range: 0..0,
                    source_char_range: (base_chars + cur_char)..(base_chars + cur_char + 1),
                    source_byte_range: (base_bytes + cur_byte)..(base_bytes + cur_byte + 1),
                    text: "`".to_string(),
                    style: base_style,
                    is_syntax_marker: true,
                    is_hidden: !is_cursor_on_line,
                });

                // Code content
                let code_content = &text[cur_byte + 1..close_b];
                let code_chars = code_content.chars().count();
                out.push(DisplaySegment {
                    visual_char_range: 0..0,
                    source_char_range: (base_chars + cur_char + 1)..(base_chars + cur_char + 1 + code_chars),
                    source_byte_range: (base_bytes + cur_byte + 1)..(base_bytes + close_b),
                    text: code_content.to_string(),
                    style: SegmentStyle {
                        code: true,
                        heading_level,
                        ..Default::default()
                    },
                    is_syntax_marker: false,
                    is_hidden: false,
                });

                // Closing `
                out.push(DisplaySegment {
                    visual_char_range: 0..0,
                    source_char_range: (base_chars + cur_char + 1 + code_chars)..(base_chars + cur_char + 2 + code_chars),
                    source_byte_range: (base_bytes + close_b)..(base_bytes + close_b + 1),
                    text: "`".to_string(),
                    style: base_style,
                    is_syntax_marker: true,
                    is_hidden: !is_cursor_on_line,
                });

                cur_byte = close_b + 1;
                cur_char += 2 + code_chars;
                plain_start_b = cur_byte;
                plain_start_c = cur_char;
                continue;
            }
        }

        // 2. Bold: **text**
        if cur_byte + 1 < len && bytes[cur_byte] == b'*' && bytes[cur_byte + 1] == b'*' {
            if let Some(close_rel) = text[cur_byte + 2..].find("**") {
                let close_b = cur_byte + 2 + close_rel;
                flush_plain!(cur_byte, cur_char);

                // Opening **
                out.push(DisplaySegment {
                    visual_char_range: 0..0,
                    source_char_range: (base_chars + cur_char)..(base_chars + cur_char + 2),
                    source_byte_range: (base_bytes + cur_byte)..(base_bytes + cur_byte + 2),
                    text: "**".to_string(),
                    style: base_style,
                    is_syntax_marker: true,
                    is_hidden: !is_cursor_on_line,
                });

                // Bold content
                let bold_content = &text[cur_byte + 2..close_b];
                let bold_chars = bold_content.chars().count();
                out.push(DisplaySegment {
                    visual_char_range: 0..0,
                    source_char_range: (base_chars + cur_char + 2)..(base_chars + cur_char + 2 + bold_chars),
                    source_byte_range: (base_bytes + cur_byte + 2)..(base_bytes + close_b),
                    text: bold_content.to_string(),
                    style: SegmentStyle {
                        bold: true,
                        heading_level,
                        ..Default::default()
                    },
                    is_syntax_marker: false,
                    is_hidden: false,
                });

                // Closing **
                out.push(DisplaySegment {
                    visual_char_range: 0..0,
                    source_char_range: (base_chars + cur_char + 2 + bold_chars)..(base_chars + cur_char + 4 + bold_chars),
                    source_byte_range: (base_bytes + close_b)..(base_bytes + close_b + 2),
                    text: "**".to_string(),
                    style: base_style,
                    is_syntax_marker: true,
                    is_hidden: !is_cursor_on_line,
                });

                cur_byte = close_b + 2;
                cur_char += 4 + bold_chars;
                plain_start_b = cur_byte;
                plain_start_c = cur_char;
                continue;
            }
        }

        // 3. Strikethrough: ~~text~~
        if cur_byte + 1 < len && bytes[cur_byte] == b'~' && bytes[cur_byte + 1] == b'~' {
            if let Some(close_rel) = text[cur_byte + 2..].find("~~") {
                let close_b = cur_byte + 2 + close_rel;
                flush_plain!(cur_byte, cur_char);

                // Opening ~~
                out.push(DisplaySegment {
                    visual_char_range: 0..0,
                    source_char_range: (base_chars + cur_char)..(base_chars + cur_char + 2),
                    source_byte_range: (base_bytes + cur_byte)..(base_bytes + cur_byte + 2),
                    text: "~~".to_string(),
                    style: base_style,
                    is_syntax_marker: true,
                    is_hidden: !is_cursor_on_line,
                });

                // Strike content
                let strike_content = &text[cur_byte + 2..close_b];
                let strike_chars = strike_content.chars().count();
                out.push(DisplaySegment {
                    visual_char_range: 0..0,
                    source_char_range: (base_chars + cur_char + 2)..(base_chars + cur_char + 2 + strike_chars),
                    source_byte_range: (base_bytes + cur_byte + 2)..(base_bytes + close_b),
                    text: strike_content.to_string(),
                    style: SegmentStyle {
                        strikethrough: true,
                        heading_level,
                        ..Default::default()
                    },
                    is_syntax_marker: false,
                    is_hidden: false,
                });

                // Closing ~~
                out.push(DisplaySegment {
                    visual_char_range: 0..0,
                    source_char_range: (base_chars + cur_char + 2 + strike_chars)..(base_chars + cur_char + 4 + strike_chars),
                    source_byte_range: (base_bytes + close_b)..(base_bytes + close_b + 2),
                    text: "~~".to_string(),
                    style: base_style,
                    is_syntax_marker: true,
                    is_hidden: !is_cursor_on_line,
                });

                cur_byte = close_b + 2;
                cur_char += 4 + strike_chars;
                plain_start_b = cur_byte;
                plain_start_c = cur_char;
                continue;
            }
        }

        // 4. Italic: *text* (excluding **)
        if bytes[cur_byte] == b'*' && (cur_byte + 1 == len || bytes[cur_byte + 1] != b'*') {
            if let Some(close_rel) = text[cur_byte + 1..].find('*') {
                if close_rel > 0 {
                    let close_b = cur_byte + 1 + close_rel;
                    flush_plain!(cur_byte, cur_char);

                    // Opening *
                    out.push(DisplaySegment {
                        visual_char_range: 0..0,
                        source_char_range: (base_chars + cur_char)..(base_chars + cur_char + 1),
                        source_byte_range: (base_bytes + cur_byte)..(base_bytes + cur_byte + 1),
                        text: "*".to_string(),
                        style: base_style,
                        is_syntax_marker: true,
                        is_hidden: !is_cursor_on_line,
                    });

                    // Italic content
                    let italic_content = &text[cur_byte + 1..close_b];
                    let italic_chars = italic_content.chars().count();
                    out.push(DisplaySegment {
                        visual_char_range: 0..0,
                        source_char_range: (base_chars + cur_char + 1)..(base_chars + cur_char + 1 + italic_chars),
                        source_byte_range: (base_bytes + cur_byte + 1)..(base_bytes + close_b),
                        text: italic_content.to_string(),
                        style: SegmentStyle {
                            italic: true,
                            heading_level,
                            ..Default::default()
                        },
                        is_syntax_marker: false,
                        is_hidden: false,
                    });

                    // Closing *
                    out.push(DisplaySegment {
                        visual_char_range: 0..0,
                        source_char_range: (base_chars + cur_char + 1 + italic_chars)..(base_chars + cur_char + 2 + italic_chars),
                        source_byte_range: (base_bytes + close_b)..(base_bytes + close_b + 1),
                        text: "*".to_string(),
                        style: base_style,
                        is_syntax_marker: true,
                        is_hidden: !is_cursor_on_line,
                    });

                    cur_byte = close_b + 1;
                    cur_char += 2 + italic_chars;
                    plain_start_b = cur_byte;
                    plain_start_c = cur_char;
                    continue;
                }
            }
        }

        // 5. Link: [label](url)
        if bytes[cur_byte] == b'[' {
            if let Some(close_bracket) = text[cur_byte + 1..].find(']') {
                let close_bracket_b = cur_byte + 1 + close_bracket;
                if text[close_bracket_b + 1..].starts_with('(') {
                    if let Some(close_paren) = text[close_bracket_b + 2..].find(')') {
                        let close_paren_b = close_bracket_b + 2 + close_paren;
                        flush_plain!(cur_byte, cur_char);

                        // Opening [
                        out.push(DisplaySegment {
                            visual_char_range: 0..0,
                            source_char_range: (base_chars + cur_char)..(base_chars + cur_char + 1),
                            source_byte_range: (base_bytes + cur_byte)..(base_bytes + cur_byte + 1),
                            text: "[".to_string(),
                            style: base_style,
                            is_syntax_marker: true,
                            is_hidden: !is_cursor_on_line,
                        });

                        // Link label
                        let label_content = &text[cur_byte + 1..close_bracket_b];
                        let label_chars = label_content.chars().count();
                        out.push(DisplaySegment {
                            visual_char_range: 0..0,
                            source_char_range: (base_chars + cur_char + 1)..(base_chars + cur_char + 1 + label_chars),
                            source_byte_range: (base_bytes + cur_byte + 1)..(base_bytes + close_bracket_b),
                            text: label_content.to_string(),
                            style: SegmentStyle {
                                link: true,
                                heading_level,
                                ..Default::default()
                            },
                            is_syntax_marker: false,
                            is_hidden: false,
                        });

                        // Closing ](url)
                        let url_marker_content = &text[close_bracket_b..=close_paren_b];
                        let url_marker_chars = url_marker_content.chars().count();
                        out.push(DisplaySegment {
                            visual_char_range: 0..0,
                            source_char_range: (base_chars + cur_char + 1 + label_chars)..(base_chars + cur_char + 1 + label_chars + url_marker_chars),
                            source_byte_range: (base_bytes + close_bracket_b)..(base_bytes + close_paren_b + 1),
                            text: url_marker_content.to_string(),
                            style: base_style,
                            is_syntax_marker: true,
                            is_hidden: !is_cursor_on_line,
                        });

                        cur_byte = close_paren_b + 1;
                        cur_char += 1 + label_chars + url_marker_chars;
                        plain_start_b = cur_byte;
                        plain_start_c = cur_char;
                        continue;
                    }
                }
            }
        }

        // Advance single character
        let ch = text[cur_byte..].chars().next().unwrap();
        cur_byte += ch.len_utf8();
        cur_char += 1;
    }

    flush_plain!(len, cur_char);
}
