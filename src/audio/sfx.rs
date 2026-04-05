use bevy::prelude::*;

#[derive(Resource)]
pub struct SfxHandles {
    pub correct_key: Handle<AudioSource>,
    pub wrong_key: Handle<AudioSource>,
    pub line_complete: Handle<AudioSource>,
    pub record_break: Handle<AudioSource>,
}

impl FromWorld for SfxHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        SfxHandles {
            correct_key: asset_server.load("audio/correct_key.ogg"),
            wrong_key: asset_server.load("audio/wrong_key.ogg"),
            line_complete: asset_server.load("audio/line_complete.ogg"),
            record_break: asset_server.load("audio/record_break.ogg"),
        }
    }
}

#[derive(Message)]
pub enum SfxEvent {
    CorrectKey,
    WrongKey,
    LineComplete,
    RecordBreak,
}

pub struct SfxPlugin;

impl Plugin for SfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SfxEvent>()
            .init_resource::<SfxHandles>()
            .add_systems(Update, play_sfx);
    }
}

fn play_sfx(mut commands: Commands, mut events: MessageReader<SfxEvent>, sfx: Res<SfxHandles>) {
    for event in events.read() {
        let source = match event {
            SfxEvent::CorrectKey => sfx.correct_key.clone(),
            SfxEvent::WrongKey => sfx.wrong_key.clone(),
            SfxEvent::LineComplete => sfx.line_complete.clone(),
            SfxEvent::RecordBreak => sfx.record_break.clone(),
        };
        commands.spawn(AudioPlayer::new(source));
    }
}
