use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

use super::GameState;
use crate::audio::sfx::SfxEvent;
use crate::data::text_model::{Difficulty, Language};
use crate::resources::font_assets::FontAssets;
use crate::resources::game_config::GameConfig;
use crate::resources::game_data::CurrentPassage;
use crate::systems::difficulty::{generate_hidden_positions, HiddenChars, HIDDEN_PLACEHOLDER};

const COLOR_CORRECT: Color = Color::srgb(0.29, 0.85, 0.50);
const COLOR_WRONG: Color = Color::srgb(0.97, 0.45, 0.44);
const COLOR_REF_ACTIVE: Color = Color::srgb(0.88, 0.88, 0.88);
const COLOR_REF_DIM: Color = Color::srgb(0.45, 0.45, 0.55);
const COLOR_REF_DONE: Color = Color::srgb(0.35, 0.65, 0.45);
const COLOR_HUD: Color = Color::srgb(0.7, 0.7, 0.8);
const COLOR_CURSOR: Color = Color::srgb(0.88, 0.88, 0.88);

const MAX_CHARS_EN: usize = 60;
const MAX_CHARS_ZH: usize = 25;

#[derive(Resource)]
struct TypingState {
    lines: Vec<String>,
    current_line: usize,
    user_input: String,
    completed_inputs: Vec<String>,
    start_time: Option<f64>,
    total_keystrokes: u32,
    correct_keystrokes: u32,
    input_dirty: bool,
}

#[derive(Resource, Clone)]
pub struct GameResult {
    pub passage_id: String,
    pub title: String,
    pub elapsed_secs: f64,
    pub kpm: f64,
    pub accuracy: f64,
    pub total_chars: usize,
    pub is_new_record: bool,
}

#[derive(Component)]
struct RefLineText(usize);

#[derive(Component)]
struct InputLineText(usize);

#[derive(Component)]
struct HudTimer;

#[derive(Component)]
struct HudKpm;

#[derive(Resource)]
struct CursorTimer {
    timer: Timer,
    visible: bool,
}

#[derive(Resource)]
struct GamePaused {
    is_paused: bool,
    pause_start: Option<f64>,
    total_paused: f64,
}

#[derive(Component)]
struct PauseOverlay;

pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_playing)
            .add_systems(OnExit(GameState::Playing), cleanup_playing)
            .add_systems(
                Update,
                handle_pause_input.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    handle_typing_input,
                    update_cursor_blink,
                    update_input_display,
                    update_ref_line_colors,
                    update_hud,
                    check_completion,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing).and(not_paused)),
            );
    }
}

fn split_lines(content: &str, language: Language) -> Vec<String> {
    let max_chars = match language {
        Language::En => MAX_CHARS_EN,
        Language::Zh => MAX_CHARS_ZH,
    };

    match language {
        Language::En => split_english(content, max_chars),
        Language::Zh => split_chinese(content, max_chars),
    }
}

