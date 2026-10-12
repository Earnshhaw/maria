mod launch;
mod boot;
mod config;
mod gather_games;
mod gui;
mod style;
mod api_keys;
mod id;
mod dirs;
mod gather_runners;
use iced::{Font, Settings, Theme, application};
use crate::{boot::boot, gui::State};

fn main() -> iced::Result {
    application(boot, State::update, State::view).settings(
        Settings {
            vsync: false,
            default_font: Font::with_name("JetBrains Mono"),
            antialiasing: true,
            ..Default::default()
        })
    .theme(Theme::KanagawaWave)
    .run()
}