use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::{
    Arc, Battery, Canvas, Cap, Color, Column, Cpu, Full, Horizontal, Image, Key, Keyboard, Layer,
    LayerWindow, Margin, Memory, Monitor, Padding, Parent, Rectangle, Row, Service, Shape, Size,
    SpaceBetween, Stack, Start, Text, Vertical, Weight, Widget, Zone, children, shapes,
};

use crate::modules::network::NetworkStatus;
use crate::modules::streams;
use crate::theme;

// the same width as the app launcher, so both overlays feel the same
const WIDTH_RATIO: f32 = 0.52;
const PADDING: f32 = 16.0;
const GAP: f32 = 8.0;
const RING_SIZE: f32 = 96.0;
const RING_STROKE: f32 = 7.0;

struct PanelState {
    open: bool,
}

impl Service for PanelState {
    fn new() -> Self {
        Self { open: false }
    }

    fn listen() {}
}

pub fn toggle() {
    let open = PanelState::read().open;
    let mut state = PanelState::write();
    state.open = !open;
}

/// Whether the panel is open, for the modules that only move while it shows.
pub fn is_open() -> bool {
    PanelState::read().open
}

pub fn close() {
    let mut state = PanelState::write();
    state.open = false;
}

/// Lets a keybinding open the panel: `amane ipc call panel toggle`.
pub fn ipc(args: &[String]) -> String {
    match args.first().map(String::as_str) {
        Some("open") => {
            let mut state = PanelState::write();
            state.open = true;
            "abierto".to_string()
        }
        Some("close") => {
            close();
            "cerrado".to_string()
        }
        Some("state") => {
            if PanelState::read().open {
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

#[derive(Debug, PartialEq)]
struct SystemInfo {
    hostname: String,
    os: String,
    kernel: String,
    uptime_seconds: u64,
    disk_used: u64,
    disk_available: u64,
    temperature: Option<u32>,
}

impl Service for SystemInfo {
    fn new() -> Self {
        Self {
            hostname: read_trim("/etc/hostname").unwrap_or_else(|| String::from("localhost")),
            os: os_name(),
            kernel: read_trim("/proc/sys/kernel/osrelease").unwrap_or_default(),
            uptime_seconds: 0,
            disk_used: 0,
            disk_available: 0,
            temperature: None,
        }
    }

    fn listen() {
        loop {
            let uptime_seconds = uptime();
            let (disk_used, disk_available) = disk_usage();
            let temperature = cpu_temperature();

            let changed = {
                let current = SystemInfo::read();
                current.uptime_seconds != uptime_seconds
                    || current.disk_used != disk_used
                    || current.disk_available != disk_available
                    || current.temperature != temperature
            };

            if changed {
                let mut current = SystemInfo::write();
                current.uptime_seconds = uptime_seconds;
                current.disk_used = disk_used;
                current.disk_available = disk_available;
                current.temperature = temperature;
            }

            thread::sleep(Duration::from_secs(5));
        }
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let open = PanelState::read().open;

    // the window covers the whole screen, so a click outside the box closes it
    let child = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::TRANSPARENT)
        .align_child(amane::Center, amane::Center)
        .on_click(|_| close())
        .child(content(monitor.width as f32 * WIDTH_RATIO));

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
        .namespace("amane-system-panel")
        .visible(open)
        .on_key(|key| {
            if key == Key::Escape {
                close();
            }
        })
        .child(child)
}

fn content(width: f32) -> Rectangle {
    let column = column();

    Rectangle::new()
        .width(width)
        .height(box_height(&column))
        .fill(theme::LAUNCHER_BACKGROUND)
        .radius(14.0)
        .padding(Padding {
            top: PADDING,
            right: 18.0,
            bottom: PADDING,
            left: 18.0,
        })
        // the clicks inside the box do not close it
        .on_click(|_| {})
        .child(column)
}

/// The height of the system panel, so the other panels can have the same size.
pub fn height() -> f32 {
    box_height(&column())
}

fn box_height(column: &Column) -> f32 {
    // the column knows how tall it is, so the box never has to guess
    match Widget::height(column) {
        Size::Fixed(height) => height + PADDING * 2.0,
        _ => 0.0,
    }
}

fn column() -> Column {
    let cpu = Cpu::read().percent();
    let memory = Memory::read();
    let battery = Battery::read();
    let info = SystemInfo::read();
    let network = NetworkStatus::read();

    let disk_total = info.disk_used.saturating_add(info.disk_available);
    let disk_percent = percent(info.disk_used, disk_total);

    let mut items: Vec<Box<dyn Widget>> = vec![
        Box::new(header(&info.hostname)),
        Box::new(divider()),
        Box::new(rings(cpu, memory.percent(), disk_percent)),
    ];

    items.push(Box::new(section("RED")));
    items.push(Box::new(stat_row("Conexión", connection(&network))));
    items.push(Box::new(stat_row("Descarga", network.download_rate())));

    if battery.present() {
        items.push(Box::new(section("BATERÍA")));
        items.push(Box::new(stat_row("Carga", battery_label(&battery))));
    }

    if let Some(temperature) = info.temperature {
        items.push(Box::new(section("TEMPERATURA")));
        items.push(Box::new(stat_row("Sensor", format!("{temperature} °C"))));
    }

    items.push(Box::new(section("EQUIPO")));
    items.push(Box::new(stat_row("Sistema", info.os.clone())));
    items.push(Box::new(stat_row("Kernel", info.kernel.clone())));
    items.push(Box::new(stat_row(
        "Uptime",
        format_uptime(info.uptime_seconds),
    )));
    items.push(Box::new(actions()));

    Column::new(items).gap(GAP)
}

fn header(hostname: &str) -> Rectangle {
    let username = env::var("USER").unwrap_or_else(|_| String::from("usuario"));

    Rectangle::new().width(Parent).height(50.0).child(
        Row::new(children![
            avatar(),
            Column::new(children![
                Text::new(username)
                    .size(16.0)
                    .weight(Weight::Bold)
                    .color(Color::from(theme::PANEL_FOREGROUND)),
                Text::new(hostname)
                    .size(12.0)
                    .color(Color::from(theme::PANEL_MUTED)),
            ])
            .width(Parent)
            .justify(amane::Center)
            .align(Start),
            close_button(),
        ])
        .gap(10.0)
        .height(Parent)
        .align(amane::Center),
    )
}

fn avatar() -> Rectangle {
    let face = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".face"))
        .unwrap_or_default();

    if face.exists() {
        return Rectangle::new()
            .width(46.0)
            .height(46.0)
            .fill(Image::contain(face))
            .radius(Full)
            .clip();
    }

    Rectangle::new()
        .width(46.0)
        .height(46.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(Full)
        .align_child(amane::Center, amane::Center)
        .child(
            Text::new("")
                .size(23.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}

fn close_button() -> Rectangle {
    Rectangle::new()
        .width(28.0)
        .height(28.0)
        .fill(Color::TRANSPARENT)
        .radius(7.0)
        .align_child(amane::Center, amane::Center)
        .cursor(amane::Pointer)
        .on_click(|_| close())
        .child(
            Text::new("×")
                .size(20.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::PANEL_MUTED)),
        )
}

fn section(label: &str) -> Text {
    Text::new(label)
        .size(11.0)
        .weight(Weight::Bold)
        .color(Color::from(theme::PANEL_MUTED))
}

fn divider() -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(1.0)
        .fill(theme::PANEL_BORDER)
        .opacity(0.5)
}

fn stat_row(label: &str, value: String) -> Row {
    Row::new(children![
        Text::new(label)
            .size(13.0)
            .color(Color::from(theme::PANEL_FOREGROUND)),
        Text::new(value)
            .size(13.0)
            .weight(Weight::Bold)
            .color(Color::from(theme::PANEL_FOREGROUND)),
    ])
    .width(Parent)
    .height(20.0)
    .justify(SpaceBetween)
    .align(amane::Center)
}

// the rings of the dashboard: a circle with the value in the middle
fn rings(cpu: u8, memory: u8, disk: u8) -> Row {
    Row::new(children![
        streams::runcat_view(64.0, 56.0),
        ring(cpu, "CPU"),
        ring(memory, "RAM"),
        ring(disk, "Disco"),
    ])
    .width(Parent)
    .height(RING_SIZE + 22.0)
    .gap(28.0)
    .justify(amane::SpaceEvenly)
    .align(amane::Center)
}

fn ring(percent: u8, label: &str) -> Column {
    let circle = Stack::new(children![
        Canvas::new()
            .width(RING_SIZE)
            .height(RING_SIZE)
            .shapes(shapes![
                Arc::new()
                    .stroke(RING_STROKE, theme::PANEL_BORDER)
                    .opacity(0.4),
                Arc::new()
                    .sweep(360.0 * f32::from(percent) / 100.0)
                    .stroke(RING_STROKE, ring_color(percent))
                    .cap(Cap::Round),
            ]),
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .align_child(amane::Center, amane::Center)
            .child(
                Text::new(format!("{percent}%"))
                    .size(15.0)
                    .weight(Weight::Bold)
                    .color(Color::from(theme::PANEL_FOREGROUND)),
            ),
    ])
    .width(RING_SIZE)
    .height(RING_SIZE);

    Column::new(children![
        circle,
        Text::new(label)
            .size(11.0)
            .color(Color::from(theme::PANEL_MUTED)),
    ])
    .gap(4.0)
    .align(amane::Center)
}

// the same thresholds wayle uses for its rings
fn ring_color(percent: u8) -> &'static str {
    match percent {
        85.. => theme::PANEL_DANGER,
        60..=84 => theme::PANEL_WARNING,
        _ => theme::PANEL_SUCCESS,
    }
}

fn connection(network: &NetworkStatus) -> String {
    if !network.connected() {
        return String::from("Sin conexión");
    }

    let kind = if network.wireless() {
        "Wi-Fi"
    } else {
        "Ethernet"
    };

    match network.interface() {
        Some(interface) => format!("{kind} · {interface}"),
        None => kind.to_string(),
    }
}

fn battery_label(battery: &Battery) -> String {
    let state = if battery.charging() {
        "cargando"
    } else if battery.full() {
        "carga completa"
    } else {
        "por cable"
    };

    format!("{}% · {state}", battery.percent())
}

fn actions() -> Row {
    Row::new(children![
        action_button("Bloquear", "swaylock -f", false),
        action_button("Salir", "swaymsg exit", false),
        action_button("Reiniciar", "systemctl reboot", false),
        action_button("Apagar", "systemctl poweroff", true),
    ])
    .width(Parent)
    .height(34.0)
    .gap(6.0)
    .justify(amane::SpaceEvenly)
    .align(amane::Center)
}

fn action_button(label: &'static str, command: &'static str, danger: bool) -> Rectangle {
    let background = if danger {
        theme::PANEL_DANGER
    } else {
        theme::MODULE_BACKGROUND
    };
    let foreground = if danger {
        theme::PANEL_FOREGROUND
    } else {
        theme::MODULE_FOREGROUND
    };

    Rectangle::new()
        .width(82.0)
        .height(34.0)
        .fill(background)
        .radius(7.0)
        .align_child(amane::Center, amane::Center)
        .cursor(amane::Pointer)
        .on_click(move |_| {
            close();
            amane::spawn(command);
        })
        .child(
            Text::new(label)
                .size(12.0)
                .weight(Weight::Bold)
                .color(Color::from(foreground)),
        )
}

fn read_trim(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn os_name() -> String {
    let os_release = fs::read_to_string("/etc/os-release").unwrap_or_default();

    for line in os_release.lines() {
        if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
            return value.trim_matches('"').to_string();
        }
    }

    String::from("Linux")
}

fn uptime() -> u64 {
    read_trim("/proc/uptime")
        .and_then(|text| text.split_whitespace().next()?.parse::<f64>().ok())
        .map(|seconds| seconds as u64)
        .unwrap_or(0)
}

fn disk_usage() -> (u64, u64) {
    let Ok(output) = Command::new("df").args(["-PB1", "/"]).output() else {
        return (0, 0);
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let Some(line) = text.lines().last() else {
        return (0, 0);
    };
    let fields: Vec<_> = line.split_whitespace().collect();

    if fields.len() < 4 {
        return (0, 0);
    }

    (
        fields[2].parse().unwrap_or(0),
        fields[3].parse().unwrap_or(0),
    )
}

fn cpu_temperature() -> Option<u32> {
    for entry in fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        let Some(name) = read_trim(&entry.path().join("name").to_string_lossy()) else {
            continue;
        };
        if !matches!(name.as_str(), "coretemp" | "k10temp" | "zenpower") {
            continue;
        }

        for index in 1..=10 {
            let path = entry.path().join(format!("temp{index}_input"));
            let Some(value) = read_trim(&path.to_string_lossy()) else {
                continue;
            };
            if let Ok(millidegrees) = value.parse::<i64>() {
                return Some((millidegrees / 1000) as u32);
            }
        }
    }

    None
}

fn percent(used: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }

    ((used as f64 * 100.0 / total as f64) as u8).min(100)
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;

    if days > 0 {
        format!("{days} d {hours} h")
    } else if hours > 0 {
        format!("{hours} h {minutes} min")
    } else {
        format!("{minutes} min")
    }
}

#[cfg(test)]
mod tests {
    use super::{format_uptime, percent, ring_color};
    use crate::theme;

    #[test]
    fn formats_uptime_and_percentages() {
        assert_eq!(format_uptime(60), "1 min");
        assert_eq!(format_uptime(3_660), "1 h 1 min");
        assert_eq!(format_uptime(90_000), "1 d 1 h");
        assert_eq!(percent(1, 3), 33);
        assert_eq!(percent(1, 0), 0);
        assert_eq!(percent(3, 2), 100);
    }

    #[test]
    fn the_ring_color_follows_the_wayle_thresholds() {
        assert_eq!(ring_color(10), theme::PANEL_SUCCESS);
        assert_eq!(ring_color(60), theme::PANEL_WARNING);
        assert_eq!(ring_color(85), theme::PANEL_DANGER);
    }
}
