use bevy::prelude::*;

use crate::data::text_model::{Difficulty, Grade, Language};

#[derive(Resource, Debug, Clone)]
pub struct GameConfig {
    pub language: Option<Language>,
    pub grade: Option<Grade>,
    pub difficulty: Option<Difficulty>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            language: None,
            grade: None,
            difficulty: None,
        }
    }
}

impl GameConfig {
    pub fn reset(&mut self) {
        self.language = None;
        self.grade = None;
        self.difficulty = None;
    }
}
