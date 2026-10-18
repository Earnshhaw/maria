use crate::{
    config::read_config,
    data_model::{ProtonVariant, Runner},
    dirs::persistence_dirs_valid,
    gather_runners::fetch_runners,
    gui::{State, process_loaded_games},
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
