use amane::{Monitor, Rectangle, Scroll, Service, Text, Weight, Workspaces};

use crate::theme;

pub fn view(monitor: &Monitor) -> Rectangle {
    let workspaces = Workspaces::read();

    let current = workspaces
        .list()
        .iter()
        .find(|workspace| workspace.output() == Some(monitor.name.as_str()) && workspace.active())
        .or_else(|| {
            workspaces.list().iter().find(|workspace| {
                workspace.output() == Some(monitor.name.as_str()) && workspace.focused()
            })
        });

    let label = current
        .map(|workspace| {
            workspace
                .name()
                .map(str::to_owned)
                .unwrap_or_else(|| workspace.index().to_string())
        })
        .unwrap_or_else(|| "—".to_owned());

    let monitor_name = monitor.name.clone();

    Rectangle::new()
        .width(34.0)
        .height(24.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(amane::Center, amane::Center)
        .on_scroll(move |scroll| scroll_workspaces(&monitor_name, scroll))
        .child(
            Text::new(label)
                .size(14.0)
                .weight(Weight::Bold)
                .color(theme::MODULE_FOREGROUND.into()),
        )
}

fn scroll_workspaces(monitor: &str, scroll: Scroll) {
    if scroll.y == 0.0 {
        return;
    }

    let target_id = {
        let workspaces = Workspaces::read();
        let workspaces: Vec<_> = workspaces
            .list()
            .iter()
            .filter(|workspace| workspace.output() == Some(monitor))
            .collect();

        let Some(current) = workspaces
            .iter()
            .position(|workspace| workspace.focused())
            .or_else(|| workspaces.iter().position(|workspace| workspace.active()))
        else {
            return;
        };

        let previous = scroll.y < 0.0;
        let target = cycle_index(current, workspaces.len(), previous);

        workspaces[target].id()
    };

    Workspaces::focus(target_id);
}

fn cycle_index(current: usize, len: usize, previous: bool) -> usize {
    if previous {
        (current + len - 1) % len
    } else {
        (current + 1) % len
    }
}

#[cfg(test)]
mod tests {
    use super::cycle_index;

    #[test]
    fn scrolls_between_workspaces_and_wraps_around() {
        assert_eq!(cycle_index(1, 4, true), 0);
        assert_eq!(cycle_index(1, 4, false), 2);
        assert_eq!(cycle_index(0, 4, true), 3);
        assert_eq!(cycle_index(3, 4, false), 0);
    }
}
