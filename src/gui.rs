use std::{process::ExitStatus, rc::Rc, sync::Arc};

use iced::{Element, Length, Task, widget::{button, column, container, image::{self, Handle}, mouse_area, row, scrollable, text}};
use tokio::{process::Child, spawn, task::JoinSet};

use crate::{boot::PLACEHOLDER_GRID, dirs::app_data_dir, gather_games::{CError, fetch_steam_games, load_saved_games, save_to_disk}, launch::{GameEntry, LocalEntry, Runner, SteamEntry}, style::container_style};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub game_entries: Vec<(GameEntry, Handle)>,
    pub runners: Vec<Runner>,
    pub running_game: Option<GameEntry>
}

#[derive(Debug, Clone)]
pub enum Message {
    FetchSteamGames,
    SteamGamesFetched(Result<Vec<SteamEntry>, CError>),
    SyncGamesToDisk,
    Synced(Result<(), CError>),
    LaunchGame(String),
    GameLaunched((Result<Arc<Child>, CError>, String)),
    WatchProccess(Arc<Child>),
    ProcessResult(Result<ExitStatus, CError>)
}

fn panel_el<'a>(entry: &GameEntry, handle: &Handle) -> Element<'a, Message> {
    container(
        mouse_area(
    column![
        image::Image::new(handle).height(200),
    ]).on_double_click(Message::LaunchGame(entry.id_as_string()))).style(container_style)
    .into()
}

impl State {
    pub fn view(&self) -> Element<'_, Message> {
        let panels = scrollable(self.game_entries.iter().fold(column![], |acc, (entry, handle)| {
            let pa = acc.push(panel_el(entry, handle));
            pa
        })).height(Length::Fill);
        let fetch_steam = button("Steam").on_press(Message::FetchSteamGames);
        
        row![fetch_steam, panels].into()
    }      
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchSteamGames => fetch_steam_games_cmd(self),
            Message::SteamGamesFetched(result) => steam_games_fetched(self, result),
            Message::SyncGamesToDisk => sync_games(self),
            Message::Synced(status) => synced(self, status),
            Message::LaunchGame(strid) => launch_game(self, strid),
            Message::GameLaunched((res, id)) => launched(self, res, id),
            Message::WatchProccess(child) => watch_proccess(child),
            Message::ProcessResult(res) => child_terminated(self, res)
        }
    }
}

fn child_terminated(state: &mut State, result: Result<ExitStatus, CError>) -> Task<Message> {
    match result {
        Ok(s) => {
            println!("{}", s.code().unwrap());
        },
        Err(e) => {
           eprintln!("{e} Process failed ?");
        }
    }
    state.running_game.take();
    Task::none()
}
fn watch_proccess(child: Arc<Child>) -> Task<Message> {
    let mut inner = Arc::into_inner(child).unwrap();
    Task::perform(async move {
        inner.wait().await.map_err(|_| CError::IOError)
    }, Message::ProcessResult)
}
fn launched(state: &mut State, result: Result<Arc<Child>, CError>, id: String) -> Task<Message> {
    match result {
        Ok(ch) => {
            let (entry, _) = state.game_entries.iter().find(|(entry, _)| entry.id_as_string() == id).unwrap();
            state.running_game = Some(entry.to_owned());
            Task::done(Message::WatchProccess(ch))
        },
        Err(e) => {eprintln!("{}",e); Task::none()}
    }
}
fn launch_game(state: &mut State, strid: String) -> Task<Message> {
    let running_game = state.game_entries.iter().find(|stored| stored.0.id_as_string() == strid).cloned();
    if let Some(game) = running_game {
        Task::perform(async move {
            let (game, _) = game;
            (game.run_game().await, strid)
        },  Message::GameLaunched)
    } else {
        Task::none()
    }
    
}
fn synced(state: &mut State, status: Result<(), CError>) -> Task<Message> {
    match status {
        Ok(_) => {},
        Err(e) => eprintln!("{}", e)
    };
    let mut game_entries: Vec<(GameEntry, Handle)> = vec![];
    for entry in load_saved_games().unwrap() {
        match entry {
            GameEntry::Local(game) => {
                if !game.get_grid().try_exists().unwrap() {
                    let handle = Handle::from_path(app_data_dir().unwrap().join(format!("grids/{}", PLACEHOLDER_GRID)));
                    game_entries.push((GameEntry::Local(game), handle));
                } else {
                    let handle = Handle::from_path(game.get_grid());
                    game_entries.push((GameEntry::Local(game), handle));
                }
            },
            GameEntry::Steam(game) => {
                if !game.get_grid().try_exists().unwrap() {
                    let handle = Handle::from_path(app_data_dir().unwrap().join(format!("grids/{}", PLACEHOLDER_GRID)));
                    game_entries.push((GameEntry::Steam(game), handle));
                } else {
                    let handle = Handle::from_path(game.get_grid());
                    game_entries.push((GameEntry::Steam(game), handle));
                }
            }
        }
    }
    state.game_entries = game_entries;
    Task::none()
}
fn sync_games(state: &mut State) -> Task<Message> {
    let c = state.game_entries.iter().map(|(entry, _)| entry.clone()).collect();
    Task::perform(async move {
        save_to_disk(c).await
    }, Message::Synced)
}
fn fetch_steam_games_cmd(state: &mut State) -> Task<Message> {
    Task::perform(async move {
        fetch_steam_games().await
    }, Message::SteamGamesFetched)
}
fn steam_games_fetched(state: &mut State, result: Result<Vec<SteamEntry>, CError>) -> Task<Message> {
        match result {
            Ok(entries) => {
                let mut new_entries = Vec::new();
                for entry in entries {
                    if state.game_entries.iter().any(|(e, _)| {
                        match &e {
                            GameEntry::Local(_) => {false},
                            GameEntry::Steam(game) => {game.get_appid() == entry.get_appid()}
                        }
                    }) {
                        continue;
                    }
                    let handle = Handle::from_path(entry.get_grid());
                    new_entries.push((GameEntry::Steam(entry), handle));
                }
                state.game_entries.extend(new_entries);
                Task::done(Message::SyncGamesToDisk)
                
            },
            Err(e) => {
                eprintln!("{}", e);
                Task::none()
            },
        }
        //Task::done(Message::SyncGamesToDisk)
}