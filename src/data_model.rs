use crate::{gather_runners::{RunnerEntry}};
use std::{collections::BTreeMap, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GameEntry {
    Steam(SteamEntry),
    Local(LocalEntry)
}

impl GameEntry {
    pub fn _get_title(&self) -> String {
            match &self {
                GameEntry::Local(f) => f.get_title().to_owned(),
                GameEntry::Steam(f) => f.get_title().to_owned()
            }
        }
        pub fn get_id(&self) -> String {
            match &self {
                GameEntry::Local(s) => {s.get_id().to_string()},
                GameEntry::Steam(s) => {s.get_appid().to_string()}
            }
    }
}

impl Default for GameEntry {
    fn default() -> Self {
        GameEntry::Local(LocalEntry::default())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SteamEntry {
    appid: u64,
    title: String,
    grid_path: PathBuf
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LocalEntry {
    title: String, 
    id: Uuid, 
    grid_path: PathBuf,
    binary_path: PathBuf, 
    working_dir: PathBuf,
    wrappers: Vec<String>,
    env_vars: BTreeMap<String, String>, 
    launch_args: Vec<String>, 
    runner: Runner
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
        runner: Runner) -> LocalEntry 
    {
        LocalEntry { 
            title: title.into(), 
            grid_path: grid_path.into(),
            id: id.into(), 
            binary_path: binary_path.into(), 
            working_dir: working_dir.into(), 
            wrappers: wrappers,
            env_vars: env_vars, 
            launch_args: launch_args, 
            runner: runner }
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
    pub fn new(appid: impl Into<u64>, title: impl Into<String>, grid_path: impl Into<PathBuf>) -> SteamEntry {
        SteamEntry { appid: appid.into(), title: title.into(), grid_path: grid_path.into() }
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

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, PartialOrd, )]
pub enum Runner {
    #[default]
    Native,
    Wine,
    Proton(ProtonVariant),
}

impl ToString for Runner {
    fn to_string(&self) -> String {
        match self {
            Self::Native => String::from("Native"),
            Self::Wine => String::from("Wine"),
            Self::Proton(pr_vr) => pr_vr.to_string()
        }
    }
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum ProtonVariant {
    GEProton, //latest ge proton
    #[default]
    UMUProton, //umu-proton, latest stable valve proton + umu fixes
    Custom(RunnerEntry) //path to a local proton folder, must have proton file inside of it 
}

impl ToString for ProtonVariant {
    fn to_string(&self) -> String {
        match self {
            Self::GEProton => String::from("GEProton"),
            Self::UMUProton => String::from("UMUProton"),
            Self::Custom(r_e) => String::from(r_e.name())
        }
    }
}

impl ProtonVariant {
    pub fn get_string(&self) -> String {
        match self {
            Self::GEProton => String::from("GE-Proton"),
            Self::UMUProton => String::from("UMU-Proton"),
            Self::Custom(p) => String::from(p.path().to_string_lossy())
        }
    }
}
