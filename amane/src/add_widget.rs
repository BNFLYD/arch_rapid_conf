use amane::{
    Center, Color, Column, Horizontal, Key, Keyboard, Layer, LayerWindow, Margin, Monitor, Padding,
    Parent, Rectangle, Row, Service, Start, Text, Vertical, Weight, Widget, Zone,
};

use crate::theme;
use crate::widgets;

/*
 * The panel that adds widgets to the board. It is its own window, drawn over
 * the start menu, with the same shape as the app launcher: a full screen
 * transparent root that closes on click, and a centered box with the list.
 */

const WIDTH: f32 = 380.0;
const PADDING: f32 = 12.0;
const ROW: f32 = 46.0;
const GAP: f32 = 6.0;
const TITLE: f32 = 24.0;

struct AddState {
    open: bool,
    // the plus of the editing strip is under the pointer
    hovered: bool,
    // the last attempt found no room on the board
    full: bool,
}

impl Service for AddState {
    fn new() -> Self {
        Self {
            open: false,
            hovered: false,
            full: false,
        }
    }

    // the panel only changes with input, so there is nothing to poll
    fn listen() {}
}

pub fn open() {
    let mut state = AddState::write();

    state.open = true;
    state.full = false;
}

pub fn close() {
    AddState::write().open = false;
}

pub fn is_open() -> bool {
    AddState::read().open
}

pub fn hovered() -> bool {
    AddState::read().hovered
}

pub fn set_hovered(inside: bool) {
    let mut state = AddState::write();

    if state.hovered == inside {
        return;
    }

    state.hovered = inside;
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let (open, full) = {
        let state = AddState::read();
        (state.open, state.full)
    };

    let mut items: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new("Agregar widget")
            .size(13.0)
            .weight(Weight::Bold)
            .color(Color::from(theme::PANEL_FOREGROUND)),
    )];

    for spec in widgets::specs() {
        items.push(Box::new(row(spec)));
    }

    if full {
        items.push(Box::new(
            Text::new("Sin espacio libre")
                .size(11.0)
                .color(Color::from(theme::PANEL_DANGER)),
        ));
    }

    let rows = widgets::specs().len() as f32;
    let height =
        PADDING * 2.0 + TITLE + rows * ROW + (rows + 1.0) * GAP + if full { 16.0 } else { 0.0 };

    let panel = Rectangle::new()
        .width(WIDTH)
        .height(height)
        .fill(theme::LAUNCHER_BACKGROUND)
        .radius(14.0)
        .padding(Padding {
            top: PADDING,
            right: PADDING,
            bottom: PADDING,
            left: PADDING,
        })
        // the clicks inside the box do not close it
        .on_click(|_| {})
        .child(Column::new(items).gap(GAP));

    let child = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::TRANSPARENT)
        .align_child(Center, Center)
        .on_click(|_| close())
        .child(panel);

    LayerWindow::new()
        .width(monitor.width as f32)
        .height(monitor.height as f32)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Left)
        .margin(Margin {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .keyboard(Keyboard::Exclusive)
        .space(Zone::Ignore)
        .namespace("amane-add-widget")
        .visible(open)
        .on_key(|key| {
            if key == Key::Escape {
                close();
            }
        })
        .child(child)
}

fn row(spec: &'static widgets::Spec) -> Rectangle {
    let labels: Vec<Box<dyn Widget>> = vec![
        Box::new(
            Text::new(spec.title)
                .size(13.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        ),
        Box::new(
            Text::new(spec.description)
                .size(11.0)
                .color(Color::from(theme::PANEL_MUTED)),
        ),
    ];

    let size = format!("{}×{}", spec.default_size.0, spec.default_size.1);

    Rectangle::new()
        .width(Parent)
        .height(ROW)
        .radius(10.0)
        .padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 4.0,
            left: 8.0,
        })
        .align_child(Start, Center)
        .cursor(amane::Pointer)
        .on_click(move |_| choose(spec.kind))
        .child(
            Row::new(vec![
                Box::new(
                    Text::new(spec.icon)
                        .size(16.0)
                        .color(Color::from(theme::PANEL_MUTED)),
                ) as Box<dyn Widget>,
                Box::new(Column::new(labels).gap(2.0)),
                Box::new(
                    Text::new(size)
                        .size(11.0)
                        .color(Color::from(theme::PANEL_MUTED)),
                ),
            ])
            .gap(10.0)
            .height(Parent)
            .align(Center),
        )
}

fn choose(kind: widgets::Kind) {
    if crate::home::add(kind) {
        close();

        return;
    }

    // no room: the panel stays open and says so
    AddState::write().full = true;
}
