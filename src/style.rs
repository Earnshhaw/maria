use iced::{Border, Color, Theme, color, widget::container};

pub fn sidebar_container(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(color!(80, 65, 75).into()),
        border: Border {
            color: Color::BLACK,
            width: 1.0,
            radius: 5.0.into()
        },
        ..Default::default()
    }
}

pub fn gamegrid_container(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(color!(75, 55, 65).into()),
        border: Border {
            color: Color::BLACK,
            width: 1.0,
            radius: 5.0.into()
        },
        ..Default::default()
    }
}

pub fn topbar_container(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(color!(80, 75, 80).into()),
        border: Border {
            color: Color::BLACK,
            width: 1.0,
            radius: 5.0.into()
        },
        ..Default::default()
    }
}
