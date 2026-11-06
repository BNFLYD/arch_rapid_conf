use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use amane::Service;

/// How many times each app was launched, so the launcher can show the most
/// used ones first.
pub struct Usage {
    // the launches made from this launcher
    own: HashMap<String, u32>,
    // the ones wofi counted before, kept apart so they are not counted twice
    seed: HashMap<String, u32>,
}

impl Service for Usage {
    fn new() -> Self {
        Self {
            own: read_counts(),
            seed: seed_from_wofi(),
        }
    }

    // the counts only change when an app is launched, so there is nothing to poll
    fn listen() {}
}

impl Usage {
    pub fn count(&self, name: &str) -> u32 {
        self.own.get(name).copied().unwrap_or(0) + self.seed.get(name).copied().unwrap_or(0)
    }

    /// Counts one more launch and keeps the file up to date.
    pub fn record(&mut self, name: &str) {
        let count = self.own.entry(name.to_string()).or_insert(0);
        *count += 1;

        write_counts(&self.own);
    }
}

fn cache_folder() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/mori".to_string());
            PathBuf::from(home).join(".cache")
        });

    base.join("amane")
}

fn counts_path() -> PathBuf {
    cache_folder().join("launcher-usage")
}

fn read_counts() -> HashMap<String, u32> {
    let mut counts = HashMap::new();

    let Ok(text) = fs::read_to_string(counts_path()) else {
        return counts;
    };

    for line in text.lines() {
        let Some((name, count)) = line.split_once('\t') else {
            continue;
        };

        if let Ok(count) = count.trim().parse() {
            counts.insert(name.to_string(), count);
        }
    }

    counts
}

fn write_counts(counts: &HashMap<String, u32>) {
    let folder = cache_folder();

    if fs::create_dir_all(&folder).is_err() {
        return;
    }

    let mut text = String::new();

    for (name, count) in counts {
        text.push_str(&format!("{name}\t{count}\n"));
    }

    let _ = fs::write(counts_path(), text);
}

/// The counts wofi kept while it was the launcher: a number and the desktop
/// file it belongs to.
fn seed_from_wofi() -> HashMap<String, u32> {
    let mut counts = HashMap::new();

    let path = cache_folder().parent().map(|cache| cache.join("wofi-drun"));

    let Some(path) = path else {
        return counts;
    };

    let Ok(text) = fs::read_to_string(path) else {
        return counts;
    };

    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(count), Some(desktop)) = (parts.next(), parts.next()) else {
            continue;
        };

        let Ok(count) = count.parse::<u32>() else {
            continue;
        };

        let Some(name) = desktop_name(Path::new(desktop)) else {
            continue;
        };

        *counts.entry(name).or_insert(0) += count;
    }

    counts
}

/// The name a desktop file shows, the same one the launcher lists.
fn desktop_name(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;

    for line in text.lines() {
        if let Some(name) = line.strip_prefix("Name=") {
            let name = name.trim();

            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::desktop_name;

    #[test]
    fn takes_the_name_of_a_desktop_file() {
        let folder = std::env::temp_dir().join("amane-usage-test");

        std::fs::create_dir_all(&folder).unwrap();

        let path = folder.join("zen.desktop");
        std::fs::write(
            &path,
            "[Desktop Entry]\nName=Zen\nName[es]=Zen\nExec=zen %u\n",
        )
        .unwrap();

        assert_eq!(desktop_name(&path), Some("Zen".to_string()));

        std::fs::remove_dir_all(&folder).ok();
    }

    #[test]
    fn a_desktop_file_without_a_name_has_none() {
        assert_eq!(desktop_name(Path::new("/does/not/exist.desktop")), None);
    }
}
