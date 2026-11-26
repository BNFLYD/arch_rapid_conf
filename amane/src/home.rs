use std::cell::Cell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use amane::{Center, Color, Parent, Point, Rectangle, Service, Stack, Text, Widget};

use crate::menu;
use crate::theme;
use crate::widgets;

/*
 * The board of page 0: which widget sits in which cells and the options each
 * one keeps. The widgets themselves are drawn from this data on every frame,
 * and the services they show are read while drawing, so nothing here can hold
 * a stale copy of anything.
 *
 * Cells, not pixels: a widget of 3x2 covers three columns and two rows, which
 * is what makes the layout the same on any monitor.
 *
 * Holding a widget still for a moment turns the editing mode on. There it is
 * dragged from cell to cell, pushing whatever it lands on to the nearest free
 * spot, and dropped on the strip above the grid to be removed. There is no
 * release event, so the deletion is decided by time and the drag ends with the
 * click the library does report.
 */

// how long a widget has to be held still to turn the editing mode on
const HOLD: Duration = Duration::from_secs(2);
// a hold that never resolved must not live forever
const HOLD_GIVE_UP: Duration = Duration::from_secs(6);
// how far the pointer may move and still count as holding
const HOLD_SLACK: f32 = 4.0;
// the ticker looks at the hold and the file this often
const TICK: Duration = Duration::from_millis(80);
const CHECK_FILE: Duration = Duration::from_secs(1);
// the thin strip at the top of the block that deletes what stays on it; it is
// reserved always, so the widgets never move when the editing mode changes
const TRASH_STRIP: f32 = 24.0;
const TRASH_ICON: f32 = 16.0;
// the corner square that resizes a widget
const HANDLE: f32 = 20.0;
// how long a widget has to stay on the band before it goes
const TRASH_DELAY: Duration = Duration::from_millis(400);
// two drags closer than this belong to the same gesture
const CONTINUATION: Duration = Duration::from_millis(300);

thread_local! {
    // set by a resize event, so a motion without one means the button was let go
    // (the library reports no release of its own)
    static RESIZED: Cell<bool> = const { Cell::new(false) };
}

pub struct Instance {
    // stable identity: it does not change when the widget moves
    pub id: String,
    pub kind: widgets::Kind,

    // the top left cell
    pub col: usize,
    pub row: usize,

    // how many cells it covers
    pub cols: usize,
    pub rows: usize,

    // what this instance was configured with, like the command of a shortcut
    pub options: HashMap<String, String>,
}

struct PendingPress {
    started: Instant,
    point: (f32, f32),
}

struct DragState {
    id: String,
    // where inside the widget it was grabbed, so pressing does not move it
    grabbed: (f32, f32),
    // the last cell it was placed on, so the push only happens when it changes
    last_cell: (usize, usize),
    // when it was left on the band, if it is there
    deleting: Option<Instant>,
    // the last drag event, to tell a new press from the same gesture
    last: Instant,
}

struct ResizeState {
    id: String,
    // the last size it took, to skip the events that change nothing
    last: (usize, usize),
}

pub struct Board {
    pub instances: Vec<Instance>,
    pub editing: bool,
    // the resize in progress, if any
    resize: Option<ResizeState>,
    pending: Option<PendingPress>,
    dragging: Option<DragState>,
    // the click that comes with the hold must not open anything nor leave the mode
    swallow_click: bool,
    // the cells the board has right now, written by the view
    grid: Option<(usize, usize)>,
    // when the file was read last, so our own saves do not come back as reloads
    loaded_at: Option<SystemTime>,
}

impl Service for Board {
    fn new() -> Self {
        Self {
            instances: load(),
            editing: false,
            resize: None,
            pending: None,
            dragging: None,
            swallow_click: false,
            grid: None,
            loaded_at: stamp(),
        }
    }

    // the hold and the layout file need a tick, so this service polls
    fn listen() {
        let mut last_check = Instant::now();

        loop {
            thread::sleep(TICK);

            hold_tick();

            if last_check.elapsed() >= CHECK_FILE {
                last_check = Instant::now();
                reload_tick();
            }
        }
    }
}

