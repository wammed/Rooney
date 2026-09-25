use std::path::{Component, Path, PathBuf};
use url::Url;

/// Normalizes a path:
/// 1. If the path exists on disk, `canonicalize()` is used to resolve symlinks and real location.
/// 2. If it does not exist (e.g. a new file to be created), ensures it is absolute
///    and resolves `.` and `..` components logically without failing.
pub fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }

    let path_str = path.to_string_lossy();
    let abs_path = if path_str.starts_with("~/") || path_str == "~" {
        if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.home_dir().join(path_str.trim_start_matches("~/"))
        } else {
            path.to_path_buf()
        }
    } else if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };

    let mut components = Vec::new();
    for comp in abs_path.components() {
        match comp {
            Component::Prefix(..) => components.push(comp),
            Component::RootDir => {
                components.clear();
                components.push(comp);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(last) = components.last() {
                    if !matches!(last, Component::RootDir | Component::Prefix(..)) {
                        components.pop();
                    }
                }
            }
            Component::Normal(..) => components.push(comp),
        }
    }
    components.into_iter().collect()
}

/// Parses a command line argument string into a normalized local PathBuf.
/// Handles:
/// - `file://` URIs (with WHATWG URL parsing and percent-decoding)
/// - Relative paths (resolved against current working directory)
/// - Absolute paths
/// - Tilde (`~`) paths
pub fn parse_path_or_uri(input: &str) -> Option<PathBuf> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check if it's a URL with a file scheme (e.g. file:///path/to/file or file://localhost/path)
    if trimmed.starts_with("file:") {
        if let Ok(url) = Url::parse(trimmed) {
            if let Ok(path) = url.to_file_path() {
                return Some(normalize_path(&path));
            }
        }
        // Fallback for non-standard file: URIs
        let is_triple_slash =
            trimmed.starts_with("file:///") || trimmed.starts_with("file://localhost/");
        let stripped = trimmed
            .strip_prefix("file://localhost/")
            .or_else(|| trimmed.strip_prefix("file:///"))
            .or_else(|| trimmed.strip_prefix("file://"))
            .or_else(|| trimmed.strip_prefix("file:"))
            .unwrap_or(trimmed);

        let decoded = percent_decode_str(stripped);
        let final_path_str = if is_triple_slash && !decoded.starts_with('/') {
            format!("/{}", decoded)
        } else {
            decoded
        };
        return Some(normalize_path(Path::new(&final_path_str)));
    }

    Some(normalize_path(Path::new(trimmed)))
}

fn percent_decode_str(input: &str) -> String {
    let mut bytes = Vec::with_capacity(input.len());
    let mut iter = input.bytes();
    while let Some(b) = iter.next() {
        if b == b'%' {
            let h1 = iter.next();
            let h2 = iter.next();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                if let Ok(hex_str) = std::str::from_utf8(&[h1, h2]) {
                    if let Ok(val) = u8::from_str_radix(hex_str, 16) {
                        bytes.push(val);
                        continue;
                    }
                }
                bytes.push(b'%');
                bytes.push(h1);
                bytes.push(h2);
                continue;
            }
            bytes.push(b'%');
            if let Some(h1) = h1 {
                bytes.push(h1);
            }
            if let Some(h2) = h2 {
                bytes.push(h2);
            }
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| input.to_string())
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppFlags {
    pub files: Vec<PathBuf>,
}

impl AppFlags {
    pub fn from_args<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let mut files = Vec::new();
        for item in args {
            let s: String = item.into();
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed == "--" || trimmed.starts_with('-') {
                continue;
            }
            if let Some(path) = parse_path_or_uri(trimmed) {
                files.push(path);
            }
        }
        Self { files }
    }
}
