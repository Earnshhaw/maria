mod launch;
mod gather_runners;
use std::{collections::BTreeMap};
use launch::{Launch, GameCfg, Runner, GameEntry};
use crate::gather_runners::init_runners;


fn main() {
    let launch = GameCfg::new(0, "", "", BTreeMap::new(), &[], Runner::Wine);
    
    let entry = GameEntry {
        title: "".to_string(),
        launch: Launch::Local(launch),
    };

    println!("{:#?}", init_runners())
}