impl Board {
    // the file is written only here, and never while loading
    fn save(&mut self) {
        let path = path();
        let temporary = path.with_extension("conf.tmp");

        if fs::write(&temporary, write_back(&self.instances)).is_err() {
            return;
        }

        if fs::rename(&temporary, &path).is_err() {
            return;
        }

        // the file we just wrote is the one we already have
        self.loaded_at = stamp();
    }
}

/// Whether the board is being edited, so Escape and the background click know
/// what to do first.
pub fn editing() -> bool {
    Board::read().editing
}

/// Leaves the editing mode and keeps what changed.
pub fn stop_editing() {
    let mut board = Board::write();

    if !board.editing {
        return;
    }

    board.editing = false;
    board.dragging = None;
    board.resize = None;
    board.pending = None;

    board.save();
}

/// Closing the menu throws away any gesture and keeps what was edited.
pub fn reset_edit() {
    stop_editing();

    let mut board = Board::write();

    board.pending = None;
    board.dragging = None;
    board.resize = None;
    board.swallow_click = false;
}

/// Going to the other page drops a hold that was still armed.
pub fn cancel_pending() {
    Board::write().pending = None;
}

fn remember_grid(columns: usize, rows: usize) {
    let mut board = Board::write();

    if board.grid == Some((columns, rows)) {
        return;
    }

    board.grid = Some((columns, rows));
}

/// Adds a widget of that kind on the first free spot that takes its default
/// size. Returns false when nothing fits.
pub fn add(kind: widgets::Kind) -> bool {
    let Some(spec) = widgets::SPECS.iter().find(|spec| spec.kind == kind) else {
        return false;
    };

    let mut board = Board::write();

    // before the board is drawn once, the grid of a 1920x1080 screen is a fair guess
    let dimensions = board.grid.unwrap_or((17, 9));

    let Some((col, row)) = first_fit(&board.instances, spec.default_size, dimensions) else {
        return false;
    };

    let id = next_id(&board.instances, kind);

    board.instances.push(Instance {
        id,
        kind,
        col,
        row,
        cols: spec.default_size.0,
        rows: spec.default_size.1,
        options: HashMap::new(),
    });

    board.save();

    true
}

// the first free spot, reading the grid from the top left
fn first_fit(
    instances: &[Instance],
    size: (usize, usize),
    board_size: (usize, usize),
) -> Option<(usize, usize)> {
    let (columns, rows) = board_size;

    for row in 0..rows {
        for col in 0..columns {
            if col + size.0 > columns || row + size.1 > rows {
                continue;
            }

            let free = instances
                .iter()
                .all(|other| !overlaps_cells(col, row, size.0, size.1, other));

            if free {
                return Some((col, row));
            }
        }
    }

    None
}

// whether a widget of that size fits there, ignoring itself
fn fits(
    instances: &[Instance],
    id: &str,
    col: usize,
    row: usize,
    cols: usize,
    rows: usize,
) -> bool {
    instances
        .iter()
        .filter(|other| other.id != id)
        .all(|other| !overlaps_cells(col, row, cols, rows, other))
}

fn next_id(instances: &[Instance], kind: widgets::Kind) -> String {
    let mut number = 1;

    loop {
        let id = format!("{}-{number}", kind.word());

        if instances.iter().all(|instance| instance.id != id) {
            return id;
        }

        number += 1;
    }
}

/// Arms the hold that turns the editing mode on, or cancels it when the pointer
/// moves. Returns whether it was a press waiting to become a hold, in which case
/// there is nothing else to do with it. A widget or the empty board behave the
/// same here, so nothing about the press is kept.
fn hold(point: Point) -> bool {
    let mut board = Board::write();

    if let Some(pending) = board.pending.as_ref() {
        let moved = (point.x - pending.point.0).hypot(point.y - pending.point.1) > HOLD_SLACK;

        if moved {
            board.pending = None;
        }

        return true;
    }

    // a new press: the click a previous hold spent does not reach this gesture
    board.swallow_click = false;

    if board.editing {
        return false;
    }

    board.pending = Some(PendingPress {
        started: Instant::now(),
        point: (point.x, point.y),
    });

    true
}

/// A press on an empty spot of the board: holding it also opens the editing mode.
pub fn background_drag(point: Point) {
    hold(point);
}

