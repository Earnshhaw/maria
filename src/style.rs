use iced::{Border, Color, Theme, widget::container};

pub fn container_style(theme: &Theme) -> container::Style {
    container::Style {
        border: Border {
            color: Color::BLACK,
            width: 1.0,
            radius: 5.0.into()
        },
        ..Default::default()
    }
}