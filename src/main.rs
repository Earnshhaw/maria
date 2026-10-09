mod launch;
mod boot;
mod config;
mod gather_games;
mod gui;
mod api_keys;
mod id;
mod dirs;
mod gather_runners;
use iced::application;
use crate::{boot::boot, gui::State};

fn main() -> iced::Result {
    application(boot, State::update, State::view)
        .run()
}