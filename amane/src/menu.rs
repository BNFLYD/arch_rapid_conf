use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use amane::{
    Animation, Apps, Color, Column, DesktopApp, Easing, Key, Pointer, Rectangle, Row, Service,
    Widget,
};

use crate::theme;
use crate::usage::Usage;
use crate::widgets::tile;

/*
 * The two pages of the start menu share one grid of cells, as many as fit in
 * the space the block has when it is fully open, so it adapts to any monitor
 * without a fixed column count:
 *
 * - page 0, the menu itself: the empty grid where the widgets and shortcuts go
 * - page 1, the drawer: every app, in the order the user picked and then
 *   alphabetically
 *
 * The state keeps the order and the cell selected with the keyboard; the apps
 * themselves are looked up in the Apps service while drawing, so the drawer
 * shows them as soon as the scan finishes and can never hold a stale list.
 *
 * The order persists in ~/.cache/amane/menu-grid: lines of "index name". Apps
 * that are not in the order yet come after it, in alphabetical order.
 */

pub(crate) const CELL: f32 = 96.0;
pub(crate) const GAP: f32 = 10.0;
// how long the slide between the two pages takes
const PAGE_ROLL: Duration = Duration::from_millis(200);

/// The two pages of the menu: the menu itself on top and the drawer below,
/// slid inside the block that clips them.
struct Pager {
    // true while the drawer is the page being shown
    drawer: bool,
    page: Animation<f32>,
}

impl Service for Pager {
    fn new() -> Self {
        Self {
            drawer: false,
            page: Animation::new(0.0).duration(PAGE_ROLL).easing(Easing::Out),
        }
    }

    // the page only moves on a key press, so there is nothing to poll
    fn listen() {}
}

/// 0 shows the menu, 1 the drawer, in between while it slides.
pub fn page() -> f32 {
    Pager::read().page.value()
}

/// Where the page is going, not where it is, so the keys answer at once.
pub fn on_drawer() -> bool {
    Pager::read().drawer
}

/// Tab: goes down to the drawer, or back up to the menu.
pub fn toggle_page() {
    // a hold that was still armed belongs to the page that is going away
    crate::home::cancel_pending();

    let mut pager = Pager::write();

    let drawer = !pager.drawer;

    pager.drawer = drawer;
    pager.page.to(if drawer { 1.0 } else { 0.0 });
}

/// The menu always opens on its own page.
pub fn reset_page() {
    {
        let mut pager = Pager::write();

        pager.drawer = false;
        pager.page.to(0.0);
    }

    // the marker only comes back with the first arrow
    Grid::write().selected = None;
}

pub struct Grid {
    // only the order the user picked: the apps are resolved while drawing
    order: Vec<String>,
    // index of the selected cell over the resolved list, none when the menu is closed
    selected: Option<usize>,
}

impl Service for Grid {
    fn new() -> Self {
        // never writes: without a file the order comes from the app names
        Self {
            order: load_order(),
            selected: None,
        }
    }

    // the grid only changes when it is reordered, so there is nothing to poll
    fn listen() {}
}

pub fn cols(width: f32) -> usize {
    // one column less than what fits, so the grid keeps air around it
    ((((width + GAP) / (CELL + GAP)) as usize).saturating_sub(1)).max(1)
}

pub fn rows(height: f32) -> usize {
    ((height + GAP) / (CELL + GAP)) as usize
}

/// Walks the cells of the block. `cell_at` decides what each one shows, so both
/// pages share one grid and the widgets of page 0 land exactly where the apps
/// of the drawer are.
fn slots(width: f32, height: f32, mut cell_at: impl FnMut(usize) -> Rectangle) -> Column {
    let columns = cols(width);

    let mut lines: Vec<Box<dyn Widget>> = Vec::new();

    for row in 0..rows(height) {
        let mut cells: Vec<Box<dyn Widget>> = Vec::new();

        for column in 0..columns {
            cells.push(Box::new(cell_at(row * columns + column)));
        }

        lines.push(Box::new(Row::new(cells).gap(GAP)) as Box<dyn Widget>);
    }

    Column::new(lines).gap(GAP)
}

/// Page 1: every app, with the marker of the keyboard.
pub fn drawer(width: f32, height: f32) -> Column {
    let apps = Apps::read();
    let order = Grid::read();

    let resolved = resolve(&apps, &order.order);
    let selected = order.selected;

    slots(width, height, |index| {
        cell(resolved.get(index), selected == Some(index))
    })
}

