use std::fs;
use std::path::PathBuf;

use amane::{Apps, Color, Column, DesktopApp, Pointer, Rectangle, Row, Service, Widget};

use crate::theme;
use crate::usage::Usage;
use crate::widgets::tile::{self, Shortcut};

/*
 * The grid of the start menu: as many cells as fit in the space the block has
 * when it is fully open, so it adapts to any monitor without a fixed column
 * count. The tiles (shortcuts first, widgets later) are dropped into these
 * slots and dragged around, so the geometry lives in functions the drag can
 * reuse to know which slot the pointer is over.
 *
 * The order of the slots persists in ~/.cache/amane/menu-grid: lines of
 * "index name". Apps that are not on the grid yet take the first free slots,
 * in order of use.
 */

const CELL: f32 = 96.0;
const GAP: f32 = 10.0;

enum Item {
    App(Shortcut),
}

pub struct Grid {
    slots: Vec<Option<Item>>,
}

impl Service for Grid {
    fn new() -> Self {
        let apps = Apps::read();
        let usage = Usage::read();

        // most used first, ties in alphabetical order, like the launcher
        let mut ordered: Vec<&DesktopApp> = apps.list().iter().collect();
        ordered.sort_by(|a, b| {
            usage
                .count(b.name())
                .cmp(&usage.count(a.name()))
                .then_with(|| a.name().cmp(b.name()))
        });

        let mut slots = load(apps.list());

        // apps that are not on the grid yet take the first free slots
        for app in ordered {
            let placed = slots.iter().any(
                |slot| matches!(slot, Some(Item::App(shortcut)) if shortcut.name == app.name()),
            );

            if placed {
                continue;
            }

            match slots.iter().position(|slot| slot.is_none()) {
                Some(index) => slots[index] = Some(Item::App(shortcut_of(app))),
                None => slots.push(Some(Item::App(shortcut_of(app)))),
            }
        }

        save(&slots);

        Self { slots }
    }

    // the grid only changes when it is edited, it never polls
    fn listen() {}
}

pub fn cols(width: f32) -> usize {
    // one column less than what fits, so the grid keeps air around it
    ((((width + GAP) / (CELL + GAP)) as usize).saturating_sub(1)).max(1)
}

pub fn rows(height: f32) -> usize {
    ((height + GAP) / (CELL + GAP)) as usize
}

/// The grid that fills the menu block. The size is the one the block has when
/// fully open, never the animated one, so the roll does not reflow it.
pub fn grid(width: f32, height: f32) -> Column {
    let state = Grid::read();
    let columns = cols(width);

    let mut lines: Vec<Box<dyn Widget>> = Vec::new();

    for row in 0..rows(height) {
        let mut cells: Vec<Box<dyn Widget>> = Vec::new();

        for column in 0..columns {
            let index = row * columns + column;

            let slot = state.slots.get(index).and_then(|slot| slot.as_ref());

            cells.push(Box::new(cell(slot)));
        }

        lines.push(Box::new(Row::new(cells).gap(GAP)) as Box<dyn Widget>);
    }

    Column::new(lines).gap(GAP)
}

// an empty slot keeps its faint tile so the grid stays visible; a taken one
// shows the shortcut and launches it on click
fn cell(slot: Option<&Item>) -> Rectangle {
    let base = Rectangle::new()
        .width(CELL)
        .height(CELL)
        .fill(Color::from(theme::LAUNCHER_SELECTED))
        .clip();

    match slot {
        Some(Item::App(shortcut)) => {
            let name = shortcut.name.clone();
            let exec = shortcut.exec.clone();

            base.cursor(Pointer)
                .on_click(move |_| launch(&name, &exec))
                .align_child(amane::Center, amane::Center)
                .child(tile::view(shortcut))
        }
        None => base,
    }
}

fn launch(name: &str, exec: &str) {
    Usage::write().record(name);
    amane::spawn(exec);
}

fn shortcut_of(app: &DesktopApp) -> Shortcut {
    Shortcut {
        name: app.name().to_string(),
        icon: app.icon_path().map(PathBuf::from),
        exec: app.exec().to_string(),
    }
}

// "index name" per line; names may have spaces, the index never does
fn parse_index(line: &str) -> Option<(usize, &str)> {
    let (index, name) = line.split_once(' ')?;
    let index = index.parse().ok()?;

    Some((index, name))
}

fn path() -> PathBuf {
    crate::usage::cache_folder().join("menu-grid")
}

fn load(apps: &[DesktopApp]) -> Vec<Option<Item>> {
    let Ok(text) = fs::read_to_string(path()) else {
        return Vec::new();
    };

    let mut slots: Vec<Option<Item>> = Vec::new();

    for line in text.lines() {
        let Some((index, name)) = parse_index(line) else {
            continue;
        };

        // a shortcut that is no longer installed leaves an empty slot
        let Some(app) = apps.iter().find(|app| app.name() == name) else {
            continue;
        };

        while slots.len() <= index {
            slots.push(None);
        }

        slots[index] = Some(Item::App(shortcut_of(app)));
    }

    slots
}

fn save(slots: &[Option<Item>]) {
    let mut text = String::new();

    for (index, slot) in slots.iter().enumerate() {
        if let Some(Item::App(shortcut)) = slot {
            text.push_str(&format!("{} {}\n", index, shortcut.name));
        }
    }

    let _ = fs::create_dir_all(crate::usage::cache_folder());
    let _ = fs::write(path(), text);
}

#[cfg(test)]
mod tests {
    use super::{cols, parse_index, rows};

    #[test]
    fn fits_one_column_less_than_the_width_allows() {
        assert_eq!(cols(1906.0), 17);
        assert_eq!(cols(400.0), 2);
    }

    #[test]
    fn fits_as_many_rows_as_the_height_allows() {
        assert_eq!(rows(1022.0), 9);
        assert_eq!(rows(200.0), 1);
    }

    #[test]
    fn reads_the_slot_and_the_name_of_a_line() {
        assert_eq!(parse_index("12 Firefox Web"), Some((12, "Firefox Web")));
        assert_eq!(parse_index("not a slot"), None);
    }
}
