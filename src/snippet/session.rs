use super::parser::parse_snippet_template;
use ropey::Rope;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivePlaceholder {
    pub index: usize,
    pub char_range: Range<usize>,
    pub is_mirror: bool,
}

#[derive(Debug, Clone)]
pub struct PlaceholderSession {
    pub placeholders: Vec<ActivePlaceholder>,
    pub current_step: usize,
    pub final_caret_char_idx: usize,
}

impl PlaceholderSession {
    /// Creates a new placeholder session based on a template and insertion position.
    pub fn new(
        template: &str,
        insert_char_idx: usize,
        selected_text: Option<&str>,
    ) -> (String, Option<Self>) {
        let parsed = parse_snippet_template(template, selected_text);

        if parsed.placeholders.is_empty() {
            let _final_caret = insert_char_idx + parsed.final_caret_char_offset;
            return (parsed.text, None);
        }

        let mut active_list = Vec::with_capacity(parsed.placeholders.len());
        for p in &parsed.placeholders {
            active_list.push(ActivePlaceholder {
                index: p.index,
                char_range: (insert_char_idx + p.start_char)..(insert_char_idx + p.end_char),
                is_mirror: p.is_mirror,
            });
        }

        let final_caret = insert_char_idx + parsed.final_caret_char_offset;

        let session = Self {
            placeholders: active_list,
            current_step: 0,
            final_caret_char_idx: final_caret,
        };

        (parsed.text, Some(session))
    }

    /// Returns the unique placeholder indices in order of navigation (e.g. [1, 2, 3]).
    pub fn unique_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self.placeholders.iter().map(|p| p.index).collect();
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    /// Returns the current active placeholder's character range, if any.
    pub fn current_range(&self) -> Option<Range<usize>> {
        let indices = self.unique_indices();
        let target_idx = *indices.get(self.current_step)?;
        self.placeholders
            .iter()
            .find(|p| p.index == target_idx && !p.is_mirror)
            .map(|p| p.char_range.clone())
    }

    /// Advances to the next placeholder. If at the end, returns None indicating transition to $0.
    pub fn next_step(&mut self) -> Option<Range<usize>> {
        let indices = self.unique_indices();
        if self.current_step + 1 < indices.len() {
            self.current_step += 1;
            self.current_range()
        } else {
            None
        }
    }

    /// Moves back to the previous placeholder.
    pub fn prev_step(&mut self) -> Option<Range<usize>> {
        if self.current_step > 0 {
            self.current_step -= 1;
            self.current_range()
        } else {
            self.current_range()
        }
    }

    /// Adjusts placeholder offsets when text is edited in the buffer.
    pub fn adjust_offsets(&mut self, edit_start: usize, old_len: usize, new_len: usize) {
        let delta = (new_len as isize) - (old_len as isize);
        if delta == 0 {
            return;
        }

        let indices = self.unique_indices();
        let current_target_idx = indices.get(self.current_step).copied();

        for p in &mut self.placeholders {
            if Some(p.index) == current_target_idx && !p.is_mirror {
                // Currently active placeholder expanded or contracted
                let new_end = ((p.char_range.end as isize) + delta).max(p.char_range.start as isize) as usize;
                p.char_range.end = new_end;
            } else if p.char_range.start >= edit_start {
                // Subsequent placeholders shift right or left
                let new_start = ((p.char_range.start as isize) + delta).max(0) as usize;
                let new_end = ((p.char_range.end as isize) + delta).max(new_start as isize) as usize;
                p.char_range = new_start..new_end;
            }
        }

        if self.final_caret_char_idx >= edit_start {
            self.final_caret_char_idx =
                ((self.final_caret_char_idx as isize) + delta).max(0) as usize;
        }
    }
}

/// Converts a character index to (line, col) in a Rope.
pub fn char_to_line_col(rope: &Rope, char_idx: usize) -> (usize, usize) {
    let char_idx = char_idx.min(rope.len_chars());
    let line = rope.char_to_line(char_idx);
    let line_start_char = rope.line_to_char(line);
    let col = char_idx.saturating_sub(line_start_char);
    (line, col)
}
