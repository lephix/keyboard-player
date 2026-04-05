use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::text_model::{Grade, Language, TextPassage};

pub fn load_passages_from_dir(dir: &Path) -> HashMap<(Language, Grade), Vec<TextPassage>> {
    let mut map: HashMap<(Language, Grade), Vec<TextPassage>> = HashMap::new();

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Failed to read texts directory {:?}: {}", dir, err);
            return map;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Failed to read {:?}: {}", path, err);
                continue;
            }
        };

        let passage: TextPassage = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(err) => {
                eprintln!("Failed to parse {:?}: {}", path, err);
                continue;
            }
        };

        map.entry((passage.language, passage.grade))
            .or_default()
            .push(passage);
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_passages_from_assets() {
        let map = load_passages_from_dir(Path::new("assets/texts"));
        let total: usize = map.values().map(|v| v.len()).sum();
        assert_eq!(total, 6);
    }

    #[test]
    fn loads_correct_language_grade_keys() {
        let map = load_passages_from_dir(Path::new("assets/texts"));
        assert!(map.contains_key(&(Language::En, Grade::Elementary)));
        assert!(map.contains_key(&(Language::En, Grade::Middle)));
        assert!(map.contains_key(&(Language::En, Grade::High)));
        assert!(map.contains_key(&(Language::Zh, Grade::Elementary)));
        assert!(map.contains_key(&(Language::Zh, Grade::Middle)));
        assert!(map.contains_key(&(Language::Zh, Grade::High)));
    }

    #[test]
    fn nonexistent_dir_returns_empty() {
        let map = load_passages_from_dir(Path::new("/nonexistent/path/12345"));
        assert!(map.is_empty());
    }

    #[test]
    fn passages_have_nonempty_content() {
        let map = load_passages_from_dir(Path::new("assets/texts"));
        for passages in map.values() {
            for p in passages {
                assert!(!p.content.is_empty(), "passage {} has empty content", p.id);
                assert!(!p.title.is_empty(), "passage {} has empty title", p.id);
            }
        }
    }
}
