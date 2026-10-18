use crate::{
    data_model::{GameEntry, Runner},
    dirs::prefix_dir,
    error::Error,
};
use std::fs::create_dir_all;
use tokio::process::{Child, Command};

fn make_prefix(game: &GameEntry) {
    match &game {
        GameEntry::Local(game) => match game.get_runner() {
            Runner::Native => {
                return;
            }
            _ => {
                create_dir_all(prefix_dir(&game.get_id())).unwrap();
            }
        },
        GameEntry::Steam(_) => {
            return;
        }
    }
}

pub async fn run_game(game: &GameEntry) -> Result<Child, Error> {
    make_prefix(game);
    command_build(game)
        .spawn()
        .map_err(|e| Error::IOError(e.to_string()))
}

fn command_build(entry: &GameEntry) -> Command {
    match &entry {
        GameEntry::Steam(steam) => {
            let mut cmd = Command::new("steam");
            cmd.args(["-applaunch", &steam.get_appid().to_string()]);
            cmd
        }
        GameEntry::Local(game) => {
            let game_wrappers = game.get_wrappers();
            let mut cmd = match game.get_runner() {
                Runner::Native => {
                    if let Some(first) = game_wrappers.first() {
                        let mut cmd = Command::new(first);
                        cmd.args(&game_wrappers[1..]);

                        cmd.arg(game.get_binary_path());
                        cmd
                    } else {
                        Command::new(game.get_binary_path())
                    }
                }
                Runner::Wine => {
                    let mut initial = match game_wrappers.first() {
                        Some(first) => {
                            let mut cmd = Command::new(first);
                            cmd.args(&game_wrappers[1..]);
                            cmd.args(["wine", "start", "/unix"]);
                            cmd
                        }
                        None => {
                            let mut cmd = Command::new("wine");
                            cmd.args(["start", "/unix"]);
                            cmd
                        }
                    };
                    initial
                        .envs([("WINEPREFIX", prefix_dir(game.get_id()))])
                        .arg(game.get_binary_path());
                    initial
                }
                Runner::Proton(variant) => {
                    let mut initial = match game_wrappers.first() {
                        Some(first) => {
                            let mut cmd = Command::new(first);
                            cmd.args(&game_wrappers[1..]);
                            cmd.arg("umu-run");
                            cmd
                        }
                        None => Command::new("umu-run"),
                    };
                    initial
                        .envs([
                            ("PROTONPATH", variant.get_string()),
                            ("WINEPREFIX", prefix_dir(game.get_id())),
                        ])
                        .arg(game.get_binary_path());
                    initial
                }
            };
            cmd.args(game.get_launch_args())
                .envs(game.get_env_vars())
                .current_dir(game.get_working_dir());
            cmd
        }
    }
}
