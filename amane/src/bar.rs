use amane::{
    Center, End, Full, Layer, LayerWindow, Margin, Monitor, Padding, Parent, Rectangle, Row, Start,
    Vertical, Widget, Zone, children,
};

use crate::modules;
use crate::theme;

pub fn view(monitor: &Monitor) -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(44.0)
        .anchor_vertical(Vertical::Bottom)
        .margin(Margin {
            top: 0,
            right: 7,
            bottom: 0,
            left: 7,
        })
        .layer(Layer::Top)
        .space(Zone::Reserve)
        .namespace("amane-bar")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(43.0)
                .fill(theme::BAR_BACKGROUND)
                .radius_top_left(7.0)
                .radius_top_right(7.0)
                .padding(Padding {
                    top: 0.0,
                    right: 7.0,
                    bottom: 0.0,
                    left: 7.0,
                })
                .translate(0.0, 1.0)
                .child(
                    Row::new(children![
                        left_section(monitor),
                        center_section(),
                        right_section(),
                    ])
                    .height(Parent)
                    .align(Center),
                ),
        )
}

fn left_section(monitor: &Monitor) -> Rectangle {
    section(
        Start,
        Row::new(children![
            modules::workspaces::view(monitor),
            modules::streams::runcat(),
        ])
        .gap(7.0)
        .height(Parent)
        .align(Center),
    )
}

fn center_section() -> Rectangle {
    section(
        Center,
        Row::new(children![
            modules::streams::cava_left(),
            modules::launcher::view(),
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