/// Page 0: the empty grid where the widgets and shortcuts will go. It paints
/// nothing but keeps the positions.
/// Page 0: the board of widgets, over the same cells the drawer uses.
pub fn home(width: f32, height: f32) -> impl Widget {
    crate::home::view(width, height)
}

// the saved order first, only for the apps still installed, then the rest in
// alphabetical order: the menu is static, it does not follow the launcher
fn resolve<'a>(apps: &'a Apps, order: &[String]) -> Vec<&'a DesktopApp> {
    let mut resolved: Vec<&DesktopApp> = Vec::new();

    for name in order {
        if let Some(app) = apps.list().iter().find(|app| app.name() == name) {
            resolved.push(app);
        }
    }

    let mut rest: Vec<&DesktopApp> = apps
        .list()
        .iter()
        .filter(|app| !resolved.iter().any(|placed| placed.name() == app.name()))
        .collect();

    // ignoring case: "amane" comes before "Bluefire", not after
    rest.sort_by_cached_key(|app| app.name().to_lowercase());

    resolved.extend(rest);
    resolved
}

// a taken cell shows the app and launches it on click; an empty one keeps its
// place in the grid and paints nothing
fn cell(app: Option<&&DesktopApp>, selected: bool) -> Rectangle {
    let base = Rectangle::new().width(CELL).height(CELL).clip();

    // the selection is marked without bringing back the background squares:
    // a soft halo around the chosen one
    let base = if selected {
        base.fill(Color::from(theme::MENU_SELECTION))
            .radius(theme::BAR_RADIUS)
    } else {
        base
    };

    let Some(app) = app else {
        return base;
    };

    let name = app.name().to_string();
    let exec = app.exec().to_string();

    base.cursor(Pointer)
        .on_click(move |_| launch(&name, &exec))
        .align_child(amane::Center, amane::Center)
        .child(tile::view(app))
}

fn launch(name: &str, exec: &str) {
    Usage::write().record(name);

    // like windows: the menu leaves before the app shows up
    crate::bar::menu_close();

    amane::spawn(exec);
}

/// Handles one key press in the grid. `columns` comes from whoever builds the
/// window, since only it knows how wide the block is.
pub fn press(key: Key, columns: usize) {
    match key {
        // the only keys that bring the marker up
        Key::Left | Key::Right | Key::Up | Key::Down => move_selection(key, columns),
        // both of them open the marked app
        Key::Enter | Key::Space => open_selected(),
        _ => {}
    }
}

fn total() -> usize {
    // the resolved order holds every installed app
    Apps::read().list().len()
}

fn move_selection(key: Key, columns: usize) {
    let total = total();

    if total == 0 {
        return;
    }

    let mut grid = Grid::write();

    // the first arrow only lights the first cell: moving too would make the
    // first press jump a whole row
    let Some(current) = grid.selected else {
        grid.selected = Some(0);
        return;
    };

    let next = match key {
        Key::Left => current.saturating_sub(1),
        Key::Right => current + 1,
        Key::Up => current.saturating_sub(columns),
        Key::Down => current + columns,
        _ => current,
    };

    // no wrapping: on the first cell, left and up do nothing
    grid.selected = Some(next.min(total - 1));
}

fn open_selected() {
    let selected = Grid::read().selected;
    // without a marker there is nothing to open
    let Some(index) = selected else {
        return;
    };

    // the guards end here: opening the app closes the menu, which writes the
    // grid, and a read held while writing the same service would wait forever
    let chosen = {
        let apps = Apps::read();
        let order = Grid::read();

        let resolved = resolve(&apps, &order.order);

        resolved
            .get(index)
            .map(|app| (app.name().to_string(), app.exec().to_string()))
    };

    if let Some((name, exec)) = chosen {
        launch(&name, &exec);
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

fn load_order() -> Vec<String> {
    let Ok(text) = fs::read_to_string(path()) else {
        return Vec::new();
    };

    let mut placed: Vec<(usize, String)> = text
        .lines()
        .filter_map(parse_index)
        .map(|(index, name)| (index, name.to_string()))
        .collect();

    // the file keeps a position per line, but the grid now packs the apps in
    // a plain sequence
    placed.sort_by_key(|(index, _)| *index);

    placed.into_iter().map(|(_, name)| name).collect()
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
