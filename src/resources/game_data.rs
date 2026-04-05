use bevy::prelude::*;
use std::collections::HashMap;

use crate::data::text_model::{Grade, Language, TextPassage};

#[derive(Resource)]
pub struct TextLibrary {
    pub passages: HashMap<(Language, Grade), Vec<TextPassage>>,
}

#[derive(Resource, Clone)]
pub struct CurrentPassage {
    pub passage: TextPassage,
}
