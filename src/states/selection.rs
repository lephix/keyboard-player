use bevy::prelude::*;
use rand::seq::SliceRandom;

use super::GameState;
use crate::data::text_model::{Difficulty, Grade, Language};
use crate::resources::font_assets::FontAssets;
use crate::resources::game_config::GameConfig;
use crate::resources::game_data::{CurrentPassage, TextLibrary};

const BTN_NORMAL: Color = Color::srgb(0.15, 0.15, 0.25);
const BTN_HOVERED: Color = Color::srgb(0.25, 0.25, 0.40);
const BTN_PRESSED: Color = Color::srgb(0.29, 0.69, 0.50);
const BTN_FOCUSED: Color = Color::srgb(0.29, 0.85, 0.50);
const TEXT_PRIMARY: Color = Color::srgb(0.88, 0.88, 0.88);
const TEXT_SECONDARY: Color = Color::srgb(0.6, 0.6, 0.7);

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum SelectionStep {
    #[default]
    Language,
    Grade,
    Difficulty,
}

#[derive(Resource, Default)]
struct SelectionState {
    step: SelectionStep,
    font: Option<Handle<Font>>,
    focused_index: usize,
}

#[derive(Component)]
struct SelectionUiRoot;

#[derive(Component)]
struct ButtonIndex(usize);

#[derive(Component)]
struct LanguageButton(Language);

#[derive(Component)]
struct GradeButton(Grade);

#[derive(Component)]
struct DifficultyButton(Difficulty);

#[derive(Component)]
struct BackButton;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
            .add_systems(OnEnter(GameState::Selection), on_enter_selection)
            .add_systems(
                OnExit(GameState::Selection),
                |mut state: ResMut<SelectionState>| {
                    state.step = SelectionStep::Language;
                    state.focused_index = 0;
                },
            )
            .add_systems(
                Update,
                (
                    handle_language_buttons,
                    handle_grade_buttons,
                    handle_difficulty_buttons,
                    handle_back_button,
                    handle_keyboard_selection,
                    handle_button_visuals,
                    handle_escape_key,
                )
                    .run_if(in_state(GameState::Selection)),
            );
    }
}

fn on_enter_selection(
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
    mut sel_state: ResMut<SelectionState>,
    fonts: Res<FontAssets>,
) {
    config.reset();
    sel_state.step = SelectionStep::Language;
    sel_state.focused_index = 0;
    sel_state.font = Some(fonts.pixel_font.clone());
    let font = fonts.pixel_font.clone();
    spawn_language_ui(&mut commands, &font);
}

fn spawn_language_ui(commands: &mut Commands, font: &Handle<Font>) {
    commands
        .spawn((
            DespawnOnExit(GameState::Selection),
            SelectionUiRoot,
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
        .with_children(|root| {
            root.spawn((
                Text::new("选择语言 / Select Language"),
                TextFont {
                    font: font.clone(),
                    font_size: 42.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));

            spawn_hint(root, font);

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                ..default()
            })
            .with_children(|row| {
                spawn_btn(row, "English", LanguageButton(Language::En), font, 0);
                spawn_btn(row, "中文", LanguageButton(Language::Zh), font, 1);
            });
        });
}

fn spawn_grade_ui(commands: &mut Commands, font: &Handle<Font>) {
    commands
        .spawn((
            DespawnOnExit(GameState::Selection),
            SelectionUiRoot,
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
        .with_children(|root| {
            root.spawn((
                Text::new("选择年级 / Select Grade"),
                TextFont {
                    font: font.clone(),
                    font_size: 42.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));

            spawn_hint(root, font);

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                ..default()
            })
            .with_children(|row| {
                spawn_btn(row, "小学", GradeButton(Grade::Elementary), font, 0);
                spawn_btn(row, "初中", GradeButton(Grade::Middle), font, 1);
                spawn_btn(row, "高中", GradeButton(Grade::High), font, 2);
            });

            spawn_back(root, font);
        });
}

fn spawn_difficulty_ui(commands: &mut Commands, font: &Handle<Font>) {
    commands
        .spawn((
            DespawnOnExit(GameState::Selection),
            SelectionUiRoot,
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
        .with_children(|root| {
            root.spawn((
                Text::new("选择难度 / Select Difficulty"),
                TextFont {
                    font: font.clone(),
                    font_size: 42.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));

            spawn_hint(root, font);

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                ..default()
            })
            .with_children(|row| {
                spawn_btn(row, "简单", DifficultyButton(Difficulty::Easy), font, 0);
                spawn_btn(row, "困难", DifficultyButton(Difficulty::Hard), font, 1);
            });

            spawn_back(root, font);
        });
}

fn spawn_hint(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent.spawn((
        Text::new("← → Select   Enter Confirm   Esc Back"),
        TextFont {
            font: font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.4, 0.4, 0.5)),
    ));
}

fn spawn_btn<M: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: M,
    font: &Handle<Font>,
    index: usize,
) {
    parent
        .spawn((
            Button,
            marker,
            ButtonIndex(index),
            Node {
                width: Val::Px(200.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            BorderColor::all(TEXT_SECONDARY),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 26.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));
        });
}

fn spawn_back(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            margin: UiRect::top(Val::Px(10.0)),
            ..default()
        })
        .with_children(|wrapper| {
            wrapper
                .spawn((
                    Button,
                    BackButton,
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.18)),
                    BorderColor::all(TEXT_SECONDARY),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("< Back"),
                        TextFont {
                            font: font.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));
                });
        });
}