fn split_english(content: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in content.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn split_chinese(content: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut char_count = 0;

    for ch in content.chars() {
        current.push(ch);
        char_count += 1;

        let is_punct = matches!(ch, '。' | '！' | '？' | '；' | '\n');
        if char_count >= max_chars || is_punct {
            lines.push(current.trim().to_string());
            current = String::new();
            char_count = 0;
        }
    }
    if !current.trim().is_empty() {
        lines.push(current.trim().to_string());
    }
    lines
}

fn setup_playing(
    mut commands: Commands,
    passage: Res<CurrentPassage>,
    config: Res<GameConfig>,
    fonts: Res<FontAssets>,
    mut window: Single<&mut Window>,
) {
    let font = fonts.pixel_font.clone();
    let language = config.language.unwrap_or(Language::En);

    if language == Language::Zh {
        window.ime_enabled = true;
    }
    let difficulty = config.difficulty.unwrap_or(Difficulty::Easy);
    let lines = split_lines(&passage.passage.content, language);
    let line_count = lines.len();

    let hidden = if difficulty == Difficulty::Hard {
        generate_hidden_positions(&lines, language)
    } else {
        HiddenChars::default()
    };

    commands.insert_resource(hidden.clone());

    let display_lines: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(line_idx, line)| {
            let hidden_set = hidden.positions.get(line_idx);
            line.chars()
                .enumerate()
                .map(|(char_idx, ch)| {
                    if hidden_set.map_or(false, |s| s.contains(&char_idx)) {
                        HIDDEN_PLACEHOLDER
                    } else {
                        ch
                    }
                })
                .collect()
        })
        .collect();

    commands.insert_resource(TypingState {
        lines: lines.clone(),
        current_line: 0,
        user_input: String::new(),
        completed_inputs: Vec::new(),
        start_time: None,
        total_keystrokes: 0,
        correct_keystrokes: 0,
        input_dirty: true,
    });

    commands.insert_resource(CursorTimer {
        timer: Timer::from_seconds(0.53, TimerMode::Repeating),
        visible: true,
    });

    commands.insert_resource(GamePaused {
        is_paused: false,
        pause_start: None,
        total_paused: 0.0,
    });

    commands
        .spawn((
            DespawnOnExit(GameState::Playing),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(20.0)),
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            })
            .with_children(|hud| {
                hud.spawn((
                    HudTimer,
                    Text::new("Time: 0.0s"),
                    TextFont {
                        font: font.clone(),
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(COLOR_HUD),
                ));
                hud.spawn((
                    HudKpm,
                    Text::new("KPM: 0"),
                    TextFont {
                        font: font.clone(),
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(COLOR_HUD),
                ));
            });

            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                padding: UiRect::horizontal(Val::Px(40.0)),
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|scroll| {
                for i in 0..line_count {
                    scroll
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            margin: UiRect::bottom(Val::Px(12.0)),
                            ..default()
                        })
                        .with_children(|line_block| {
                            line_block.spawn((
                                RefLineText(i),
                                Text::new(display_lines[i].clone()),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(if i == 0 {
                                    COLOR_REF_ACTIVE
                                } else {
                                    COLOR_REF_DIM
                                }),
                            ));

                            line_block.spawn((
                                InputLineText(i),
                                Text::new(""),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(COLOR_CORRECT),
                                Node {
                                    min_height: Val::Px(28.0),
                                    ..default()
                                },
                            ));
                        });
                }
            });
        });
}

fn cleanup_playing(mut commands: Commands, mut window: Single<&mut Window>) {
    commands.remove_resource::<TypingState>();
    commands.remove_resource::<HiddenChars>();
    commands.remove_resource::<CursorTimer>();
    commands.remove_resource::<GamePaused>();
    window.ime_enabled = false;
}

fn not_paused(paused: Option<Res<GamePaused>>) -> bool {
    paused.map_or(true, |p| !p.is_paused)
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<GamePaused>,
    mut commands: Commands,
    time: Res<Time>,
    fonts: Res<FontAssets>,
    mut next_state: ResMut<NextState<GameState>>,
    overlay: Query<Entity, With<PauseOverlay>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if paused.is_paused {
            paused.is_paused = false;
            if let Some(start) = paused.pause_start.take() {
                paused.total_paused += time.elapsed_secs_f64() - start;
            }
            for entity in &overlay {
                commands.entity(entity).despawn();
            }
        } else {
            paused.is_paused = true;
            paused.pause_start = Some(time.elapsed_secs_f64());
            spawn_pause_overlay(&mut commands, &fonts);
        }
        return;
    }

    if !paused.is_paused {
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        paused.is_paused = false;
        if let Some(start) = paused.pause_start.take() {
            paused.total_paused += time.elapsed_secs_f64() - start;
        }
        for entity in &overlay {
            commands.entity(entity).despawn();
        }
    }

    if keyboard.just_pressed(KeyCode::KeyQ) {
        next_state.set(GameState::MainMenu);
    }
}

