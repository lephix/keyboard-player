pub mod menu;
pub mod playing;
pub mod result;
pub mod selection;

use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum GameState {
    #[default]
    MainMenu,
    Selection,
    Playing,
    Result,
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_plugins((
            menu::MenuPlugin,
            selection::SelectionPlugin,
            playing::PlayingPlugin,
            result::ResultPlugin,
        ));
    }
}