fn despawn_ui(commands: &mut Commands, ui_roots: &Query<Entity, With<SelectionUiRoot>>) {
    for entity in ui_roots.iter() {
        commands.entity(entity).despawn();
    }
}

fn option_count(step: SelectionStep) -> usize {
    match step {
        SelectionStep::Language => 2,
        SelectionStep::Grade => 3,
        SelectionStep::Difficulty => 2,
    }
}

fn start_game(
    config: &GameConfig,
    commands: &mut Commands,
    library: &TextLibrary,
    next_state: &mut NextState<GameState>,
) {
    let lang = config.language.expect("language must be set");
    let grade = config.grade.expect("grade must be set");
    if let Some(passages) = library.passages.get(&(lang, grade)) {
        if let Some(passage) = passages.choose(&mut rand::thread_rng()) {
            commands.insert_resource(CurrentPassage {
                passage: passage.clone(),
            });
            next_state.set(GameState::Playing);
            return;
        }
    }
    eprintln!("No passages found for {:?} {:?}", lang, grade);
    next_state.set(GameState::MainMenu);
}

fn handle_keyboard_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sel_state: ResMut<SelectionState>,
    mut config: ResMut<GameConfig>,
    mut commands: Commands,
    ui_roots: Query<Entity, With<SelectionUiRoot>>,
    library: Res<TextLibrary>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let count = option_count(sel_state.step);

    if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::ArrowDown) {
        sel_state.focused_index = (sel_state.focused_index + 1) % count;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::ArrowUp) {
        sel_state.focused_index = if sel_state.focused_index == 0 {
            count - 1
        } else {
            sel_state.focused_index - 1
        };
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        let font = sel_state.font.clone().unwrap_or_default();
        match sel_state.step {
            SelectionStep::Language => {
                config.language = Some(if sel_state.focused_index == 0 {
                    Language::En
                } else {
                    Language::Zh
                });
                sel_state.step = SelectionStep::Grade;
                sel_state.focused_index = 0;
                despawn_ui(&mut commands, &ui_roots);
                spawn_grade_ui(&mut commands, &font);
            }
            SelectionStep::Grade => {
                config.grade = Some(match sel_state.focused_index {
                    0 => Grade::Elementary,
                    1 => Grade::Middle,
                    _ => Grade::High,
                });
                sel_state.step = SelectionStep::Difficulty;
                sel_state.focused_index = 0;
                despawn_ui(&mut commands, &ui_roots);
                spawn_difficulty_ui(&mut commands, &font);
            }
            SelectionStep::Difficulty => {
                config.difficulty = Some(if sel_state.focused_index == 0 {
                    Difficulty::Easy
                } else {
                    Difficulty::Hard
                });
                start_game(&config, &mut commands, &library, &mut next_state);
            }
        }
    }
}