// the drag of a widget: the press arms the hold, and in the editing mode the
// drags after it move the widget cell by cell
fn drag(id: &str, point: Point, page: (f32, f32), origin: (f32, f32)) {
    if hold(point) {
        return;
    }

    let mut board = Board::write();

    // a press in the editing mode starts the drag, and the drags right after it
    // move the widget; one that comes much later is a new press
    let moving = board
        .dragging
        .as_ref()
        .map(|drag| drag.id == id && drag.last.elapsed() < CONTINUATION)
        .unwrap_or(false);

    if !moving {
        let Some(found) = board
            .instances
            .iter()
            .find(|instance| instance.id == id)
            .map(|instance| (instance.col, instance.row))
        else {
            return;
        };

        board.dragging = Some(DragState {
            id: String::from(id),
            grabbed: (point.x, point.y),
            last_cell: found,
            deleting: None,
            last: Instant::now(),
        });
    }

    let Some((widget_cols, widget_rows)) = board
        .instances
        .iter()
        .find(|instance| instance.id == id)
        .map(|instance| (instance.cols, instance.rows))
    else {
        return;
    };

    let grabbed = board
        .dragging
        .as_ref()
        .map(|drag| drag.grabbed)
        .unwrap_or((0.0, 0.0));

    // the pointer over the reserved strip is the only way to delete
    let over_the_band = (0.0..TRASH_STRIP).contains(&(origin.1 + point.y));

    if let Some(drag) = board.dragging.as_mut() {
        drag.last = Instant::now();

        if over_the_band {
            drag.deleting.get_or_insert_with(Instant::now);

            return;
        }

        drag.deleting = None;
    }

    let columns = menu::cols(page.0);
    let rows = menu::rows(page.1 - TRASH_STRIP);

    let candidate = candidate_cell(
        origin,
        (point.x, point.y),
        grabbed,
        (widget_cols, widget_rows),
        columns,
        rows,
    );

    if board
        .dragging
        .as_ref()
        .map(|drag| drag.last_cell == candidate)
        .unwrap_or(true)
    {
        return;
    }

    if apply_move(&mut board.instances, id, candidate, columns, rows)
        && let Some(drag) = board.dragging.as_mut()
    {
        drag.last_cell = candidate;
    }
}

/// Page 0: the widgets laid out over the cells they cover.
pub fn view(width: f32, height: f32) -> impl Widget {
    let columns = menu::cols(width);
    // the strip is not part of the grid, so the rows come from what is left
    let rows = menu::rows(height - TRASH_STRIP);

    // the panel that adds widgets needs to know how many cells there are, and
    // it draws on another window; this is written before the read below, on the
    // same thread, so the lock is taken twice in a row and never at once
    remember_grid(columns, rows);

    let board = Board::read();

    // the board needs a target of its own, or holding an empty spot does
    // nothing: it goes first, so the widgets on top win the hit test
    let background = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::TRANSPARENT)
        .on_drag(background_drag)
        .on_move(|_| settle())
        .on_click(|_| background_click());

    let mut items: Vec<Box<dyn Widget>> = vec![Box::new(background)];

    let over_the_strip = board
        .dragging
        .as_ref()
        .map(|drag| drag.deleting.is_some())
        .unwrap_or(false);

    // reserved always, drawn only while the board is edited
    if board.editing {
        items.push(Box::new(trash_strip(cells_width(columns), over_the_strip)));
    }

    // with nothing on the board the strip is useless, but there has to be a way
    // to add the first widget without holding anything
    if !board.editing && board.instances.is_empty() {
        items.push(Box::new(only_the_plus(cells_width(columns))));
    }

    for instance in &board.instances {
        let deleting = board
            .dragging
            .as_ref()
            .map(|drag| drag.id == instance.id && drag.deleting.is_some())
            .unwrap_or(false);

        let resizing = board
            .resize
            .as_ref()
            .filter(|resize| resize.id == instance.id)
            .map(|_| (instance.cols, instance.rows));

        let look = widgets::Look {
            width: cells_width(instance.cols),
            height: cells_height(instance.rows),
            editing: board.editing,
            deleting,
            resizing,
        };

        let spec = widgets::spec(instance.kind);

        // the point a drag reports is measured from the corner this frame drew,
        // so that corner goes with the handler; reading it from the state again
        // would mix two frames and the widget would run away from the pointer
        let origin = (x(instance.col), y(instance.row));

        let placed = (spec.view)(instance, &look)
            .translate(origin.0, origin.1)
            .on_drag({
                let id = instance.id.clone();
                let page = (width, height);

                move |point| drag(&id, point, page, origin)
            });

        items.push(Box::new(placed) as Box<dyn Widget>);

        // the corner that resizes: it goes after the body, so it wins the
        // pointer inside its square and the body keeps moving the widget. It is
        // always drawn while editing, so grabbing it never depends on a hover
        if board.editing {
            let handle = resize_handle(instance, width, height);

            items.push(Box::new(handle) as Box<dyn Widget>);
        }
    }

    Stack::new(items)
        .width(cells_width(columns))
        .height(TRASH_STRIP + cells_height(rows))
}

