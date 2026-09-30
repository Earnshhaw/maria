use std::{collections::BTreeMap, path::PathBuf};

fn main() {
    let game = GameCfg { title: String::from("sh2"), 
        id: 0,
        binary_path: "/usr/local/games/Konami/Silent Hill 2/sh2pc.exe".to_string(), 
        working_dir:"/usr/local/games/Konami/Silent Hill 2".to_string(), 
        runner: Runner::Wine,
        store: Store::None, 
        launch_args: Vec::new(),
        env_vars: BTreeMap::new()
    };
    run_pipeline(&game);
}

struct GameCfg {
    title: String,
    id: u64,
    binary_path: String,
    working_dir: String,
    runner: Runner,
    store: Store,
    launch_args: Vec<String>,
    env_vars: BTreeMap<String, String>
}

enum Runner {
    ByPath(PathBuf),
    Fixed(FixedRunner),
    Wine,
    Native
}

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

#[derive(Default)]
enum FixedRunner {
    ProtonGE, //latest ge proton
    #[default]
    UMUProton, //umu-proton, latest stable valve proton + umu fixes
}

impl FixedRunner {
    fn get_string(&self) -> String {
        match self {
            Self::ProtonGE => String::from("GE-Proton"),
            Self::UMUProton => String::from("UMU-Proton")
        }
    }
}

fn run_pipeline(entry: &GameCfg) {
    let data_dir = dirs::data_dir().unwrap();
    let prefix_dir = data_dir.join(format!("maria/{}/prefix/", entry.title));
    
    let protonpath = match &entry.runner {
        Runner::ByPath(path) => {
            path.to_string_lossy().into_owned()
        },
        Runner::Fixed(fixed) => {
            fixed.get_string()
        }
        Runner::Wine => {
            wine_run(entry, prefix_dir); 
            return;
        },
        Runner::Native => {
            native_run(entry);
            return;
        }
    };

    let mut cmd = std::process::Command::new("umu-run");
    
    cmd
    .envs([("PROTONPATH", protonpath), ("WINEPREFIX", prefix_dir.to_string_lossy().into_owned()), ("STORE", entry.store.get_string())])
    .arg(&entry.binary_path)
    .current_dir(&entry.working_dir)
    .args(&entry.launch_args)
    .spawn().unwrap();
}

fn native_run(entry: &GameCfg) {
    std::process::Command::new(format!("./{}", entry.binary_path)).envs(&entry.env_vars);
}

fn wine_run(entry: &GameCfg, wine_prefix: PathBuf) {
    let mut cmd = std::process::Command::new("wine");
    cmd.env("WINEPREFIX", wine_prefix).args(["start", "/unix"]).arg(&entry.binary_path).envs(&entry.env_vars).args(&entry.launch_args).current_dir(&entry.working_dir).spawn().unwrap();
}