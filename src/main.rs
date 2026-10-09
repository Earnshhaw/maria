mod launch;
mod boot;
mod config;
mod library;
mod gui;
mod api_keys;
mod gather_runners;
use iced::application;
use crate::{boot::boot, gui::State};

fn main() -> iced::Result {
    let (boot, _) = boot();
    application(boot, State::update, State::view)
        .run()
}