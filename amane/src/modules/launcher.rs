use amane::{Button, Image, Padding, Parent, Pointer, Rectangle};

use super::PADDING;
use crate::app_launcher;
use crate::system_panel;

const LOGO: &str = "/home/mori/Documents/assets/ui/logo.svg";
const LOGO_WIDTH: f32 = 33.0;

pub fn view() -> Rectangle {
    Rectangle::new()
        .width(LOGO_WIDTH + PADDING * 2.0)
        .height(Parent)
        .padding(Padding {
            top: 0.0,
            right: PADDING,
            bottom: 0.0,
            left: PADDING,
        })
        .cursor(Pointer)
        .on_click(|button| match button {
            Button::Left => {
                system_panel::close();
                app_launcher::toggle();
            }
            Button::Right => system_panel::toggle(),
            Button::Middle => {}
        })
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Image::contain(LOGO)),
        )
}