// the width of a run of cells, with the gaps between them
pub fn cells_width(cols: usize) -> f32 {
    cols as f32 * menu::CELL + cols.saturating_sub(1) as f32 * menu::GAP
}

pub fn cells_height(rows: usize) -> f32 {
    rows as f32 * menu::CELL + rows.saturating_sub(1) as f32 * menu::GAP
}

pub fn x(col: usize) -> f32 {
    col as f32 * (menu::CELL + menu::GAP)
}

pub fn y(row: usize) -> f32 {
    TRASH_STRIP + row as f32 * (menu::CELL + menu::GAP)
}

// the plus to add widgets on the left, the trash centred, and a hairline that
// keeps the strip apart from the first row
fn trash_strip(width: f32, active: bool) -> Stack {
    let color = if active {
        theme::TRASH_ICON_ACTIVE
    } else {
        theme::TRASH_ICON
    };

    let trash = Rectangle::new()
        .width(width)
        .height(TRASH_STRIP)
        .align_child(Center, Center)
        .child(
            Text::new("\u{f1f8}")
                .size(TRASH_ICON)
                .color(Color::from(color)),
        );

    let hairline = Rectangle::new()
        .width(width)
        .height(1.0)
        .fill(theme::TRASH_LINE)
        .translate(0.0, TRASH_STRIP - 1.0);

    Stack::new(vec![
        Box::new(trash) as Box<dyn Widget>,
        Box::new(plus()),
        Box::new(hairline),
    ])
    .width(width)
    .height(TRASH_STRIP)
}

// the strip alone, for a board with nothing on it: the only way to add
fn only_the_plus(width: f32) -> Stack {
    let hairline = Rectangle::new()
        .width(width)
        .height(1.0)
        .fill(theme::TRASH_LINE)
        .translate(0.0, TRASH_STRIP - 1.0);

    Stack::new(vec![
        Box::new(plus()) as Box<dyn Widget>,
        Box::new(hairline),
    ])
    .width(width)
    .height(TRASH_STRIP)
}

// the plus that opens the catalog of widgets
fn plus() -> Rectangle {
    let color = if crate::add_widget::hovered() {
        theme::ADD_ICON_ACTIVE
    } else {
        theme::ADD_ICON
    };

    Rectangle::new()
        .width(28.0)
        .height(TRASH_STRIP)
        .translate(8.0, 0.0)
        .align_child(Center, Center)
        .cursor(amane::Pointer)
        .on_hover(crate::add_widget::set_hovered)
        .on_click(|_| crate::add_widget::open())
        .child(
            Text::new("\u{eadc}")
                .size(TRASH_ICON)
                .color(Color::from(color)),
        )
}

/// Every motion of the board comes through here, which makes it the place to
/// notice that a resize ended: a motion with no resize in the same event means
/// the button was let go.
fn settle() {
    let acted = RESIZED.with(|resized| resized.replace(false));

    if acted {
        return;
    }

    let mut board = Board::write();

    board.resize = None;
}

// the square in the corner that resizes the widget
fn resize_handle(instance: &Instance, width: f32, height: f32) -> Rectangle {
    // the corner of the handle as this frame draws it: the drag reports its
    // point from here, so the two always travel together
    let origin = (
        x(instance.col) + cells_width(instance.cols) - HANDLE,
        y(instance.row) + cells_height(instance.rows) - HANDLE,
    );

    Rectangle::new()
        .width(HANDLE)
        .height(HANDLE)
        .translate(origin.0, origin.1)
        .fill(theme::EDIT_BORDER)
        .radius(2.0)
        .on_drag({
            let id = instance.id.clone();
            let page = (width, height);

            move |point| resize_drag(&id, point, page, origin)
        })
        .on_click(|_| {
            clicked();
        })
}

