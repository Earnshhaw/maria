use std::collections::{BTreeMap};
use std::fs::{create_dir_all};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

pub struct GameEntry {
    pub title: String,
    pub launch: Launch,
}

impl GameEntry {
    pub fn get_id(&self) -> u64 {
        match &self.launch {
            Launch::Steam { appid } => {*appid},
            Launch::Local(gc) => {gc.id}
        }
    }
    pub fn get_cfg(&self) -> Option<GameCfg> {
        match &self.launch {
            Launch::Local(cfg) => {Some(cfg.clone())}
            Launch::Steam { appid } => {None}
        }
    }
}

pub enum Launch {
    Steam {appid: u64},
    Local(GameCfg),
}

#[derive(Clone)]
pub struct GameCfg {
    pub id: u64,
    binary_path: PathBuf,
    working_dir: PathBuf,
    env_vars: BTreeMap<String, String>,
    launch_args: Vec<String>,
    runner: Runner
}

impl GameCfg {
    pub fn new(id: u64, binary_path: impl Into<PathBuf>, working_dir: impl Into<PathBuf>, envs_vars: BTreeMap<String, String>, launch_args: &[&str], runner: Runner) -> GameCfg {
        GameCfg {
            id: id,
            binary_path: binary_path.into(),
            working_dir: working_dir.into(),
            env_vars: envs_vars,
            launch_args: launch_args.to_vec().into_iter().map(|e| e.to_string()).collect(), 
            runner: runner
        }
    }
    
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }
    pub fn env_vars(&self) -> &BTreeMap<String, String> {
        &self.env_vars
    }
    pub fn launch_args(&self) -> &Vec<String> {
        &self.launch_args
    }
    pub fn runner(&self) -> &Runner {
        &self.runner
    }
}

#[derive(Debug, Clone)]
pub enum Runner {
    Native,
    Wine,
    Proton(ProtonVariant, Store),
}

#[derive(Default, Clone, Debug)]
pub enum ProtonVariant {
    GEProton, //latest ge proton
    #[default]
    UMUProton, //umu-proton, latest stable valve proton + umu fixes
    Custom(PathBuf) //path to a local proton folder, must have proton file inside of it 
}

impl ProtonVariant {
    pub fn get_string(&self) -> String {
        match self {
            Self::GEProton => String::from("GE-Proton"),
            Self::UMUProton => String::from("UMU-Proton"),
            Self::Custom(p) => String::from(p.to_string_lossy())
        }
    }
}

#[derive(Clone, Debug)]
pub enum Store {
    Steam,
    EGS,
    None,
}

impl Store {
    pub fn get_string(&self) -> String {
        match self {
            Self::EGS => {
                String::from("egs")
            },
            Self::Steam => {
                String::from("steam")
            }
            Self::None => {
                String::from("none")
            }
        }
    }
}

fn command_build(entry: &GameEntry) -> Command {
    match &entry.launch {
        Launch::Steam { appid } => {
            let mut cmd = Command::new("steam");
            cmd.args(["-applaunch", &appid.to_string()]);
            cmd
        }
        Launch::Local(cfg) => {
            match &cfg.runner {
                Runner::Native => {
                    let mut cmd = Command::new(cfg.binary_path());
                    cmd
                        .args(cfg.launch_args())
                        .envs(cfg.env_vars())
                        .current_dir(cfg.working_dir());
                    cmd
                },
                Runner::Wine => {
                    let mut cmd = Command::new("wine");
                    cmd
                        .current_dir(cfg.working_dir())
                        .env("WINEPREFIX", prefix_dir(&cfg))
                        .args(["start", "/unix"])
                        .arg(cfg.binary_path())
                        .envs(cfg.env_vars())
                        .args(cfg.launch_args());
                    cmd
                },
                Runner::Proton(variant, store) => {
                    let mut cmd = Command::new("umu-run");
                    cmd
                        .current_dir(cfg.working_dir()) //questionable ?
                        .envs([("PROTONPATH", variant.get_string()), ("WINEPREFIX", prefix_dir(&cfg)), ("STORE", store.get_string())])
                        .arg(cfg.binary_path())
                        .envs(cfg.env_vars())
                        .args(cfg.launch_args());
                    cmd
                }   
            }
        },
    }
}

pub fn process_run(entry: &GameEntry) -> Result<Child, std::io::Error> {
    if let Some(cfg) = entry.get_cfg() {
        if !matches!(cfg.runner, Runner::Native) {
            create_dir_all(prefix_dir(&entry.get_cfg().unwrap())).expect("Failed to create prefix folder");
        }
    }
    command_build(entry).spawn()
}

pub fn prefix_dir(entry: &GameCfg) -> String {
    let data_dir = dirs::data_dir().expect("No data dir ?");
    data_dir.join(format!("maria/{}/prefix", entry.id)).to_string_lossy().into_owned()
}