use amane::{
    Center, Color, Column, End, Parent, Rectangle, Row, Service, SpaceBetween, Stack, Text, Weight,
    Widget,
};

use crate::home::Instance;
use crate::modules::activity::{Activity, Day};
use crate::theme;
use crate::widgets::Look;

const CELL: f32 = 10.0;
const GAP: f32 = 2.0;
const COLUMNS: usize = 53;
const WEEKDAY_LABELS: f32 = 26.0;
const MONTH_LABELS: f32 = 14.0;
const GRID_WIDTH: f32 = COLUMNS as f32 * CELL + (COLUMNS - 1) as f32 * GAP;
const GRID_HEIGHT: f32 = 7.0 * CELL + 6.0 * GAP;
const WIDTH: f32 = WEEKDAY_LABELS + GRID_WIDTH;
const HEIGHT: f32 = MONTH_LABELS + GRID_HEIGHT;

// Local commits, grouped by their committer day. Remotes and visibility do not matter.
pub fn view(_instance: &Instance, look: &Look) -> Rectangle {
    let activity = Activity::read();
    let unit = if activity.total == 1 {
        "commit"
    } else {
        "commits"
    };

    let header = Row::new(vec![
        Box::new(
            Text::new("Actividad local")
                .size(11.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        ) as Box<dyn Widget>,
        Box::new(
            Text::new(format!("{} {unit} · último año", activity.total))
                .size(9.0)
                .color(Color::from(theme::PANEL_MUTED)),
        ),
    ])
    .width(Parent)
    .justify(SpaceBetween);

    let content = Column::new(vec![
        Box::new(header) as Box<dyn Widget>,
        Box::new(graph(&activity.days)),
        Box::new(legend()),
    ])
    .width(WIDTH)
    .gap(4.0);

    drop(activity);

    let tile = Rectangle::new()
        .width(look.width)
        .height(look.height)
        .fill(theme::MODULE_BACKGROUND)
        .radius(theme::BAR_RADIUS)
        .align_child(Center, Center)
        .on_click(|_| {
            crate::home::clicked();
        });

    let tile = match look.resizing {
        Some(size) => tile.child(Stack::new(vec![
            Box::new(content) as Box<dyn Widget>,
            Box::new(crate::widgets::size_label(size, look.width, look.height)),
        ])),
        None => tile.child(content),
    };

    if !look.editing {
        tile
    } else if look.deleting {
        tile.border(1.0, theme::DELETE_BORDER)
    } else {
        tile.border(1.0, theme::EDIT_BORDER)
    }
}

fn graph(days: &[Day]) -> Stack {
    if days.is_empty() {
        return Stack::new(vec![Box::new(
            Rectangle::new()
                .width(WIDTH)
                .height(HEIGHT)
                .align_child(Center, Center)
                .child(
                    Text::new("Leyendo tus commits locales…")
                        .size(9.0)
                        .color(Color::from(theme::PANEL_MUTED)),
                ),
        )])
        .width(WIDTH)
        .height(HEIGHT);
    }

    let mut items: Vec<Box<dyn Widget>> = vec![Box::new(month_labels(days))];

    for (row, label) in [(1, "Lun"), (3, "Mié"), (5, "Vie")] {
        items.push(Box::new(label_at(
            label,
            0.0,
            MONTH_LABELS + row as f32 * (CELL + GAP),
        )));
    }

    for (index, day) in days.iter().enumerate() {
        let column = index / 7;
        let row = index % 7;

        items.push(Box::new(
            Rectangle::new()
                .width(CELL)
                .height(CELL)
                .fill(theme::HEATMAP[level(day.count)])
                .radius(2.0)
                .translate(
                    WEEKDAY_LABELS + column as f32 * (CELL + GAP),
                    MONTH_LABELS + row as f32 * (CELL + GAP),
                ),
        ));
    }

    Stack::new(items).width(WIDTH).height(HEIGHT)
}

fn month_labels(days: &[Day]) -> Stack {
    const MONTHS: [&str; 12] = [
        "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
    ];

    let mut labels: Vec<Box<dyn Widget>> = Vec::new();
    let mut previous = None;
    let mut last_column = None;

    for (index, day) in days.iter().enumerate() {
        if previous == Some(day.month) {
            continue;
        }

        previous = Some(day.month);
        let column = index / 7;

        // Prefer the first full month over a partial starting month if they collide.
        if last_column.is_some_and(|last| column < last + 3) {
            if labels.len() == 1 {
                labels.clear();
            } else {
                continue;
            }
        }

        last_column = Some(column);

        if let Some(month) = day
            .month
            .checked_sub(1)
            .and_then(|month| MONTHS.get(month as usize))
        {
            labels.push(Box::new(label_at(
                month,
                WEEKDAY_LABELS + column as f32 * (CELL + GAP),
                0.0,
            )));
        }
    }

    Stack::new(labels).width(WIDTH).height(CELL)
}

fn label_at(text: &'static str, x: f32, y: f32) -> Rectangle {
    Rectangle::new()
        .width(24.0)
        .height(CELL)
        .child(
            Text::new(text)
                .size(8.0)
                .color(Color::from(theme::PANEL_MUTED)),
        )
        .translate(x, y)
}

fn legend() -> Row {
    let mut items: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new("Menos")
            .size(8.0)
            .color(Color::from(theme::PANEL_MUTED)),
    )];

    for color in theme::HEATMAP {
        items.push(Box::new(
            Rectangle::new()
                .width(CELL)
                .height(CELL)
                .fill(color)
                .radius(2.0),
        ));
    }

    items.push(Box::new(
        Text::new("Más")
            .size(8.0)
            .color(Color::from(theme::PANEL_MUTED)),
    ));

    Row::new(items)
        .width(Parent)
        .gap(4.0)
        .align(Center)
        .justify(End)
}

fn level(count: u32) -> usize {
    match count {
        0 => 0,
        1..=2 => 1,
        3..=5 => 2,
        6..=9 => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::level;

    #[test]
    fn activity_has_five_readable_levels() {
        assert_eq!([0, 1, 3, 6, 10].map(level), [0, 1, 2, 3, 4]);
    }
}
