use bevy::prelude::*;

use super::GameState;
use crate::resources::font_assets::FontAssets;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_menu)
            .add_systems(Update, menu_input.run_if(in_state(GameState::MainMenu)));
    }
}

fn spawn_menu(mut commands: Commands, fonts: Res<FontAssets>) {
    let font = fonts.pixel_font.clone();
    commands
        .spawn((
            DespawnOnExit(GameState::MainMenu),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Keyboard Player"),
                TextFont {
                    font: font.clone(),
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(0.88, 0.88, 0.88)),
            ));

            parent.spawn((
                Text::new("Press ENTER or SPACE to Start"),
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
            ));
        });
}

fn menu_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Selection);
    }
}
