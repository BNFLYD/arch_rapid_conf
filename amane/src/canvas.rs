use amane::{Color, Fill, Full, Image, Layer, LayerWindow, Monitor, Parent, Rectangle, Zone};

use crate::wallpaper::{self, Backdrop};

pub fn view(monitor: &Monitor) -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(Full)
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .namespace("amane-canvas")
        // always mapped: painting a fullscreen surface late takes a visible
        // moment. no input region, so no click is lost here
        .visible(true)
        .click_through()
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(backdrop(monitor)),
        )
}

// the same picture sway paints, so opening the menu looks like the apps
// simply went away
fn backdrop(monitor: &Monitor) -> Fill {
    if !crate::bar::menu_open() {
        // closed: the canvas paints nothing
        return Color::TRANSPARENT.into();
    }

    match wallpaper::for_monitor(monitor) {
        // the same framing as sway's "fill": cover the screen and cut the rest
        Backdrop::Image(path) => Image::cover(path)
            .thumbnail(monitor.width, monitor.height)
            .into(),
        Backdrop::Color(color) => Color::from(color.as_str()).into(),
    }
}
