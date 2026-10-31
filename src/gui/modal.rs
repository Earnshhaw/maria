use iced::{Length, alignment};

use crate::gui::gui_prelude::{
    Element, GameEdit, KeyValue, Message, SetMethod, State, button, container, opaque, pick_list,
    row, stack, text_input, image
};

pub fn modal<'a>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    stack!(base.into(), opaque(content.into())).into()
}

pub fn edit_game_modal(state: &State) -> Element<'_, Message> {
    let entry = state.game_form.as_ref().unwrap();
    let title = text_input("Title", &entry.title).on_input(|i| GameEdit::SetTitle(i).into());
    let poster = button(image(&entry.grid_path))
        .on_press(GameEdit::SetGridRfd.into());
    let binary_path = row![
        text_input("Path to executable", &entry.binary_path)
            .on_input(|i| GameEdit::SetBinary(SetMethod::TextInput(i)).into()),
        button("rfd").on_press(GameEdit::SetBinary(SetMethod::Rfd).into())
    ];
    let working_dir = row![
        text_input("Working directory", &entry.working_dir)
            .on_input(|i| GameEdit::SetWorkingDir(SetMethod::TextInput(i)).into()),
        button("rfd").on_press(GameEdit::SetWorkingDir(SetMethod::Rfd).into())
    ];
    let runner = pick_list(state.runners.as_ref(), entry.runner.as_ref(), |i| {
        GameEdit::SetRunner(i).into()
    });
    let launch_args = text_input("Launch arguments, for ex. -dx11", &entry.launch_args)
        .on_input(|i| GameEdit::SetLaunchArguments(i).into());
    let wrappers = text_input("Wrappers, for ex. mangohud", &entry.wrappers)
        .on_input(|i| GameEdit::SetWrapper(i).into());
    let env_fields =
        entry
            .env_vars
            .iter()
            .enumerate()
            .fold(iced::widget::column![], |acc, (i, (k, v))| {
                acc.push(row![
                    text_input("Key", k).on_input(move |k| GameEdit::SetEnvVars(
                        k,
                        KeyValue::Key(i)
                    )
                    .into()),
                    text_input("Value", v).on_input(move |v| GameEdit::SetEnvVars(
                        v,
                        KeyValue::Value(i),
                    )
                    .into()),
                    button("X").on_press(GameEdit::RemoveEnvField(i).into())
                ])
            });
    let env_fields = env_fields
        .push(button("Add an enviroment variable").on_press(GameEdit::AddEnvField.into()));

    let new_envs = container(env_fields.align_x(alignment::Alignment::Center));
    let finalization = row![
        button("Confirm").on_press(GameEdit::ConfirmEntry.into()),
        button("Cancel").on_press(GameEdit::CancelEntry.into())
    ];
    container(iced::widget::column![
        poster.padding(2).height(300).width(200),
        title.width(400),
        binary_path.width(400),
        working_dir.width(400),
        runner.width(400),
        launch_args.width(400),
        wrappers.width(400),
        new_envs.width(400),
        finalization.width(400)
    ].spacing(5).align_x(alignment::Alignment::Center)).center(Length::Fill)
    .into()
}
