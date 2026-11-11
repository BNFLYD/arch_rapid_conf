use std::time::Duration;

use amane::{
    Animation, Center, Color, Easing, End, Full, Key, Keyboard, Layer, LayerWindow, Margin,
    Monitor, Padding, Parent, Rectangle, Row, Service, Start, Vertical, Widget, Zone, children,
};

use crate::modules;
use crate::theme;

// how long the roll up takes
const ROLL: Duration = Duration::from_millis(150);

// the menu state lives in a service because the canvas, the menu and the bar
// are three windows, and each one is drawn on its own thread
struct Roll {
    open: bool,
    amount: Animation<f32>,
}

impl Service for Roll {
    fn new() -> Self {
        Self {
            open: false,
            amount: Animation::new(0.0).duration(ROLL).easing(Easing::Out),
        }
    }

    fn listen() {}
}

pub fn menu_toggle() {
    let mut state = Roll::write();
    let open = !state.open;

    state.open = open;
    state.amount.to(if open { 1.0 } else { 0.0 });
}

pub fn menu_close() {
    let mut state = Roll::write();

    state.open = false;
    state.amount.to(0.0);
}

pub fn menu_open() -> bool {
    Roll::read().open
}

/// 0 closed, 1 fully rolled up. Reading it keeps both windows asking for frames.
pub fn menu_amount() -> f32 {
    Roll::read().amount.value()
}

/// Lets a keybinding open the menu: `amane ipc call start toggle`.
pub fn menu_ipc(args: &[String]) -> String {
    match args.first().map(String::as_str) {
        Some("open") => {
            let mut state = Roll::write();
            state.open = true;
            state.amount.to(1.0);
            "abierto".to_string()
        }
        Some("close") => {
            menu_close();
            "cerrado".to_string()
        }
        Some("state") => {
            if menu_open() {
                "abierto".to_string()
            } else {
                "cerrado".to_string()
            }
        }
        _ => {
            menu_toggle();
            "alternado".to_string()
        }
    }
}

/// The menu block. It sits on the overlay layer, under the bar, which is
/// mapped after it.
pub fn start_menu(monitor: &Monitor) -> LayerWindow {
    let open = menu_open();
    let amount = menu_amount();

    // the area above the bar, which the block may fill
    let area = monitor.height as f32 - theme::MENU_MARGIN - theme::BAR_WINDOW_HEIGHT;

    // the block grows from the top edge of the bar upwards
    let block = Rectangle::new()
        .width(Parent)
        .height(area * amount)
        .fill(theme::BAR_BACKGROUND)
        .radius_top_left(theme::BAR_RADIUS)
        .radius_top_right(theme::BAR_RADIUS)
        .clip()
        // a click inside the block has to land somewhere, or it reaches the
        // frame and closes the menu
        .on_click(|_| {});

    // the transparent frame: its padding is the side margin, the air above
    // the block and the whole bar window below it
    let root = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::TRANSPARENT)
        .padding(Padding {
            top: theme::MENU_MARGIN,
            right: theme::SIDE_MARGIN,
            bottom: theme::BAR_WINDOW_HEIGHT,
            left: theme::SIDE_MARGIN,
        })
        .align_child(Start, End)
        // the block has no close handler, so only the frame closes the menu
        .on_click(|_| menu_close())
        .child(block);

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .layer(Layer::Overlay)
        .keyboard(Keyboard::Exclusive)
        .space(Zone::Ignore)
        .namespace("amane-start-menu")
        // the window stays up while the block rolls back down
        .visible(open || amount > 0.0)
        .on_key(|key| {
            if key == Key::Escape {
                menu_close();
            }
        })
        .child(root)
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let amount = menu_amount();

    LayerWindow::new()
        .width(Full)
        .height(theme::BAR_WINDOW_HEIGHT)
        .anchor_vertical(Vertical::Bottom)
        .margin(Margin {
            // the window carries the air below the pill, so the space it
            // reserves stays at BAR_HEIGHT: 51 - 7
            top: -(theme::MENU_MARGIN as i32),
            right: theme::SIDE_MARGIN as i32,
            bottom: 0,
            left: theme::SIDE_MARGIN as i32,
        })
        // on the overlay layer, mapped after the menu, so it stays above it
        .layer(Layer::Overlay)
        .space(Zone::Reserve)
        .namespace("amane-bar")
        .child(
            // at the bottom of the window, where the bar lives
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::TRANSPARENT)
                .align_child(Start, End)
                .child(pill(monitor, amount)),
        )
}

fn pill(monitor: &Monitor, amount: f32) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        // the pill loses its air and its top corners as the menu deploys and
        // gains the bottom ones: both end up as one floating piece
        .height(theme::BAR_HEIGHT - 1.0 + amount)
        .fill(theme::BAR_BACKGROUND)
        .radius_top_left(theme::BAR_RADIUS * (1.0 - amount))
        .radius_top_right(theme::BAR_RADIUS * (1.0 - amount))
        .radius_bottom_left(theme::BAR_RADIUS * amount)
        .radius_bottom_right(theme::BAR_RADIUS * amount)
        .padding(Padding {
            top: 0.0,
            right: theme::SIDE_MARGIN,
            bottom: 0.0,
            left: theme::SIDE_MARGIN,
        })
        // open, it lifts off the bottom edge of the screen
        .translate(0.0, -theme::MENU_MARGIN * amount)
        .child(
            Row::new(children![
                left_section(),
                center_section(monitor),
                right_section(),
            ])
            .height(Parent)
            .align(Center),
        )
}

fn left_section() -> Rectangle {
    section(
        Start,
        Row::new(children![modules::launcher::view(),])
            .gap(7.0)
            .height(Parent)
            .align(Center),
    )
}

fn center_section(monitor: &Monitor) -> Rectangle {
    section(
        Center,
        Row::new(children![
            modules::streams::cava_left(),
            modules::workspaces::view(monitor),
            modules::streams::cava_right(),
        ])
        .gap(7.0)
        .height(Parent)
        .align(Center),
    )
}

fn right_section() -> Rectangle {
    section(
        End,
        Row::new(children![
            modules::network::view(),
            modules::volume::view(),
            modules::clock::view(),
        ])
        .gap(7.0)
        .height(Parent)
        .align(Center),
    )
}

fn section(alignment: impl Into<amane::Align>, content: impl Widget + 'static) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(alignment, Center)
        .child(content)
}
