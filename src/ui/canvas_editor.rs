use crate::editor::pane::EditorPane;
use crate::theme::EditorTheme;
use cosmic::iced::mouse;
use cosmic::iced::widget::canvas::{Action, Event, Frame, Geometry, Path, Program, Stroke, Text};
use cosmic::iced::{Color, Font, Pixels, Point, Rectangle};

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

    fn gutter_width(&self) -> f32 {
        let lines = self.pane.buffer.line_count().max(100);
        let digits = lines.to_string().len().max(3);
        (digits as f32) * self.char_width + 24.0
    }
}

impl<'a, Message: 'static> Program<Message, cosmic::Theme, cosmic::Renderer> for EditorCanvas<'a> {
    type State = CanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.is_dragging = true;
                None
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                None
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
        let total_lines = buffer.line_count();

        // 1. Draw Background (with Wayland alpha)
        let bg_color = self.theme.background_with_alpha();
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg_color);

        // Dimming overlay if configured
        if self.theme.dimming > 0.0 {
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::from_rgba(0.0, 0.0, 0.0, self.theme.dimming * 0.8),
            );
        }

        // 2. Draw Gutter Background
        let gutter_rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: gutter,
            height: bounds.height,
        };
        frame.fill_rectangle(gutter_rect.position(), gutter_rect.size(), self.theme.config.gutter_bg);

        // Gutter Divider Line
        frame.stroke(
            &Path::line(Point::new(gutter, 0.0), Point::new(gutter, bounds.height)),
            Stroke::default()
                .with_color(self.theme.config.border)
                .with_width(1.0),
        );

        // Visible range calculation
        let start_line = (self.pane.scroll_y / self.line_height).floor() as usize;
        let visible_lines = (bounds.height / self.line_height).ceil() as usize + 2;
        let end_line = (start_line + visible_lines).min(total_lines);

        let font_leak: &'static str = Box::leak(self.font_name.to_string().into_boxed_str());
        let font = Font::with_name(font_leak);
        let text_size = Pixels(self.font_size);

        // 3. Highlight Current Line
        if buffer.cursor.0 >= start_line && buffer.cursor.0 < end_line {
            let cur_y = (buffer.cursor.0 as f32) * self.line_height - self.pane.scroll_y;
            let cur_line_rect = Rectangle {
                x: gutter,
                y: cur_y,
                width: bounds.width - gutter,
                height: self.line_height,
            };
            frame.fill_rectangle(
                cur_line_rect.position(),
                cur_line_rect.size(),
                self.theme.config.current_line_bg,
            );
        }

        // 4. Selection Highlight
        if let Some(anchor) = buffer.selection_anchor {
            let (sel_start, sel_end) = if (buffer.cursor.0, buffer.cursor.1) < (anchor.0, anchor.1) {
                (buffer.cursor, anchor)
            } else {
                (anchor, buffer.cursor)
            };

            for line_idx in sel_start.0..=sel_end.0 {
                if line_idx >= start_line && line_idx < end_line {
                    let y = (line_idx as f32) * self.line_height - self.pane.scroll_y;
                    let line_chars = buffer.line_char_count(line_idx);

                    let start_col = if line_idx == sel_start.0 {
                        sel_start.1
                    } else {
                        0
                    };
                    let end_col = if line_idx == sel_end.0 {
                        sel_end.1
                    } else {
                        line_chars
                    };

                    if start_col < end_col {
                        let sel_x = gutter + (start_col as f32) * self.char_width - self.pane.scroll_x;
                        let sel_w = ((end_col - start_col) as f32) * self.char_width;

                        let sel_rect = Rectangle {
                            x: sel_x,
                            y,
                            width: sel_w,
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

        // 5. Draw Lines (Gutter line number & Syntax highlighted code)
        for line_idx in start_line..end_line {
            let y = (line_idx as f32) * self.line_height - self.pane.scroll_y;

            let line_num_str = format!("{}", line_idx + 1);
            let num_color = if line_idx == buffer.cursor.0 {
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

            if let Some(line_text) = buffer.line_text(line_idx) {
                let spans = self.pane.highlighter.highlight_line(&line_text, line_idx);

                if spans.is_empty() {
                    frame.fill_text(Text {
                        content: line_text,
                        position: Point::new(gutter + 10.0 - self.pane.scroll_x, y + 2.0),
                        color: self.theme.config.fg,
                        size: text_size,
                        font,
                        ..Default::default()
                    });
                } else {
                    let chars: Vec<char> = line_text.chars().collect();
                    let mut cur_col = 0;

                    for span in spans {
                        if span.start_col > cur_col && cur_col < chars.len() {
                            let unhighlighted: String = chars[cur_col..span.start_col.min(chars.len())].iter().collect();
                            let span_x = gutter + 10.0 + (cur_col as f32) * self.char_width - self.pane.scroll_x;
                            frame.fill_text(Text {
                                content: unhighlighted,
                                position: Point::new(span_x, y + 2.0),
                                color: self.theme.config.fg,
                                size: text_size,
                                font,
                                ..Default::default()
                            });
                        }

                        let start = span.start_col.min(chars.len());
                        let end = span.end_col.min(chars.len());
                        if start < end {
                            let span_text: String = chars[start..end].iter().collect();
                            let span_x = gutter + 10.0 + (start as f32) * self.char_width - self.pane.scroll_x;
                            let span_color = span.token_type.color(&self.theme.config);

                            frame.fill_text(Text {
                                content: span_text,
                                position: Point::new(span_x, y + 2.0),
                                color: span_color,
                                size: text_size,
                                font,
                                ..Default::default()
                            });
                        }
                        cur_col = end;
                    }

                    if cur_col < chars.len() {
                        let remaining: String = chars[cur_col..].iter().collect();
                        let span_x = gutter + 10.0 + (cur_col as f32) * self.char_width - self.pane.scroll_x;
                        frame.fill_text(Text {
                            content: remaining,
                            position: Point::new(span_x, y + 2.0),
                            color: self.theme.config.fg,
                            size: text_size,
                            font,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // 6. Draw Cursor (Caret)
        if self.is_focused && buffer.cursor.0 >= start_line && buffer.cursor.0 < end_line {
            let cursor_y = (buffer.cursor.0 as f32) * self.line_height - self.pane.scroll_y;
            let cursor_x = gutter + 10.0 + (buffer.cursor.1 as f32) * self.char_width - self.pane.scroll_x;

            let caret_rect = Rectangle {
                x: cursor_x,
                y: cursor_y + 1.0,
                width: 2.0,
                height: self.line_height - 2.0,
            };
            frame.fill_rectangle(
                caret_rect.position(),
                caret_rect.size(),
                self.theme.config.cursor,
            );

            // 7. Draw Local AI FIM Ghost Text (Inline Preview)
            if let Some(ref ghost) = self.pane.ghost_text {
                if !ghost.is_empty() {
                    let first_line = ghost.lines().next().unwrap_or(ghost);
                    let ghost_x = cursor_x + 2.0;

                    frame.fill_text(Text {
                        content: first_line.to_string(),
                        position: Point::new(ghost_x, cursor_y + 2.0),
                        color: self.theme.config.ghost_text,
                        size: text_size,
                        font,
                        ..Default::default()
                    });
                }
            }
        }

        // 8. Active Focused Border (in Split View)
        if self.is_focused {
            frame.stroke(
                &Path::rectangle(Point::ORIGIN, bounds.size()),
                Stroke::default()
                    .with_color(self.theme.config.accent)
                    .with_width(1.5),
            );
        }

        vec![frame.into_geometry()]
    }
}
