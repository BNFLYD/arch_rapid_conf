use std::ops::Range;
use std::path::PathBuf;

use amane::{
    Apps, Color, Column, DesktopApp, Horizontal, Image, Key, Keyboard, Layer, LayerWindow, Margin,
    Monitor, Padding, Parent, Rectangle, Row, Scroll, Service, Text, TextInput, Vertical, Weight,
    Zone,
};

use crate::theme;
use crate::usage::Usage;

// the launcher takes half the screen, like the old wofi window
const WIDTH_RATIO: f32 = 0.52;
const SEARCH_HEIGHT: f32 = 46.0;
const ROW_HEIGHT: f32 = 46.0;
// the window always shows this many rows, so it never changes its height
const ROWS: usize = 4;
const PADDING: f32 = 10.0;
const GAP: f32 = 6.0;
const SEARCH_ID: &str = "amane-launcher-search";

struct LauncherState {
    open: bool,
    query: String,
    selected: usize,
    // what the wheel moved, kept for the parts that do not make a whole row
    scroll: f32,
}

impl Service for LauncherState {
    fn new() -> Self {
        Self {
            open: false,
            query: String::new(),
            selected: 0,
            scroll: 0.0,
        }
    }

    // the state only changes with input, so there is nothing to poll
    fn listen() {}
}

/// What the launcher shows for every app: the fields the view needs, owned so
/// the click handlers can keep them.
struct Entry {
    name: String,
    description: Option<String>,
    icon: Option<PathBuf>,
    exec: String,
    count: u32,
}

pub fn toggle() {
    if LauncherState::read().open {
        close();
        return;
    }

    {
        let mut state = LauncherState::write();
        state.open = true;
        state.query.clear();
        state.selected = 0;
    }

    // the input keeps what was typed, so it starts empty every time
    TextInput::set_text(SEARCH_ID, "");
}

pub fn close() {
    let mut state = LauncherState::write();
    state.open = false;
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let (open, query, selected) = {
        let state = LauncherState::read();
        (state.open, state.query.clone(), state.selected)
    };

    let entries = if open { filtered(&query) } else { Vec::new() };
    let selected = selected.min(entries.len().saturating_sub(1));
    let rows = visible_range(entries.len(), selected);

    // the height is always the same, no matter how many apps match
    let height = PADDING * 2.0 + SEARCH_HEIGHT + ROWS as f32 * (ROW_HEIGHT + GAP);

    let child = if open {
        content(&entries, &rows, selected)
    } else {
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(Color::TRANSPARENT)
    };

    LayerWindow::new()
        .width(monitor.width as f32 * WIDTH_RATIO)
        .height(height)
        .anchor_vertical(Vertical::Middle)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .keyboard(Keyboard::Exclusive)
        .space(Zone::Ignore)
        .namespace("amane-launcher")
        .visible(open)
        .on_key(|key| match key {
            Key::Escape => close(),
            Key::Up => move_selection(-1),
            Key::Down => move_selection(1),
            _ => {}
        })
        .child(child)
}

fn content(entries: &[Entry], rows: &Range<usize>, selected: usize) -> Rectangle {
    let mut items: Vec<Box<dyn amane::Widget>> = vec![Box::new(search_box())];

    if entries.is_empty() {
        items.push(Box::new(empty_message()));
    } else {
        for (index, entry) in entries.iter().enumerate().skip(rows.start).take(rows.len()) {
            items.push(Box::new(entry_row(entry, index == selected)));
        }
    }

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(theme::LAUNCHER_BACKGROUND)
        .radius(14.0)
        .border(1.0, theme::PANEL_BORDER)
        .padding(Padding {
            top: PADDING,
            right: PADDING,
            bottom: PADDING,
            left: PADDING,
        })
        .on_scroll(scroll_selection)
        .child(Column::new(items).gap(GAP))
}

fn search_box() -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(SEARCH_HEIGHT)
        .fill(theme::LAUNCHER_SEARCH)
        .radius_top_left(12.0)
        .radius_top_right(12.0)
        .radius_bottom_left(4.0)
        .radius_bottom_right(4.0)
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .align_child(amane::Start, amane::Center)
        .child(
            TextInput::new(SEARCH_ID)
                .width(Parent)
                .size(15.0)
                .color(Color::from(theme::MODULE_FOREGROUND))
                .placeholder("Buscar…")
                .focused()
                .on_change(|text| {
                    let mut state = LauncherState::write();
                    state.query = text;
                    state.selected = 0;
                })
                .on_submit(|_| launch_selected()),
        )
}

