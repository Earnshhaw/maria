mod launch;
mod config;
mod library;
mod id;
mod gather_runners;
mod steam_games;
use std::{collections::BTreeMap};
use launch::{Launch, GameCfg, Runner, GameEntry};
use crate::{config::{AppConfig, Display}, steam_games::fetch_steam_games};


fn main() {
    let launch = GameCfg::new("", "", BTreeMap::new(), &[], Runner::Wine);
    
    let entry = GameEntry {
        title: "".to_string(),
        launch: Launch::Local(launch.clone()),
    };

    let new_cfg = {
        AppConfig { version: "0.1.0".to_string(), display: Display { mode: "sidebar".to_string() } }
    };
    
    let x = fetch_steam_games().unwrap();
    println!("{:#?}", x)
}