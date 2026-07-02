use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

use constants::CONFIG_PATH;

pub type Config = HashMap<String, HashMap<String, String>>;

pub fn home_dir() -> Option<OsString> {
    env::var_os("HOME")
}

pub fn expand_path(path: &str) -> PathBuf {
    if let Some(home) = home_dir().map(PathBuf::from) {
        return home.join(path);
    }
    PathBuf::from(path)
}

pub fn read_config() -> Config {
    let contents = read_to_string(config_path()).unwrap_or_default();

    let mut config: Config = HashMap::new();
    let mut current_section = String::new();

    for line in contents.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            config.entry(current_section.clone()).or_default();
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            config
                .entry(current_section.clone())
                .or_default()
                .insert(key, value);
        }
    }

    config
}

pub fn get_value<'a>(config: &'a Config, section: &str, key: &str) -> Option<&'a str> {
    config.get(section)?.get(key).map(|s| s.as_str())
}

pub fn write_config(section: String, key: String, value: String) {
    let contents = read_to_string(config_path()).unwrap_or_default();

    let header = format!("[{section}]");
    let entry = format!("{key} = \"{value}\"");

    let mut out = String::new();
    let mut in_target_section = false;
    let mut section_found = false;
    let mut key_written = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        let is_header = trimmed.starts_with('[') && trimmed.ends_with(']');
        let is_target_key = in_target_section
            && trimmed
                .split('=')
                .next()
                .map(|k| k.trim() == key)
                .unwrap_or(false);

        if is_header {
            if in_target_section && !key_written {
                out.push_str(&entry);
                out.push('\n');
                key_written = true;
            }

            in_target_section = trimmed == header;
            if in_target_section {
                section_found = true;
            }

            out.push_str(line);
            out.push('\n');
            continue;
        }

        if is_target_key {
            out.push_str(&entry);
            out.push('\n');
            key_written = true;
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    if in_target_section && !key_written {
        out.push_str(&entry);
        out.push('\n');
    }

    if !section_found {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
        out.push_str(&header);
        out.push('\n');
        out.push_str(&entry);
        out.push('\n');
    }

    let _ = write(config_path(), out);
}

fn config_path() -> PathBuf {
    expand_path(CONFIG_PATH)
}
