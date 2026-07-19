use helpers::file_types::FileType;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Default)]
struct LocStats {
    files: usize,
    lines: usize,
    code: usize,
    comment: usize,
    blank: usize,
}

pub fn loc(path: String) {
    let mut stats: HashMap<&'static str, LocStats> = HashMap::new();
    walk(&PathBuf::from(path), &mut stats);

    let mut rows: Vec<_> = stats.into_iter().collect();
    rows.sort_by_key(|(name, _)| *name);

    println!(
        "{:<15} {:>6} {:>8} {:>7} {:>10} {:>8}",
        "Language", "Files", "Lines", "Code", "Comment", "Blank"
    );
    println!("{}", "─".repeat(60));

    let mut total = LocStats::default();
    for (name, s) in &rows {
        println!(
            "{:<15} {:>6} {:>8} {:>7} {:>10} {:>8}",
            name, s.files, s.lines, s.code, s.comment, s.blank
        );
        total.files += s.files;
        total.lines += s.lines;
        total.code += s.code;
        total.comment += s.comment;
        total.blank += s.blank;
    }

    println!("{}", "─".repeat(60));
    println!(
        "{:<15} {:>6} {:>8} {:>7} {:>10} {:>8}",
        "Total", total.files, total.lines, total.code, total.comment, total.blank
    );
}

fn walk(path: &Path, stats: &mut HashMap<&'static str, LocStats>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if matches!(name, ".git" | "target" | "node_modules") {
                    continue;
                }
            }
            walk(&path, stats);
            continue;
        }

        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        let Some(file_type) = FileType::from_name(ext) else {
            continue;
        };

        count_file(&path, file_type.name(), stats);
    }
}

fn count_file(path: &Path, lang: &'static str, stats: &mut HashMap<&'static str, LocStats>) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };

    let entry = stats.entry(lang).or_default();
    entry.files += 1;

    for line in content.lines() {
        entry.lines += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            entry.blank += 1;
        } else if trimmed.starts_with("//") || trimmed.starts_with('#') {
            entry.comment += 1;
        } else {
            entry.code += 1;
        }
    }
}
