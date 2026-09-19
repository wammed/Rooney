use crate::app::Message;
use crate::editor::pane::EditorPane;
use crate::theme::EditorTheme;
use cosmic::iced::advanced::graphics::geometry::{
    Frame, Path, Stroke, Text, Renderer as GeometryRenderer,
};
use cosmic::iced::widget::canvas::Geometry;
use cosmic::iced::advanced::layout::{self, Layout};
use cosmic::iced::advanced::renderer;
use cosmic::iced::advanced::widget::tree::{self, Tree};
use cosmic::iced::advanced::{Clipboard, InputMethod, Shell, Widget, Renderer};
use cosmic::iced::event::Event;
use cosmic::iced::mouse;
use cosmic::iced::{Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Vector};
use unicode_width::UnicodeWidthChar;

fn intern_font_name(name: &str) -> &'static str {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};

    static CACHE: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashSet::new()));
    let mut set = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(&interned) = set.get(name) {
        interned
    } else {
        let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
        set.insert(leaked);
        leaked
    }
}

#[derive(Default)]
pub struct CanvasState {
    pub is_dragging: bool,
    pub is_dragging_scrollbar: bool,
    pub scrollbar_drag_offset_y: f32,
}

pub struct EditorCanvas<'a> {
    pub pane: &'a EditorPane,
    pub theme: &'a EditorTheme,
    pub is_focused: bool,
    pub font_name: &'a str,
    pub font_size: f32,
    pub line_height: f32,
    pub char_width: f32,
}

#[derive(Debug, Clone)]
pub struct VisualRow {
    pub line_idx: usize,
    pub char_start: usize,
    pub char_end: usize,
    pub is_first_subrow: bool,
    pub y: f32,
}

type GlyphMetricKey = (char, u32, &'static str);
type GlyphMetricsMap = std::sync::RwLock<std::collections::HashMap<GlyphMetricKey, f32>>;

static METRICS_CACHE: std::sync::OnceLock<GlyphMetricsMap> = std::sync::OnceLock::new();
static FONT_SYSTEM: std::sync::OnceLock<std::sync::Mutex<cosmic_text::FontSystem>> =
    std::sync::OnceLock::new();

/// Clears the global glyph advance cache. Call this when font family, size, or themes are changed.
pub fn clear_glyph_cache() {
    if let Some(cache) = METRICS_CACHE.get() {
        if let Ok(mut guard) = cache.write() {
            guard.clear();
        }
    }
}

fn measure_glyph_advance(c: char, font_size: f32, font_name: &str) -> f32 {
    if c == '\t' {
        return font_size * 0.60 * 4.0;
    }
    // Fast path: ASCII printable characters have uniform monospace width
    if (' '..='~').contains(&c) {
        return font_size * 0.60;
    }

    use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};
    use std::collections::HashMap;
    use std::sync::RwLock;

    let interned_name = intern_font_name(font_name);
    let key = (c, font_size.to_bits(), interned_name);

    let cache = METRICS_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    if let Ok(guard) = cache.read() {
        if let Some(&w) = guard.get(&key) {
            return w;
        }
    }

    let font_system_lock = FONT_SYSTEM.get_or_init(|| std::sync::Mutex::new(FontSystem::new()));
    let mut measured_w = None;

    if let Ok(mut fs) = font_system_lock.lock() {
        let metrics = Metrics::new(font_size, (font_size * 1.5).round());
        let mut buffer = Buffer::new(&mut fs, metrics);
        let mut s = [0u8; 4];
        let str_slice = c.encode_utf8(&mut s);
        let family = if font_name == "monospace" {
            Family::Monospace
        } else {
            Family::Name(font_name)
        };
        buffer.set_text(&mut fs, str_slice, Attrs::new().family(family), Shaping::Advanced);
        buffer.shape_until_scroll(&mut fs, false);

        let mut total_w = 0.0;
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                total_w += glyph.w;
            }
        }
        if total_w > 0.0 {
            measured_w = Some(total_w);
        }
    }

    let w = measured_w.unwrap_or_else(|| {
        if c.width_cjk().unwrap_or(1) == 2 {
            font_size * 1.0
        } else {
            font_size * 0.60
        }
    });

    if let Ok(mut guard) = cache.write() {
        guard.insert(key, w);
    }

    w
}

