#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placeholder {
    pub index: usize,
    pub default_value: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub is_mirror: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSnippet {
    pub text: String,
    pub placeholders: Vec<Placeholder>,
    pub final_caret_char_offset: usize,
}

/// Parses a snippet template containing `$0`, `${1:default}`, `$1`, etc.
/// If `selected_text` is provided and non-empty, it replaces the default value of the first placeholder (`$1`).
pub fn parse_snippet_template(template: &str, selected_text: Option<&str>) -> ParsedSnippet {
    let mut out = String::with_capacity(template.len());
    let mut placeholders = Vec::new();
    let mut final_caret_char = None;
    let mut first_placeholder_applied = false;

    // Track default values by placeholder index so mirrored `$1` can reuse the value
    let mut default_values: std::collections::HashMap<usize, String> =
        std::collections::HashMap::new();

    let bytes = template.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'\\' && i + 1 < len && (bytes[i + 1] == b'$' || bytes[i + 1] == b'}') {
            // Escaped character: literal '$' or '}'
            out.push(bytes[i + 1] as char);
            i += 2;
            continue;
        }

        if bytes[i] == b'$' && i + 1 < len {
            if bytes[i + 1] == b'{' {
                // Potential ${N:default} or ${N}
                if let Some(close_brace) = find_matching_brace(template, i + 2) {
                    let inner = &template[i + 2..close_brace];
                    if let Some((idx_str, default_part)) = inner.split_once(':') {
                        if let Ok(idx) = idx_str.trim().parse::<usize>() {
                            let mut val = default_part.to_string();
                            if idx == 1 && !first_placeholder_applied {
                                if let Some(sel) = selected_text {
                                    if !sel.is_empty() {
                                        val = sel.to_string();
                                    }
                                }
                                first_placeholder_applied = true;
                            }
                            default_values.entry(idx).or_insert_with(|| val.clone());

                            let start_byte = out.len();
                            let start_char = out.chars().count();
                            out.push_str(&val);
                            let end_byte = out.len();
                            let end_char = out.chars().count();

                            if idx == 0 {
                                final_caret_char = Some(start_char);
                            } else {
                                placeholders.push(Placeholder {
                                    index: idx,
                                    default_value: val,
                                    start_byte,
                                    end_byte,
                                    start_char,
                                    end_char,
                                    is_mirror: false,
                                });
                            }

                            i = close_brace + 1;
                            continue;
                        }
                    } else if let Ok(idx) = inner.trim().parse::<usize>() {
                        // ${N} without default value
                        let val = default_values
                            .get(&idx)
                            .cloned()
                            .unwrap_or_default();
                        let start_byte = out.len();
                        let start_char = out.chars().count();
                        out.push_str(&val);
                        let end_byte = out.len();
                        let end_char = out.chars().count();

                        if idx == 0 {
                            final_caret_char = Some(start_char);
                        } else {
                            placeholders.push(Placeholder {
                                index: idx,
                                default_value: val,
                                start_byte,
                                end_byte,
                                start_char,
                                end_char,
                                is_mirror: default_values.contains_key(&idx),
                            });
                        }

                        i = close_brace + 1;
                        continue;
                    }
                }
            } else if bytes[i + 1].is_ascii_digit() {
                // $0 or $1, $2...
                let mut num_end = i + 1;
                while num_end < len && bytes[num_end].is_ascii_digit() {
                    num_end += 1;
                }
                let num_str = &template[i + 1..num_end];
                if let Ok(idx) = num_str.parse::<usize>() {
                    if idx == 0 {
                        final_caret_char = Some(out.chars().count());
                    } else {
                        let mut val = default_values.get(&idx).cloned().unwrap_or_default();
                        if idx == 1 && !first_placeholder_applied {
                            if let Some(sel) = selected_text {
                                if !sel.is_empty() {
                                    val = sel.to_string();
                                    default_values.insert(idx, val.clone());
                                }
                            }
                            first_placeholder_applied = true;
                        }
                        let start_byte = out.len();
                        let start_char = out.chars().count();
                        out.push_str(&val);
                        let end_byte = out.len();
                        let end_char = out.chars().count();

                        placeholders.push(Placeholder {
                            index: idx,
                            default_value: val,
                            start_byte,
                            end_byte,
                            start_char,
                            end_char,
                            is_mirror: default_values.contains_key(&idx),
                        });
                    }

                    i = num_end;
                    continue;
                }
            }
        }

        // Normal character (including '$$' for math block or '$' without digits)
        let ch = template[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }

    // Sort placeholders by index (1, 2, 3...)
    // Note: Stable sort preserves order for mirrors
    placeholders.sort_by_key(|p| p.index);

    let final_caret_char_offset = final_caret_char.unwrap_or_else(|| out.chars().count());

    ParsedSnippet {
        text: out,
        placeholders,
        final_caret_char_offset,
    }
}

fn find_matching_brace(s: &str, start_pos: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 1;
    let mut i = start_pos;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'{' {
            depth += 1;
        } else if bytes[i] == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

