use iced::{Element, Task, widget::{image::Handle, row}};

use crate::launch::{GameEntry, LocalEntry, Runner};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub game_entries: Vec<(GameEntry, Handle)>,
    pub runners: Vec<Runner>,
}

#[derive(Debug, Clone)]
pub enum Message {
    
}

impl State {
    pub fn view(&self) -> Element<'_, Message> {
        row![].into()
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
        }
    }
}