use crate::gui::gui_prelude::{
    ProtonVariant, Runner, State, fetch_runners, persistence_dirs_valid, process_loaded_games,
    read_config,
};

pub fn boot() -> State {
    if !persistence_dirs_valid() {
        panic!("Persistence dirs could not be initialized. Aborting.");
    }
    let config = read_config().unwrap_or_default();

    let game_entries = process_loaded_games();
    let mut runners: Vec<Runner> = fetch_runners()
        .unwrap_or_default()
        .into_iter()
        .map(|r| Runner::Proton(ProtonVariant::Custom(r)))
        .collect();
    if cfg!(target_os = "linux") {
        runners.extend(vec![
            Runner::Native,
            Runner::Wine,
            Runner::Proton(ProtonVariant::GEProton),
            Runner::Proton(ProtonVariant::UMUProton),
        ]);
    } else {
        runners = vec![Runner::Native]
    }

    State::new(game_entries, runners, config)
}