// a press on the handle starts the resize, and the drags after it grow or
// shrink the widget cell by cell, stopping at the last size that fitted
fn resize_drag(id: &str, point: Point, page: (f32, f32), origin: (f32, f32)) {
    RESIZED.with(|resized| resized.set(true));

    let mut board = Board::write();

    // a press starts the resize; the drags after it only move the far corner
    if board
        .resize
        .as_ref()
        .map(|resize| resize.id != id)
        .unwrap_or(true)
    {
        board.resize = Some(ResizeState {
            id: String::from(id),
            last: (0, 0),
        });
    }

    let Some(instance) = board
        .instances
        .iter()
        .find(|instance| instance.id == id)
        .map(|instance| {
            (
                instance.col,
                instance.row,
                instance.cols,
                instance.rows,
                instance.kind,
            )
        })
    else {
        return;
    };

    let Some(spec) = widgets::SPECS.iter().find(|spec| spec.kind == instance.4) else {
        return;
    };

    let columns = menu::cols(page.0);
    let rows = menu::rows(page.1 - TRASH_STRIP);
    let step = menu::CELL + menu::GAP;

    // where the pointer is, in the same space the widget was laid out in: the
    // far corner follows it, so growing never depends on accumulating deltas
    let here = (origin.0 + point.x, origin.1 + point.y);

    let wanted_cols = ((here.0 - x(instance.0)) / step).round().max(1.0) as i64;
    let wanted_rows = ((here.1 - y(instance.1)) / step).round().max(1.0) as i64;

    let mut cols = wanted_cols.max(spec.min_size.0 as i64);
    let mut rows_wanted = wanted_rows.max(spec.min_size.1 as i64);

    if let Some((max_cols, max_rows)) = spec.max_size {
        cols = cols.min(max_cols as i64);
        rows_wanted = rows_wanted.min(max_rows as i64);
    }

    cols = cols.min((columns - instance.0) as i64);
    rows_wanted = rows_wanted.min((rows - instance.1) as i64);

    let candidate = (cols.max(1) as usize, rows_wanted.max(1) as usize);

    let last = board
        .resize
        .as_ref()
        .map(|resize| resize.last)
        .unwrap_or((0, 0));

    if candidate == last || candidate == (instance.2, instance.3) {
        return;
    }

    if !fits(
        &board.instances,
        id,
        instance.0,
        instance.1,
        candidate.0,
        candidate.1,
    ) {
        return;
    }

    if let Some(instance) = board.instances.iter_mut().find(|item| item.id == id) {
        instance.cols = candidate.0;
        instance.rows = candidate.1;
    }

    if let Some(resize) = board.resize.as_mut() {
        resize.last = candidate;
    }
}

// the cell the widget's corner goes to, from where the pointer is and the
// offset it grabbed: a press keeps it where it is, and one cell of motion moves
// it one cell, so the widget keeps up with the pointer
fn candidate_cell(
    origin: (f32, f32),
    point: (f32, f32),
    grabbed: (f32, f32),
    widget: (usize, usize),
    columns: usize,
    rows: usize,
) -> (usize, usize) {
    let col = cell_of(origin.0 + point.0 - grabbed.0);
    let row = row_of(origin.1 + point.1 - grabbed.1);

    (
        col.min(columns.saturating_sub(widget.0)),
        row.min(rows.saturating_sub(widget.1)),
    )
}

// the row a point of the board falls on, counted from the first row of widgets
fn row_of(point: f32) -> usize {
    (((point - TRASH_STRIP) / (menu::CELL + menu::GAP))
        .round()
        .max(0.0)) as usize
}

// the cell a point of the board falls on
fn cell_of(point: f32) -> usize {
    (point / (menu::CELL + menu::GAP)).round().max(0.0) as usize
}

/// Ends a hold or a drag. Returns whether the click is the one that came with
/// the hold, so the caller spends it on nothing else.
pub fn clicked() -> bool {
    let mut board = Board::write();

    board.pending = None;
    board.dragging = None;
    board.resize = None;

    std::mem::take(&mut board.swallow_click)
}

/// A click on an empty spot of the board: it ends a hold, and in the editing
/// mode it leaves it, unless it is the click that came with the hold.
pub fn background_click() {
    // the click that ends a hold is spent: it must not leave the mode either
    if clicked() {
        return;
    }

    if editing() {
        stop_editing();
    }
}