impl<'a> EditorCanvas<'a> {
    #[inline]
    pub fn clear_glyph_cache() {
        clear_glyph_cache();
    }

    #[inline]
    pub fn char_advance(c: char, font_size: f32) -> f32 {
        Self::char_advance_with_font(c, font_size, "JetBrainsMono Nerd Font")
    }

    #[inline]
    pub fn char_advance_with_font(c: char, font_size: f32, font_name: &str) -> f32 {
        measure_glyph_advance(c, font_size, font_name)
    }

    #[inline]
    pub fn glyph_advance(&self, c: char) -> f32 {
        measure_glyph_advance(c, self.font_size, self.font_name)
    }


    pub fn new(
        pane: &'a EditorPane,
        theme: &'a EditorTheme,
        is_focused: bool,
        font_name: &'a str,
        font_size: f32,
    ) -> Self {
        let line_height = (font_size * 1.5).round();
        let char_width = font_size * 0.60;

        Self {
            pane,
            theme,
            is_focused,
            font_name,
            font_size,
            line_height,
            char_width,
        }
    }

    pub fn gutter_width(&self) -> f32 {
        let lines = self.pane.buffer.line_count().max(100);
        let digits = lines.to_string().len().max(3);
        (digits as f32) * self.char_width + 24.0
    }

    /// Safely slices a string slice by character (Unicode scalar value) indices.
    /// Returns an empty slice if out of bounds. Pure ASCII strings are sliced in O(1).
    pub fn slice_by_char_indices(text: &str, start_char: usize, end_char: usize) -> &str {
        if start_char >= end_char {
            return "";
        }
        if text.is_ascii() {
            let s = start_char.min(text.len());
            let e = end_char.min(text.len());
            return &text[s..e];
        }
        let mut start_byte = text.len();
        let mut end_byte = text.len();
        let mut cur_char = 0;
        for (b, _) in text.char_indices() {
            if cur_char == start_char {
                start_byte = b;
            }
            if cur_char == end_char {
                end_byte = b;
                break;
            }
            cur_char += 1;
        }
        if start_char >= cur_char {
            return "";
        }
        &text[start_byte..end_byte]
    }

    #[inline]
    pub fn total_content_height(&self) -> f32 {
        let total_lines = self.pane.buffer.line_count();
        (total_lines as f32) * self.line_height
    }

    pub fn build_viewport_visual_rows(
        &self,
        bounds_width: f32,
        scroll_y: f32,
        bounds_height: f32,
    ) -> Vec<VisualRow> {
        let total_lines = self.pane.buffer.line_count();
        if total_lines == 0 {
            return Vec::new();
        }

        let first_visible_line = (scroll_y / self.line_height).floor().max(0.0) as usize;
        let visible_count = (bounds_height / self.line_height).ceil().max(1.0) as usize;
        let margin = 5;
        let start_line = first_visible_line.saturating_sub(margin).min(total_lines.saturating_sub(1));
        let end_line = (first_visible_line + visible_count + margin).min(total_lines);

        let gutter = self.gutter_width();
        let avail_width = (bounds_width - gutter - 24.0).max(120.0);

        let mut visual_rows = Vec::new();

        for line_idx in start_line..end_line {
            let line_base_y = (line_idx as f32) * self.line_height;
            let line_text = self.pane.buffer.line_text(line_idx).unwrap_or_default();

            if line_text.is_empty() {
                visual_rows.push(VisualRow {
                    line_idx,
                    char_start: 0,
                    char_end: 0,
                    is_first_subrow: true,
                    y: line_base_y,
                });
                continue;
            }

            let mut start = 0;
            let mut cur_row_w = 0.0;
            let mut is_first = true;
            let mut total_chars = 0;
            let mut subrow_idx = 0;

            for (i, c) in line_text.chars().enumerate() {
                total_chars = i + 1;
                let w = self.glyph_advance(c);
                if cur_row_w + w > avail_width && i > start {
                    visual_rows.push(VisualRow {
                        line_idx,
                        char_start: start,
                        char_end: i,
                        is_first_subrow: is_first,
                        y: line_base_y + (subrow_idx as f32) * self.line_height,
                    });
                    start = i;
                    cur_row_w = 0.0;
                    is_first = false;
                    subrow_idx += 1;
                }
                cur_row_w += w;
            }

            visual_rows.push(VisualRow {
                line_idx,
                char_start: start,
                char_end: total_chars,
                is_first_subrow: is_first,
                y: line_base_y + (subrow_idx as f32) * self.line_height,
            });
        }

        visual_rows
    }