fn spawn_pause_overlay(commands: &mut Commands, fonts: &FontAssets) {
    let font = fonts.pixel_font.clone();
    commands
        .spawn((
            PauseOverlay,
            DespawnOnExit(GameState::Playing),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(10),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font: font.clone(),
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(0.88, 0.88, 0.88)),
            ));
            root.spawn((
                Text::new("Enter / Esc  -  Resume"),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
            ));
            root.spawn((
                Text::new("Q  -  Quit (no score saved)"),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
            ));
        });
}

fn handle_typing_input(
    mut kb_events: MessageReader<KeyboardInput>,
    mut ime_events: MessageReader<Ime>,
    mut state: ResMut<TypingState>,
    time: Res<Time>,
    mut sfx_events: MessageWriter<SfxEvent>,
) {
    let Some(ref_line) = state.lines.get(state.current_line).cloned() else {
        return;
    };

    for event in ime_events.read() {
        if let Ime::Commit { value, .. } = event {
            if state.start_time.is_none() {
                state.start_time = Some(time.elapsed_secs_f64());
            }
            for ch in value.chars() {
                if state.user_input.chars().count() >= ref_line.chars().count() {
                    break;
                }
                state.user_input.push(ch);
                state.total_keystrokes += 1;
                state.input_dirty = true;

                let input_len = state.user_input.chars().count();
                let ref_char = ref_line.chars().nth(input_len - 1);
                if ref_char == Some(ch) {
                    state.correct_keystrokes += 1;
                    sfx_events.write(SfxEvent::CorrectKey);
                } else {
                    sfx_events.write(SfxEvent::WrongKey);
                }
            }
        }
    }

    for event in kb_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match (&event.logical_key, &event.text) {
            (Key::Backspace, _) => {
                if !state.user_input.is_empty() {
                    state.user_input.pop();
                    state.input_dirty = true;
                }
            }
            (_, Some(inserted_text)) => {
                if inserted_text.chars().all(|c| !c.is_control()) {
                    if state.start_time.is_none() {
                        state.start_time = Some(time.elapsed_secs_f64());
                    }
                    for ch in inserted_text.chars() {
                        if state.user_input.chars().count() >= ref_line.chars().count() {
                            break;
                        }
                        state.user_input.push(ch);
                        state.total_keystrokes += 1;
                        state.input_dirty = true;

                        let input_len = state.user_input.chars().count();
                        let ref_char = ref_line.chars().nth(input_len - 1);
                        if ref_char == Some(ch) {
                            state.correct_keystrokes += 1;
                            sfx_events.write(SfxEvent::CorrectKey);
                        } else {
                            sfx_events.write(SfxEvent::WrongKey);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn update_cursor_blink(
    time: Res<Time>,
    mut cursor_timer: ResMut<CursorTimer>,
    mut state: ResMut<TypingState>,
) {
    cursor_timer.timer.tick(time.delta());
    if cursor_timer.timer.just_finished() {
        cursor_timer.visible = !cursor_timer.visible;
        state.input_dirty = true;
    }
}

fn update_input_display(
    mut state: ResMut<TypingState>,
    mut input_texts: Query<(Entity, &InputLineText, &mut Text)>,
    mut commands: Commands,
    fonts: Res<FontAssets>,
    cursor_timer: Res<CursorTimer>,
) {
    if !state.input_dirty {
        return;
    }
    state.input_dirty = false;

    let current = state.current_line;
    let ref_line = match state.lines.get(current) {
        Some(l) => l.clone(),
        None => return,
    };

    let font = fonts.pixel_font.clone();
    let cursor_color = if cursor_timer.visible {
        TextColor(COLOR_CURSOR)
    } else {
        TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0))
    };

    for (entity, marker, mut text) in &mut input_texts {
        if marker.0 < current {
            if let Some(completed) = state.completed_inputs.get(marker.0) {
                if !completed.is_empty() {
                    commands.entity(entity).despawn_related::<Children>();
                    **text = completed.clone();
                    commands.entity(entity).insert(TextColor(COLOR_CORRECT));
                }
            }
            continue;
        }

        if marker.0 != current {
            continue;
        }

        let segments = build_segments(&state.user_input, &ref_line);
        commands.entity(entity).despawn_related::<Children>();

        if segments.is_empty() {
            **text = "▏".to_string();
            commands.entity(entity).insert(cursor_color);
        } else {
            **text = segments[0].0.clone();
            commands.entity(entity).insert(TextColor(segments[0].1));

            for seg in &segments[1..] {
                let child = commands
                    .spawn((
                        TextSpan::new(seg.0.clone()),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(seg.1),
                    ))
                    .id();
                commands.entity(entity).add_child(child);
            }

            let cursor_child = commands
                .spawn((
                    TextSpan::new("▏"),
                    TextFont {
                        font: font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    cursor_color,
                ))
                .id();
            commands.entity(entity).add_child(cursor_child);
        }
    }
}

fn build_segments(input: &str, reference: &str) -> Vec<(String, Color)> {
    let mut segments: Vec<(String, Color)> = Vec::new();
    let ref_chars: Vec<char> = reference.chars().collect();

    for (i, ch) in input.chars().enumerate() {
        let color = if i < ref_chars.len() && ch == ref_chars[i] {
            COLOR_CORRECT
        } else {
            COLOR_WRONG
        };

        if let Some(last) = segments.last_mut() {
            if last.1 == color {
                last.0.push(ch);
                continue;
            }
        }
        segments.push((ch.to_string(), color));
    }
    segments
}

fn update_ref_line_colors(
    state: Res<TypingState>,
    mut ref_texts: Query<(&RefLineText, &mut TextColor)>,
) {
    if !state.is_changed() {
        return;
    }

    for (marker, mut color) in &mut ref_texts {
        *color = if marker.0 < state.current_line {
            TextColor(COLOR_REF_DONE)
        } else if marker.0 == state.current_line {
            TextColor(COLOR_REF_ACTIVE)
        } else {
            TextColor(COLOR_REF_DIM)
        };
    }
}

fn update_hud(
    state: Res<TypingState>,
    time: Res<Time>,
    paused: Res<GamePaused>,
    mut timer_text: Query<&mut Text, (With<HudTimer>, Without<HudKpm>)>,
    mut kpm_text: Query<&mut Text, (With<HudKpm>, Without<HudTimer>)>,
) {
    let elapsed = match state.start_time {
        Some(start) => time.elapsed_secs_f64() - start - paused.total_paused,
        None => 0.0,
    };

    for mut text in &mut timer_text {
        **text = format!("Time: {:.1}s", elapsed);
    }

    let kpm = if elapsed > 0.0 {
        (state.total_keystrokes as f64 / elapsed) * 60.0
    } else {
        0.0
    };

    for mut text in &mut kpm_text {
        **text = format!("KPM: {:.0}", kpm);
    }
}

fn check_completion(
    mut state: ResMut<TypingState>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    paused: Res<GamePaused>,
    passage: Res<CurrentPassage>,
    config: Res<GameConfig>,
    mut sfx_events: MessageWriter<SfxEvent>,
) {
    let current = state.current_line;
    let ref_line = match state.lines.get(current) {
        Some(l) => l.clone(),
        None => return,
    };

    if state.user_input == ref_line {
        let completed = state.user_input.clone();
        state.completed_inputs.push(completed);
        state.user_input.clear();
        state.current_line += 1;
        state.input_dirty = true;
        sfx_events.write(SfxEvent::LineComplete);

        if state.current_line >= state.lines.len() {
            let elapsed = match state.start_time {
                Some(start) => time.elapsed_secs_f64() - start - paused.total_paused,
                None => 0.0,
            };
            let kpm = if elapsed > 0.0 {
                (state.total_keystrokes as f64 / elapsed) * 60.0
            } else {
                0.0
            };
            let accuracy = if state.total_keystrokes > 0 {
                (state.correct_keystrokes as f64 / state.total_keystrokes as f64) * 100.0
            } else {
                100.0
            };

            let lang_str = format!("{:?}", config.language.unwrap_or(Language::En));
            let grade_str = format!(
                "{:?}",
                config
                    .grade
                    .unwrap_or(crate::data::text_model::Grade::Elementary)
            );
            let diff_str = format!("{:?}", config.difficulty.unwrap_or(Difficulty::Easy));

            let record = crate::storage::records::PracticeRecord {
                passage_id: passage.passage.id.clone(),
                language: lang_str,
                grade: grade_str,
                difficulty: diff_str,
                elapsed_secs: elapsed,
                kpm,
                accuracy,
                total_chars: state.lines.iter().map(|l| l.chars().count()).sum(),
                timestamp: chrono_now(),
            };

            let is_new_record = crate::storage::records::add_record(record);

            if is_new_record {
                sfx_events.write(SfxEvent::RecordBreak);
            }

            commands.insert_resource(GameResult {
                passage_id: passage.passage.id.clone(),
                title: passage.passage.title.clone(),
                elapsed_secs: elapsed,
                kpm,
                accuracy,
                total_chars: state.lines.iter().map(|l| l.chars().count()).sum(),
                is_new_record,
            });

            next_state.set(GameState::Result);
        }
    }
}

fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_english_basic() {
        let result = split_english("The quick brown fox jumps over", 15);
        assert_eq!(result, vec!["The quick brown", "fox jumps over"]);
    }

    #[test]
    fn split_english_single_word_exceeds_max() {
        let result = split_english("superlongword", 5);
        assert_eq!(result, vec!["superlongword"]);
    }

    #[test]
    fn split_english_empty() {
        let result = split_english("", 60);
        assert!(result.is_empty());
    }

    #[test]
    fn split_english_exact_fit() {
        let result = split_english("abc def", 7);
        assert_eq!(result, vec!["abc def"]);
    }

    #[test]
    fn split_chinese_on_punctuation() {
        let result = split_chinese("床前明月光。疑是地上霜。", 100);
        assert_eq!(result, vec!["床前明月光。", "疑是地上霜。"]);
    }

    #[test]
    fn split_chinese_empty() {
        let result = split_chinese("", 25);
        assert!(result.is_empty());
    }

    #[test]
    fn split_chinese_max_chars_wraps() {
        let result = split_chinese("一二三四五六七八九十", 5);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].chars().count(), 5);
    }

    #[test]
    fn build_segments_all_correct() {
        let segs = build_segments("abc", "abc");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].0, "abc");
        assert_eq!(segs[0].1, COLOR_CORRECT);
    }

    #[test]
    fn build_segments_all_wrong() {
        let segs = build_segments("xyz", "abc");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].0, "xyz");
        assert_eq!(segs[0].1, COLOR_WRONG);
    }

    #[test]
    fn build_segments_mixed() {
        let segs = build_segments("axc", "abc");
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].1, COLOR_CORRECT);
        assert_eq!(segs[1].1, COLOR_WRONG);
        assert_eq!(segs[2].1, COLOR_CORRECT);
    }

    #[test]
    fn build_segments_empty_input() {
        let segs = build_segments("", "abc");
        assert!(segs.is_empty());
    }

    #[test]
    fn split_lines_dispatches_by_language() {
        let en_result = split_lines("hello world", Language::En);
        assert!(!en_result.is_empty());

        let zh_result = split_lines("你好世界。", Language::Zh);
        assert!(!zh_result.is_empty());
    }
}
