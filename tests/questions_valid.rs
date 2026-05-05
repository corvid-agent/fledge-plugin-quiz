//! Integration tests that validate all question JSON files are well-formed.

use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct Question {
    question: String,
    answer: String,
    #[serde(default)]
    difficulty: Option<String>,
}

fn questions_dir() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/questions"))
}

#[test]
fn all_json_files_parse_successfully() {
    let dir = questions_dir();
    assert!(dir.exists(), "questions/ directory must exist");

    let mut count = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            let data = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
            let questions: Vec<Question> = serde_json::from_str(&data)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
            assert!(
                !questions.is_empty(),
                "{} must contain at least one question",
                path.display()
            );
            count += questions.len();
        }
    }
    assert!(count > 0, "Expected at least one question file");
}

#[test]
fn all_questions_have_non_empty_fields() {
    let dir = questions_dir();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            let data = fs::read_to_string(&path).unwrap();
            let questions: Vec<Question> = serde_json::from_str(&data).unwrap();

            for (i, q) in questions.iter().enumerate() {
                assert!(
                    !q.question.trim().is_empty(),
                    "{}[{}]: question must not be empty",
                    path.display(),
                    i
                );
                assert!(
                    !q.answer.trim().is_empty(),
                    "{}[{}]: answer must not be empty",
                    path.display(),
                    i
                );
            }
        }
    }
}

#[test]
fn difficulty_values_are_valid() {
    let valid = ["easy", "medium", "hard"];
    let dir = questions_dir();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            let data = fs::read_to_string(&path).unwrap();
            let questions: Vec<Question> = serde_json::from_str(&data).unwrap();

            for (i, q) in questions.iter().enumerate() {
                if let Some(ref d) = q.difficulty {
                    assert!(
                        valid.contains(&d.as_str()),
                        "{}[{}]: difficulty '{}' is not one of {:?}",
                        path.display(),
                        i,
                        d,
                        valid
                    );
                }
            }
        }
    }
}

#[test]
fn no_duplicate_questions_within_topic() {
    let dir = questions_dir();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            let data = fs::read_to_string(&path).unwrap();
            let questions: Vec<Question> = serde_json::from_str(&data).unwrap();

            let mut seen = std::collections::HashSet::new();
            for q in &questions {
                let normalized = q.question.trim().to_lowercase();
                assert!(
                    seen.insert(normalized.clone()),
                    "Duplicate question in {}: '{}'",
                    path.display(),
                    q.question
                );
            }
        }
    }
}
