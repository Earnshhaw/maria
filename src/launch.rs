#![allow(unused)]
use std::collections::{BTreeMap};
use std::fs::{create_dir_all};
use std::path::{Path, PathBuf};
use iced::widget::grid;
use serde::{Deserialize, Serialize};
use tokio::io;
use tokio::process::{Child, Command};
use uuid::Uuid;
use crate::{gather_runners::RunnerEntry, dirs::prefix_dir};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GameEntry {
    Steam(SteamEntry),
    Local(LocalEntry)
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
        env_vars: BTreeMap<String, String>, 
        launch_args: &[&str], 
        runner: Runner) -> LocalEntry 
    {
        LocalEntry { 
            title: title.into(), 
            grid_path: grid_path.into(),
            id: id.into(), 
            binary_path: binary_path.into(), 
            working_dir: working_dir.into(), 
            env_vars: env_vars, 
            launch_args: launch_args.into_iter().map(|e| e.to_string()).collect(), 
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

impl GameEntry {
    fn make_prefix(&self) {
        match &self {
            GameEntry::Local(game) => {
                match game.runner {
                    Runner::Native => {return},
                    _ => {
                        create_dir_all(prefix_dir(&game.id)).unwrap();
                    }
                }
            },
            GameEntry::Steam(..) => {return}
        }
    }
    
    pub async fn run_game(&self) -> io::Result<Child> {
        self.make_prefix();
        command_build(self).spawn()
    }

}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub enum Runner {
    Native,
    #[default]
    Wine,
    Proton(ProtonVariant),
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub enum ProtonVariant {
    GEProton, //latest ge proton
    #[default]
    UMUProton, //umu-proton, latest stable valve proton + umu fixes
    Custom(RunnerEntry) //path to a local proton folder, must have proton file inside of it 
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

fn command_build(entry: &GameEntry) -> Command {
    match &entry {
        GameEntry::Steam(steam) => {
            let mut cmd = Command::new("steam");
            cmd.args(["-appEntryType", &steam.appid.to_string()]);
            cmd
        }
        GameEntry::Local(game) => {
            match game.get_runner() {
                Runner::Native => {
                    let mut cmd = Command::new(game.get_binary_path());
                    cmd
                        .args(game.get_launch_args())
                        .envs(game.get_env_vars())
                        .current_dir(game.get_working_dir());
                    cmd
                },
                Runner::Wine => {
                    let mut cmd = Command::new("wine");
                    cmd
                        .current_dir(game.get_working_dir())
                        .env("WINEPREFIX", prefix_dir(game.get_id()))
                        .args(["start", "/unix"])
                        .arg(game.get_binary_path())
                        .envs(game.get_env_vars())
                        .args(game.get_launch_args());
                    cmd
                },
                Runner::Proton(variant) => {
                    let mut cmd = Command::new("umu-run");
                    cmd
                        .current_dir(game.get_working_dir()) //questionable ?
                        .envs([("PROTONPATH", variant.get_string()), ("WINEPREFIX", prefix_dir(game.get_id()))])
                        .arg(game.get_binary_path())
                        .envs(game.get_env_vars())
                        .args(game.get_launch_args());
                    cmd
                }   
            }
        },
    }
}

