use crate::editor::buffer::TextBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedLineInfo {
    pub line_idx: usize,
    pub subrow_count: usize,
    pub extra_rows: usize, // subrow_count - 1
    pub cum_extra_before: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineWrapModel {
    pub wrapped_lines: Vec<WrappedLineInfo>,
    pub total_extra_rows: usize,
}

impl LineWrapModel {
    pub fn new() -> Self {
        Self {
            wrapped_lines: Vec::new(),
            total_extra_rows: 0,
        }
    }

    pub fn total_visual_rows(&self, total_logical_lines: usize) -> usize {
        total_logical_lines + self.total_extra_rows
    }

    pub fn line_to_visual_row(&self, line_idx: usize) -> usize {
        match self
            .wrapped_lines
            .binary_search_by_key(&line_idx, |w| w.line_idx)
        {
            Ok(idx) => line_idx + self.wrapped_lines[idx].cum_extra_before,
            Err(idx) => {
                if idx == 0 {
                    line_idx
                } else {
                    let prev = &self.wrapped_lines[idx - 1];
                    line_idx + prev.cum_extra_before + prev.extra_rows
                }
            }
        }
    }

    pub fn visual_row_to_line(&self, vrow_idx: usize) -> (usize, usize) {
        if self.wrapped_lines.is_empty() {
            return (vrow_idx, 0);
        }
        let idx = self.wrapped_lines.partition_point(|w| {
            let v_start = w.line_idx + w.cum_extra_before;
            v_start <= vrow_idx
        });
        if idx == 0 {
            return (vrow_idx, 0);
        }
        let w = &self.wrapped_lines[idx - 1];
        let v_start = w.line_idx + w.cum_extra_before;
        let v_end = v_start + w.subrow_count;
        if vrow_idx < v_end {
            (w.line_idx, vrow_idx - v_start)
        } else {
            let cum_extra = w.cum_extra_before + w.extra_rows;
            (vrow_idx.saturating_sub(cum_extra), 0)
        }
    }

    pub fn subrow_count(&self, line_idx: usize) -> usize {
        match self
            .wrapped_lines
            .binary_search_by_key(&line_idx, |w| w.line_idx)
        {
            Ok(idx) => self.wrapped_lines[idx].subrow_count,
            Err(_) => 1,
        }
    }

    pub fn build(
        buffer: &TextBuffer,
        avail_width: f32,
        glyph_advance: impl Fn(char) -> f32,
    ) -> Self {
        let total_lines = buffer.line_count();
        if total_lines == 0 {
            return Self::new();
        }

        let mut wrapped_lines = Vec::new();
        let mut total_extra = 0;

        // Minimum bytes for a line to possibly exceed avail_width
        // ASCII char is 1 byte, minimum width ~5.0px at small font sizes
        let min_wrap_bytes = (avail_width / 14.0).floor() as usize;

        // O(1) fast path: if the longest line in the buffer cannot exceed avail_width,
        // no lines in the buffer can possibly wrap. Zero iterations, zero allocations.
        if buffer.max_line_len < min_wrap_bytes {
            return Self::new();
        }

        for (line_idx, line_slice) in buffer.rope.lines().enumerate() {
            // Zero-allocation check on line byte length
            if line_slice.len_bytes() >= min_wrap_bytes {
                let line_str = line_slice.to_string();
                let line_trimmed = line_str.trim_end_matches(['\r', '\n']);
                let subrows = compute_line_subrows(line_trimmed, avail_width, &glyph_advance);
                if subrows.len() > 1 {
                    let subrow_count = subrows.len();
                    let extra = subrow_count - 1;
                    wrapped_lines.push(WrappedLineInfo {
                        line_idx,
                        subrow_count,
                        extra_rows: extra,
                        cum_extra_before: total_extra,
                    });
                    total_extra += extra;
                }
            }
        }

        Self {
            wrapped_lines,
            total_extra_rows: total_extra,
        }
    }
}

pub fn compute_line_subrows(
    line_text: &str,
    avail_width: f32,
    glyph_advance: impl Fn(char) -> f32,
) -> Vec<(usize, usize)> {
    let line_text = line_text.trim_end_matches(['\r', '\n']);
    if line_text.is_empty() {
        return vec![(0, 0)];
    }

    let mut subrows = Vec::new();
    let mut start_char_idx = 0;
    let mut cur_row_w = 0.0;
    let mut char_count = 0;

    for (_, c) in line_text.char_indices() {
        let w = glyph_advance(c);
        if cur_row_w + w > avail_width && char_count > start_char_idx {
            subrows.push((start_char_idx, char_count));
            start_char_idx = char_count;
            cur_row_w = 0.0;
        }
        cur_row_w += w;
        char_count += 1;
    }
    subrows.push((start_char_idx, char_count));
    subrows
}
