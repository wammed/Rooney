use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct FontManager {
    pub available_fonts: Vec<String>,
    pub current_font: String,
    pub font_size: f32,
    pub line_height: f32,
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FontManager {
    pub fn new() -> Self {
        let mut families = BTreeSet::new();

        if let Ok(output) = std::process::Command::new("fc-list")
            .arg(":")
            .arg("family")
            .output()
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                for line in text.lines() {
                    for family in line.split(',') {
                        let trimmed = family.trim();
                        if !trimmed.is_empty() {
                            families.insert(trimmed.to_string());
                        }
                    }
                }
            }
        }

        let mut font_list: Vec<String> = families.into_iter().collect();

        // Prioritize Nerd Fonts and Monospace at the top
        font_list.sort_by(|a, b| {
            let a_is_nerd = a.to_lowercase().contains("nerd");
            let b_is_nerd = b.to_lowercase().contains("nerd");
            let a_is_mono = a.to_lowercase().contains("mono");
            let b_is_mono = b.to_lowercase().contains("mono");

            match (a_is_nerd, b_is_nerd) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => match (a_is_mono, b_is_mono) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.to_lowercase().cmp(&b.to_lowercase()),
                },
            }
        });

        let default_font = font_list
            .iter()
            .find(|f| f.contains("JetBrainsMono Nerd Font"))
            .or_else(|| font_list.iter().find(|f| f.contains("Hack Nerd Font")))
            .or_else(|| font_list.iter().find(|f| f.to_lowercase().contains("nerd font")))
            .or_else(|| font_list.iter().find(|f| f.to_lowercase().contains("mono")))
            .cloned()
            .unwrap_or_else(|| "monospace".to_string());

        Self {
            available_fonts: font_list,
            current_font: default_font,
            font_size: 14.0,
            line_height: 22.0,
        }
    }

    pub fn set_font(&mut self, font_name: String) {
        self.current_font = font_name;
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size.clamp(9.0, 36.0);
        self.line_height = (self.font_size * 1.5).round();
    }
}
