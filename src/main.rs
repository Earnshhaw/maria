mod launch;
mod config;
mod gather_runners;
mod steam_games;
use std::{collections::BTreeMap};
use launch::{Launch, GameCfg, Runner, GameEntry};
use crate::{config::{AppConfig, Display}};


fn main() {
    let launch = GameCfg::new(0, "", "", BTreeMap::new(), &[], Runner::Wine);
    
    let entry = GameEntry {
        title: "".to_string(),
        launch: Launch::Local(launch),
    };

    let new_cfg = {
        AppConfig { version: "0.1.0".to_string(), display: Display { mode: "sidebar".to_string() } }
    };
    
    if let Ok(s) = config::update_config(new_cfg) {
        println!("{:#?}", s);
    }

    

    //println!("{:#?}", init_runners())
}