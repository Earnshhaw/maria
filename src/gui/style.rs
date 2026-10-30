use iced::border::Radius;
use iced::widget::{button};
use iced::{Border, Color, Shadow};

pub fn button_card(theme: &iced::Theme, status: button::Status) -> button::Style {
    let border_color = match status {
        button::Status::Hovered => {
            theme.palette().primary
        },
        _ => {
            Color::TRANSPARENT
        }
    };


    button::Style {
        background: Some(Color::TRANSPARENT.into()),
        border: Border {
            width: 1.0,
            color: border_color,
            radius: Radius::new(2)
        },
        ..Default::default()
    }
}
