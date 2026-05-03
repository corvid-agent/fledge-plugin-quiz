use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process;

use crossterm::{
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
};
use rand::prelude::*;
use rand::rngs::StdRng;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct Question {
    question: String,
    answer: String,
    #[serde(default = "default_difficulty")]
    difficulty: String,
}

fn default_difficulty() -> String {
    "medium".to_string()
}

fn plugin_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("FLEDGE_PLUGIN_DIR")
            .unwrap_or_else(|_| {
                eprintln!("FLEDGE_PLUGIN_DIR not set — fledge >= 0.15.3 sets it automatically");
                process::exit(1);
            }),
    )
}

fn questions_dir() -> PathBuf {
    plugin_dir().join("questions")
}

fn load_questions(path: &Path) -> Vec<Question> {
    let data = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", path.display(), e);
        process::exit(1);
    });
    serde_json::from_str(&data).unwrap_or_else(|e| {
        eprintln!("Failed to parse {}: {}", path.display(), e);
        process::exit(1);
    })
}

fn available_topics(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut topics = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                topics.push((name, path));
            }
        }
    }
    topics.sort_by(|a, b| a.0.cmp(&b.0));
    topics
}

fn difficulty_color(d: &str) -> Color {
    match d {
        "easy" => Color::Green,
        "medium" => Color::Yellow,
        "hard" => Color::Red,
        _ => Color::White,
    }
}

fn print_header(out: &mut impl Write, text: &str) {
    queue!(
        out,
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print(text),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("\n"),
    )
    .ok();
}

fn cmd_topics() {
    let dir = questions_dir();
    let topics = available_topics(&dir);
    let out = io::stdout();
    let mut w = io::BufWriter::new(out.lock());

    print_header(&mut w, "Available Topics");
    queue!(w, Print("\n")).ok();

    for (name, path) in &topics {
        let questions = load_questions(path);
        let easy = questions.iter().filter(|q| q.difficulty == "easy").count();
        let med = questions.iter().filter(|q| q.difficulty == "medium").count();
        let hard = questions.iter().filter(|q| q.difficulty == "hard").count();

        queue!(
            w,
            Print("  "),
            SetForegroundColor(Color::White),
            SetAttribute(Attribute::Bold),
            Print(format!("{:<15}", name)),
            SetAttribute(Attribute::Reset),
            ResetColor,
            Print(format!("{} questions  ", questions.len())),
            SetForegroundColor(Color::Green),
            Print(format!("{}e ", easy)),
            SetForegroundColor(Color::Yellow),
            Print(format!("{}m ", med)),
            SetForegroundColor(Color::Red),
            Print(format!("{}h", hard)),
            ResetColor,
            Print("\n"),
        )
        .ok();
    }

    w.flush().ok();
}

fn cmd_play(topic: Option<&str>, count: usize, hard_only: bool) {
    let dir = questions_dir();
    let topics = available_topics(&dir);

    if topics.is_empty() {
        eprintln!("No question files found in {}", dir.display());
        process::exit(1);
    }

    let mut rng = StdRng::from_os_rng();

    let (topic_name, topic_path) = if let Some(t) = topic {
        topics
            .iter()
            .find(|(name, _)| name == t)
            .cloned()
            .unwrap_or_else(|| {
                eprintln!(
                    "Topic '{}' not found. Run 'fledge quiz topics' to see available topics.",
                    t
                );
                process::exit(1);
            })
    } else {
        topics[rng.random_range(0..topics.len())].clone()
    };

    let mut questions = load_questions(&topic_path);

    if hard_only {
        questions.retain(|q| q.difficulty == "hard");
        if questions.is_empty() {
            eprintln!("No hard questions in topic '{}'", topic_name);
            process::exit(1);
        }
    }

    questions.shuffle(&mut rng);
    let actual_count = count.min(questions.len());
    let selected = &questions[..actual_count];

    let out = io::stdout();
    let mut w = io::BufWriter::new(out.lock());
    let stdin = io::stdin();

    // Header
    queue!(
        w,
        Print("\n"),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print(format!("══ Developer Quiz: {} ══", topic_name)),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("\n"),
        SetForegroundColor(Color::DarkGrey),
        Print(format!("{} questions", actual_count)),
        ResetColor,
        Print("\n\n"),
    )
    .ok();
    w.flush().ok();

    let mut correct = 0;

    for (i, q) in selected.iter().enumerate() {
        let dc = difficulty_color(&q.difficulty);

        // Question number and difficulty
        queue!(
            w,
            SetForegroundColor(Color::White),
            SetAttribute(Attribute::Bold),
            Print(format!("  Q{}/{}", i + 1, actual_count)),
            SetAttribute(Attribute::Reset),
            Print("  "),
            SetForegroundColor(dc),
            Print(format!("[{}]", q.difficulty)),
            ResetColor,
            Print("\n"),
        )
        .ok();

        // Question text
        queue!(
            w,
            Print("  "),
            SetForegroundColor(Color::White),
            Print(&q.question),
            ResetColor,
            Print("\n"),
        )
        .ok();

        // Prompt
        queue!(
            w,
            Print("\n  "),
            SetForegroundColor(Color::DarkGrey),
            Print("→ "),
            ResetColor,
        )
        .ok();
        w.flush().ok();

        let mut answer = String::new();
        stdin.lock().read_line(&mut answer).ok();
        let answer = answer.trim().to_lowercase();
        let expected = q.answer.trim().to_lowercase();

        if answer == expected {
            correct += 1;
            queue!(
                w,
                Print("  "),
                SetForegroundColor(Color::Green),
                SetAttribute(Attribute::Bold),
                Print("  ✓ Correct!"),
                SetAttribute(Attribute::Reset),
                ResetColor,
                Print("\n\n"),
            )
            .ok();
        } else {
            queue!(
                w,
                Print("  "),
                SetForegroundColor(Color::Red),
                SetAttribute(Attribute::Bold),
                Print("  ✗ Wrong"),
                SetAttribute(Attribute::Reset),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("  Answer: {}", q.answer)),
                ResetColor,
                Print("\n\n"),
            )
            .ok();
        }
        w.flush().ok();
    }

    // Score summary
    let pct = if actual_count > 0 {
        (correct as f64 / actual_count as f64 * 100.0) as usize
    } else {
        0
    };
    let bar_width = 20;
    let filled = (pct * bar_width) / 100;
    let empty = bar_width - filled;

    let score_color = if pct >= 80 {
        Color::Green
    } else if pct >= 50 {
        Color::Yellow
    } else {
        Color::Red
    };

    queue!(
        w,
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("  ══ Results ══"),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("\n\n"),
        Print("  Score: "),
        SetForegroundColor(score_color),
        SetAttribute(Attribute::Bold),
        Print(format!("{}/{}", correct, actual_count)),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print(format!("  ({}%)\n", pct)),
        Print("  "),
        SetForegroundColor(score_color),
        Print(format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))),
        ResetColor,
        Print("\n\n"),
    )
    .ok();

    // Message
    let msg = if pct == 100 {
        "Perfect score!"
    } else if pct >= 80 {
        "Great job!"
    } else if pct >= 50 {
        "Not bad, keep learning."
    } else {
        "Keep studying!"
    };

    queue!(
        w,
        Print("  "),
        SetForegroundColor(score_color),
        Print(msg),
        ResetColor,
        Print("\n\n"),
    )
    .ok();
    w.flush().ok();
}

