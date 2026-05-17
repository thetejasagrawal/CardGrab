use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Local, Timelike, Utc};

use crate::scanner::FileInfo;

fn sanitize_segment(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c.is_whitespace() || c == '.');
    if trimmed.is_empty() {
        "_".to_string()
    } else {
        trimmed.to_string()
    }
}

fn replace_token(out: &mut String, token: &str, value: &str) {
    let needle = format!("{{{}}}", token);
    if !value.is_empty() {
        *out = out.replace(&needle, &sanitize_segment(value));
    } else {
        *out = out.replace(&needle, "_");
    }
}

pub fn render_file_path(pattern: &str, file: &FileInfo) -> PathBuf {
    let when_utc: DateTime<Utc> = file
        .shot_at
        .or(file.mtime)
        .unwrap_or_else(|| Utc::now());
    let when_local = when_utc.with_timezone(&Local);

    let year = format!("{:04}", when_local.year());
    let month = format!("{:02}", when_local.month());
    let day = format!("{:02}", when_local.day());
    let date = format!("{}-{}-{}", year, month, day);
    let time = format!(
        "{:02}-{:02}-{:02}",
        when_local.hour(),
        when_local.minute(),
        when_local.second()
    );

    let camera = file.camera_model.clone().unwrap_or_else(|| "Camera".into());
    let lens = file.lens.clone().unwrap_or_else(|| "Lens".into());
    let kind = file.kind.folder_name().to_string();
    let ext = file.ext.clone();
    let orig = file
        .src_abs
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();

    let mut s = pattern.to_string();
    replace_token(&mut s, "year", &year);
    replace_token(&mut s, "month", &month);
    replace_token(&mut s, "day", &day);
    replace_token(&mut s, "date", &date);
    replace_token(&mut s, "time", &time);
    replace_token(&mut s, "camera", &camera);
    replace_token(&mut s, "lens", &lens);
    replace_token(&mut s, "kind", &kind);
    replace_token(&mut s, "ext", &ext);
    replace_token(&mut s, "orig_name", &orig);

    let mut p = PathBuf::new();
    for seg in s.split('/').filter(|s| !s.is_empty()) {
        p.push(seg);
    }

    // Always append the original filename at the end if pattern doesn't already
    // end with {orig_name}.{ext} producing a filename.
    let pattern_ends_with_file = pattern.trim_end_matches('/').ends_with("}");
    let already_has_extension = p
        .file_name()
        .and_then(|s| s.to_str())
        .map(|n| n.contains('.'))
        .unwrap_or(false);

    if !(pattern_ends_with_file && already_has_extension) {
        let filename = file
            .src_abs
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        p.push(filename);
    }
    p
}

pub fn render_full(dest_root: &Path, pattern: &str, file: &FileInfo) -> PathBuf {
    dest_root.join(render_file_path(pattern, file))
}

pub fn preview_paths(pattern: &str, files: &[FileInfo], n: usize) -> Vec<String> {
    files
        .iter()
        .take(n)
        .map(|f| {
            render_file_path(pattern, f)
                .to_string_lossy()
                .to_string()
        })
        .collect()
}
