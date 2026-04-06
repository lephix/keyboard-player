mod audio;
mod data;
mod resources;
mod states;
mod storage;
mod systems;

use bevy::prelude::*;
use std::path::Path;

use audio::sfx::SfxPlugin;
use data::text_loader;
use resources::font_assets::FontAssets;
use resources::game_config::GameConfig;
use resources::game_data::TextLibrary;

fn main() {
    // macOS .app bundle: set CWD to Contents/Resources/ so Bevy finds assets/
    #[cfg(target_os = "macos")]
    {
        if let Ok(exe) = std::env::current_exe()
            && let Some(macos_dir) = exe.parent()
            && macos_dir.ends_with("Contents/MacOS")
            && let Some(contents) = macos_dir.parent()
        {
            let resources = contents.join("Resources");
            if resources.exists() {
                let _ = std::env::set_current_dir(&resources);
            }
        }
    }

    let passages = text_loader::load_passages_from_dir(Path::new("assets/texts"));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Keyboard Player".into(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.102, 0.102, 0.180)))
        .insert_resource(TextLibrary { passages })
        .init_resource::<GameConfig>()
        .init_resource::<FontAssets>()
        .add_plugins(states::StatesPlugin)
        .add_plugins(SfxPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
