use crate::gui::gui_prelude::{
    ContextMenu, Element, GameEdit, GameEntry, Item, Length, Menu, Message, State, button,
    container, grid, scrollable, text,
};

const PANESPLIT: f32 = 2 as f32 / 3 as f32;
pub fn game_grid_view(state: &State) -> Element<'_, Message> {
    let grid = scrollable(
        grid(state.game_entries().iter().map(|(entry, handle)| {
            ContextMenu::new(
                {
                    button(
                        iced::widget::image(handle) //UNDERLAY
                            .border_radius(10)
                            .content_fit(iced::ContentFit::Contain)
                            .height(Length::Fill),
                    )
                    .on_press(Message::LaunchGame(entry.get_id())) //UNDERLAY
                },
                || {
                    //OVERLAY
                    let button_m = match entry {
                        GameEntry::Local(_) => {
                            iced::widget::column![button("Edit").on_press(
                                Message::GameEditOverlay(GameEdit::EditLocalGame(entry.clone()))
                            )]
                        }
                        GameEntry::Steam(_) => {
                            iced::widget::column![]
                        }
                    };
                    let cx = button_m
                        .push(button("Delete").on_press(Message::DeleteEntry(entry.get_id())));
                    cx.into()
                }, //OVERLAY
            )
            .into()
        }))
        .height(grid::Sizing::AspectRatio(PANESPLIT))
        .fluid(200)
        .spacing(5),
    );

    container(grid).width(Length::Fill).padding(10).into()
}
#[allow(unused)]
pub fn sidebar_view(state: &State) -> Element<'_, Message> {
    let els = text!("SIDEBAR");
    container(els).into()
}

#[allow(unused)]
pub fn topbar_view(state: &State) -> Element<'_, Message> {
    let options = vec![
        Item::new(
            button("Add a local game").on_press(Message::GameEditOverlay(GameEdit::AddLocalGame)),
        )
        .close_on_click(true),
        Item::new(button("Fetch steam library").on_press(Message::FetchSteamGames))
            .close_on_click(true),
    ];
    let cx = iced_aw::menu_bar!((
        text("Add a game"),
        Menu::new(options).width(Length::Fixed(125.0))
    ));
    container(cx).width(Length::Fill).into()
}
