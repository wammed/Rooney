use crate::app::Message;
use crate::editor::pane::EditorPane;
use crate::theme::EditorTheme;
use cosmic::iced::mouse;
use cosmic::iced::widget::canvas::{Action, Event, Frame, Geometry, Path, Program, Stroke, Text};
use cosmic::iced::{Color, Font, Pixels, Point, Rectangle};
use unicode_width::UnicodeWidthChar;

#[derive(Default)]
pub struct CanvasState {
    pub is_dragging: bool,
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
struct VisualRow {
    line_idx: usize,
    char_start: usize,
    char_end: usize,
    is_first_subrow: bool,
}

impl<'a> EditorCanvas<'a> {
    pub fn new(
        pane: &'a EditorPane,
        theme: &'a EditorTheme,
        is_focused: bool,
        font_name: &'a str,
        font_size: f32,
    ) -> Self {
        let line_height = (font_size * 1.5).round();
        let char_width = (font_size * 0.60).round();

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

    fn build_visual_rows(&self, bounds_width: f32) -> Vec<VisualRow> {
        let gutter = self.gutter_width();
        let avail_width = (bounds_width - gutter - 24.0).max(120.0);
        let max_cols = ((avail_width / self.char_width).floor() as usize).max(10);

        let mut visual_rows = Vec::new();
        let total_lines = self.pane.buffer.line_count();

        for line_idx in 0..total_lines {
            let line_text = self.pane.buffer.line_text(line_idx).unwrap_or_default();
            let chars: Vec<char> = line_text.chars().collect();

            if chars.is_empty() {
                visual_rows.push(VisualRow {
                    line_idx,
                    char_start: 0,
                    char_end: 0,
                    is_first_subrow: true,
                });
                continue;
            }

            let mut start = 0;
            let mut cur_col_width = 0;
            let mut is_first = true;

            for (i, &c) in chars.iter().enumerate() {
                let w = c.width().unwrap_or(1).max(1);
                if cur_col_width + w > max_cols && i > start {
                    visual_rows.push(VisualRow {
                        line_idx,
                        char_start: start,
                        char_end: i,
                        is_first_subrow: is_first,
                    });
                    start = i;
                    cur_col_width = 0;
                    is_first = false;
                }
                cur_col_width += w;
            }

            visual_rows.push(VisualRow {
                line_idx,
                char_start: start,
                char_end: chars.len(),
                is_first_subrow: is_first,
            });
        }

        visual_rows
    }

    pub fn cursor_screen_pos(&self, bounds: Rectangle) -> Option<Point> {
        let gutter = self.gutter_width();
        let visual_rows = self.build_visual_rows(bounds.width);
        let cursor = self.pane.buffer.cursor;

        for (v_idx, row) in visual_rows.iter().enumerate() {
            if row.line_idx == cursor.0 && cursor.1 >= row.char_start && cursor.1 <= row.char_end {
                let y = (v_idx as f32) * self.line_height - self.pane.scroll_y;
                let line_text = self.pane.buffer.line_text(cursor.0).unwrap_or_default();
                let chars: Vec<char> = line_text.chars().collect();

                let mut col_offset = 0;
                for i in row.char_start..cursor.1.min(chars.len()) {
                    let w = chars[i].width().unwrap_or(1).max(1);
                    col_offset += w;
                }

                let x = gutter + 10.0 + (col_offset as f32) * self.char_width - self.pane.scroll_x;
                return Some(Point::new(x, y));
            }
        }
        None
    }
}

impl<'a> Program<Message, cosmic::Theme, cosmic::Renderer> for EditorCanvas<'a> {
    type State = CanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    state.is_dragging = true;
                    let gutter = self.gutter_width();
                    let visual_rows = self.build_visual_rows(bounds.width);

                    let clicked_v_idx = ((pos.y + self.pane.scroll_y) / self.line_height).floor() as isize;
                    let clicked_v_idx = clicked_v_idx.max(0) as usize;

