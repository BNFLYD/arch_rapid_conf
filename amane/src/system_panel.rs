use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::{
    Battery, Color, Column, Cpu, Full, Horizontal, Image, Key, Keyboard, Layer, LayerWindow,
    Margin, Memory, Padding, Parent, Rectangle, Row, Service, SpaceBetween, Start, Text, Vertical,
    Weight, Zone, children,
};

use crate::theme;

const WIDTH: f32 = 390.0;
const BASE_HEIGHT: f32 = 380.0;
const CONTENT_WIDTH: f32 = WIDTH - 36.0;

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
    let mut state = PanelState::write();
    state.open = !state.open;
}

pub fn close() {
    let mut state = PanelState::write();
    state.open = false;
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

pub fn view() -> LayerWindow {
    let open = PanelState::read().open;
    let info = SystemInfo::read();
    let battery = Battery::read();
    let extra_rows = u8::from(info.temperature.is_some()) + u8::from(battery.present());
    let height = BASE_HEIGHT + f32::from(extra_rows) * 28.0;

    let child = if open {
        content()
    } else {
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(Color::TRANSPARENT)
    };

    LayerWindow::new()
        .width(WIDTH)
        .height(height)
        .anchor_vertical(Vertical::Middle)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .keyboard(Keyboard::OnDemand)
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

fn content() -> Rectangle {
    let cpu = Cpu::read().percent();
    let memory = Memory::read();
    let battery = Battery::read();
    let info = SystemInfo::read();

    let memory_used = memory.used_kib().saturating_mul(1024);
    let memory_total = memory.total_kib().saturating_mul(1024);
    let disk_total = info.disk_used.saturating_add(info.disk_available);
    let disk_percent = percent(info.disk_used, disk_total);
    let mut children: Vec<Box<dyn amane::Widget>> = vec![
        Box::new(header(&info.hostname)),
        Box::new(divider()),
        Box::new(section("RENDIMIENTO")),
        Box::new(stat_row("CPU", format!("{cpu}%"))),
        Box::new(progress_bar(cpu)),
        Box::new(stat_row(
            "Memoria",
            format!("{} / {}", gibibytes(memory_used), gibibytes(memory_total)),
        )),
        Box::new(progress_bar(memory.percent())),
        Box::new(stat_row(
            "Disco",
            format!("{} / {}", gibibytes(info.disk_used), gibibytes(disk_total)),
        )),
        Box::new(progress_bar(disk_percent)),
    ];

    if let Some(temperature) = info.temperature {
        children.push(Box::new(stat_row(
            "Temperatura",
            format!("{temperature} °C"),
        )));
    }

    if battery.present() {
        let state = if battery.charging() {
            "cargando"
        } else if battery.full() {
            "carga completa"
        } else {
            "por cable"
        };
        children.push(Box::new(stat_row(
            "Batería",
            format!("{}% · {state}", battery.percent()),
        )));
    }

    children.push(Box::new(section("EQUIPO")));
    children.push(Box::new(stat_row("Sistema", info.os.clone())));
    children.push(Box::new(stat_row("Kernel", info.kernel.clone())));
    children.push(Box::new(stat_row(
        "Uptime",
        format_uptime(info.uptime_seconds),
    )));
    children.push(Box::new(actions()));

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(theme::PANEL_BACKGROUND)
        .radius(14.0)
        .border(1.0, theme::PANEL_BORDER)
        .padding(Padding {
            top: 16.0,
            right: 18.0,
            bottom: 16.0,
            left: 18.0,
        })
        .child(Column::new(children).gap(8.0))
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
            Text::new("")
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

fn progress_bar(percent: u8) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(6.0)
        .fill(theme::PANEL_BORDER)
        .radius(3.0)
        .align_child(Start, amane::Center)
        .child(
            Rectangle::new()
                .width(CONTENT_WIDTH * f32::from(percent) / 100.0)
                .height(6.0)
                .fill(theme::MODULE_BACKGROUND)
                .radius(3.0),
        )
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

fn gibibytes(bytes: u64) -> String {
    format!("{:.1} GiB", bytes as f64 / 1024.0_f64.powi(3))
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
    use super::{format_uptime, percent};

    #[test]
    fn formats_uptime_and_percentages() {
        assert_eq!(format_uptime(60), "1 min");
        assert_eq!(format_uptime(3_660), "1 h 1 min");
        assert_eq!(format_uptime(90_000), "1 d 1 h");
        assert_eq!(percent(1, 3), 33);
        assert_eq!(percent(1, 0), 0);
        assert_eq!(percent(3, 2), 100);
    }
}