    #[allow(dead_code)]
    fn build_visual_rows(&self, bounds_width: f32) -> Vec<VisualRow> {
        self.build_viewport_visual_rows(bounds_width, 0.0, f32::MAX)
    }

    pub fn cursor_screen_pos(&self, bounds: Rectangle) -> Option<Point> {
        let gutter = self.gutter_width();
        let cursor = self.pane.buffer.cursor;
        let line_text = self.pane.buffer.line_text(cursor.0).unwrap_or_default();
        let avail_width = (bounds.width - gutter - 24.0).max(120.0);

        let mut start = 0;
        let mut cur_row_w = 0.0;
        let mut subrow_idx = 0;
        let mut target_subrow_start = 0;
        let mut target_subrow_idx = 0;

        let chars: Vec<char> = line_text.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            let w = self.glyph_advance(c);
            if cur_row_w + w > avail_width && i > start {
                if cursor.1 >= start && cursor.1 <= i {
                    target_subrow_start = start;
                    target_subrow_idx = subrow_idx;
                    break;
                }
                start = i;
                cur_row_w = 0.0;
                subrow_idx += 1;
            }
            cur_row_w += w;
        }
        if cursor.1 >= start {
            target_subrow_start = start;
            target_subrow_idx = subrow_idx;
        }

        let y = ((cursor.0 + target_subrow_idx) as f32) * self.line_height - self.pane.scroll_y.get();
        let mut pixel_offset = 0.0;
        if cursor.1 > target_subrow_start {
            for &ch in chars.iter().skip(target_subrow_start).take(cursor.1.saturating_sub(target_subrow_start)) {
                pixel_offset += self.glyph_advance(ch);
            }
        }

