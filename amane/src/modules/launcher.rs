use amane::{Button, Image, Parent, Pointer, Rectangle};

use crate::system_panel;

const LOGO: &str = "/home/mori/Documents/assets/ui/logo.svg";

pub fn view() -> Rectangle {
    Rectangle::new()
        .width(33.0)
        .height(Parent)
        .fill(Image::contain(LOGO))
        .cursor(Pointer)
        .on_click(|button| match button {
            Button::Left => {
                system_panel::close();
                amane::spawn("wofi --show drun");
            }
            Button::Right => system_panel::toggle(),
            Button::Middle => {}
        })
}