// moves the widget to a cell, pushing whoever is under it to the nearest free
// spot; when someone does not fit, the whole move is undone
fn apply_move(
    instances: &mut [Instance],
    id: &str,
    candidate: (usize, usize),
    columns: usize,
    rows: usize,
) -> bool {
    let Some(index) = instances.iter().position(|instance| instance.id == id) else {
        return false;
    };

    if candidate.0 + instances[index].cols > columns || candidate.1 + instances[index].rows > rows {
        return false;
    }

    let before: Vec<(usize, usize)> = instances
        .iter()
        .map(|instance| (instance.col, instance.row))
        .collect();

    instances[index].col = candidate.0;
    instances[index].row = candidate.1;

    // a widget pushed aside may land on another one, so this repeats; a move
    // that keeps pushing longer than the widgets it has is undone
    for _ in 0..=instances.len() {
        let under = {
            let target = &instances[index];

            instances
                .iter()
                .position(|other| other.id != id && overlaps(target, other))
        };

        let Some(under) = under else {
            return true;
        };

        let Some(spot) = nearest_free(instances, under, columns, rows) else {
            undo(instances, &before);

            return false;
        };

        instances[under].col = spot.0;
        instances[under].row = spot.1;
    }

    undo(instances, &before);

    false
}

fn undo(instances: &mut [Instance], before: &[(usize, usize)]) {
    for (instance, (col, row)) in instances.iter_mut().zip(before) {
        instance.col = *col;
        instance.row = *row;
    }
}

// the free cell closest to where the widget is now, so it moves as little as
// possible; the scan starts at the top left, which wins the ties
fn nearest_free(
    instances: &[Instance],
    index: usize,
    columns: usize,
    rows: usize,
) -> Option<(usize, usize)> {
    let moving = &instances[index];

    let mut best: Option<((usize, usize), usize)> = None;

    for row in 0..rows {
        for col in 0..columns {
            if col + moving.cols > columns || row + moving.rows > rows {
                continue;
            }

            let free = instances.iter().enumerate().all(|(other_index, other)| {
                other_index == index || !overlaps_cells(col, row, moving.cols, moving.rows, other)
            });

            if !free {
                continue;
            }

            let distance = col.abs_diff(moving.col) + row.abs_diff(moving.row);

            if best.map(|(_, closest)| distance < closest).unwrap_or(true) {
                best = Some(((col, row), distance));
            }
        }
    }

    best.map(|(cell, _)| cell)
}

fn overlaps(a: &Instance, b: &Instance) -> bool {
    overlaps_cells(a.col, a.row, a.cols, a.rows, b)
}

fn overlaps_cells(col: usize, row: usize, cols: usize, rows: usize, other: &Instance) -> bool {
    col < other.col + other.cols
        && other.col < col + cols
        && row < other.row + other.rows
        && other.row < row + rows
}

// the ticker: the hold that turns the editing mode on, and the widget left on
// the strip for long enough
fn hold_tick() {
    let mut board = Board::write();

    let held = board
        .pending
        .as_ref()
        .map(|pending| pending.started.elapsed() >= HOLD)
        .unwrap_or(false);

    if held {
        board.editing = true;
        board.pending = None;
        // the click that accompanies the hold is spent
        board.swallow_click = true;

        return;
    }

    let gave_up = board
        .pending
        .as_ref()
        .map(|pending| pending.started.elapsed() >= HOLD_GIVE_UP)
        .unwrap_or(false);

    if gave_up {
        board.pending = None;
    }

    let gone = board
        .dragging
        .as_ref()
        .and_then(|drag| drag.deleting)
        .map(|since| since.elapsed() >= TRASH_DELAY)
        .unwrap_or(false);

    if gone && let Some(drag) = board.dragging.take() {
        board.instances.retain(|instance| instance.id != drag.id);
        board.save();
    }
}

