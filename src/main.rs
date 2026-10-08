mod launch;
mod config;
mod library;
mod id;
mod gather_runners;
use std::{collections::BTreeMap};
use launch::{Runner};
use crate::{launch::LocalEntry, library::{fetch_local_games}};


fn main() {
    let x = fetch_local_games().unwrap();
    let id = x.iter().find(|x| x.get_title() == "sh2").unwrap().get_id();
    let z = LocalEntry::new("sh2r", *id, "/usr/local/games/Konami/Silent Hill 2/sh2pc.exe", "/usr/local/games/Konami/Silent Hill 2", BTreeMap::new(), &[], Runner::Wine);
    z.save().unwrap()
}