                    let (target_line, target_col) = if let Some(row) = visual_rows.get(clicked_v_idx) {
                        let line_text = self.pane.buffer.line_text(row.line_idx).unwrap_or_default();
                        let chars: Vec<char> = line_text.chars().collect();
                        let rel_x = (pos.x - gutter - 10.0 + self.pane.scroll_x).max(0.0);

                        let mut acc_width = 0.0;
                        let mut chosen_col = row.char_start;

                        for i in row.char_start..row.char_end.min(chars.len()) {
                            let w = chars[i].width().unwrap_or(1).max(1);
                            let char_pixel_w = (w as f32) * self.char_width;
                            if acc_width + char_pixel_w / 2.0 >= rel_x {
                                break;
                            }
                            acc_width += char_pixel_w;
                            chosen_col = i + 1;
                        }
                        (row.line_idx, chosen_col)
                    } else if let Some(last) = visual_rows.last() {
                        (last.line_idx, last.char_end)
                    } else {
                        (0, 0)
                    };

                    return Some(Action::publish(Message::ClickPane(
                        self.pane.id,
                        target_line,
                        target_col,
                    )));
                }
                None
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                None
            }

            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let y_delta = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * self.line_height * 2.0,
                    mouse::ScrollDelta::Pixels { y, .. } => *y,
                };
                Some(Action::publish(Message::ScrollPane(self.pane.id, -y_delta)))
            }

            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &cosmic::Renderer,
        _theme: &cosmic::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
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

        let font_leak: &'static str = Box::leak(self.font_name.to_string().into_boxed_str());
        let font = Font::with_name(font_leak);
        let text_size = Pixels(self.font_size);

        // 3. Build soft-wrapped visual rows
        let visual_rows = self.build_visual_rows(bounds.width);

        // 4. Current Line highlight
        for (v_idx, row) in visual_rows.iter().enumerate() {
            if row.line_idx == buffer.cursor.0 {
                let y = (v_idx as f32) * self.line_height - self.pane.scroll_y;
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

        // 5. Draw lines (Gutter and soft-wrapped text)
        for (v_idx, row) in visual_rows.iter().enumerate() {
            let y = (v_idx as f32) * self.line_height - self.pane.scroll_y;

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
                let chars: Vec<char> = line_text.chars().collect();
                if row.char_start < chars.len() {
                    let sub_end = row.char_end.min(chars.len());
                    let spans = self.pane.highlighter.highlight_line(&line_text, row.line_idx);

                    let mut cur_col = row.char_start;
                    let mut cur_pixel_x = gutter + 10.0 - self.pane.scroll_x;

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

                        let segment: String = chars[cur_col..token_end].iter().collect();
                        let mut seg_cols = 0;
                        for ch in chars[cur_col..token_end].iter() {
                            seg_cols += ch.width().unwrap_or(1).max(1);
                        }

                        frame.fill_text(Text {
                            content: segment,
                            position: Point::new(cur_pixel_x, y + 2.0),
                            color: token_color,
                            size: text_size,
                            font,
                            ..Default::default()
                        });

                        cur_pixel_x += (seg_cols as f32) * self.char_width;
                        cur_col = token_end;
                    }
                }
            }
        }

        // 6. Draw Cursor (Caret) & IME Preedit
        if self.is_focused {
            if let Some(cursor_pt) = self.cursor_screen_pos(bounds) {
                if cursor_pt.y + self.line_height >= 0.0 && cursor_pt.y <= bounds.height {
                    let caret_rect = Rectangle {
                        x: cursor_pt.x,
                        y: cursor_pt.y + 1.0,
                        width: 2.0,
                        height: self.line_height - 2.0,
                    };
                    frame.fill_rectangle(
                        caret_rect.position(),
                        caret_rect.size(),
                        self.theme.config.cursor,
                    );

                    // 7. Render IME Preedit text (Composing Japanese string)
                    let mut preedit_offset_x = 0.0;
                    if let Some((ref preedit_str, _)) = self.pane.preedit {
                        if !preedit_str.is_empty() {
                            let preedit_x = cursor_pt.x + 3.0;
                            let mut preedit_cols = 0;
                            for c in preedit_str.chars() {
                                preedit_cols += c.width().unwrap_or(1).max(1);
                            }
                            let preedit_w = (preedit_cols as f32) * self.char_width;

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

                            preedit_offset_x = preedit_w + 4.0;
                        }
                    }

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

        vec![frame.into_geometry()]
    }
}
