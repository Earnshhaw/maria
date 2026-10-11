use iced::{widget::image::Handle};
use crate::{config::read_config, dirs::{app_data_dir, persistence_dirs_valid}, gather_games::load_saved_games, gather_runners::fetch_runners, gui::State, launch::{GameEntry, ProtonVariant, Runner}};

pub const PLACEHOLDER_GRID: &str = "blank.png";

pub fn boot() -> State {
    if !persistence_dirs_valid() {
        panic!("Persistence dirs could not be initialized. Aborting.");
    }
    let config = read_config().unwrap_or_default();
    
    let mut game_entries: Vec<(GameEntry, Handle)> = vec![];
    for entry in load_saved_games().unwrap() {
        match entry {
            GameEntry::Local(game) => {
                if !game.get_grid().try_exists().unwrap() {
                    let handle = Handle::from_path(app_data_dir().join(format!("grids/{}", PLACEHOLDER_GRID)));
                    game_entries.push((GameEntry::Local(game), handle));
                } else {
                    let handle = Handle::from_path(game.get_grid());
                    game_entries.push((GameEntry::Local(game), handle));
                }
            },
            GameEntry::Steam(game) => {
                if !game.get_grid().try_exists().unwrap() {
                    let handle = Handle::from_path(app_data_dir().join(format!("grids/{}", PLACEHOLDER_GRID)));
                    game_entries.push((GameEntry::Steam(game), handle));
                } else {
                    let handle = Handle::from_path(game.get_grid());
                    game_entries.push((GameEntry::Steam(game), handle));
                }
            }
        }
    }
    let mut runners: Vec<Runner> = fetch_runners().unwrap_or_default().into_iter().map(|r| {
        Runner::Proton(ProtonVariant::Custom(r))
    }).collect();
    if cfg!(target_os = "linux") {
        runners.extend(vec![Runner::Native, Runner::Wine, Runner::Proton(ProtonVariant::GEProton), Runner::Proton(ProtonVariant::UMUProton)]);
    }
    
    State::new(game_entries, runners, config)
}
