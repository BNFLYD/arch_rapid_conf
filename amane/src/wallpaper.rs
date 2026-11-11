use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use amane::Monitor;

use crate::theme;

pub enum Backdrop {
    Image(PathBuf),
    Color(String),
}

// parsed once: amane starts together with sway (exec_always)
static FOUND: LazyLock<HashMap<String, Backdrop>> = LazyLock::new(|| parse(&sway_config()));

// only used when sway declares no wallpaper this shell can draw
static FALLBACK: LazyLock<Backdrop> =
    LazyLock::new(|| Backdrop::Color(String::from(theme::BACKDROP_FALLBACK)));

/// The wallpaper sway paints on this monitor.
pub fn for_monitor(monitor: &Monitor) -> &'static Backdrop {
    FOUND
        .get(&monitor.name)
        .or_else(|| FOUND.get("*"))
        .unwrap_or(&FALLBACK)
}

fn parse(path: &Path) -> HashMap<String, Backdrop> {
    std::fs::read_to_string(path)
        .map(|text| parse_text(&text))
        .unwrap_or_default()
}

fn parse_text(text: &str) -> HashMap<String, Backdrop> {
    let mut found = HashMap::new();

    for line in text.lines() {
        let line = line.trim();

        // the commented out wallpapers are just examples
        if line.starts_with('#') {
            continue;
        }

        // "output <name> bg <path|#hex> <mode>"
        let words = words(line);

        if words.first() != Some(&"output") || words.get(2) != Some(&"bg") {
            continue;
        }

        let (Some(name), Some(value)) = (words.get(1), words.get(3)) else {
            continue;
        };

        let backdrop = if value.starts_with('#') {
            Backdrop::Color(String::from(*value))
        } else {
            Backdrop::Image(expand(value))
        };

        // the last match wins, the same as in sway
        found.insert(String::from(*name), backdrop);
    }

    found
}

// splits on spaces, keeping a quoted word in one piece and without its quotes
fn words(line: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let mut start = None;
    let mut quoted = false;

    for (index, character) in line.char_indices() {
        match character {
            '"' => quoted = !quoted,
            ' ' | '\t' if !quoted => {
                if let Some(from) = start.take() {
                    words.push(line[from..index].trim_matches('"'));
                }
            }
            _ => {
                start.get_or_insert(index);
            }
        }
    }

    if let Some(from) = start {
        words.push(line[from..].trim_matches('"'));
    }

    words
}

fn expand(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }

    PathBuf::from(path)
}

fn sway_config() -> PathBuf {
    let config =
        std::env::var("SWAY_CONFIG").unwrap_or_else(|_| String::from(".config/sway/config"));

    if config.starts_with('/') {
        return PathBuf::from(config);
    }

    match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(config),
        Err(_) => PathBuf::from(config),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(backdrop: &Backdrop) -> String {
        match backdrop {
            Backdrop::Image(path) => path.display().to_string(),
            Backdrop::Color(color) => color.clone(),
        }
    }

    #[test]
    fn keeps_a_quoted_wallpaper_in_one_piece() {
        assert_eq!(
            words(r#"output * bg "/a path/fondo.jpg" fill"#),
            vec!["output", "*", "bg", "/a path/fondo.jpg", "fill"]
        );
    }

    #[test]
    fn reads_the_wallpapers_of_the_sway_config() {
        let found = parse_text(
            "# output * bg /home/mori/ejemplo.jpg fill\n\
             output * bg /mnt/d/imagenes/fondos/landscape.jpg fill\n\
             # output * bg /home/mori/Pictures/STRAY.png fill\n\
             output DP-1 bg \"#aabbcc\" solid_color",
        );

        assert_eq!(value(&found["*"]), "/mnt/d/imagenes/fondos/landscape.jpg");
        assert_eq!(value(&found["DP-1"]), "#aabbcc");
        assert_eq!(found.len(), 2);
    }
}
