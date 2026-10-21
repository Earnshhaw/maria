mod api_keys;
mod config;
mod data_model;
mod dirs;
mod disk_sync;
mod error;
mod gather_games;
mod gather_runners;
mod gui;
mod id;
mod launch;
mod misc;
use crate::gui::{State, boot};
use iced::{Font, Settings, Theme, application};

fn main() -> iced::Result {
    application(boot, State::update, State::view)
        .settings(Settings {
            vsync: false,
            default_font: Font::with_name("JetBrainsMono Nerd Font"),
            antialiasing: true,
            ..Default::default()
        })
        .theme(Theme::KanagawaWave)
        .run()
}
