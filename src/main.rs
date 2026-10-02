use std::collections::{BTreeMap};
use std::fs::{create_dir_all};
use std::path::{PathBuf};
use std::process::{Child, Command};

fn main() {
    let launch = GameCfg { id: 0, 
        binary_path: PathBuf::from("/usr/local/games/Konami/Silent Hill 2/sh2pc.exe"), 
        working_dir: PathBuf::from("/usr/local/games/Konami/Silent Hill 2"), 
        env_vars: BTreeMap::new(), 
        launch_args: Vec::new(), 
        runner: Runner::Wine };
    
    let entry = GameEntry {
        title: "sh2".to_string(),
        launch: Launch::Local(launch.clone()),
    };
    match process_run(&entry) {
        Ok(mut ch) => {println!("{}", ch.wait().unwrap().code().unwrap())},
        Err(e) => eprintln!("{}", e)
    }
    println!("{}", prefix_dir(&launch));
}

struct GameEntry {
    title: String,
    launch: Launch,
}

impl GameEntry {
    fn get_id(&self) -> u64 {
        match &self.launch {
            Launch::Steam { appid } => {*appid},
            Launch::Local(gc) => {gc.id}
        }
    }
    fn get_cfg(&self) -> Option<GameCfg> {
        match &self.launch {
            Launch::Local(cfg) => {Some(cfg.clone())}
            Launch::Steam { appid } => {None}
        }
    }
}

enum Launch {
    Steam {appid: u64},
    Local(GameCfg),
}

#[derive(Clone)]
struct GameCfg {
    id: u64,
    binary_path: PathBuf,
    working_dir: PathBuf,
    env_vars: BTreeMap<String, String>,
    launch_args: Vec<String>,
    runner: Runner
}

#[derive(Debug, Clone)]
enum Runner {
    Native,
    Wine,
    Proton(ProtonVariant, Store),
}

#[derive(Default, Clone, Debug)]
enum ProtonVariant {
    GEProton, //latest ge proton
    #[default]
    UMUProton, //umu-proton, latest stable valve proton + umu fixes
    Custom(PathBuf) //path to a local proton folder, must have proton file inside of it 
}

impl ProtonVariant {
    fn get_string(&self) -> String {
        match self {
            Self::GEProton => String::from("GE-Proton"),
            Self::UMUProton => String::from("UMU-Proton"),
            Self::Custom(p) => String::from(p.to_string_lossy())
        }
    }
}

#[derive(Clone, Debug)]
enum Store {
    Steam,
    EGS,
    None,
}

impl Store {
    fn get_string(&self) -> String {
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
                    let mut cmd = Command::new(&cfg.binary_path);
                    cmd.args(&cfg.launch_args).envs(&cfg.env_vars).current_dir(&cfg.working_dir);
                    cmd
                },
                Runner::Wine => {
                    let mut cmd = Command::new("wine");
                    cmd.current_dir(&cfg.working_dir).env("WINEPREFIX", prefix_dir(&cfg)).args(["start", "/unix"]).arg(&cfg.binary_path).envs(&cfg.env_vars).args(&cfg.launch_args);
                    cmd
                },
                Runner::Proton(variant, store) => {
                    let mut cmd = Command::new("umu-run");
                    cmd
                        .envs([("PROTONPATH", variant.get_string()), ("WINEPREFIX", prefix_dir(&cfg)), ("STORE", store.get_string())])
                        .arg(&cfg.binary_path)
                        .envs(&cfg.env_vars)
                        .args(&cfg.launch_args);
                    cmd
                }   
            }
        },
    }
}

fn process_run(entry: &GameEntry) -> Result<Child, std::io::Error> {
    if !matches!(entry.launch, Launch::Steam {appid}) {
        create_dir_all(prefix_dir(&entry.get_cfg().unwrap())).expect("Failed to create prefix folder");
    }
    command_build(entry).spawn()
}

fn prefix_dir(entry: &GameCfg) -> String {
    let data_dir = dirs::data_dir().expect("No data dir ?");
    data_dir.join(format!("maria/{}/prefix", entry.id)).to_string_lossy().into_owned()
}