fn cmd_random() {
    let dir = questions_dir();
    let topics = available_topics(&dir);

    if topics.is_empty() {
        eprintln!("No question files found.");
        process::exit(1);
    }

    let mut rng = StdRng::from_os_rng();
    let (topic_name, topic_path) = &topics[rng.random_range(0..topics.len())];
    let questions = load_questions(topic_path);
    let q = &questions[rng.random_range(0..questions.len())];

    let out = io::stdout();
    let mut w = io::BufWriter::new(out.lock());
    let stdin = io::stdin();

    let dc = difficulty_color(&q.difficulty);

    queue!(
        w,
        Print("\n"),
        SetForegroundColor(Color::DarkGrey),
        Print(format!("  [{}]", topic_name)),
        Print("  "),
        SetForegroundColor(dc),
        Print(format!("[{}]", q.difficulty)),
        ResetColor,
        Print("\n\n"),
        Print("  "),
        SetForegroundColor(Color::White),
        SetAttribute(Attribute::Bold),
        Print(&q.question),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("\n\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("  Answer (or Enter to reveal): "),
        ResetColor,
    )
    .ok();
    w.flush().ok();

    let mut answer = String::new();
    stdin.lock().read_line(&mut answer).ok();
    let answer = answer.trim();

    if answer.is_empty() {
        queue!(
            w,
            Print("  "),
            SetForegroundColor(Color::Cyan),
            Print(format!("Answer: {}", q.answer)),
            ResetColor,
            Print("\n\n"),
        )
        .ok();
    } else {
        let norm_ans = answer.to_lowercase();
        let norm_exp = q.answer.trim().to_lowercase();
        if norm_ans == norm_exp {
            queue!(
                w,
                Print("  "),
                SetForegroundColor(Color::Green),
                SetAttribute(Attribute::Bold),
                Print("✓ Correct!"),
                SetAttribute(Attribute::Reset),
                ResetColor,
                Print("\n\n"),
            )
            .ok();
        } else {
            queue!(
                w,
                Print("  "),
                SetForegroundColor(Color::Red),
                SetAttribute(Attribute::Bold),
                Print("✗ Wrong"),
                SetAttribute(Attribute::Reset),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("  Answer: {}", q.answer)),
                ResetColor,
                Print("\n\n"),
            )
            .ok();
        }
    }
    w.flush().ok();
}

fn print_usage() {
    let out = io::stdout();
    let mut w = io::BufWriter::new(out.lock());
    queue!(
        w,
        Print("fledge quiz — developer trivia (Rust TUI)\n\n"),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("USAGE:\n"),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("    fledge quiz <command> [options]\n\n"),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("COMMANDS:\n"),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("    play [topic]        Answer questions (random topic if omitted)\n"),
        Print("    topics              List available topics\n"),
        Print("    random              Single random question\n\n"),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("OPTIONS:\n"),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("    --count <n>         Number of questions (default: 5)\n"),
        Print("    --hard              Only hard questions\n\n"),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("EXAMPLES:\n"),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("    fledge quiz play\n"),
        Print("    fledge quiz play http --count 10\n"),
        Print("    fledge quiz play rust --hard\n"),
        Print("    fledge quiz random\n"),
    )
    .ok();
    w.flush().ok();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "play" | "p" => {
            let mut topic: Option<&str> = None;
            let mut count = 5usize;
            let mut hard = false;
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--count" => {
                        i += 1;
                        count = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(5);
                    }
                    "--hard" => hard = true,
                    s if !s.starts_with('-') && topic.is_none() => topic = Some(s),
                    _ => {}
                }
                i += 1;
            }
            cmd_play(topic, count, hard);
        }
        "topics" | "t" => cmd_topics(),
        "random" | "r" => cmd_random(),
        "scores" | "s" => {
            println!("Score tracking coming in v0.3.0");
        }
        "help" | "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown command: {}", cmd);
            print_usage();
            process::exit(1);
        }
    }
}
