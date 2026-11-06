mod app_launcher;
mod bar;
mod modules;
mod system_panel;
mod theme;
mod usage;

use amane::App;

fn main() {
    App::new()
        .font(theme::FONT)
        .window(system_panel::view)
        .window_per_monitor(app_launcher::view)
        .window_per_monitor(bar::view)
        .ipc("launcher", app_launcher::ipc)
        .run();
}
