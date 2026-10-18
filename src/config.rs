#![allow(unused)]
use crate::{dirs::app_config_dir, error::Error};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, create_dir_all, read_to_string, write},
    io::Write,
};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppConfig {
    pub version: String,
    pub display: Display,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub enum Display {
    #[default]
    GridView,
    DetailsView,
}

pub fn read_config() -> Result<AppConfig, Error> {
    let c_dir = app_config_dir();
    if !c_dir.try_exists()? {
        create_dir_all(&c_dir)?;
    }
    let c_file = c_dir.join("config.toml");
    if !c_file.try_exists()? {
        let mut new_cfg = File::create_new(c_file)?;
        let content = toml::to_string_pretty(&AppConfig::default())?;
        new_cfg.write_all(&content.into_bytes());
        return Ok(AppConfig::default());
    }
    let contents = read_to_string(&c_file)?;
    let des = toml::from_str(&contents)?;
    Ok(des)
}

pub fn update_config(new_cfg: AppConfig) -> Result<(), Error> {
    let c_dir = app_config_dir();
    if !c_dir.try_exists()? {
        create_dir_all(&c_dir)?;
    }
    let c_file = c_dir.join("config.toml");
    if !c_file.try_exists()? {
        File::create_new(&c_file)?;
    }
    let ser_cfg = toml::to_string_pretty(&new_cfg)?;
    write(c_file, ser_cfg)?;
    Ok(())
}
