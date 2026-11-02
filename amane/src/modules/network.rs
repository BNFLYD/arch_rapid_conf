use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use amane::{Color, Rectangle, Service, Text};

use crate::theme;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Sample {
    connected: bool,
    wireless: bool,
    bytes_per_second: u64,
}

pub struct NetworkStatus {
    sample: Sample,
    interface: Option<String>,
    previous: Option<(Instant, u64)>,
}

impl Service for NetworkStatus {
    fn new() -> Self {
        let mut status = Self {
            sample: Sample::default(),
            interface: None,
            previous: None,
        };

        status.refresh();

        status
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    fn update(&mut self) -> bool {
        let sample = self.refresh();
        let changed = sample != self.sample;

        self.sample = sample;

        changed
    }
}

impl NetworkStatus {
    fn refresh(&mut self) -> Sample {
        let interface = default_interface();
        let connected = interface.as_deref().map(is_connected).unwrap_or(false);
        let wireless = interface.as_deref().map(is_wireless).unwrap_or(false);

        let bytes = interface.as_deref().and_then(|name| received_bytes(name));

        let same_interface = self.interface.as_deref() == interface.as_deref();
        let previous = if same_interface {
            self.previous.take()
        } else {
            None
        };
        let now = Instant::now();

        let bytes_per_second = match (bytes, previous) {
            (Some(current), Some((last_time, last_bytes))) => current
                .saturating_sub(last_bytes)
                .saturating_mul(1_000)
                .checked_div(last_time.elapsed().as_millis().max(1) as u64)
                .unwrap_or(0),
            _ => 0,
        };

        self.interface = interface;
        self.previous = bytes.map(|value| (now, value));

        Sample {
            connected,
            wireless,
            bytes_per_second,
        }
    }
}

pub fn view() -> Rectangle {
    let network = NetworkStatus::read();

    let label = if network.sample.connected {
        let icon = if network.sample.wireless {
            ""
        } else {
            ""
        };

        format!("{icon}  {}", format_rate(network.sample.bytes_per_second))
    } else {
        String::from("󰖟 N/A")
    };

    Rectangle::new()
        .width(118.0)
        .height(24.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(amane::Center, amane::Center)
        .child(
            Text::new(label)
                .size(14.0)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}

fn default_interface() -> Option<String> {
    let routes = fs::read_to_string("/proc/net/route").ok()?;

    default_interface_from_routes(&routes)
}

fn default_interface_from_routes(routes: &str) -> Option<String> {
    routes
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();

            if fields.len() < 7 || fields[1] != "00000000" {
                return None;
            }

            let flags = u32::from_str_radix(fields[3], 16).ok()?;
            if flags & 0x1 == 0 {
                return None;
            }

            let metric = fields[6].parse::<u64>().ok()?;

            Some((metric, fields[0].to_owned()))
        })
        .min_by_key(|(metric, _)| *metric)
        .map(|(_, interface)| interface)
}

fn is_connected(interface: &str) -> bool {
    let state =
        fs::read_to_string(format!("/sys/class/net/{interface}/operstate")).unwrap_or_default();
    let state = state.trim();

    if state == "up" {
        return true;
    }

    if state == "unknown" {
        return fs::read_to_string(format!("/sys/class/net/{interface}/carrier"))
            .map(|value| value.trim() == "1")
            .unwrap_or(false);
    }

    false
}

fn is_wireless(interface: &str) -> bool {
    let base = Path::new("/sys/class/net").join(interface);

    base.join("wireless").exists() || base.join("phy80211").exists()
}

fn received_bytes(interface: &str) -> Option<u64> {
    let stats = fs::read_to_string("/proc/net/dev").ok()?;

    received_bytes_from_stats(&stats, interface)
}

fn received_bytes_from_stats(stats: &str, interface: &str) -> Option<u64> {
    stats.lines().skip(1).find_map(|line| {
        let (name, values) = line.split_once(':')?;

        if name.trim() != interface {
            return None;
        }

        values.split_whitespace().next()?.parse().ok()
    })
}

fn format_rate(bytes_per_second: u64) -> String {
    let bits = bytes_per_second.saturating_mul(8);

    if bits < 1_000 {
        format!("{bits} b/s")
    } else if bits < 1_000_000 {
        format!("{:.0} Kb/s", bits as f64 / 1_000.0)
    } else if bits < 1_000_000_000 {
        format!("{:.1} Mb/s", bits as f64 / 1_000_000.0)
    } else {
        format!("{:.1} Gb/s", bits as f64 / 1_000_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_default_route() {
        let routes = "Iface\tDestination\tGateway\tFlags\n\
eth0\t00000000\t01000000\t0003\t0\t0\t100\t00000000\t0\t0\t0\n\
wlan0\t00000000\t02000000\t0001\t0\t0\t10\t00000000\t0\t0\t0\n";

        assert_eq!(
            default_interface_from_routes(routes).as_deref(),
            Some("wlan0")
        );
    }

    #[test]
    fn reads_received_bytes() {
        let stats = "Inter-| face |bytes\n  eth0: 1234 10 0 0 0 0 0 0 20 1\n";

        assert_eq!(received_bytes_from_stats(stats, "eth0"), Some(1234));
    }

    #[test]
    fn formats_network_rates() {
        assert_eq!(format_rate(0), "0 b/s");
        assert_eq!(format_rate(125), "1 Kb/s");
        assert_eq!(format_rate(125_000), "1.0 Mb/s");
    }
}
