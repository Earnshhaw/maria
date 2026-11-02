use iced::border::Radius;
use iced::widget::button;
use iced::{Border, Color};

pub fn button_card(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Color::TRANSPARENT.into()),
        border: Border {
            width: 1.0,
            color: Color::TRANSPARENT,
            radius: Radius::new(20),
        },
        ..Default::default()
    }
}
