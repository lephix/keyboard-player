use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Zh,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::En => write!(f, "English"),
            Language::Zh => write!(f, "中文"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Grade {
    Elementary,
    Middle,
    High,
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Grade::Elementary => write!(f, "小学"),
            Grade::Middle => write!(f, "初中"),
            Grade::High => write!(f, "高中"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Difficulty {
    Easy,
    Hard,
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Difficulty::Easy => write!(f, "简单"),
            Difficulty::Hard => write!(f, "困难"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextPassage {
    pub id: String,
    pub language: Language,
    pub grade: Grade,
    pub title: String,
    pub author: Option<String>,
    pub content: String,
}
