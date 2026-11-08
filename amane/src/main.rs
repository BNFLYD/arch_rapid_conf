mod app_launcher;
mod bar;
mod modules;
mod network_panel;
mod system_panel;
mod theme;
mod usage;
mod volume_panel;

use amane::App;

fn main() {
    App::new()
        .font(theme::FONT)
        .window_per_monitor(system_panel::view)
        .window_per_monitor(app_launcher::view)
        // after the launcher, so the password field takes the keyboard focus
        .window_per_monitor(network_panel::view)
        .window_per_monitor(volume_panel::view)
        .window_per_monitor(bar::view)
        .ipc("launcher", app_launcher::ipc)
        .ipc("panel", system_panel::ipc)
        .ipc("network", network_panel::ipc)
        .ipc("volume", volume_panel::ipc)
        .run();
}
