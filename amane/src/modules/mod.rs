use amane::{Size, Widget};

pub(crate) const PADDING: f32 = 7.0;
pub(crate) const GAP: f32 = 7.0;

pub(crate) fn measured_width(widget: &impl Widget) -> f32 {
    match widget.width() {
        Size::Fixed(width) => width,
        Size::Parent => 0.0,
    }
}

pub mod clock;
pub mod launcher;
pub mod network;
pub mod streams;
pub mod volume;
pub mod workspaces;