        let x = gutter + 10.0 + pixel_offset - self.pane.scroll_x.get();
        Some(Point::new(x, y))
    }

    pub fn pos_to_char_coords(&self, pos: Point, bounds: Rectangle) -> (usize, usize) {
        let gutter = self.gutter_width();
        let total_lines = self.pane.buffer.line_count();
        if total_lines == 0 {
            return (0, 0);
        }

        let clicked_line = ((pos.y + self.pane.scroll_y.get()) / self.line_height).floor().max(0.0) as usize;
        let clicked_line = clicked_line.min(total_lines - 1);
        let line_text = self.pane.buffer.line_text(clicked_line).unwrap_or_default();
        let avail_width = (bounds.width - gutter - 24.0).max(120.0);

        let chars: Vec<char> = line_text.chars().collect();
        if chars.is_empty() {
            return (clicked_line, 0);
        }

        let rel_x = (pos.x - gutter - 10.0 + self.pane.scroll_x.get()).max(0.0);

        let mut subrows = Vec::new();
        let mut start = 0;
        let mut cur_row_w = 0.0;
        for (i, &c) in chars.iter().enumerate() {
            let w = self.glyph_advance(c);
            if cur_row_w + w > avail_width && i > start {
                subrows.push((start, i));
                start = i;
                cur_row_w = 0.0;
            }
            cur_row_w += w;
        }
        subrows.push((start, chars.len()));

        let line_top_y = (clicked_line as f32) * self.line_height - self.pane.scroll_y.get();
        let subrow_idx = (((pos.y - line_top_y) / self.line_height).floor().max(0.0) as usize).min(subrows.len() - 1);

        let (sub_start, sub_end) = subrows[subrow_idx];
        let mut acc_width = 0.0;
        let mut chosen_col = sub_start;

        if sub_end > sub_start {
            for (idx_offset, &ch) in chars[sub_start..sub_end].iter().enumerate() {
                let char_pixel_w = self.glyph_advance(ch);
                if acc_width + char_pixel_w / 2.0 >= rel_x {
                    break;
                }
                acc_width += char_pixel_w;
                chosen_col = sub_start + idx_offset + 1;
            }
        }
        (clicked_line, chosen_col)
    }

    pub fn draw_frame(
        &self,
        renderer: &cosmic::Renderer,
        bounds: Rectangle,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let gutter = self.gutter_width();
        let buffer = &self.pane.buffer;

        // 1. Background (with Wayland alpha)
        let bg_color = self.theme.background_with_alpha();
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg_color);

        if self.theme.dimming > 0.0 {
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::from_rgba(0.0, 0.0, 0.0, self.theme.dimming * 0.8),
            );
        }

        // 2. Gutter Background
        let gutter_rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: gutter,
            height: bounds.height,
        };
        frame.fill_rectangle(gutter_rect.position(), gutter_rect.size(), self.theme.config.gutter_bg);

        frame.stroke(
            &Path::line(Point::new(gutter, 0.0), Point::new(gutter, bounds.height)),
            Stroke::default()
                .with_color(self.theme.config.border)
                .with_width(1.0),
        );

        let font = Font::with_name(intern_font_name(self.font_name));
        let text_size = Pixels(self.font_size);

        // 3. Viewport virtualized visual rows
        let total_content_height = self.total_content_height();
        let max_scroll = (total_content_height - bounds.height).max(0.0);

        // 3.1 Auto-scrolling to keep cursor visible if requested
        if self.pane.needs_scroll_to_cursor.get() {
            let cursor = buffer.cursor;
            let cursor_top = (cursor.0 as f32) * self.line_height;
            let cursor_bottom = cursor_top + self.line_height;
            let margin = (2.0 * self.line_height).min(bounds.height * 0.25).max(0.0);

            let mut scroll = self.pane.scroll_y.get();
            if cursor_top < scroll + margin {
                scroll = (cursor_top - margin).max(0.0);
            } else if cursor_bottom > scroll + bounds.height - margin {
                scroll = (cursor_bottom + margin - bounds.height).max(0.0);
            }
            self.pane.scroll_y.set(scroll.clamp(0.0, max_scroll));
            self.pane.needs_scroll_to_cursor.set(false);
        }

        let cur_scroll_y = self.pane.scroll_y.get().clamp(0.0, max_scroll);
        self.pane.scroll_y.set(cur_scroll_y);
        let cur_scroll_x = self.pane.scroll_x.get();

        let visual_rows = self.build_viewport_visual_rows(bounds.width, cur_scroll_y, bounds.height);

        // 4. Current Line highlight
        for row in &visual_rows {
            if row.line_idx == buffer.cursor.0 {
                let y = row.y - cur_scroll_y;
                if y + self.line_height >= 0.0 && y <= bounds.height {
                    let cur_line_rect = Rectangle {
                        x: gutter,
                        y,
                        width: bounds.width - gutter,
                        height: self.line_height,
                    };
                    frame.fill_rectangle(
                        cur_line_rect.position(),
                        cur_line_rect.size(),
                        self.theme.config.current_line_bg,
                    );
                }
            }
        }

        // 4.1 Draw Selection Highlight (if any text is selected)
        if let Some(anchor) = self.pane.buffer.selection_anchor {
            if anchor != buffer.cursor {
                let (sel_start, sel_end) = if (buffer.cursor.0, buffer.cursor.1) < (anchor.0, anchor.1) {
                    (buffer.cursor, anchor)
                } else {
                    (anchor, buffer.cursor)
                };

                for row in &visual_rows {
                    let y = row.y - cur_scroll_y;
                    if y + self.line_height < 0.0 || y > bounds.height {
                        continue;
                    }

                    if row.line_idx >= sel_start.0 && row.line_idx <= sel_end.0 {
                        let line_text = buffer.line_text(row.line_idx).unwrap_or_default();

                        let line_sel_start = if row.line_idx == sel_start.0 {
                            sel_start.1.max(row.char_start)
                        } else {
                            row.char_start
                        };

                        let line_sel_end = if row.line_idx == sel_end.0 {
                            sel_end.1.min(row.char_end)
                        } else {
                            row.char_end
                        };

                        if line_sel_start < line_sel_end {
                            let mut start_x_offset = 0.0;
                            if line_sel_start > row.char_start {
                                for ch in line_text.chars().skip(row.char_start).take(line_sel_start - row.char_start) {
                                    start_x_offset += self.glyph_advance(ch);
                                }
                            }

                            let mut sel_w = 0.0;
                            for ch in line_text.chars().skip(line_sel_start).take(line_sel_end - line_sel_start) {
                                sel_w += self.glyph_advance(ch);
                            }

                            let sel_rect = Rectangle {
                                x: gutter + 10.0 + start_x_offset - cur_scroll_x,
                                y,
                                width: sel_w.max(4.0),
                                height: self.line_height,
                            };
                            frame.fill_rectangle(
                                sel_rect.position(),
                                sel_rect.size(),
                                self.theme.config.selection_bg,
                            );
                        }
                    }
                }
            }
        }

        // 4.2 Draw Search Match Highlights
        if !self.pane.search_matches.is_empty() {
            for (match_idx, &(m_line, m_start, m_end)) in self.pane.search_matches.iter().enumerate() {
                for row in &visual_rows {
                    let y = row.y - cur_scroll_y;
                    if y + self.line_height < 0.0 || y > bounds.height {
                        continue;
                    }

                    if row.line_idx == m_line {
                        let match_vis_start = m_start.max(row.char_start);
                        let match_vis_end = m_end.min(row.char_end);

                        if match_vis_start < match_vis_end {
                            let line_text = buffer.line_text(row.line_idx).unwrap_or_default();

                            let mut x_offset = 0.0;
                            if match_vis_start > row.char_start {
                                for ch in line_text.chars().skip(row.char_start).take(match_vis_start - row.char_start) {
                                    x_offset += self.glyph_advance(ch);
                                }
                            }

                            let mut match_w = 0.0;
                            for ch in line_text.chars().skip(match_vis_start).take(match_vis_end - match_vis_start) {
                                match_w += self.glyph_advance(ch);
                            }

                            let is_current = match_idx == self.pane.current_match_idx;
                            let match_rect = Rectangle {
                                x: gutter + 10.0 + x_offset - cur_scroll_x,
                                y,
                                width: match_w.max(4.0),
                                height: self.line_height,
                            };

                            let match_color = if is_current {
                                Color::from_rgba(1.0, 0.8, 0.2, 0.6)
                            } else {
                                Color::from_rgba(1.0, 0.9, 0.3, 0.3)
                            };
                            frame.fill_rectangle(match_rect.position(), match_rect.size(), match_color);

                            if is_current {
                                frame.stroke(
                                    &Path::rectangle(match_rect.position(), match_rect.size()),
                                    Stroke::default().with_color(Color::WHITE).with_width(1.0),
                                );
                            }
                        }
                    }
                }
            }
        }

        // 5. Draw lines (Gutter and soft-wrapped text)
        for row in &visual_rows {
            let y = row.y - cur_scroll_y;

            // Vertical clipping: only render rows inside visible window
            if y + self.line_height < 0.0 {
                continue;
            }
            if y > bounds.height {
                break;
            }

            // Gutter line number (only on the first visual subrow)
            if row.is_first_subrow {
                let line_num_str = format!("{}", row.line_idx + 1);
                let num_color = if row.line_idx == buffer.cursor.0 {
                    self.theme.config.current_line_num
                } else {
                    self.theme.config.gutter_fg
                };

                frame.fill_text(Text {
                    content: line_num_str,
                    position: Point::new(gutter - 12.0, y + 2.0),
                    color: num_color,
                    size: text_size,
                    font,
                    align_x: cosmic::iced::alignment::Horizontal::Right.into(),
                    ..Default::default()
                });
            }

            if let Some(line_text) = buffer.line_text(row.line_idx) {
                if row.char_start < row.char_end {
                    let sub_end = row.char_end;
                    let spans = self.pane.highlighter.highlight_line(&line_text, row.line_idx);

                    let mut cur_col = row.char_start;
                    let mut cur_pixel_x = gutter + 10.0 - cur_scroll_x;

                    while cur_col < sub_end {
                        let span = spans.iter().find(|s| s.start_col <= cur_col && cur_col < s.end_col);
                        let (token_end, token_color) = if let Some(s) = span {
                            (s.end_col.min(sub_end), s.token_type.color(&self.theme.config))
                        } else {
                            let next_start = spans
                                .iter()
                                .map(|s| s.start_col)
                                .filter(|&start| start > cur_col && start < sub_end)
                                .min()
                                .unwrap_or(sub_end);
                            (next_start, self.theme.config.fg)
                        };

                        let segment = Self::slice_by_char_indices(&line_text, cur_col, token_end);
                        let mut seg_w = 0.0;
                        for ch in segment.chars() {
                            seg_w += self.glyph_advance(ch);
                        }

                        frame.fill_text(Text {
                            content: segment.to_string(),
                            position: Point::new(cur_pixel_x, y + 2.0),
                            color: token_color,
                            size: text_size,
                            font,
                            ..Default::default()
                        });

                        cur_pixel_x += seg_w;
                        cur_col = token_end;
                    }
                }
            }
        }

        // 6. Draw Cursor (Caret) & IME Preedit
        if self.is_focused {
            if let Some(cursor_pt) = self.cursor_screen_pos(bounds) {
                if cursor_pt.y + self.line_height >= 0.0 && cursor_pt.y <= bounds.height {
                    // 7. Render IME Preedit text (Composing Japanese string)
                    let mut preedit_offset_x = 0.0;
                    let mut caret_x = cursor_pt.x;

                    if let Some((ref preedit_str, ref sel)) = self.pane.preedit {
                        if !preedit_str.is_empty() {
                            let preedit_x = cursor_pt.x;
                            let mut preedit_w = 0.0;
                            for c in preedit_str.chars() {
                                preedit_w += self.glyph_advance(c);
                            }

                            // Highlight underlay for preedit
                            frame.fill_rectangle(
                                Point::new(preedit_x, cursor_pt.y),
                                cosmic::iced::Size::new(preedit_w, self.line_height),
                                Color::from_rgba(0.2, 0.4, 0.8, 0.25),
                            );

                            frame.fill_text(Text {
                                content: preedit_str.clone(),
                                position: Point::new(preedit_x, cursor_pt.y + 2.0),
                                color: self.theme.config.accent,
                                size: text_size,
                                font,
                                ..Default::default()
                            });

                            // Underline for IME preedit
                            frame.stroke(
                                &Path::line(
                                    Point::new(preedit_x, cursor_pt.y + self.line_height - 1.0),
                                    Point::new(preedit_x + preedit_w, cursor_pt.y + self.line_height - 1.0),
                                ),
                                Stroke::default()
                                    .with_color(self.theme.config.accent)
                                    .with_width(2.0),
                            );

                            // The caret is placed at the end of the preedit (or selection)
                            let sel_end = sel.as_ref().map(|r| r.end).unwrap_or(preedit_str.chars().count());
                            let mut sel_w = 0.0;
                            for c in preedit_str.chars().take(sel_end) {
                                sel_w += self.glyph_advance(c);
                            }
                            caret_x = preedit_x + sel_w;
                            preedit_offset_x = preedit_w + 4.0;
                        }
                    }

                    let caret_rect = Rectangle {
                        x: caret_x,
                        y: cursor_pt.y + 1.0,
                        width: 2.0,
                        height: self.line_height - 2.0,
                    };
                    frame.fill_rectangle(
                        caret_rect.position(),
                        caret_rect.size(),
                        self.theme.config.cursor,
                    );

                    // 8. Draw Local AI FIM Ghost Text (Inline Suggestion)
                    if let Some(ref ghost) = self.pane.ghost_text {
                        if !ghost.is_empty() && self.pane.preedit.is_none() {
                            let first_line = ghost.lines().next().unwrap_or(ghost);
                            let ghost_x = cursor_pt.x + 3.0 + preedit_offset_x;

                            frame.fill_text(Text {
                                content: first_line.to_string(),
                                position: Point::new(ghost_x, cursor_pt.y + 2.0),
                                color: self.theme.config.ghost_text,
                                size: text_size,
                                font,
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }

        // 9. Active Focused Border
        if self.is_focused {
            frame.stroke(
                &Path::rectangle(Point::ORIGIN, bounds.size()),
                Stroke::default()
                    .with_color(self.theme.config.accent)
                    .with_width(2.0),
            );
        }

        // 10. Right-edge Scrollbar
        if total_content_height > bounds.height {
            let scrollbar_width = 12.0;
            let track_rect = Rectangle {
                x: bounds.width - scrollbar_width,
                y: 0.0,
                width: scrollbar_width,
                height: bounds.height,
            };
            frame.fill_rectangle(
                track_rect.position(),
                track_rect.size(),
                Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            );
            frame.stroke(
                &Path::line(
                    Point::new(bounds.width - scrollbar_width, 0.0),
                    Point::new(bounds.width - scrollbar_width, bounds.height),
                ),
                Stroke::default()
                    .with_color(self.theme.config.border)
                    .with_width(1.0),
            );

            let thumb_height = ((bounds.height / total_content_height) * bounds.height)
                .max(28.0)
                .min(bounds.height);
            let max_thumb_y = (bounds.height - thumb_height).max(0.0);
            let thumb_y = if max_scroll > 0.0 {
                (cur_scroll_y / max_scroll) * max_thumb_y
            } else {
                0.0
            };
            let thumb_color = if self.is_focused {
                let mut c = self.theme.config.accent;
                c.a = 0.75;
                c
            } else {
                let mut c = self.theme.config.border;
                c.a = 0.6;
                c
            };
            let thumb_rect = Rectangle {
                x: bounds.width - scrollbar_width + 2.0,
                y: thumb_y,
                width: scrollbar_width - 4.0,
                height: thumb_height,
            };
            frame.fill_rectangle(
                thumb_rect.position(),
                thumb_rect.size(),
                thumb_color,
            );
        }

        vec![frame.into_geometry()]
    }
}

impl<'a> Widget<Message, cosmic::Theme, cosmic::Renderer> for EditorCanvas<'a> {
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<CanvasState>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(CanvasState::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &cosmic::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fill)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &cosmic::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<CanvasState>();
        let bounds = layout.bounds();

        if state.is_dragging_scrollbar {
            return mouse::Interaction::Pointer;
        }

        if let Some(pos) = cursor.position_in(bounds) {
            if pos.x >= bounds.width - 14.0 {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Text
            }
        } else {
            mouse::Interaction::None
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &cosmic::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<CanvasState>();

        // Enable IME when this editor pane is focused
        if self.is_focused {
            let gutter = self.gutter_width();
            let (cursor_x, cursor_y) = if let Some(pt) = self.cursor_screen_pos(bounds) {
                (
                    pt.x.clamp(0.0, bounds.width),
                    pt.y.clamp(0.0, (bounds.height - self.line_height).max(0.0)),
                )
            } else {
                (gutter + 10.0, 0.0)
            };

            let cursor_rect = Rectangle::new(
                Point::new(bounds.x + cursor_x, bounds.y + cursor_y),
                Size::new(self.char_width.max(2.0), self.line_height),
            );

            shell.request_input_method(&InputMethod::<&str>::Enabled {
                cursor: cursor_rect,
                purpose: cosmic::iced::advanced::input_method::Purpose::Normal,
                preedit: self.pane.preedit.as_ref().map(|(s, sel)| {
                    cosmic::iced::advanced::input_method::Preedit {
                        content: s.as_str(),
                        selection: sel.clone(),
                        text_size: Some(Pixels(self.font_size)),
                    }
                }),
            });
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let total_content_height = self.total_content_height();
                    let max_scroll = (total_content_height - bounds.height).max(0.0);

                    if total_content_height > bounds.height && pos.x >= bounds.width - 14.0 {
                        let thumb_height = ((bounds.height / total_content_height) * bounds.height)
                            .max(28.0)
                            .min(bounds.height);
                        let max_thumb_y = (bounds.height - thumb_height).max(0.0);
                        let cur_scroll_y = self.pane.scroll_y.get().clamp(0.0, max_scroll);
                        let thumb_y = if max_scroll > 0.0 {
                            (cur_scroll_y / max_scroll) * max_thumb_y
                        } else {
                            0.0
                        };

                        state.is_dragging_scrollbar = true;
                        if pos.y >= thumb_y && pos.y <= thumb_y + thumb_height {
                            state.scrollbar_drag_offset_y = pos.y - thumb_y;
                        } else {
                            state.scrollbar_drag_offset_y = thumb_height / 2.0;
                            let target_thumb_y = (pos.y - thumb_height / 2.0).clamp(0.0, max_thumb_y);
                            let new_scroll_y = if max_thumb_y > 0.0 {
                                (target_thumb_y / max_thumb_y) * max_scroll
                            } else {
                                0.0
                            };
                            shell.publish(Message::SetScrollY(self.pane.id, new_scroll_y));
                        }
                        shell.capture_event();
                    } else {
                        state.is_dragging = true;
                        let (target_line, target_col) = self.pos_to_char_coords(pos, bounds);
                        shell.publish(Message::ClickPane(
                            self.pane.id,
                            target_line,
                            target_col,
                        ));
                        shell.capture_event();
                    }
                }
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging_scrollbar {
                    if let Some(global_pos) = cursor.position() {
                        let rel_y = global_pos.y - bounds.y;
                        let total_content_height = self.total_content_height();
                        let max_scroll = (total_content_height - bounds.height).max(0.0);
                        if total_content_height > bounds.height && max_scroll > 0.0 {
                            let thumb_height = ((bounds.height / total_content_height) * bounds.height)
                                .max(28.0)
                                .min(bounds.height);
                            let max_thumb_y = (bounds.height - thumb_height).max(0.0);
                            let target_thumb_y = (rel_y - state.scrollbar_drag_offset_y).clamp(0.0, max_thumb_y);
                            let new_scroll_y = if max_thumb_y > 0.0 {
                                (target_thumb_y / max_thumb_y) * max_scroll
                            } else {
                                0.0
                            };
                            shell.publish(Message::SetScrollY(self.pane.id, new_scroll_y));
                            shell.capture_event();
                        }
                    }
                } else if state.is_dragging {
                    if let Some(pos) = cursor.position_in(bounds) {
                        let (line, col) = self.pos_to_char_coords(pos, bounds);
                        shell.publish(Message::DragSelect(self.pane.id, line, col));
                        shell.capture_event();
                    } else if let Some(global_pos) = cursor.position() {
                        let clamped_x = (global_pos.x - bounds.x).clamp(0.0, bounds.width);
                        let clamped_y = (global_pos.y - bounds.y).clamp(0.0, bounds.height);
                        let (line, col) = self.pos_to_char_coords(Point::new(clamped_x, clamped_y), bounds);
                        shell.publish(Message::DragSelect(self.pane.id, line, col));
                        shell.capture_event();
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                state.is_dragging_scrollbar = false;
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if pos.x < bounds.width - 14.0 {
                        if self.pane.buffer.selected_text().is_none() {
                            let (target_line, target_col) = self.pos_to_char_coords(pos, bounds);
                            shell.publish(Message::ClickPane(
                                self.pane.id,
                                target_line,
                                target_col,
                            ));
                        }
                        shell.publish(Message::OpenContextMenu(
                            self.pane.id,
                            bounds.x + pos.x,
                            bounds.y + pos.y,
                        ));
                        shell.capture_event();
                    }
                }
            }

            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let y_delta = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * self.line_height * 2.0,
                    mouse::ScrollDelta::Pixels { y, .. } => *y,
                };
                shell.publish(Message::ScrollPane(self.pane.id, -y_delta));
                shell.capture_event();
            }

            _ => {}
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut cosmic::Renderer,
        _theme: &cosmic::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let layers = self.draw_frame(renderer, bounds);
        renderer.with_layer(bounds, |renderer| {
            renderer.with_translation(
                Vector::new(bounds.x, bounds.y),
                |renderer| {
                    for layer in layers {
                        GeometryRenderer::draw_geometry(renderer, layer);
                    }
                },
            );
        });
    }
}

impl<'a> From<EditorCanvas<'a>> for Element<'a, Message, cosmic::Theme, cosmic::Renderer> {
    fn from(canvas: EditorCanvas<'a>) -> Self {
        Element::new(canvas)
    }
}
