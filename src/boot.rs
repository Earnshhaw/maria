use std::sync::Arc;

use iced::{widget::image::Handle};
use crate::{dirs::app_data_dir, gather_games::load_saved_games, gather_runners::fetch_runners, gui::State, launch::{GameEntry, ProtonVariant, Runner}};

pub const PLACEHOLDER_GRID: &str = "blank.png";

pub fn boot() -> State {
    let mut game_entries: Vec<(GameEntry, Handle)> = vec![];
    for entry in load_saved_games().unwrap() {
        match entry {
            GameEntry::Local(game) => {
                if !game.get_grid().try_exists().unwrap() {
                    let handle = Handle::from_path(app_data_dir().unwrap().join(format!("grids/{}", PLACEHOLDER_GRID)));
                    game_entries.push((GameEntry::Local(game), handle));
                } else {
                    let handle = Handle::from_path(game.get_grid());
                    game_entries.push((GameEntry::Local(game), handle));
                }
            },
            GameEntry::Steam(game) => {
                if !game.get_grid().try_exists().unwrap() {
                    let handle = Handle::from_path(app_data_dir().unwrap().join(format!("grids/{}", PLACEHOLDER_GRID)));
                    game_entries.push((GameEntry::Steam(game), handle));
                } else {
                    let handle = Handle::from_path(game.get_grid());
                    game_entries.push((GameEntry::Steam(game), handle));
                }
            }
        }
    }
    let mut runners: Vec<Runner> = fetch_runners().into_iter().map(|r| Runner::Proton(ProtonVariant::Custom(r))).collect();
    runners.extend(vec![Runner::Native, Runner::Wine, Runner::Proton(ProtonVariant::GEProton), Runner::Proton(ProtonVariant::UMUProton)]);

    
    State {
        game_entries: game_entries,
        runners: runners,
        running_game: None
    }
}