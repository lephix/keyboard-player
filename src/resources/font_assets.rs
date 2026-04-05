use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct FontAssets {
    pub pixel_font: Handle<Font>,
}

impl FromWorld for FontAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        FontAssets {
            pixel_font: asset_server.load("fonts/ark-pixel-12px-zh.otf"),
        }
    }
}
