#![allow(unused)]
use crate::gather_runners::RunnerEntry;
use iced::widget::image::Handle;
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use std::{
    collections::BTreeMap,
    ops::Deref,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialOrd, PartialEq, Eq, Hash)]
pub enum GameEntry {
    Steam(SteamEntry),
    Local(LocalEntry),
}

#[derive(Debug, Clone)]
pub struct Game {
    entry: GameEntry,
    handle: Handle,
}

impl Game {
    pub fn new(entry: GameEntry, handle: Handle) -> Game {
        Game { entry, handle }
    }
    pub fn get_id(&self) -> GameId {
        match &self.entry {
            GameEntry::Local(g) => GameId(g.get_id().to_string()),
            GameEntry::Steam(g) => GameId(g.get_appid().to_string()),
        }
    }
    pub fn get_grid(&self) -> PathBuf {
        match &self.entry {
            GameEntry::Local(g) => g.get_grid().to_owned(),
            GameEntry::Steam(g) => g.get_grid().to_owned(),
        }
    }
    pub fn get_entry(&self) -> GameEntry {
        match &self.entry {
            GameEntry::Local(g) => GameEntry::Local(g.to_owned()),
            GameEntry::Steam(g) => GameEntry::Steam(g.to_owned()),
        }
    }
    pub fn get_handle(&self) -> &Handle {
        &self.handle
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct GameId(String);

impl std::fmt::Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<Uuid> for GameId {
    fn from(value: Uuid) -> Self {
        GameId(value.to_string())
    }
}

impl From<u64> for GameId {
    fn from(value: u64) -> Self {
        GameId(value.to_string())
    }
}

impl GameEntry {
    pub fn get_title(&self) -> String {
        match &self {
            GameEntry::Local(f) => f.get_title().to_owned(),
            GameEntry::Steam(f) => f.get_title().to_owned(),
        }
    }
    pub fn get_id(&self) -> GameId {
        match &self {
            GameEntry::Local(s) => s.get_id().to_owned().into(),
            GameEntry::Steam(s) => s.get_appid().into(),
        }
    }
    pub fn get_grid(&self) -> PathBuf {
        match self {
            GameEntry::Local(g) => g.get_grid().to_owned(),
            GameEntry::Steam(g) => g.get_grid().to_owned(),
        }
    }
}

impl Default for GameEntry {
    fn default() -> Self {
        GameEntry::Local(LocalEntry::default())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct SteamEntry {
    appid: u64,
    title: String,
    grid_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct LocalEntry {
    title: String,
    id: Uuid,
    grid_path: PathBuf,
    binary_path: PathBuf,
    working_dir: PathBuf,
    wrappers: Vec<String>,
    env_vars: BTreeMap<String, String>,
    launch_args: Vec<String>,
    runner: Runner,
}

impl LocalEntry {
    pub fn new(
        title: impl Into<String>,
        id: impl Into<Uuid>,
        grid_path: impl Into<PathBuf>,
        binary_path: impl Into<PathBuf>,
        working_dir: impl Into<PathBuf>,
        wrappers: Vec<String>,
        env_vars: BTreeMap<String, String>,
        launch_args: Vec<String>,
        runner: Runner,
    ) -> LocalEntry {
        LocalEntry {
            title: title.into(),
            grid_path: grid_path.into(),
            id: id.into(),
            binary_path: binary_path.into(),
            working_dir: working_dir.into(),
            wrappers: wrappers,
            env_vars: env_vars,
            launch_args: launch_args,
            runner: runner,
        }
    }
    pub fn get_title(&self) -> &str {
        &self.title
    }
    pub fn get_grid(&self) -> &Path {
        &self.grid_path
    }
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
    pub fn get_binary_path(&self) -> &Path {
        &self.binary_path
    }
    pub fn get_working_dir(&self) -> &Path {
        &self.working_dir
    }
    pub fn get_env_vars(&self) -> &BTreeMap<String, String> {
        &self.env_vars
    }
    pub fn get_launch_args(&self) -> &Vec<String> {
        &self.launch_args
    }
    pub fn get_runner(&self) -> &Runner {
        &self.runner
    }
    pub fn get_wrappers(&self) -> &Vec<String> {
        &self.wrappers
    }
}

impl SteamEntry {
    pub fn new(
        appid: impl Into<u64>,
        title: impl Into<String>,
        grid_path: impl Into<PathBuf>,
    ) -> SteamEntry {
        SteamEntry {
            appid: appid.into(),
            title: title.into(),
            grid_path: grid_path.into(),
        }
    }

    pub fn get_appid(&self) -> u64 {
        self.appid
    }
    pub fn get_title(&self) -> &str {
        &self.title
    }
    pub fn get_grid(&self) -> &Path {
        &self.grid_path
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Runner {
    #[default]
    Native,
    Wine,
    Proton(ProtonVariant),
}

impl std::fmt::Display for Runner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Runner::Native => {f.write_str("Native")},
            Runner::Proton(variant) => {f.write_str(&variant.to_string())},
            Runner::Wine => {f.write_str("Wine")}
        }
    }

}

#[derive(Default, Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ProtonVariant {
    GEProton, //latest ge proton
    #[default]
    UMUProton, //umu-proton, latest stable valve proton + umu fixes
    Custom(RunnerEntry), //path to a local proton folder, must have proton file inside of it
}

impl std::fmt::Display for ProtonVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GEProton => {f.write_str("GEProton")},
            Self::UMUProton => {f.write_str("UMUProton")},
            Self::Custom(r_e) => {f.write_str(r_e.name())},
        }
    }
}

impl ProtonVariant {
    pub fn as_string(&self) -> String {
        match self {
            Self::GEProton => String::from("GE-Proton"),
            Self::UMUProton => String::from("UMU-Proton"),
            Self::Custom(p) => String::from(p.path().to_string_lossy()),
        }
    }
}
