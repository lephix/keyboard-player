use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeRecord {
    pub passage_id: String,
    pub language: String,
    pub grade: String,
    pub difficulty: String,
    pub elapsed_secs: f64,
    pub kpm: f64,
    pub accuracy: f64,
    pub total_chars: usize,
    pub timestamp: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RecordStore {
    pub records: Vec<PracticeRecord>,
}

impl RecordStore {
    pub fn best_kpm(&self) -> Option<f64> {
        self.records
            .iter()
            .map(|r| r.kpm)
            .fold(None, |max, kpm| Some(max.map_or(kpm, |m: f64| m.max(kpm))))
    }

    pub fn best_kpm_for(&self, language: &str, grade: &str) -> Option<f64> {
        self.records
            .iter()
            .filter(|r| r.language == language && r.grade == grade)
            .map(|r| r.kpm)
            .fold(None, |max, kpm| Some(max.map_or(kpm, |m: f64| m.max(kpm))))
    }

    pub fn best_records_by_category(&self) -> HashMap<(String, String), PracticeRecord> {
        let mut best: HashMap<(String, String), PracticeRecord> = HashMap::new();
        for record in &self.records {
            let key = (record.language.clone(), record.grade.clone());
            if let Some(existing) = best.get(&key) {
                if record.kpm > existing.kpm {
                    best.insert(key, record.clone());
                }
            } else {
                best.insert(key, record.clone());
            }
        }
        best
    }
}

fn records_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("kb-player");
    fs::create_dir_all(&dir).ok();
    dir.join("records.json")
}

pub fn load_records() -> RecordStore {
    let path = records_path();
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => RecordStore::default(),
    }
}

pub fn save_records(store: &RecordStore) {
    let path = records_path();
    if let Ok(json) = serde_json::to_string_pretty(store) {
        if let Err(e) = fs::write(&path, json) {
            eprintln!("Failed to save records: {}", e);
        }
    }
}

pub fn add_record(record: PracticeRecord) -> bool {
    let mut store = load_records();
    let prev_best = store.best_kpm();
    store.records.push(record.clone());
    save_records(&store);
    prev_best.map_or(true, |best| record.kpm > best)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(kpm: f64) -> PracticeRecord {
        PracticeRecord {
            passage_id: "test".into(),
            language: "En".into(),
            grade: "Elementary".into(),
            difficulty: "Easy".into(),
            elapsed_secs: 10.0,
            kpm,
            accuracy: 100.0,
            total_chars: 50,
            timestamp: "0".into(),
        }
    }

    #[test]
    fn empty_store_best_kpm_is_none() {
        let store = RecordStore::default();
        assert!(store.best_kpm().is_none());
    }

    #[test]
    fn best_kpm_returns_highest() {
        let store = RecordStore {
            records: vec![make_record(50.0), make_record(120.0), make_record(80.0)],
        };
        assert_eq!(store.best_kpm(), Some(120.0));
    }

    #[test]
    fn best_kpm_for_filters_by_language_grade() {
        let mut r1 = make_record(100.0);
        r1.language = "Zh".into();
        let r2 = make_record(50.0);
        let store = RecordStore {
            records: vec![r1, r2],
        };
        assert_eq!(store.best_kpm_for("En", "Elementary"), Some(50.0));
        assert_eq!(store.best_kpm_for("Zh", "Elementary"), Some(100.0));
        assert_eq!(store.best_kpm_for("En", "High"), None);
    }

    #[test]
    fn json_round_trip() {
        let store = RecordStore {
            records: vec![make_record(75.5)],
        };
        let json = serde_json::to_string(&store).unwrap();
        let restored: RecordStore = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.records.len(), 1);
        assert_eq!(restored.records[0].kpm, 75.5);
    }

    #[test]
    fn best_records_by_category_picks_highest_kpm() {
        let r1 = make_record(50.0);
        let r2 = make_record(100.0);
        let store = RecordStore {
            records: vec![r1, r2],
        };
        let best = store.best_records_by_category();
        let key = ("En".to_string(), "Elementary".to_string());
        assert_eq!(best[&key].kpm, 100.0);
    }
}
