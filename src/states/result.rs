use bevy::prelude::*;

use super::playing::GameResult;
use super::GameState;
use crate::resources::font_assets::FontAssets;

const TEXT_PRIMARY: Color = Color::srgb(0.88, 0.88, 0.88);
const TEXT_SECONDARY: Color = Color::srgb(0.6, 0.6, 0.7);
const TEXT_HIGHLIGHT: Color = Color::srgb(0.29, 0.85, 0.50);

pub struct ResultPlugin;

impl Plugin for ResultPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Result), spawn_result)
            .add_systems(Update, result_input.run_if(in_state(GameState::Result)));
    }
}

fn spawn_result(mut commands: Commands, result: Option<Res<GameResult>>, fonts: Res<FontAssets>) {
    let font = fonts.pixel_font.clone();
    let (title, elapsed, kpm, accuracy, total_chars, is_new_record) = match result {
        Some(r) => (
            r.title.clone(),
            r.elapsed_secs,
            r.kpm,
            r.accuracy,
            r.total_chars,
            r.is_new_record,
        ),
        None => ("--".into(), 0.0, 0.0, 0.0, 0, false),
    };

    commands
        .spawn((
            DespawnOnExit(GameState::Result),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
        ))
        .with_children(|root| {
            if is_new_record {
                root.spawn((
                    Text::new("★ NEW RECORD! ★"),
                    TextFont {
                        font: font.clone(),
                        font_size: 36.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.84, 0.0)),
                ));
            }

            root.spawn((
                Text::new("练习完成！"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(TEXT_HIGHLIGHT),
            ));

            root.spawn((
                Text::new(format!("\"{}\"", title)),
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));

            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                row_gap: Val::Px(12.0),
                margin: UiRect::vertical(Val::Px(20.0)),
                ..default()
            })
            .with_children(|stats| {
                spawn_stat_row(stats, "用时", &format!("{:.1} 秒", elapsed), &font);
                spawn_stat_row(stats, "KPM", &format!("{:.0}", kpm), &font);
                spawn_stat_row(stats, "正确率", &format!("{:.1}%", accuracy), &font);
                spawn_stat_row(stats, "总字符", &format!("{}", total_chars), &font);
            });

            root.spawn((
                Text::new("Press SPACE to return to menu"),
                TextFont {
                    font: font.clone(),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
            ));
        });
}

fn spawn_stat_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    font: &Handle<Font>,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{}:", label)),
                TextFont {
                    font: font.clone(),
                    font_size: 26.0,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
            ));
            row.spawn((
                Text::new(value),
                TextFont {
                    font: font.clone(),
                    font_size: 26.0,
                    ..default()
                },
                TextColor(TEXT_HIGHLIGHT),
            ));
        });
}

fn result_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::MainMenu);
    }
}