fn entry_row(entry: &Entry, selected: bool) -> Rectangle {
    let background = if selected {
        Color::from(theme::LAUNCHER_SELECTED)
    } else {
        Color::TRANSPARENT
    };

    let mut labels: Vec<Box<dyn amane::Widget>> = vec![Box::new(
        Text::new(&entry.name)
            .size(14.0)
            .weight(Weight::Bold)
            .color(Color::from(theme::MODULE_FOREGROUND)),
    )];

    if let Some(description) = &entry.description {
        labels.push(Box::new(
            Text::new(description)
                .size(11.0)
                .color(Color::from(theme::PANEL_MUTED)),
        ));
    }

    let mut items: Vec<Box<dyn amane::Widget>> = Vec::new();

    if let Some(icon) = &entry.icon {
        items.push(Box::new(
            Rectangle::new()
                .width(30.0)
                .height(30.0)
                .fill(Image::contain(icon.clone()))
                .clip(),
        ));
    }

    items.push(Box::new(Column::new(labels).gap(2.0)));

    let exec = entry.exec.clone();
    let name = entry.name.clone();

    Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .fill(background)
        .radius(10.0)
        .padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 4.0,
            left: 8.0,
        })
        .align_child(amane::Start, amane::Center)
        .cursor(amane::Pointer)
        .on_click(move |_| {
            Usage::write().record(&name);
            amane::spawn(&exec);
            close();
        })
        .child(
            Row::new(items)
                .gap(10.0)
                .height(Parent)
                .align(amane::Center),
        )
}

/// The message takes the place of a row, so the list keeps its shape.
fn empty_message() -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .padding(Padding {
            top: 0.0,
            right: 8.0,
            bottom: 0.0,
            left: 48.0,
        })
        .align_child(amane::Start, amane::Center)
        .child(
            Text::new("Sin resultados")
                .size(13.0)
                .color(Color::from(theme::PANEL_MUTED)),
        )
}

/// The apps that match what was typed: the most used ones first, and the
/// others in alphabetical order.
fn filtered(query: &str) -> Vec<Entry> {
    let query = query.trim().to_lowercase();

    let mut entries: Vec<Entry> = Apps::read()
        .list()
        .iter()
        .filter(|app| matches(app, &query))
        .map(entry)
        .collect();

    entries.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    entries
}

fn matches(app: &DesktopApp, query: &str) -> bool {
    matches_text(app.name(), app.description(), query)
}

fn matches_text(name: &str, description: Option<&str>, query: &str) -> bool {
    let query = query.trim().to_lowercase();

    if query.is_empty() {
        return true;
    }

    name.to_lowercase().contains(&query)
        || description.is_some_and(|text| text.to_lowercase().contains(&query))
}

fn entry(app: &DesktopApp) -> Entry {
    let name = app.name().to_string();

    Entry {
        count: Usage::read().count(&name),
        name,
        description: app.description().map(str::to_string),
        icon: app.icon_path().map(PathBuf::from),
        exec: app.exec().to_string(),
    }
}

/// The rows on screen: a window over the list that follows the selection.
fn visible_range(len: usize, selected: usize) -> Range<usize> {
    if len <= ROWS {
        return 0..len;
    }

    let start = selected.saturating_sub(ROWS / 2).min(len - ROWS);

    start..start + ROWS
}

fn move_selection(delta: isize) {
    let query = LauncherState::read().query.clone();
    let count = filtered(&query).len();

    if count == 0 {
        return;
    }

    let mut state = LauncherState::write();
    let next = (state.selected as isize + delta).rem_euclid(count as isize);
    state.selected = next as usize;
}

/// The wheel moves the selection, the parts that do not make a whole row are kept.
fn scroll_selection(scroll: Scroll) {
    if scroll.y == 0.0 {
        return;
    }

    let query = LauncherState::read().query.clone();
    let count = filtered(&query).len();

    if count == 0 {
        return;
    }

    let mut state = LauncherState::write();
    state.scroll += scroll.y;

    let lines = state.scroll.trunc();
    state.scroll -= lines;

    if lines == 0.0 {
        return;
    }

    let next = (state.selected as f32 + lines).rem_euclid(count as f32);
    state.selected = next as usize;
}

fn launch_selected() {
    let (query, selected) = {
        let state = LauncherState::read();
        (state.query.clone(), state.selected)
    };

    if let Some(entry) = filtered(&query).get(selected) {
        Usage::write().record(&entry.name);
        amane::spawn(&entry.exec);
    }

    close();
}

#[cfg(test)]
mod tests {
    use super::{ROWS, matches_text, visible_range};

    #[test]
    fn filters_by_name_and_description() {
        assert!(matches_text("Firefox", Some("Web Browser"), "fire"));
        assert!(matches_text("Firefox", Some("Web Browser"), "BROWS"));
        assert!(!matches_text("Firefox", Some("Web Browser"), "krit"));
        assert!(matches_text("Firefox", None, "  "));
    }

    #[test]
    fn keeps_the_selection_on_screen() {
        assert_eq!(visible_range(3, 1), 0..3);
        assert_eq!(visible_range(ROWS + 4, 0), 0..ROWS);
        assert_eq!(visible_range(ROWS + 4, ROWS + 3), 4..4 + ROWS);
    }

    #[test]
    fn the_list_is_never_longer_than_the_window() {
        assert!(visible_range(100, 50).len() <= ROWS);
    }
}
