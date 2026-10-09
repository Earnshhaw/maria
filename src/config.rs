#![allow(unused)]
use std::{error::Error, fs::{File, create_dir_all, read_to_string, write}, path::PathBuf};
use serde::{Deserialize, Serialize};
use crate::dirs::app_config_dir;


#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AppConfig {
    pub version: String,
    pub display: Display
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Display {
    pub mode: String
}

pub fn read_config() -> Result<AppConfig, Box<dyn Error>> {
    let c_dir = app_config_dir();
    if !c_dir.try_exists()? {
        create_dir_all(&c_dir)?;
    }
    let c_file = c_dir.join("config.toml");
    if !c_file.try_exists()? {
        File::create_new(c_file)?;
        return Ok(AppConfig::default());
    }
    let contents = read_to_string(&c_file)?;
    let des = toml::from_str(&contents)?;
    Ok(des)
}

pub fn update_config(new_cfg: AppConfig) -> Result<(), Box<dyn Error>> {
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