// the file is the user's: editing it with an editor shows up within a second,
// without restarting the shell
fn reload_tick() {
    let mut board = Board::write();

    // never step on a gesture in progress
    if board.editing || board.dragging.is_some() || board.pending.is_some() {
        return;
    }

    let stamp = stamp();

    if stamp.is_none() || board.loaded_at == stamp {
        return;
    }

    let text = fs::read_to_string(path()).unwrap_or_default();
    let parsed = parse(&text);

    // a file that has content but no widget is an editor halfway through
    // writing, not an empty board
    if parsed.is_empty() && !text.trim().is_empty() {
        return;
    }

    board.instances = parsed;
    board.loaded_at = stamp;
}

fn load() -> Vec<Instance> {
    fs::read_to_string(path())
        .map(|text| parse(&text))
        .unwrap_or_default()
}

fn stamp() -> Option<SystemTime> {
    fs::metadata(path()).and_then(|meta| meta.modified()).ok()
}

fn path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));

    PathBuf::from(home).join(".config/amane/widgets.conf")
}

// "version 1", then one line per widget and one per option; a line that does
// not read is skipped, so a broken file never takes the rest with it
fn parse(text: &str) -> Vec<Instance> {
    let mut instances: Vec<Instance> = Vec::new();
    let mut options: HashMap<String, HashMap<String, String>> = HashMap::new();

    for line in text.lines() {
        let mut words = line.split_whitespace();

        match words.next() {
            Some("instance") => {
                let (Some(id), Some(kind), Some(col), Some(row), Some(cols), Some(rows)) = (
                    words.next(),
                    words.next(),
                    words.next(),
                    words.next(),
                    words.next(),
                    words.next(),
                ) else {
                    continue;
                };

                let Some(kind) = widgets::Kind::parse(kind) else {
                    continue;
                };

                let (Ok(col), Ok(row), Ok(cols), Ok(rows)) =
                    (col.parse(), row.parse(), cols.parse(), rows.parse())
                else {
                    continue;
                };

                instances.push(Instance {
                    id: String::from(id),
                    kind,
                    col,
                    row,
                    cols,
                    rows,
                    options: HashMap::new(),
                });
            }
            Some("option") => {
                let (Some(id), Some(key)) = (words.next(), words.next()) else {
                    continue;
                };

                // the value is the rest of the line: it may have spaces
                let value = words.collect::<Vec<_>>().join(" ");

                options
                    .entry(String::from(id))
                    .or_default()
                    .insert(String::from(key), value);
            }
            _ => {}
        }
    }

    // options may come before or after their widget
    for instance in &mut instances {
        if let Some(found) = options.remove(&instance.id) {
            instance.options = found;
        }
    }

    instances
}

