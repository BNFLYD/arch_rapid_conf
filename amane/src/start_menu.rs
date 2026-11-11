use std::cell::RefCell;
use std::time::Duration;

use amane::{
    Animation, Canvas, Color, Easing, End, Horizontal, Key, Keyboard, Layer, LayerWindow, Margin,
    Monitor, Parent, Rectangle, Service, Start, Vertical, Zone,
};

use crate::theme;

// the bar is 44 px tall and stays visible under the menu
const BAR_HEIGHT: i32 = 44;

// how long the menu takes to unroll from the bar to the top of the screen.
// it is long on purpose: the window is big at the end of the unroll and a big
// window takes about 120 ms to paint, so a short one would show only two frames
const UNROLL: Duration = Duration::from_millis(500);

struct StartMenuState {
    open: bool,
}

impl Service for StartMenuState {
    fn new() -> Self {
        Self { open: false }
    }

    fn listen() {}
}

pub fn toggle() {
    // the read ends before the write: both on the same service would wait
    // for each other
    let is_open = StartMenuState::read().open;

    let mut state = StartMenuState::write();
    state.open = !is_open;
}

pub fn close() {
    let mut state = StartMenuState::write();
    state.open = false;
}

/// Lets a keybinding open the menu: `amane ipc call start toggle`.
pub fn ipc(args: &[String]) -> String {
    match args.first().map(String::as_str) {
        Some("open") => {
            let mut state = StartMenuState::write();
            state.open = true;
            "abierto".to_string()
        }
        Some("close") => {
            close();
            "cerrado".to_string()
        }
        Some("state") => {
            if StartMenuState::read().open {
                "abierto".to_string()
            } else {
                "cerrado".to_string()
            }
        }
        _ => {
            toggle();
            "alternado".to_string()
        }
    }
}

// the unroll: the menu grows from the bar up to the top of the screen.
// while it moves, the library keeps asking for frames on its own.
thread_local! {
    static HEIGHT: RefCell<Animation<f32>> = RefCell::new(
        Animation::new(0.0).duration(UNROLL).easing(Easing::Out)
    );
}

fn unroll(open: bool) -> f32 {
    HEIGHT.with(|height| {
        let mut animation = height.borrow_mut();

        animation.to(if open { 1.0 } else { 0.0 });
        animation.value()
    })
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let open = StartMenuState::read().open;
    let amount = unroll(open);

    // the area above the bar, where the menu unrolls
    let area = monitor.height as f32 - BAR_HEIGHT as f32;

    // the canvas is always the full height of that area and sits on the bar,
    // so the top of it appears as the menu grows
    let canvas = Canvas::new().width(Parent).height(area).shapes(Vec::new());

    let menu = Rectangle::new()
        .width(Parent)
        .height(area * amount)
        .fill(theme::START_MENU_BACKGROUND)
        .clip()
        .align_child(Start, End)
        // the canvas has no click handler, so a click on it lands here
        .on_click(|_| close())
        .child(canvas);

    let child = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::TRANSPARENT)
        .align_child(Start, End)
        .on_click(|_| close())
        .child(menu);

    LayerWindow::new()
        .width(monitor.width as f32)
        // the size of the window never changes: resizing it on every frame
        // makes wayland pile up the configures and the shell loses its
        // surfaces with a protocol error
        .height(area)
        .anchor_vertical(Vertical::Bottom)
        .anchor_horizontal(Horizontal::Left)
        // the margin leaves the bar alone: without it the menu would sit on
        // top of it, the window is anchored to the bottom of the screen
        .margin(Margin {
            top: 0,
            right: 0,
            bottom: BAR_HEIGHT,
            left: 0,
        })
        .layer(Layer::Overlay)
        .keyboard(Keyboard::Exclusive)
        .space(Zone::Ignore)
        .namespace("amane-start-menu")
        // the window stays up while it rolls back up, so the closing shows too
        .visible(open || amount > 0.0)
        .on_key(|key| {
            if key == Key::Escape {
                close();
            }
        })
        .child(child)
}
