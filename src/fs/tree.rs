use cosmic::iced::Color;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileTypeIcon {
    pub glyph: &'static str,
    pub color: Color,
}

impl FileTypeIcon {
    pub fn for_path(path: &Path, is_dir: bool, is_expanded: bool) -> Self {
        if is_dir {
            return if is_expanded {
                Self {
                    glyph: "",
                    color: Color::from_rgb(0.95, 0.75, 0.3),
                }
            } else {
                Self {
                    glyph: "",
                    color: Color::from_rgb(0.90, 0.70, 0.25),
                }
            };
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name.starts_with(".git") || file_name == ".gitignore" {
            return Self {
                glyph: "",
                color: Color::from_rgb(0.95, 0.35, 0.2),
            };
        }

        match ext.as_str() {
            "rs" => Self {
                glyph: "",
                color: Color::from_rgb(0.98, 0.45, 0.15),
            },
            "toml" => Self {
                glyph: "",
                color: Color::from_rgb(0.65, 0.4, 0.8),
            },
            "md" | "markdown" => Self {
                glyph: "",
                color: Color::from_rgb(0.2, 0.75, 0.95),
            },
            "py" | "pyi" => Self {
                glyph: "",
                color: Color::from_rgb(0.25, 0.6, 0.85),
            },
            "js" | "mjs" | "cjs" => Self {
                glyph: "",
                color: Color::from_rgb(0.95, 0.85, 0.2),
            },
            "ts" | "mts" | "cts" => Self {
                glyph: "",
                color: Color::from_rgb(0.2, 0.55, 0.85),
            },
            "jsx" | "tsx" => Self {
                glyph: "",
                color: Color::from_rgb(0.3, 0.8, 0.95),
            },
            "fish" => Self {
                glyph: "󰈺",
                color: Color::from_rgb(0.2, 0.85, 0.65),
            },
            "yaml" | "yml" => Self {
                glyph: "",
                color: Color::from_rgb(0.8, 0.45, 0.85),
            },
            "ini" | "conf" | "cfg" => Self {
                glyph: "",
                color: Color::from_rgb(0.65, 0.7, 0.75),
            },
            "json" | "jsonc" => Self {
                glyph: "",
                color: Color::from_rgb(0.85, 0.85, 0.25),
            },
            "html" => Self {
                glyph: "",
                color: Color::from_rgb(0.95, 0.4, 0.2),
            },
            "css" | "scss" => Self {
                glyph: "",
                color: Color::from_rgb(0.25, 0.5, 0.95),
            },
            "sh" | "bash" | "zsh" => Self {
                glyph: "",
                color: Color::from_rgb(0.4, 0.9, 0.3),
            },
            "c" | "h" => Self {
                glyph: "",
                color: Color::from_rgb(0.35, 0.55, 0.85),
            },
            "cpp" | "hpp" | "cc" | "cxx" => Self {
                glyph: "",
                color: Color::from_rgb(0.15, 0.45, 0.8),
            },
            "png" | "jpg" | "jpeg" | "webp" | "svg" | "gif" => Self {
                glyph: "󰋩",
                color: Color::from_rgb(0.7, 0.45, 0.85),
            },
            "lock" => Self {
                glyph: "",
                color: Color::from_rgb(0.65, 0.65, 0.65),
            },
            "txt" => Self {
                glyph: "",
                color: Color::from_rgb(0.8, 0.8, 0.8),
            },
            _ => Self {
                glyph: "",
                color: Color::from_rgb(0.75, 0.75, 0.8),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub is_expanded: bool,
    pub icon: FileTypeIcon,
}

#[derive(Debug, Clone)]
pub struct FileTree {
    pub root: PathBuf,
    pub expanded_dirs: HashSet<PathBuf>,
    pub selected_path: Option<PathBuf>,
    pub items: Vec<FileItem>,
    pub is_visible: bool,
    pub width: f32,
}

impl FileTree {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let root = root
            .as_ref()
            .canonicalize()
            .unwrap_or_else(|_| root.as_ref().to_path_buf());
        let mut expanded_dirs = HashSet::new();
        expanded_dirs.insert(root.clone());

        let mut tree = Self {
            root,
            expanded_dirs,
            selected_path: None,
            items: Vec::new(),
            is_visible: true,
            width: 280.0,
        };

        tree.refresh();
        tree
    }

    pub fn refresh(&mut self) {
        let mut items = Vec::new();
        Self::scan_dir(&self.root, 0, &self.expanded_dirs, &mut items);
        self.items = items;
    }

    fn scan_dir(
        dir: &Path,
        depth: usize,
        expanded: &HashSet<PathBuf>,
        out: &mut Vec<FileItem>,
    ) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if matches!(
                name.as_str(),
                ".git"
                    | "target"
                    | "node_modules"
                    | ".venv"
                    | "venv"
                    | "__pycache__"
                    | "dist"
                    | "build"
                    | ".idea"
                    | ".vscode"
                    | ".next"
            ) {
                continue;
            }

            if path.is_dir() {
                dirs.push((name, path));
            } else {
                files.push((name, path));
            }
        }

        dirs.sort_by_key(|a| a.0.to_lowercase());
        files.sort_by_key(|a| a.0.to_lowercase());

        for (name, path) in dirs {
            let is_expanded = expanded.contains(&path);
            let icon = FileTypeIcon::for_path(&path, true, is_expanded);

            out.push(FileItem {
                path: path.clone(),
                name,
                is_dir: true,
                depth,
                is_expanded,
                icon,
            });

            if is_expanded {
                Self::scan_dir(&path, depth + 1, expanded, out);
            }
        }

        for (name, path) in files {
            let icon = FileTypeIcon::for_path(&path, false, false);
            out.push(FileItem {
                path,
                name,
                is_dir: false,
                depth,
                is_expanded: false,
                icon,
            });
        }
    }

    pub fn toggle_dir(&mut self, path: &Path) {
        if self.expanded_dirs.contains(path) {
            self.expanded_dirs.remove(path);
        } else {
            self.expanded_dirs.insert(path.to_path_buf());
        }
        self.refresh();
    }

    pub fn select(&mut self, path: PathBuf) {
        self.selected_path = Some(path);
    }

    pub fn set_root(&mut self, new_root: PathBuf) {
        let root = new_root.canonicalize().unwrap_or(new_root);
        self.root = root.clone();
        self.expanded_dirs.clear();
        self.expanded_dirs.insert(root);
        self.selected_path = None;
        self.refresh();
    }

    pub fn go_to_parent(&mut self) -> bool {
        if let Some(parent) = self.root.parent() {
            let parent = parent.to_path_buf();
            self.set_root(parent);
            true
        } else {
            false
        }
    }
}
