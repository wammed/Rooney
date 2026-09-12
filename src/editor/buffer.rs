use ropey::Rope;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct TextBuffer {
    pub rope: Rope,
    pub cursor: (usize, usize), // (line_idx, col_idx)
    pub selection_anchor: Option<(usize, usize)>,
    pub is_modified: bool,
    undo_stack: VecDeque<Rope>,
    redo_stack: VecDeque<Rope>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new("")
    }
}

impl TextBuffer {
    pub fn new(initial_text: &str) -> Self {
        let rope = Rope::from_str(initial_text);
        Self {
            rope,
            cursor: (0, 0),
            selection_anchor: None,
            is_modified: false,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line_text(&self, line_idx: usize) -> Option<String> {
        if line_idx < self.rope.len_lines() {
            let slice = self.rope.line(line_idx);
            let mut s = slice.to_string();
            if s.ends_with('\n') {
                s.pop();
                if s.ends_with('\r') {
                    s.pop();
                }
            }
            Some(s)
        } else {
            None
        }
    }

    pub fn full_text(&self) -> String {
        self.rope.to_string()
    }

    pub fn char_index(&self, line: usize, col: usize) -> usize {
        let line = line.min(self.rope.len_lines().saturating_sub(1));
        let line_char_start = self.rope.line_to_char(line);
        let line_len = self.rope.line(line).len_chars();
        let col = col.min(line_len);
        line_char_start + col
    }

    pub fn undo_stack_len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_stack_len(&self) -> usize {
        self.redo_stack.len()
    }

    fn push_undo(&mut self) {
        if self.undo_stack.len() >= 100 {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(self.rope.clone());
        self.redo_stack.clear();
        self.is_modified = true;
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(self.rope.clone());
            self.rope = prev;
            self.clamp_cursor();
            self.selection_anchor = None;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(self.rope.clone());
            self.rope = next;
            self.clamp_cursor();
            self.selection_anchor = None;
        }
    }

    pub fn clamp_cursor(&mut self) {
        let line_count = self.rope.len_lines();
        if line_count == 0 {
            self.cursor = (0, 0);
            return;
        }
        self.cursor.0 = self.cursor.0.min(line_count - 1);
        let line_chars = self.line_char_count(self.cursor.0);
        self.cursor.1 = self.cursor.1.min(line_chars);
    }

    pub fn line_char_count(&self, line: usize) -> usize {
        if line >= self.rope.len_lines() {
            return 0;
        }
        let slice = self.rope.line(line);
        let s = slice.to_string();
        s.trim_end_matches(&['\r', '\n'][..]).chars().count()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.delete_selection();
        self.push_undo();

        let char_idx = self.char_index(self.cursor.0, self.cursor.1);
        self.rope.insert_char(char_idx, ch);

        if ch == '\n' {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        } else {
            self.cursor.1 += 1;
        }
        self.selection_anchor = None;
    }

    pub fn insert_str(&mut self, s: &str) {
        self.delete_selection();
        self.push_undo();

        let char_idx = self.char_index(self.cursor.0, self.cursor.1);
        self.rope.insert(char_idx, s);

        let lines_added = s.chars().filter(|&c| c == '\n').count();
        if lines_added > 0 {
            self.cursor.0 += lines_added;
            let last_line = s.split('\n').next_back().unwrap_or("");
            self.cursor.1 = last_line.chars().count();
        } else {
            self.cursor.1 += s.chars().count();
        }
        self.selection_anchor = None;
    }

    pub fn delete_backspace(&mut self) {
        if self.delete_selection() {
            return;
        }

        let char_idx = self.char_index(self.cursor.0, self.cursor.1);
        if char_idx > 0 {
            self.push_undo();
            self.rope.remove(char_idx - 1..char_idx);

            if self.cursor.1 > 0 {
                self.cursor.1 -= 1;
            } else if self.cursor.0 > 0 {
                self.cursor.0 -= 1;
                self.cursor.1 = self.line_char_count(self.cursor.0);
            }
        }
    }

    pub fn delete_forward(&mut self) {
        if self.delete_selection() {
            return;
        }

        let char_idx = self.char_index(self.cursor.0, self.cursor.1);
        if char_idx < self.rope.len_chars() {
            self.push_undo();
            self.rope.remove(char_idx..char_idx + 1);
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        let Some(anchor) = self.selection_anchor else {
            return false;
        };

        if anchor == self.cursor {
            self.selection_anchor = None;
            return false;
        }

        self.push_undo();
        let (start, end) = if (self.cursor.0, self.cursor.1) < (anchor.0, anchor.1) {
            (self.cursor, anchor)
        } else {
            (anchor, self.cursor)
        };

        let start_idx = self.char_index(start.0, start.1);
        let end_idx = self.char_index(end.0, end.1);

        if start_idx < end_idx && end_idx <= self.rope.len_chars() {
            self.rope.remove(start_idx..end_idx);
        }

        self.cursor = start;
        self.selection_anchor = None;
        true
    }

    pub fn move_left(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.line_char_count(self.cursor.0);
        }
    }

    pub fn move_right(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let line_len = self.line_char_count(self.cursor.0);
        if self.cursor.1 < line_len {
            self.cursor.1 += 1;
        } else if self.cursor.0 + 1 < self.rope.len_lines() {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        }
    }

    pub fn move_up(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            let line_len = self.line_char_count(self.cursor.0);
            self.cursor.1 = self.cursor.1.min(line_len);
        }
    }

    pub fn move_down(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        if self.cursor.0 + 1 < self.rope.len_lines() {
            self.cursor.0 += 1;
            let line_len = self.line_char_count(self.cursor.0);
            self.cursor.1 = self.cursor.1.min(line_len);
        }
    }

    pub fn move_line_start(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor.1 = 0;
    }

    pub fn move_line_end(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor.1 = self.line_char_count(self.cursor.0);
    }

    pub fn select_all(&mut self) {
        self.selection_anchor = Some((0, 0));
        let last_line = self.rope.len_lines().saturating_sub(1);
        let last_col = self.line_char_count(last_line);
        self.cursor = (last_line, last_col);
    }

    pub fn selected_text(&self) -> Option<String> {
        let anchor = self.selection_anchor?;
        if anchor == self.cursor {
            return None;
        }
        let (start, end) = if (self.cursor.0, self.cursor.1) < (anchor.0, anchor.1) {
            (self.cursor, anchor)
        } else {
            (anchor, self.cursor)
        };

        let start_idx = self.char_index(start.0, start.1);
        let end_idx = self.char_index(end.0, end.1);

        if start_idx < end_idx && end_idx <= self.rope.len_chars() {
            Some(self.rope.slice(start_idx..end_idx).to_string())
        } else {
            None
        }
    }

    pub fn get_fim_prefix_suffix(&self, max_context_chars: usize) -> (String, String) {
        let cursor_idx = self.char_index(self.cursor.0, self.cursor.1);
        let total_chars = self.rope.len_chars();

        let prefix_start = cursor_idx.saturating_sub(max_context_chars);
        let prefix = self.rope.slice(prefix_start..cursor_idx).to_string();

        let suffix_end = (cursor_idx + max_context_chars).min(total_chars);
        let suffix = self.rope.slice(cursor_idx..suffix_end).to_string();

        (prefix, suffix)
    }

    pub fn move_word_left(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        if self.cursor.1 == 0 {
            if self.cursor.0 > 0 {
                self.cursor.0 -= 1;
                self.cursor.1 = self.line_char_count(self.cursor.0);
            }
            return;
        }

        let Some(line) = self.line_text(self.cursor.0) else {
            return;
        };
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor.1.min(chars.len());

        while col > 0 && chars[col - 1].is_whitespace() {
            col -= 1;
        }
        if col > 0 && !chars[col - 1].is_alphanumeric() && chars[col - 1] != '_' {
            while col > 0
                && !chars[col - 1].is_alphanumeric()
                && chars[col - 1] != '_'
                && !chars[col - 1].is_whitespace()
            {
                col -= 1;
            }
        } else {
            while col > 0 && (chars[col - 1].is_alphanumeric() || chars[col - 1] == '_') {
                col -= 1;
            }
        }
        self.cursor.1 = col;
    }

    pub fn move_word_right(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let line_len = self.line_char_count(self.cursor.0);
        if self.cursor.1 >= line_len {
            if self.cursor.0 + 1 < self.rope.len_lines() {
                self.cursor.0 += 1;
                self.cursor.1 = 0;
            }
            return;
        }

        let Some(line) = self.line_text(self.cursor.0) else {
            return;
        };
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor.1;

        if col < chars.len()
            && !chars[col].is_alphanumeric()
            && chars[col] != '_'
            && !chars[col].is_whitespace()
        {
            while col < chars.len()
                && !chars[col].is_alphanumeric()
                && chars[col] != '_'
                && !chars[col].is_whitespace()
            {
                col += 1;
            }
        } else {
            while col < chars.len() && (chars[col].is_alphanumeric() || chars[col] == '_') {
                col += 1;
            }
        }
        while col < chars.len() && chars[col].is_whitespace() {
            col += 1;
        }
        self.cursor.1 = col;
    }

    pub fn delete_line(&mut self) {
        self.push_undo();
        let line = self.cursor.0;
        if self.rope.len_lines() == 0 {
            return;
        }
        let start_idx = self.rope.line_to_char(line);
        let end_idx = if line + 1 < self.rope.len_lines() {
            self.rope.line_to_char(line + 1)
        } else {
            self.rope.len_chars()
        };

        if start_idx < end_idx {
            self.rope.remove(start_idx..end_idx);
        }
        self.selection_anchor = None;
        self.clamp_cursor();
    }

    pub fn duplicate_line(&mut self) {
        let line = self.cursor.0;
        if let Some(line_str) = self.line_text(line) {
            self.push_undo();
            let insert_idx = if line + 1 < self.rope.len_lines() {
                self.rope.line_to_char(line + 1)
            } else {
                let len = self.rope.len_chars();
                self.rope.insert_char(len, '\n');
                self.rope.len_chars()
            };
            self.rope.insert(insert_idx, &format!("{}\n", line_str));
            self.cursor.0 += 1;
            self.clamp_cursor();
        }
    }

    pub fn toggle_comment(&mut self, prefix: &str) {
        let (start_line, end_line) = if let Some(anchor) = self.selection_anchor {
            (self.cursor.0.min(anchor.0), self.cursor.0.max(anchor.0))
        } else {
            (self.cursor.0, self.cursor.0)
        };

        self.push_undo();
        let comment_str = format!("{} ", prefix);

        let mut all_commented = true;
        for l in start_line..=end_line {
            if let Some(txt) = self.line_text(l) {
                let trimmed = txt.trim_start();
                if !trimmed.is_empty() && !trimmed.starts_with(prefix) {
                    all_commented = false;
                    break;
                }
            }
        }

        for l in start_line..=end_line {
            if let Some(txt) = self.line_text(l) {
                let line_start = self.rope.line_to_char(l);
                if all_commented {
                    if let Some(pos) = txt.find(prefix) {
                        let to_remove = if txt[pos..].starts_with(&comment_str) {
                            comment_str.len()
                        } else {
                            prefix.len()
                        };
                        let char_pos = txt[..pos].chars().count();
                        self.rope
                            .remove(line_start + char_pos..line_start + char_pos + to_remove);
                    }
                } else {
                    let indent = txt.len() - txt.trim_start().len();
                    let char_indent = txt[..indent].chars().count();
                    self.rope.insert(line_start + char_indent, &comment_str);
                }
            }
        }
        self.clamp_cursor();
    }

    pub fn indent_selection(&mut self) {
        let (start_line, end_line) = if let Some(anchor) = self.selection_anchor {
            (self.cursor.0.min(anchor.0), self.cursor.0.max(anchor.0))
        } else {
            (self.cursor.0, self.cursor.0)
        };

        self.push_undo();
        for l in start_line..=end_line {
            let line_start = self.rope.line_to_char(l);
            self.rope.insert(line_start, "    ");
        }
        self.cursor.1 += 4;
        self.clamp_cursor();
    }

    pub fn unindent_selection(&mut self) {
        let (start_line, end_line) = if let Some(anchor) = self.selection_anchor {
            (self.cursor.0.min(anchor.0), self.cursor.0.max(anchor.0))
        } else {
            (self.cursor.0, self.cursor.0)
        };

        self.push_undo();
        for l in start_line..=end_line {
            if let Some(txt) = self.line_text(l) {
                let spaces = txt.chars().take_while(|&c| c == ' ').count().min(4);
                if spaces > 0 {
                    let line_start = self.rope.line_to_char(l);
                    self.rope.remove(line_start..line_start + spaces);
                }
            }
        }
        self.cursor.1 = self.cursor.1.saturating_sub(4);
        self.clamp_cursor();
    }
}