fn handle_language_buttons(
    mut commands: Commands,
    interactions: Query<(&Interaction, &LanguageButton), (Changed<Interaction>, With<Button>)>,
    ui_roots: Query<Entity, With<SelectionUiRoot>>,
    mut config: ResMut<GameConfig>,
    mut sel_state: ResMut<SelectionState>,
) {
    for (interaction, lang_btn) in &interactions {
        if *interaction == Interaction::Pressed {
            config.language = Some(lang_btn.0);
            sel_state.step = SelectionStep::Grade;
            sel_state.focused_index = 0;
            despawn_ui(&mut commands, &ui_roots);
            let font = sel_state.font.clone().unwrap_or_default();
            spawn_grade_ui(&mut commands, &font);
        }
    }
}

fn handle_grade_buttons(
    mut commands: Commands,
    interactions: Query<(&Interaction, &GradeButton), (Changed<Interaction>, With<Button>)>,
    ui_roots: Query<Entity, With<SelectionUiRoot>>,
    mut config: ResMut<GameConfig>,
    mut sel_state: ResMut<SelectionState>,
) {
    for (interaction, grade_btn) in &interactions {
        if *interaction == Interaction::Pressed {
            config.grade = Some(grade_btn.0);
            sel_state.step = SelectionStep::Difficulty;
            sel_state.focused_index = 0;
            despawn_ui(&mut commands, &ui_roots);
            let font = sel_state.font.clone().unwrap_or_default();
            spawn_difficulty_ui(&mut commands, &font);
        }
    }
}

fn handle_difficulty_buttons(
    interactions: Query<(&Interaction, &DifficultyButton), (Changed<Interaction>, With<Button>)>,
    mut config: ResMut<GameConfig>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    library: Res<TextLibrary>,
) {
    for (interaction, diff_btn) in &interactions {
        if *interaction == Interaction::Pressed {
            config.difficulty = Some(diff_btn.0);
            start_game(&config, &mut commands, &library, &mut next_state);
        }
    }
}

fn handle_back_button(
    mut commands: Commands,
    interactions: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    ui_roots: Query<Entity, With<SelectionUiRoot>>,
    mut sel_state: ResMut<SelectionState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            match sel_state.step {
                SelectionStep::Grade => {
                    sel_state.step = SelectionStep::Language;
                    sel_state.focused_index = 0;
                    let font = sel_state.font.clone().unwrap_or_default();
                    despawn_ui(&mut commands, &ui_roots);
                    spawn_language_ui(&mut commands, &font);
                }
                SelectionStep::Difficulty => {
                    sel_state.step = SelectionStep::Grade;
                    sel_state.focused_index = 0;
                    let font = sel_state.font.clone().unwrap_or_default();
                    despawn_ui(&mut commands, &ui_roots);
                    spawn_grade_ui(&mut commands, &font);
                }
                SelectionStep::Language => {
                    next_state.set(GameState::MainMenu);
                }
            }
        }
    }
}

fn handle_button_visuals(
    mut buttons: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&ButtonIndex>,
        ),
        (With<Button>, Without<BackButton>),
    >,
    sel_state: Res<SelectionState>,
) {
    for (interaction, mut bg, mut border, btn_idx) in &mut buttons {
        let is_focused = btn_idx.is_some_and(|idx| idx.0 == sel_state.focused_index);

        *bg = match *interaction {
            Interaction::Pressed => BTN_PRESSED.into(),
            Interaction::Hovered => BTN_HOVERED.into(),
            _ if is_focused => BTN_HOVERED.into(),
            _ => BTN_NORMAL.into(),
        };

        *border = if is_focused {
            BorderColor::all(BTN_FOCUSED)
        } else {
            BorderColor::all(TEXT_SECONDARY)
        };
    }
}

fn handle_escape_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    ui_roots: Query<Entity, With<SelectionUiRoot>>,
    mut sel_state: ResMut<SelectionState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match sel_state.step {
            SelectionStep::Language => {
                next_state.set(GameState::MainMenu);
            }
            SelectionStep::Grade => {
                sel_state.step = SelectionStep::Language;
                sel_state.focused_index = 0;
                let font = sel_state.font.clone().unwrap_or_default();
                despawn_ui(&mut commands, &ui_roots);
                spawn_language_ui(&mut commands, &font);
            }
            SelectionStep::Difficulty => {
                sel_state.step = SelectionStep::Grade;
                sel_state.focused_index = 0;
                let font = sel_state.font.clone().unwrap_or_default();
                despawn_ui(&mut commands, &ui_roots);
                spawn_grade_ui(&mut commands, &font);
            }
        }
    }
}