fn write_back(instances: &[Instance]) -> String {
    let mut text = String::from("version 1\n");

    for instance in instances {
        text.push_str(&format!(
            "\ninstance {} {} {} {} {} {}\n",
            instance.id,
            instance.kind.word(),
            instance.col,
            instance.row,
            instance.cols,
            instance.rows
        ));

        // sorted, so saving twice gives the same file
        let mut keys: Vec<&String> = instance.options.keys().collect();
        keys.sort();

        for key in keys {
            text.push_str(&format!(
                "option {} {} {}\n",
                instance.id, key, instance.options[key]
            ));
        }
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn widget(id: &str, col: usize, row: usize, cols: usize, rows: usize) -> Instance {
        Instance {
            id: String::from(id),
            kind: widgets::Kind::Clock,
            col,
            row,
            cols,
            rows,
            options: HashMap::new(),
        }
    }

    #[test]
    fn reads_the_widgets_and_their_options() {
        let board = parse(
            "# mi tablero\n\
             version 1\n\
             instance clock-1 clock 0 0 3 1\n\
             option clock-1 irrelevant 1\n\
             instance shortcut-1 shortcut 3 0 1 1\n\
             option shortcut-1 exec foot -e nvim\n\
             option shortcut-1 name Foot\n",
        );

        assert_eq!(board.len(), 2);
        assert_eq!(board[0].id, "clock-1");
        assert_eq!(board[0].cols, 3);
        assert_eq!(board[1].kind, widgets::Kind::Shortcut);
        assert_eq!(board[1].options["exec"], "foot -e nvim");
        assert_eq!(board[1].options["name"], "Foot");
    }

    #[test]
    fn skips_a_line_it_cannot_read() {
        let board = parse(
            "instance clock-1 clock 0 0 3 1\n\
             instance roto clock cero 0 3 1\n\
             instance carousel-1 carousel 0 0 4 2\n\
             option nada\n",
        );

        assert_eq!(board.len(), 1);
        assert_eq!(board[0].id, "clock-1");
    }

    #[test]
    fn saving_and_reading_gives_the_same_board() {
        let mut first = widget("clock-1", 0, 0, 3, 1);
        first
            .options
            .insert(String::from("name"), String::from("Reloj"));

        let board = parse(&write_back(&[first, widget("clock-2", 3, 0, 2, 2)]));

        assert_eq!(board.len(), 2);
        assert_eq!(board[1].id, "clock-2");
        assert_eq!(board[0].options["name"], "Reloj");
        assert_eq!(write_back(&board), write_back(&board));
    }

    #[test]
    fn pressing_a_widget_does_not_move_it() {
        // the clock at column 1, grabbed in the middle
        let origin = (x(1), y(0));
        let grabbed = (154.0, 48.0);

        assert_eq!(
            candidate_cell(origin, grabbed, grabbed, (3, 1), 17, 9),
            (1, 0)
        );
    }

    #[test]
    fn one_cell_of_motion_moves_the_widget_one_cell() {
        let origin = (x(1), y(0));
        let grabbed = (154.0, 48.0);

        // the pointer travels a whole cell to the right and two rows down
        let step = menu::CELL + menu::GAP;
        let point = (grabbed.0 + step, grabbed.1 + 2.0 * step);

        assert_eq!(
            candidate_cell(origin, point, grabbed, (3, 1), 17, 9),
            (2, 2)
        );
    }

    #[test]
    fn the_widget_stops_at_the_edge_of_the_grid() {
        let origin = (x(0), y(0));
        let grabbed = (10.0, 10.0);

        assert_eq!(
            candidate_cell(origin, (5000.0, 5000.0), grabbed, (2, 1), 17, 9),
            (15, 8)
        );
    }

    #[test]
    fn places_the_widgets_over_their_cells() {
        assert_eq!(cells_width(1), 96.0);
        assert_eq!(cells_width(3), 308.0);
        assert_eq!(x(2), 212.0);
        // the first row of widgets starts below the strip
        assert_eq!(y(0), TRASH_STRIP);
        assert_eq!(y(1), TRASH_STRIP + 106.0);
        assert_eq!(cell_of(x(2) + 10.0), 2);
    }

    #[test]
    fn moves_a_widget_to_an_empty_cell() {
        let mut board = vec![widget("a", 0, 0, 1, 1)];

        assert!(apply_move(&mut board, "a", (4, 2), 17, 9));
        assert_eq!((board[0].col, board[0].row), (4, 2));
    }

    #[test]
    fn pushes_the_widget_under_the_moved_one() {
        let mut board = vec![widget("a", 0, 0, 1, 1), widget("b", 1, 0, 1, 1)];

        assert!(apply_move(&mut board, "a", (1, 0), 17, 9));

        assert_eq!((board[0].col, board[0].row), (1, 0));
        assert_eq!((board[1].col, board[1].row), (0, 0));
    }

    #[test]
    fn undoes_the_move_when_there_is_no_room() {
        // the only spots the pushed widget could take are narrower than it
        let mut board = vec![
            widget("a", 0, 0, 1, 1),
            widget("b", 1, 0, 3, 1),
            widget("c", 0, 1, 4, 1),
        ];

        assert!(!apply_move(&mut board, "a", (1, 0), 4, 2));

        assert_eq!((board[0].col, board[0].row), (0, 0));
        assert_eq!((board[1].col, board[1].row), (1, 0));
    }

    #[test]
    fn refuses_a_cell_outside_the_grid() {
        let mut board = vec![widget("a", 0, 0, 2, 2)];

        assert!(!apply_move(&mut board, "a", (16, 0), 17, 9));
        assert_eq!((board[0].col, board[0].row), (0, 0));
    }

    #[test]
    fn moves_the_pushed_widget_to_the_nearest_free_cell() {
        let mut board = vec![widget("a", 0, 0, 1, 1), widget("b", 3, 0, 1, 1)];

        assert!(apply_move(&mut board, "a", (3, 0), 17, 9));

        // the closest free cells to (3, 0) are (2, 0) and (4, 0); the scan
        // starts at the top left, so (2, 0) wins the tie, not (0, 0)
        assert_eq!((board[1].col, board[1].row), (2, 0));
    }
}
